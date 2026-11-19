use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Parameters for listing projects
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct ListProjectsParams {
    /// Page number (1-based, default: 1)
    pub page: Option<usize>,
    /// Number of projects per page (default: 20, max: 100)
    pub page_size: Option<usize>,
}

/// Parameters for retrieving a single project
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetProjectParams {
    /// Numeric project ID or string identifier (e.g. "12" or "my-project")
    pub id_or_identifier: String,
}

/// Parameters for searching and filtering work packages
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct SearchWorkPackagesParams {
    /// Search term in subject, ID, or description
    pub query: Option<String>,
    /// Filter by numeric Project ID
    pub project_id: Option<i64>,
    /// Filter by Type ID (e.g. Task, Bug)
    pub type_id: Option<i64>,
    /// Filter by status: "open", "closed", or specific status name/id
    pub status: Option<String>,
    /// Filter by Assignee (User ID)
    pub assignee_id: Option<i64>,
    /// Filter by Sprint ID
    pub sprint_id: Option<i64>,
    /// Filter by Version ID
    pub version_id: Option<i64>,
    /// Page number (1-based, default: 1)
    pub page: Option<usize>,
    /// Number of items per page (default: 20, max: 100)
    pub page_size: Option<usize>,
}

/// Parameters for getting a single work package by ID
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetWorkPackageParams {
    /// Work package ID
    pub id: i64,
}

/// Parameters for creating a new work package
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateWorkPackageParams {
    /// Numeric Project ID where the work package will be created
    pub project_id: i64,
    /// Numeric Type ID (e.g. 1 for Task, 2 for Bug; use `list_types` to discover)
    pub type_id: i64,
    /// Title / subject of the work package
    pub subject: String,
    /// Optional description (markdown supported)
    pub description: Option<String>,
    /// Optional Status ID (use `list_statuses` to discover)
    pub status_id: Option<i64>,
    /// Optional Priority ID
    pub priority_id: Option<i64>,
    /// Optional Assignee User ID
    pub assignee_id: Option<i64>,
    /// Optional Parent Work Package ID (creates this work package as a child/subtask)
    pub parent_id: Option<i64>,
    /// Optional Sprint ID to assign this work package to a sprint
    pub sprint_id: Option<i64>,
    /// Optional Version ID to assign this work package to a version / milestone
    pub version_id: Option<i64>,
    /// Start date in ISO format YYYY-MM-DD
    pub start_date: Option<String>,
    /// Due date in ISO format YYYY-MM-DD
    pub due_date: Option<String>,
    /// Estimated time / total work (e.g. '4h', '2.5h', '30m', 'PT4H')
    pub estimated_time: Option<String>,
    /// Remaining work time (e.g. '2h', '1h30m', 'PT2H')
    pub remaining_time: Option<String>,
    /// Percentage completed (0 - 100)
    pub percentage_done: Option<i32>,
    /// Custom fields key-value map (e.g. {"customField1": "value"})
    pub custom_fields: Option<HashMap<String, serde_json::Value>>,
}

/// Parameters for updating an existing work package
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateWorkPackageParams {
    /// ID of the work package to update
    pub id: i64,
    /// Optimistic locking version. If omitted, the server automatically fetches the current version.
    pub lock_version: Option<i64>,
    /// New subject / title
    pub subject: Option<String>,
    /// New description (markdown supported)
    pub description: Option<String>,
    /// New Type ID
    pub type_id: Option<i64>,
    /// New Status ID
    pub status_id: Option<i64>,
    /// New Priority ID
    pub priority_id: Option<i64>,
    /// New Assignee User ID
    pub assignee_id: Option<i64>,
    /// New Parent Work Package ID (to set or change parent work package)
    pub parent_id: Option<i64>,
    /// New Sprint ID (or 0 / negative to remove from sprint)
    pub sprint_id: Option<i64>,
    /// New Version ID (or 0 / negative to remove from version)
    pub version_id: Option<i64>,
    /// Start date in ISO format YYYY-MM-DD
    pub start_date: Option<String>,
    /// Due date in ISO format YYYY-MM-DD
    pub due_date: Option<String>,
    /// Estimated time / total work (e.g. '4h', '2.5h', '30m', 'PT4H')
    pub estimated_time: Option<String>,
    /// Remaining work time (e.g. '2h', '1h30m', 'PT2H')
    pub remaining_time: Option<String>,
    /// Percentage completed (0 - 100)
    pub percentage_done: Option<i32>,
    /// Custom fields key-value map
    pub custom_fields: Option<HashMap<String, serde_json::Value>>,
}

/// Parameters for listing types (empty)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct ListTypesParams {}

/// Parameters for listing statuses (empty)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct ListStatusesParams {}

/// Parameters for getting current authenticated user (empty)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct GetCurrentUserParams {}

/// Parameters for listing available assignees in a project
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListProjectAssigneesParams {
    /// Numeric Project ID to list eligible assignees for
    pub project_id: i64,
}

/// Parameters for adding a member to a project
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AddProjectMemberParams {
    /// Numeric Project ID to add the user to
    pub project_id: i64,
    /// Numeric User ID to add to the project
    pub user_id: i64,
    /// Optional Role ID for the membership (default: 9 for 'Member')
    pub role_id: Option<i64>,
}

/// Parameters for listing available time entry activities
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct ListTimeEntryActivitiesParams {
    /// Optional Project ID to discover activities available in that project
    pub project_id: Option<i64>,
    /// Optional Work Package ID to discover activities available for that work package
    pub work_package_id: Option<i64>,
}

/// Parameters for logging spent time on a work package
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LogTimeParams {
    /// Work package ID to log time against
    pub work_package_id: i64,
    /// Amount of time spent (e.g. '2h', '1.5h', '30m', '02:30', or 'PT2H')
    pub hours: String,
    /// Optional comment or description of the work performed
    pub comment: Option<String>,
    /// Optional date when the work was performed in YYYY-MM-DD format (defaults to current date)
    pub spent_on: Option<String>,
    /// Optional activity ID (e.g. Development, Specification). Use `list_time_entry_activities` to discover.
    pub activity_id: Option<i64>,
    /// Optional user ID who performed the work (defaults to the authenticated user)
    pub user_id: Option<i64>,
}

/// Parameters for listing sprints
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct ListSprintsParams {
    /// Optional Project ID to list sprints belonging to that project
    pub project_id: Option<i64>,
}

/// Parameters for getting sprint details
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetSprintParams {
    /// Numeric Sprint ID
    pub id: i64,
}

/// Parameters for creating a version / milestone in a project
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateVersionParams {
    /// Numeric Project ID where the version will be created
    pub project_id: i64,
    /// Name / title of the version (e.g. 'v1.0.0', 'Release 2026.1')
    pub name: String,
    /// Optional description of the version (markdown supported)
    pub description: Option<String>,
    /// Optional start date in ISO format YYYY-MM-DD
    pub start_date: Option<String>,
    /// Optional end / release date in ISO format YYYY-MM-DD
    pub end_date: Option<String>,
    /// Optional status: 'open' (default), 'locked', or 'closed'
    pub status: Option<String>,
    /// Optional sharing mode: 'none' (default), 'descendants', 'hierarchy', 'tree', or 'system'
    pub sharing: Option<String>,
}

/// Parameters for listing versions
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct ListVersionsParams {
    /// Optional Project ID to list versions belonging to that project. If omitted, lists all versions visible to the user.
    pub project_id: Option<i64>,
}

/// Parameters for getting version details
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetVersionParams {
    /// Numeric Version ID
    pub id: i64,
}
