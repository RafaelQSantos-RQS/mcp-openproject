pub mod error;
pub mod filters;
pub mod models;

use crate::config::Config;
use error::OpenProjectError;
use filters::WorkPackageFilterBuilder;
use models::{
    CreateWorkPackageRequest, HalCollection, HalProject, HalStatus, HalType, HalWorkPackage,
    PaginatedResponse, ProjectSummary, StatusSummary, TypeSummary, UpdateWorkPackageRequest,
    WorkPackageSummary,
};
use reqwest::{Method, RequestBuilder, Response, header};
use serde_json::json;
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct OpenProjectClient {
    http: reqwest::Client,
    pub config: Config,
}

impl OpenProjectClient {
    pub fn new(config: Config) -> Result<Self, OpenProjectError> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/hal+json, application/json"),
        );
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("mcp-openproject/0.1.0"),
        );

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self { http, config })
    }

    /// Internal request builder helper applying basic authentication
    fn request(&self, method: Method, endpoint: &str) -> RequestBuilder {
        let full_url = self.config.api_v3_url(endpoint);
        self.http
            .request(method, &full_url)
            .basic_auth("apikey", Some(&self.config.api_key))
    }

    /// Helper to process responses and handle error translation
    async fn handle_response(response: Response) -> Result<String, OpenProjectError> {
        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            debug!(
                "OpenProject API returned non-success status {}: {}",
                status, body
            );
            return Err(OpenProjectError::from_response(status, &body));
        }

        Ok(body)
    }

    /// Tests connection by querying root /api/v3
    pub async fn test_connection(&self) -> Result<serde_json::Value, OpenProjectError> {
        info!("Pinging OpenProject API at {}", self.config.base_url);
        let resp = self.request(Method::GET, "").send().await?;
        let body = Self::handle_response(resp).await?;
        let json_val: serde_json::Value = serde_json::from_str(&body)?;
        Ok(json_val)
    }

    /// Lists projects with pagination
    pub async fn list_projects(
        &self,
        page: usize,
        page_size: usize,
    ) -> Result<PaginatedResponse<ProjectSummary>, OpenProjectError> {
        let page = page.max(1);
        let page_size = page_size.clamp(1, 100);

        let resp = self
            .request(Method::GET, "projects")
            .query(&[
                ("offset", page.to_string()),
                ("pageSize", page_size.to_string()),
            ])
            .send()
            .await?;

        let body = Self::handle_response(resp).await?;
        let col: HalCollection<HalProject> = serde_json::from_str(&body)?;

        Ok(PaginatedResponse {
            total: col.total,
            page,
            page_size,
            items: col.embedded.elements.into_iter().map(Into::into).collect(),
        })
    }

    /// Retrieves project details by numeric ID or string identifier
    pub async fn get_project(
        &self,
        id_or_identifier: &str,
    ) -> Result<ProjectSummary, OpenProjectError> {
        let path = format!("projects/{}", id_or_identifier.trim());
        let resp = self.request(Method::GET, &path).send().await?;
        let body = Self::handle_response(resp).await?;
        let project: HalProject = serde_json::from_str(&body)?;
        Ok(project.into())
    }

    /// Lists or searches work packages matching filter criteria
    pub async fn search_work_packages(
        &self,
        filter: &WorkPackageFilterBuilder,
        page: usize,
        page_size: usize,
    ) -> Result<PaginatedResponse<WorkPackageSummary>, OpenProjectError> {
        let page = page.max(1);
        let page_size = page_size.clamp(1, 100);

        let mut req = self.request(Method::GET, "work_packages").query(&[
            ("offset", page.to_string()),
            ("pageSize", page_size.to_string()),
        ]);

        if let Some(query_str) = filter.to_query_string() {
            req = req.query(&[("filters", query_str)]);
        }

        let resp = req.send().await?;
        let body = Self::handle_response(resp).await?;
        let col: HalCollection<HalWorkPackage> = serde_json::from_str(&body)?;

        Ok(PaginatedResponse {
            total: col.total,
            page,
            page_size,
            items: col.embedded.elements.into_iter().map(Into::into).collect(),
        })
    }

    /// Retrieves full details of a work package by ID
    pub async fn get_work_package(&self, id: i64) -> Result<WorkPackageSummary, OpenProjectError> {
        let path = format!("work_packages/{}", id);
        let resp = self.request(Method::GET, &path).send().await?;
        let body = Self::handle_response(resp).await?;
        let wp: HalWorkPackage = serde_json::from_str(&body)?;
        Ok(wp.into())
    }

    /// Creates a new work package in a project
    pub async fn create_work_package(
        &self,
        req: CreateWorkPackageRequest,
    ) -> Result<WorkPackageSummary, OpenProjectError> {
        let mut links = serde_json::Map::new();
        links.insert(
            "type".into(),
            json!({ "href": format!("/api/v3/types/{}", req.type_id) }),
        );

        if let Some(sid) = req.status_id {
            links.insert(
                "status".into(),
                json!({ "href": format!("/api/v3/statuses/{}", sid) }),
            );
        }

        if let Some(pid) = req.priority_id {
            links.insert(
                "priority".into(),
                json!({ "href": format!("/api/v3/priorities/{}", pid) }),
            );
        }

        if let Some(aid) = req.assignee_id {
            links.insert(
                "assignee".into(),
                json!({ "href": format!("/api/v3/users/{}", aid) }),
            );
        }

        if let Some(parent_id) = req.parent_id {
            links.insert(
                "parent".into(),
                json!({ "href": format!("/api/v3/work_packages/{}", parent_id) }),
            );
        }

        let mut body = json!({
            "subject": req.subject,
            "_links": links,
        });

        if let Some(desc) = req.description {
            body["description"] = json!({ "raw": desc });
        }

        if let Some(sd) = req.start_date {
            body["startDate"] = json!(sd);
        }

        if let Some(dd) = req.due_date {
            body["dueDate"] = json!(dd);
        }

        if let (Some(fields), Some(obj)) = (req.custom_fields, body.as_object_mut()) {
            for (k, v) in fields {
                obj.insert(k, v);
            }
        }

        let path = format!("projects/{}/work_packages", req.project_id);
        let resp = self
            .request(Method::POST, &path)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await?;

        let resp_body = Self::handle_response(resp).await?;
        let wp: HalWorkPackage = serde_json::from_str(&resp_body)?;
        Ok(wp.into())
    }

    /// Updates a work package, automatically fetching lockVersion if omitted
    pub async fn update_work_package(
        &self,
        req: UpdateWorkPackageRequest,
    ) -> Result<WorkPackageSummary, OpenProjectError> {
        // Auto-fetch lock_version if not provided
        let current_lock_version = match req.lock_version {
            Some(v) => v,
            None => {
                debug!(
                    "lock_version omitted for work package {}, fetching current version...",
                    req.id
                );
                let current = self.get_work_package(req.id).await?;
                current.lock_version
            }
        };

        let mut body = json!({
            "lockVersion": current_lock_version,
        });

        if let Some(sub) = req.subject {
            body["subject"] = json!(sub);
        }

        if let Some(desc) = req.description {
            body["description"] = json!({ "raw": desc });
        }

        if let Some(sd) = req.start_date {
            body["startDate"] = json!(sd);
        }

        if let Some(dd) = req.due_date {
            body["dueDate"] = json!(dd);
        }

        if let Some(pd) = req.percentage_done {
            body["percentageDone"] = json!(pd);
        }

        let mut links = serde_json::Map::new();
        if let Some(tid) = req.type_id {
            links.insert(
                "type".into(),
                json!({ "href": format!("/api/v3/types/{}", tid) }),
            );
        }

        if let Some(sid) = req.status_id {
            links.insert(
                "status".into(),
                json!({ "href": format!("/api/v3/statuses/{}", sid) }),
            );
        }

        if let Some(pid) = req.priority_id {
            links.insert(
                "priority".into(),
                json!({ "href": format!("/api/v3/priorities/{}", pid) }),
            );
        }

        if let Some(aid) = req.assignee_id {
            links.insert(
                "assignee".into(),
                json!({ "href": format!("/api/v3/users/{}", aid) }),
            );
        }

        if let Some(parent_id) = req.parent_id {
            links.insert(
                "parent".into(),
                json!({ "href": format!("/api/v3/work_packages/{}", parent_id) }),
            );
        }

        if !links.is_empty() {
            body["_links"] = serde_json::Value::Object(links);
        }

        if let (Some(fields), Some(obj)) = (req.custom_fields, body.as_object_mut()) {
            for (k, v) in fields {
                obj.insert(k, v);
            }
        }

        let path = format!("work_packages/{}", req.id);
        let resp = self
            .request(Method::PATCH, &path)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await?;

        let resp_body = Self::handle_response(resp).await?;
        let wp: HalWorkPackage = serde_json::from_str(&resp_body)?;
        Ok(wp.into())
    }

    /// Lists all work package types available in OpenProject
    pub async fn list_types(&self) -> Result<Vec<TypeSummary>, OpenProjectError> {
        let resp = self.request(Method::GET, "types").send().await?;
        let body = Self::handle_response(resp).await?;
        let col: HalCollection<HalType> = serde_json::from_str(&body)?;
        Ok(col.embedded.elements.into_iter().map(Into::into).collect())
    }

    /// Lists all work package statuses available in OpenProject
    pub async fn list_statuses(&self) -> Result<Vec<StatusSummary>, OpenProjectError> {
        let resp = self.request(Method::GET, "statuses").send().await?;
        let body = Self::handle_response(resp).await?;
        let col: HalCollection<HalStatus> = serde_json::from_str(&body)?;
        Ok(col.embedded.elements.into_iter().map(Into::into).collect())
    }
}
