pub mod duration;
pub mod error;
pub mod filters;
pub mod models;

use crate::config::Config;
use duration::parse_duration;
use error::OpenProjectError;
use filters::WorkPackageFilterBuilder;
use models::{
    CreateTimeEntryRequest, CreateVersionRequest, CreateWorkPackageRequest, CustomFieldSchema,
    CustomOptionSummary, HalCollection, HalCustomOption, HalMembership, HalProject, HalSprint,
    HalStatus, HalTimeEntry, HalTimeEntryActivity, HalType, HalUser, HalVersion, HalWorkPackage,
    MembershipSummary, PaginatedResponse, ProjectSummary, SprintSummary, StatusSummary,
    TimeEntryActivitySummary, TimeEntrySummary, TypeSummary, UpdateProjectRequest,
    UpdateWorkPackageRequest, UserSummary, VersionSummary, WorkPackageSchemaSummary,
    WorkPackageSummary,
};
use reqwest::{Method, RequestBuilder, Response, header};
use serde_json::json;
use std::collections::HashMap;
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
            header::HeaderValue::from_static(concat!(
                "mcp-openproject/",
                env!("CARGO_PKG_VERSION")
            )),
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

    /// Updates an existing project's description, status, status explanation, name, etc.
    pub async fn update_project(
        &self,
        req: UpdateProjectRequest,
    ) -> Result<ProjectSummary, OpenProjectError> {
        let mut body = json!({});

        if let Some(name) = req.name {
            body["name"] = json!(name);
        }

        if let Some(desc) = req.description {
            body["description"] = json!({ "raw": desc });
        }

        if let Some(explanation) = req.status_explanation {
            body["statusExplanation"] = json!({ "raw": explanation });
        }

        if let Some(pub_flag) = req.public {
            body["public"] = json!(pub_flag);
        }

        if let Some(status_input) = req.status {
            let normalized = status_input.trim().to_lowercase();
            let mut links = serde_json::Map::new();

            if normalized.is_empty()
                || normalized == "none"
                || normalized == "null"
                || normalized == "clear"
            {
                links.insert("status".into(), json!({ "href": null }));
            } else {
                let status_id = match normalized.as_str() {
                    "on_track" | "on track" | "ontrack" | "no cronograma" => "on_track",
                    "at_risk" | "at risk" | "atrisk" | "em risco" => "at_risk",
                    "off_track" | "off track" | "offtrack" | "fora do cronograma" => "off_track",
                    "not_started" | "not started" | "não iniciado" | "nao iniciado" => {
                        "not_started"
                    }
                    "finished" | "concluído" | "concluido" => "finished",
                    "discontinued" | "descontinuado" => "discontinued",
                    other => {
                        if let Some(stripped) = other.strip_prefix("/api/v3/project_statuses/") {
                            stripped
                        } else {
                            other
                        }
                    }
                };
                links.insert(
                    "status".into(),
                    json!({ "href": format!("/api/v3/project_statuses/{status_id}") }),
                );
            }
            body["_links"] = serde_json::Value::Object(links);
        }

        let path = format!("projects/{}", req.id_or_identifier.trim());
        let resp = self
            .request(Method::PATCH, &path)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await?;

        let resp_body = Self::handle_response(resp).await?;
        let project: HalProject = serde_json::from_str(&resp_body)?;
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

        if let Some(sprint_id) = req.sprint_id {
            if sprint_id > 0 {
                links.insert(
                    "sprint".into(),
                    json!({ "href": format!("/api/v3/sprints/{sprint_id}") }),
                );
            } else {
                links.insert("sprint".into(), json!({ "href": null }));
            }
        }

        if let Some(version_id) = req.version_id {
            if version_id > 0 {
                links.insert(
                    "version".into(),
                    json!({ "href": format!("/api/v3/versions/{version_id}") }),
                );
            } else {
                links.insert("version".into(), json!({ "href": null }));
            }
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

        if let Some(pd) = req.percentage_done {
            body["percentageDone"] = json!(pd);
        }

        if let Some(et) = req.estimated_time {
            let parsed = parse_duration(&et).map_err(OpenProjectError::Validation)?;
            body["estimatedTime"] = json!(parsed);
        }

        if let Some(rt) = req.remaining_time {
            let parsed = parse_duration(&rt).map_err(OpenProjectError::Validation)?;
            body["remainingTime"] = json!(parsed);
        }

        if let Some(fields) = req.custom_fields {
            Self::apply_custom_fields(&mut body, fields);
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

        if let Some(et) = req.estimated_time {
            let parsed = parse_duration(&et).map_err(OpenProjectError::Validation)?;
            body["estimatedTime"] = json!(parsed);
        }

        if let Some(rt) = req.remaining_time {
            let parsed = parse_duration(&rt).map_err(OpenProjectError::Validation)?;
            body["remainingTime"] = json!(parsed);
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

        if let Some(sprint_id) = req.sprint_id {
            if sprint_id > 0 {
                links.insert(
                    "sprint".into(),
                    json!({ "href": format!("/api/v3/sprints/{sprint_id}") }),
                );
            } else {
                links.insert("sprint".into(), json!({ "href": null }));
            }
        }

        if let Some(version_id) = req.version_id {
            if version_id > 0 {
                links.insert(
                    "version".into(),
                    json!({ "href": format!("/api/v3/versions/{version_id}") }),
                );
            } else {
                links.insert("version".into(), json!({ "href": null }));
            }
        }

        if !links.is_empty() {
            body["_links"] = serde_json::Value::Object(links);
        }

        if let Some(fields) = req.custom_fields {
            Self::apply_custom_fields(&mut body, fields);
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

    /// Retrieves information about the currently authenticated user (associated with the API key)
    pub async fn get_current_user(&self) -> Result<UserSummary, OpenProjectError> {
        let resp = self.request(Method::GET, "users/me").send().await?;
        let body = Self::handle_response(resp).await?;
        let user: HalUser = serde_json::from_str(&body)?;
        Ok(user.into())
    }

    /// Lists users available to be assigned to work packages in a specific project
    pub async fn list_project_assignees(
        &self,
        project_id: i64,
    ) -> Result<Vec<UserSummary>, OpenProjectError> {
        let path = format!("projects/{project_id}/available_assignees");
        let resp = self.request(Method::GET, &path).send().await?;
        let body = Self::handle_response(resp).await?;
        let col: HalCollection<HalUser> = serde_json::from_str(&body)?;
        Ok(col.embedded.elements.into_iter().map(Into::into).collect())
    }

    /// Adds a user as a member of a project with a role (default role_id is 9: Member)
    pub async fn add_project_member(
        &self,
        project_id: i64,
        user_id: i64,
        role_id: Option<i64>,
    ) -> Result<MembershipSummary, OpenProjectError> {
        let role_id = role_id.unwrap_or(9);
        let body = json!({
            "_links": {
                "project": { "href": format!("/api/v3/projects/{project_id}") },
                "principal": { "href": format!("/api/v3/users/{user_id}") },
                "roles": [
                    { "href": format!("/api/v3/roles/{role_id}") }
                ]
            }
        });

        let resp = self
            .request(Method::POST, "memberships")
            .header(header::CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await?;

        let resp_body = Self::handle_response(resp).await?;
        let membership: HalMembership = serde_json::from_str(&resp_body)?;
        Ok(membership.into())
    }

    /// Lists time entry activities available for a project and/or work package
    pub async fn list_time_entry_activities(
        &self,
        project_id: Option<i64>,
        work_package_id: Option<i64>,
    ) -> Result<Vec<TimeEntryActivitySummary>, OpenProjectError> {
        let mut links = serde_json::Map::new();
        if let Some(wpid) = work_package_id {
            links.insert(
                "workPackage".into(),
                json!({ "href": format!("/api/v3/work_packages/{wpid}") }),
            );
        } else if let Some(pid) = project_id {
            links.insert(
                "project".into(),
                json!({ "href": format!("/api/v3/projects/{pid}") }),
            );
        }

        let mut body = json!({});
        if !links.is_empty() {
            body["_links"] = serde_json::Value::Object(links);
        }

        let resp = self
            .request(Method::POST, "time_entries/form")
            .header(header::CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await?;

        let resp_body = Self::handle_response(resp).await?;
        let form_val: serde_json::Value = serde_json::from_str(&resp_body)?;

        let mut activities = Vec::new();
        if let Some(allowed) = form_val
            .pointer("/_embedded/schema/activity/_embedded/allowedValues")
            .and_then(|v| v.as_array())
        {
            for item in allowed {
                if let Ok(act) = serde_json::from_value::<HalTimeEntryActivity>(item.clone()) {
                    activities.push(act.into());
                }
            }
        }

        Ok(activities)
    }

    /// Logs spent time on a work package
    pub async fn log_time(
        &self,
        req: CreateTimeEntryRequest,
    ) -> Result<TimeEntrySummary, OpenProjectError> {
        let parsed_hours = parse_duration(&req.hours).map_err(OpenProjectError::Validation)?;

        let spent_on = req
            .spent_on
            .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());

        let mut body = json!({
            "hours": parsed_hours,
            "spentOn": spent_on,
        });

        if let Some(comment) = req.comment {
            body["comment"] = json!({ "raw": comment });
        }

        let mut links = serde_json::Map::new();
        links.insert(
            "workPackage".into(),
            json!({ "href": format!("/api/v3/work_packages/{}", req.work_package_id) }),
        );

        if let Some(act_id) = req.activity_id {
            links.insert(
                "activity".into(),
                json!({ "href": format!("/api/v3/time_entries/activities/{act_id}") }),
            );
        }

        if let Some(uid) = req.user_id {
            links.insert(
                "user".into(),
                json!({ "href": format!("/api/v3/users/{uid}") }),
            );
        }

        body["_links"] = serde_json::Value::Object(links);

        let resp = self
            .request(Method::POST, "time_entries")
            .header(header::CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await?;

        let resp_body = Self::handle_response(resp).await?;
        let time_entry: HalTimeEntry = serde_json::from_str(&resp_body)?;
        Ok(time_entry.into())
    }

    /// Lists sprints, optionally filtered by project ID
    pub async fn list_sprints(
        &self,
        project_id: Option<i64>,
    ) -> Result<Vec<SprintSummary>, OpenProjectError> {
        let mut req = self.request(Method::GET, "sprints");
        if let Some(pid) = project_id {
            let filter = json!([{
                "definingWorkspace": {
                    "operator": "=",
                    "values": [pid.to_string()]
                }
            }]);
            req = req.query(&[("filters", filter.to_string())]);
        }

        let resp = req.send().await?;
        let body = Self::handle_response(resp).await?;
        let col: HalCollection<HalSprint> = serde_json::from_str(&body)?;
        Ok(col.embedded.elements.into_iter().map(Into::into).collect())
    }

    /// Retrieves details of a specific sprint by ID
    pub async fn get_sprint(&self, id: i64) -> Result<SprintSummary, OpenProjectError> {
        let path = format!("sprints/{id}");
        let resp = self.request(Method::GET, &path).send().await?;
        let body = Self::handle_response(resp).await?;
        let sprint: HalSprint = serde_json::from_str(&body)?;
        Ok(sprint.into())
    }

    /// Creates a new version/milestone for a project
    pub async fn create_version(
        &self,
        req: CreateVersionRequest,
    ) -> Result<VersionSummary, OpenProjectError> {
        let mut links = serde_json::Map::new();
        let project_link = json!({ "href": format!("/api/v3/projects/{}", req.project_id) });
        links.insert("definingProject".into(), project_link.clone());
        links.insert("project".into(), project_link);

        let mut body = json!({
            "name": req.name,
            "_links": links,
        });

        if let Some(desc) = req.description {
            body["description"] = json!({ "raw": desc });
        }

        if let Some(sd) = req.start_date {
            body["startDate"] = json!(sd);
        }

        if let Some(ed) = req.end_date {
            body["endDate"] = json!(ed);
        }

        if let Some(status) = req.status {
            body["status"] = json!(status);
        }

        if let Some(sharing) = req.sharing {
            body["sharing"] = json!(sharing);
        }

        let resp = self
            .request(Method::POST, "versions")
            .header(header::CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await?;

        let resp_body = Self::handle_response(resp).await?;
        let version: HalVersion = serde_json::from_str(&resp_body)?;
        Ok(version.into())
    }

    /// Lists versions, optionally filtered by project ID
    pub async fn list_versions(
        &self,
        project_id: Option<i64>,
    ) -> Result<Vec<VersionSummary>, OpenProjectError> {
        let path = match project_id {
            Some(pid) => format!("projects/{pid}/versions"),
            None => "versions".to_string(),
        };

        let resp = self.request(Method::GET, &path).send().await?;
        let body = Self::handle_response(resp).await?;
        let col: HalCollection<HalVersion> = serde_json::from_str(&body)?;
        Ok(col.embedded.elements.into_iter().map(Into::into).collect())
    }

    /// Retrieves details of a specific version by ID
    pub async fn get_version(&self, id: i64) -> Result<VersionSummary, OpenProjectError> {
        let path = format!("versions/{id}");
        let resp = self.request(Method::GET, &path).send().await?;
        let body = Self::handle_response(resp).await?;
        let version: HalVersion = serde_json::from_str(&body)?;
        Ok(version.into())
    }

    /// Inspects active work package schema (custom fields, types, required flags, and allowed CustomOption values)
    pub async fn get_work_package_schema(
        &self,
        project_id: i64,
        type_id: Option<i64>,
        include_standard_fields: bool,
    ) -> Result<WorkPackageSchemaSummary, OpenProjectError> {
        let mut payload = json!({});
        if let Some(tid) = type_id {
            payload["_links"] = json!({
                "type": { "href": format!("/api/v3/types/{tid}") }
            });
        }

        let path = format!("projects/{project_id}/work_packages/form");
        let resp = self
            .request(Method::POST, &path)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&payload)
            .send()
            .await?;

        let body = Self::handle_response(resp).await?;
        let form_json: serde_json::Value = serde_json::from_str(&body)?;

        let schema_obj = form_json
            .pointer("/_embedded/schema")
            .and_then(|v| v.as_object())
            .ok_or_else(|| {
                OpenProjectError::Validation(
                    "OpenProject form response did not include _embedded.schema".into(),
                )
            })?;

        let mut custom_fields = Vec::new();
        let mut standard_fields = Vec::new();

        for (key, def) in schema_obj {
            if key.starts_with('_') {
                continue;
            }
            let Some(def_obj) = def.as_object() else {
                continue;
            };
            let Some(field_type) = def_obj.get("type").and_then(|t| t.as_str()) else {
                continue;
            };

            let name = def_obj
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or(key)
                .to_string();
            let required = def_obj
                .get("required")
                .and_then(|r| r.as_bool())
                .unwrap_or(false);
            let writable = def_obj
                .get("writable")
                .and_then(|w| w.as_bool())
                .unwrap_or(false);
            let has_default = def_obj
                .get("hasDefault")
                .and_then(|d| d.as_bool())
                .unwrap_or(false);
            let location = def_obj
                .get("location")
                .and_then(|l| l.as_str())
                .unwrap_or("root")
                .to_string();

            let allowed_values = Self::extract_allowed_values(def_obj);

            let entry = CustomFieldSchema {
                key: key.clone(),
                name,
                field_type: field_type.to_string(),
                required,
                writable,
                has_default,
                location,
                allowed_values,
            };

            if key.starts_with("customField") {
                custom_fields.push(entry);
            } else if include_standard_fields {
                standard_fields.push(entry);
            }
        }

        custom_fields.sort_by(|a, b| a.key.cmp(&b.key));
        standard_fields.sort_by(|a, b| a.key.cmp(&b.key));

        Ok(WorkPackageSchemaSummary {
            project_id,
            type_id,
            custom_fields,
            standard_fields,
        })
    }

    /// Retrieves details of a specific CustomOption by ID
    pub async fn get_custom_option(
        &self,
        id: i64,
    ) -> Result<CustomOptionSummary, OpenProjectError> {
        let path = format!("custom_options/{id}");
        let resp = self.request(Method::GET, &path).send().await?;
        let body = Self::handle_response(resp).await?;
        let opt: HalCustomOption = serde_json::from_str(&body)?;
        Ok(opt.into())
    }

    /// Merges custom_fields into a Work Package request body, routing HAL link values into `_links`
    fn apply_custom_fields(
        body: &mut serde_json::Value,
        fields: HashMap<String, serde_json::Value>,
    ) {
        let is_hal_link_value = |v: &serde_json::Value| -> bool {
            if v.get("href").is_some() {
                return true;
            }
            v.as_array().is_some_and(|arr| {
                !arr.is_empty() && arr.iter().all(|item| item.get("href").is_some())
            })
        };

        for (k, v) in fields {
            if k == "_links" {
                if let Some(extra_links) = v.as_object() {
                    if body.get("_links").and_then(|l| l.as_object()).is_none() {
                        body["_links"] = json!({});
                    }
                    if let Some(target_links) = body["_links"].as_object_mut() {
                        for (lk, lv) in extra_links {
                            target_links.insert(lk.clone(), lv.clone());
                        }
                    }
                }
            } else if is_hal_link_value(&v) {
                if body.get("_links").and_then(|l| l.as_object()).is_none() {
                    body["_links"] = json!({});
                }
                if let Some(target_links) = body["_links"].as_object_mut() {
                    target_links.insert(k, v);
                }
            } else if let Some(obj) = body.as_object_mut() {
                obj.insert(k, v);
            }
        }
    }

    /// Extracts allowedValues from `_embedded.allowedValues` or `_links.allowedValues` in a schema field
    fn extract_allowed_values(
        def_obj: &serde_json::Map<String, serde_json::Value>,
    ) -> Vec<CustomOptionSummary> {
        if let Some(embedded_vals) = def_obj
            .get("_embedded")
            .and_then(|e| e.get("allowedValues"))
            .and_then(|a| a.as_array())
        {
            return embedded_vals
                .iter()
                .filter_map(|item| serde_json::from_value::<HalCustomOption>(item.clone()).ok())
                .map(Into::into)
                .collect();
        }

        if let Some(link_vals) = def_obj
            .get("_links")
            .and_then(|l| l.get("allowedValues"))
            .and_then(|a| a.as_array())
        {
            return link_vals
                .iter()
                .filter_map(|item| {
                    let href = item.get("href").and_then(|h| h.as_str())?.to_string();
                    let value = item
                        .get("title")
                        .and_then(|t| t.as_str())
                        .unwrap_or(&href)
                        .to_string();
                    let id = href
                        .trim_end_matches('/')
                        .rsplit('/')
                        .next()?
                        .parse::<i64>()
                        .ok();
                    Some(CustomOptionSummary { id, value, href })
                })
                .collect();
        }

        Vec::new()
    }
}
