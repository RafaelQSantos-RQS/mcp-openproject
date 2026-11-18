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

/// Formattable text (raw, html, markdown format, or raw string)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum HalFormattable {
    Structured {
        raw: Option<String>,
        html: Option<String>,
        format: Option<String>,
    },
    Plain(String),
}

impl HalFormattable {
    pub fn as_text(&self) -> Option<String> {
        match self {
            Self::Structured { raw, html, .. } => raw.clone().or_else(|| html.clone()),
            Self::Plain(s) => Some(s.clone()),
        }
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
    #[serde(rename = "remainingTime")]
    pub remaining_time: Option<String>,
    #[serde(rename = "spentTime")]
    pub spent_time: Option<String>,
    #[serde(rename = "percentageDone")]
    pub percentage_done: Option<i32>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
    #[serde(rename = "_links", default)]
    pub links: HashMap<String, serde_json::Value>,
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
    pub sprint_id: Option<i64>,
    pub sprint_name: Option<String>,
    pub start_date: Option<String>,
    pub due_date: Option<String>,
    pub estimated_time: Option<String>,
    pub remaining_time: Option<String>,
    pub spent_time: Option<String>,
    pub percentage_done: Option<i32>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom_fields: HashMap<String, serde_json::Value>,
}

impl From<HalWorkPackage> for WorkPackageSummary {
    fn from(wp: HalWorkPackage) -> Self {
        let extract_from_link_obj =
            |obj: &serde_json::Map<String, serde_json::Value>| -> (Option<i64>, Option<String>) {
                let href = obj.get("href").and_then(|h| h.as_str());
                let title = obj
                    .get("title")
                    .and_then(|t| t.as_str())
                    .map(ToString::to_string);
                let id = href.and_then(|h| {
                    h.trim_end_matches('/')
                        .rsplit('/')
                        .next()?
                        .parse::<i64>()
                        .ok()
                });
                (id, title)
            };

        let get_link_info = |key: &str| -> (Option<i64>, Option<String>) {
            if let Some(val) = wp.links.get(key) {
                if let Some(obj) = val.as_object() {
                    return extract_from_link_obj(obj);
                }
                if let Some(first_obj) = val
                    .as_array()
                    .and_then(|arr| arr.first())
                    .and_then(|item| item.as_object())
                {
                    return extract_from_link_obj(first_obj);
                }
            }
            (None, None)
        };

        let (project_id, project_name) = get_link_info("project");
        let (type_id, type_name) = get_link_info("type");
        let (status_id, status_name) = get_link_info("status");
        let (priority_id, priority_name) = get_link_info("priority");
        let (assignee_id, assignee_name) = get_link_info("assignee");
        let (author_id, author_name) = get_link_info("author");
        let (parent_id, _) = get_link_info("parent");
        let (sprint_id, sprint_name) = get_link_info("sprint");

        // Collect custom fields from extra properties
        let mut custom_fields = HashMap::new();
        for (k, v) in wp.extra {
            if k.starts_with("customField") && !v.is_null() {
                if let Some(raw_text) = v.get("raw").and_then(|r| r.as_str()) {
                    custom_fields.insert(k, serde_json::Value::String(raw_text.to_string()));
                } else {
                    custom_fields.insert(k, v);
                }
            }
        }

        // Also check if any customField is inside _links (e.g. list/user custom fields)
        for (k, v) in &wp.links {
            if k.starts_with("customField") && !v.is_null() && !custom_fields.contains_key(k) {
                if let Some(title) = v.get("title").and_then(|t| t.as_str()) {
                    custom_fields.insert(k.clone(), serde_json::Value::String(title.to_string()));
                } else if let Some(href) = v.get("href").and_then(|h| h.as_str()) {
                    custom_fields.insert(k.clone(), serde_json::Value::String(href.to_string()));
                }
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
            sprint_id,
            sprint_name,
            start_date: wp.start_date,
            due_date: wp.due_date,
            estimated_time: wp.estimated_time,
            remaining_time: wp.remaining_time,
            spent_time: wp.spent_time,
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
    pub parent_id: Option<i64>,
    pub sprint_id: Option<i64>,
    pub start_date: Option<String>,
    pub due_date: Option<String>,
    pub estimated_time: Option<String>,
    pub remaining_time: Option<String>,
    pub percentage_done: Option<i32>,
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
    pub parent_id: Option<i64>,
    pub sprint_id: Option<i64>,
    pub start_date: Option<String>,
    pub due_date: Option<String>,
    pub estimated_time: Option<String>,
    pub remaining_time: Option<String>,
    pub percentage_done: Option<i32>,
    pub custom_fields: Option<HashMap<String, serde_json::Value>>,
}

// -------------------------------------------------------------
// Users
// -------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct HalUser {
    pub id: i64,
    pub name: String,
    pub email: Option<String>,
    pub login: Option<String>,
    pub admin: Option<bool>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct UserSummary {
    pub id: i64,
    pub name: String,
    pub email: Option<String>,
    pub login: Option<String>,
    pub admin: bool,
    pub status: Option<String>,
}

impl From<HalUser> for UserSummary {
    fn from(u: HalUser) -> Self {
        Self {
            id: u.id,
            name: u.name,
            email: u.email,
            login: u.login,
            admin: u.admin.unwrap_or(false),
            status: u.status,
        }
    }
}

// -------------------------------------------------------------
// Memberships
// -------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct HalMembership {
    pub id: i64,
    #[serde(rename = "_links", default)]
    pub links: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct MembershipSummary {
    pub id: i64,
    pub project_id: Option<i64>,
    pub project_name: Option<String>,
    pub user_id: Option<i64>,
    pub user_name: Option<String>,
    pub roles: Vec<String>,
}

impl From<HalMembership> for MembershipSummary {
    fn from(m: HalMembership) -> Self {
        let extract_link = |key: &str| -> (Option<i64>, Option<String>) {
            if let Some(obj) = m.links.get(key).and_then(|v| v.as_object()) {
                let href = obj.get("href").and_then(|h| h.as_str());
                let title = obj
                    .get("title")
                    .and_then(|t| t.as_str())
                    .map(ToString::to_string);
                let id = href.and_then(|h| {
                    h.trim_end_matches('/')
                        .rsplit('/')
                        .next()?
                        .parse::<i64>()
                        .ok()
                });
                return (id, title);
            }
            (None, None)
        };

        let (project_id, project_name) = extract_link("project");
        let (user_id, user_name) = extract_link("principal");

        let mut roles = Vec::new();
        if let Some(arr) = m.links.get("roles").and_then(|v| v.as_array()) {
            for item in arr {
                if let Some(title) = item.get("title").and_then(|t| t.as_str()) {
                    roles.push(title.to_string());
                }
            }
        }

        Self {
            id: m.id,
            project_id,
            project_name,
            user_id,
            user_name,
            roles,
        }
    }
}

// -------------------------------------------------------------
// Time Entries & Activities
// -------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct HalTimeEntryActivity {
    pub id: i64,
    pub name: String,
    #[serde(rename = "isDefault", default)]
    pub is_default: Option<bool>,
    #[serde(default)]
    pub default: Option<bool>,
    pub position: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct TimeEntryActivitySummary {
    pub id: i64,
    pub name: String,
    pub is_default: bool,
}

impl From<HalTimeEntryActivity> for TimeEntryActivitySummary {
    fn from(a: HalTimeEntryActivity) -> Self {
        Self {
            id: a.id,
            name: a.name,
            is_default: a.is_default.or(a.default).unwrap_or(false),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct HalTimeEntry {
    pub id: i64,
    #[serde(rename = "spentOn")]
    pub spent_on: Option<String>,
    pub hours: Option<String>,
    pub comment: Option<HalFormattable>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "_links", default)]
    pub links: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct TimeEntrySummary {
    pub id: i64,
    pub hours: String,
    pub spent_on: String,
    pub comment: Option<String>,
    pub project_id: Option<i64>,
    pub project_name: Option<String>,
    pub work_package_id: Option<i64>,
    pub work_package_subject: Option<String>,
    pub user_id: Option<i64>,
    pub user_name: Option<String>,
    pub activity_id: Option<i64>,
    pub activity_name: Option<String>,
    pub created_at: Option<String>,
}

impl From<HalTimeEntry> for TimeEntrySummary {
    fn from(entry: HalTimeEntry) -> Self {
        let extract_link = |key: &str| -> (Option<i64>, Option<String>) {
            if let Some(obj) = entry.links.get(key).and_then(|v| v.as_object()) {
                let href = obj.get("href").and_then(|h| h.as_str());
                let title = obj
                    .get("title")
                    .and_then(|t| t.as_str())
                    .map(ToString::to_string);
                let id = href.and_then(|h| {
                    h.trim_end_matches('/')
                        .rsplit('/')
                        .next()?
                        .parse::<i64>()
                        .ok()
                });
                return (id, title);
            }
            (None, None)
        };

        let (project_id, project_name) = extract_link("project");
        let (work_package_id, work_package_subject) = extract_link("workPackage");
        let (user_id, user_name) = extract_link("user");
        let (activity_id, activity_name) = extract_link("activity");

        Self {
            id: entry.id,
            hours: entry.hours.unwrap_or_default(),
            spent_on: entry.spent_on.unwrap_or_default(),
            comment: entry.comment.and_then(|c| c.as_text()),
            project_id,
            project_name,
            work_package_id,
            work_package_subject,
            user_id,
            user_name,
            activity_id,
            activity_name,
            created_at: entry.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct CreateTimeEntryRequest {
    pub work_package_id: i64,
    pub hours: String,
    pub comment: Option<String>,
    pub spent_on: Option<String>,
    pub activity_id: Option<i64>,
    pub user_id: Option<i64>,
}

// -------------------------------------------------------------
// Sprints
// -------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct HalSprint {
    pub id: i64,
    pub name: String,
    #[serde(rename = "startDate")]
    pub start_date: Option<String>,
    #[serde(rename = "finishDate")]
    pub finish_date: Option<String>,
    #[serde(rename = "startedAt")]
    pub started_at: Option<String>,
    #[serde(rename = "completedAt")]
    pub completed_at: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
    #[serde(rename = "_links", default)]
    pub links: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SprintSummary {
    pub id: i64,
    pub name: String,
    pub start_date: Option<String>,
    pub finish_date: Option<String>,
    pub status: Option<String>,
    pub project_id: Option<i64>,
    pub project_name: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl From<HalSprint> for SprintSummary {
    fn from(s: HalSprint) -> Self {
        let status = s.links.get("status").and_then(|val| {
            val.get("title")
                .and_then(|t| t.as_str())
                .map(ToString::to_string)
                .or_else(|| {
                    val.get("href")
                        .and_then(|h| h.as_str())
                        .and_then(|h| h.rsplit(':').next().map(ToString::to_string))
                })
        });

        let (project_id, project_name) =
            match s.links.get("definingWorkspace").and_then(|v| v.as_object()) {
                Some(obj) => {
                    let href = obj.get("href").and_then(|h| h.as_str());
                    let title = obj
                        .get("title")
                        .and_then(|t| t.as_str())
                        .map(ToString::to_string);
                    let id = href.and_then(|h| {
                        h.trim_end_matches('/')
                            .rsplit('/')
                            .next()?
                            .parse::<i64>()
                            .ok()
                    });
                    (id, title)
                }
                None => (None, None),
            };

        Self {
            id: s.id,
            name: s.name,
            start_date: s.start_date,
            finish_date: s.finish_date,
            status,
            project_id,
            project_name,
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }
}
