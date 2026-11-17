use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Generic HAL Link representation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HalLink {
    pub href: Option<String>,
    pub title: Option<String>,
}

impl HalLink {
    /// Extracts a numeric ID from an href link like "/api/v3/projects/12"
    pub fn extract_id(&self) -> Option<i64> {
        let href = self.href.as_deref()?;
        href.trim_end_matches('/')
            .rsplit('/')
            .next()?
            .parse::<i64>()
            .ok()
    }
}

/// Formattable text (raw, html, markdown format)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HalFormattable {
    pub raw: Option<String>,
    pub html: Option<String>,
    pub format: Option<String>,
}

impl HalFormattable {
    pub fn as_text(&self) -> Option<String> {
        self.raw.clone().or_else(|| self.html.clone())
    }
}

/// Raw HAL Collection wrapper
#[derive(Debug, Clone, Deserialize)]
pub struct HalCollection<T> {
    #[serde(rename = "_type")]
    pub _type: Option<String>,
    pub total: usize,
    pub count: usize,
    #[serde(rename = "pageSize")]
    pub page_size: Option<usize>,
    pub offset: Option<usize>,
    #[serde(rename = "_embedded")]
    pub embedded: HalEmbedded<T>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HalEmbedded<T> {
    pub elements: Vec<T>,
}

/// Sanitized Paginated response delivered to LLM
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct PaginatedResponse<T> {
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
    pub items: Vec<T>,
}

// -------------------------------------------------------------
// Projects
// -------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct HalProject {
    pub id: i64,
    pub identifier: String,
    pub name: String,
    pub active: Option<bool>,
    pub public: Option<bool>,
    pub description: Option<HalFormattable>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ProjectSummary {
    pub id: i64,
    pub identifier: String,
    pub name: String,
    pub active: bool,
    pub public: bool,
    pub description: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl From<HalProject> for ProjectSummary {
    fn from(p: HalProject) -> Self {
        Self {
            id: p.id,
            identifier: p.identifier,
            name: p.name,
            active: p.active.unwrap_or(true),
            public: p.public.unwrap_or(false),
            description: p.description.and_then(|d| d.as_text()),
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}

// -------------------------------------------------------------
// Types
// -------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct HalType {
    pub id: i64,
    pub name: String,
    pub color: Option<String>,
    #[serde(rename = "isDefault")]
    pub is_default: Option<bool>,
    #[serde(rename = "isMilestone")]
    pub is_milestone: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct TypeSummary {
    pub id: i64,
    pub name: String,
    pub color: Option<String>,
    pub is_default: bool,
    pub is_milestone: bool,
}

impl From<HalType> for TypeSummary {
    fn from(t: HalType) -> Self {
        Self {
            id: t.id,
            name: t.name,
            color: t.color,
            is_default: t.is_default.unwrap_or(false),
            is_milestone: t.is_milestone.unwrap_or(false),
        }
    }
}

// -------------------------------------------------------------
// Statuses
// -------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct HalStatus {
    pub id: i64,
    pub name: String,
    #[serde(rename = "isClosed")]
    pub is_closed: Option<bool>,
    #[serde(rename = "isDefault")]
    pub is_default: Option<bool>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct StatusSummary {
    pub id: i64,
    pub name: String,
    pub is_closed: bool,
    pub is_default: bool,
    pub color: Option<String>,
}

impl From<HalStatus> for StatusSummary {
    fn from(s: HalStatus) -> Self {
        Self {
            id: s.id,
            name: s.name,
            is_closed: s.is_closed.unwrap_or(false),
            is_default: s.is_default.unwrap_or(false),
            color: s.color,
        }
    }
}

// -------------------------------------------------------------
// Work Packages
// -------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct HalWorkPackage {
    pub id: i64,
    #[serde(rename = "lockVersion")]
    pub lock_version: i64,
    pub subject: String,
    pub description: Option<HalFormattable>,
    #[serde(rename = "startDate")]
    pub start_date: Option<String>,
    #[serde(rename = "dueDate")]
    pub due_date: Option<String>,
    #[serde(rename = "estimatedTime")]
    pub estimated_time: Option<String>,
    #[serde(rename = "percentageDone")]
    pub percentage_done: Option<i32>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
    #[serde(rename = "_links", default)]
    pub links: HashMap<String, Option<HalLink>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct WorkPackageSummary {
    pub id: i64,
    pub lock_version: i64,
    pub subject: String,
    pub description: Option<String>,
    pub project_id: Option<i64>,
    pub project_name: Option<String>,
    pub type_id: Option<i64>,
    pub type_name: Option<String>,
    pub status_id: Option<i64>,
    pub status_name: Option<String>,
    pub priority_id: Option<i64>,
    pub priority_name: Option<String>,
    pub assignee_id: Option<i64>,
    pub assignee_name: Option<String>,
    pub author_id: Option<i64>,
    pub author_name: Option<String>,
    pub parent_id: Option<i64>,
    pub start_date: Option<String>,
    pub due_date: Option<String>,
    pub estimated_time: Option<String>,
    pub percentage_done: Option<i32>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom_fields: HashMap<String, serde_json::Value>,
}

impl From<HalWorkPackage> for WorkPackageSummary {
    fn from(wp: HalWorkPackage) -> Self {
        let get_link_info = |key: &str| -> (Option<i64>, Option<String>) {
            if let Some(Some(link)) = wp.links.get(key) {
                (link.extract_id(), link.title.clone())
            } else {
                (None, None)
            }
        };

        let (project_id, project_name) = get_link_info("project");
        let (type_id, type_name) = get_link_info("type");
        let (status_id, status_name) = get_link_info("status");
        let (priority_id, priority_name) = get_link_info("priority");
        let (assignee_id, assignee_name) = get_link_info("assignee");
        let (author_id, author_name) = get_link_info("author");
        let (parent_id, _) = get_link_info("parent");

        // Collect custom fields (customField1, customField2, etc.)
        let mut custom_fields = HashMap::new();
        for (k, v) in wp.extra {
            if k.starts_with("customField") && !v.is_null() {
                custom_fields.insert(k, v);
            }
        }

        Self {
            id: wp.id,
            lock_version: wp.lock_version,
            subject: wp.subject,
            description: wp.description.and_then(|d| d.as_text()),
            project_id,
            project_name,
            type_id,
            type_name,
            status_id,
            status_name,
            priority_id,
            priority_name,
            assignee_id,
            assignee_name,
            author_id,
            author_name,
            parent_id,
            start_date: wp.start_date,
            due_date: wp.due_date,
            estimated_time: wp.estimated_time,
            percentage_done: wp.percentage_done,
            created_at: wp.created_at,
            updated_at: wp.updated_at,
            custom_fields,
        }
    }
}

// -------------------------------------------------------------
// Request structs for creating and updating work packages
// -------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct CreateWorkPackageRequest {
    pub project_id: i64,
    pub type_id: i64,
    pub subject: String,
    pub description: Option<String>,
    pub status_id: Option<i64>,
    pub priority_id: Option<i64>,
    pub assignee_id: Option<i64>,
    pub start_date: Option<String>,
    pub due_date: Option<String>,
    pub custom_fields: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct UpdateWorkPackageRequest {
    pub id: i64,
    pub lock_version: Option<i64>,
    pub subject: Option<String>,
    pub description: Option<String>,
    pub type_id: Option<i64>,
    pub status_id: Option<i64>,
    pub priority_id: Option<i64>,
    pub assignee_id: Option<i64>,
    pub start_date: Option<String>,
    pub due_date: Option<String>,
    pub percentage_done: Option<i32>,
    pub custom_fields: Option<HashMap<String, serde_json::Value>>,
}
