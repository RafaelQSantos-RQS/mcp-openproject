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

/// Parameters for updating an existing project
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateProjectParams {
    /// Numeric project ID or string identifier (e.g. "117" or "my-project")
    pub id_or_identifier: String,
    /// Optional new project description (markdown supported)
    pub description: Option<String>,
    /// Optional project health status: 'on_track', 'at_risk', 'off_track', 'not_started', 'finished', 'discontinued' (or 'none' to clear)
    pub status: Option<String>,
    /// Optional status explanation / health details (markdown supported)
    pub status_explanation: Option<String>,
    /// Optional new project name
    pub name: Option<String>,
    /// Optional new project identifier (unique slug: lowercase letters, numbers and hyphens)
    pub identifier: Option<String>,
    /// Optional public/private visibility flag
    pub public: Option<bool>,
    /// Optional active flag: false archives the project, true unarchives it
    pub active: Option<bool>,
    /// Optional parent project ID (0 or negative removes the parent, making it a top-level project)
    pub parent_id: Option<i64>,
    /// Custom fields key-value map. Scalar/text fields go at root; HAL link fields (e.g. {"customField10": {"href": "/api/v3/custom_options/25"}}) are automatically placed under _links. Use `get_project_schema` to discover keys and allowed options.
    pub custom_fields: Option<HashMap<String, serde_json::Value>>,
}

/// Parameters for listing types available in a project
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListProjectTypesParams {
    /// Numeric Project ID
    pub project_id: i64,
}

/// Parameters for creating a new project
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateProjectParams {
    /// Project name (1-255 characters)
    pub name: String,
    /// Unique project identifier (1-100 characters; lowercase letters, numbers and hyphens, e.g. "my-project")
    pub identifier: String,
    /// Optional project description (markdown supported)
    pub description: Option<String>,
    /// Optional public/private visibility flag (default: false, private)
    pub public: Option<bool>,
    /// Optional parent project ID (creates this project as a subproject)
    pub parent_id: Option<i64>,
    /// Custom fields key-value map. Scalar/text fields go at root; HAL link fields (e.g. {"customField31": {"href": "/api/v3/custom_options/25"}}) are automatically placed under _links. Use `get_project_schema` to discover keys, required flags and allowed options.
    pub custom_fields: Option<HashMap<String, serde_json::Value>>,
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
    /// Filter by due date >= YYYY-MM-DD
    pub due_date_from: Option<String>,
    /// Filter by due date <= YYYY-MM-DD
    pub due_date_to: Option<String>,
    /// Filter by updated_at >= YYYY-MM-DD
    pub updated_after: Option<String>,
    /// Filter by updated_at <= YYYY-MM-DD
    pub updated_before: Option<String>,
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
    /// Custom fields key-value map. Scalar/text fields (e.g. {"customField12": true}) go at root; HAL link fields (e.g. {"customField11": [{"href": "/api/v3/custom_options/25"}]}) are automatically placed under _links. Use `get_work_package_schema` to discover keys and allowed options.
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
    /// Optional comment to record in the work package activity history alongside the update (markdown supported)
    pub comment: Option<String>,
    /// Custom fields key-value map. Scalar/text fields go at root; HAL link fields (e.g. {"customField11": [{"href": "/api/v3/custom_options/25"}]}) are automatically placed under _links.
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

/// Parameters for inspecting active work package schema and custom field options for a project/type
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetWorkPackageSchemaParams {
    /// Numeric Project ID to inspect active custom fields and allowed options for
    pub project_id: i64,
    /// Optional Work Package Type ID (e.g. 1 for Task). If omitted, OpenProject uses the project's default type.
    pub type_id: Option<i64>,
    /// Whether to also include standard OpenProject fields (subject, status, priority, etc.) in the response (default: false)
    pub include_standard_fields: Option<bool>,
}

/// Parameters for inspecting the project schema and custom field options
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct GetProjectSchemaParams {
    /// Whether to also include standard OpenProject fields (name, identifier, public, etc.) in the response (default: false)
    pub include_standard_fields: Option<bool>,
}

/// Parameters for retrieving a single custom option by ID
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetCustomOptionParams {
    /// Numeric Custom Option ID (e.g. 25 from /api/v3/custom_options/25)
    pub id: i64,
}

/// Parameters for listing work package priorities (empty)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct ListPrioritiesParams {}

/// Parameters for adding a comment to a work package
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AddWorkPackageCommentParams {
    /// Work package ID to comment on
    pub work_package_id: i64,
    /// Comment text (markdown supported)
    pub comment: String,
    /// Whether this is an internal comment (default: false)
    pub internal: Option<bool>,
}

/// Parameters for listing activities and comments of a work package
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListWorkPackageActivitiesParams {
    /// Work package ID
    pub work_package_id: i64,
}

/// Parameters for listing time entries
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct ListTimeEntriesParams {
    /// Filter by Work Package ID
    pub work_package_id: Option<i64>,
    /// Filter by Project ID
    pub project_id: Option<i64>,
    /// Filter by User ID
    pub user_id: Option<i64>,
    /// Filter by spent_on >= YYYY-MM-DD
    pub spent_from: Option<String>,
    /// Filter by spent_on <= YYYY-MM-DD
    pub spent_to: Option<String>,
    /// Page number (1-based, default: 1)
    pub page: Option<usize>,
    /// Number of items per page (default: 20, max: 100)
    pub page_size: Option<usize>,
}

/// Parameters for listing relations/dependencies of a work package
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListRelationsParams {
    /// Work package ID
    pub work_package_id: i64,
}

/// Parameters for creating a relation/dependency between two work packages
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateRelationParams {
    /// Source work package ID
    pub from_id: i64,
    /// Target work package ID
    pub to_id: i64,
    /// Relation type: 'relates', 'blocks', 'blocked' (or 'blocked_by'), 'precedes', 'follows', 'duplicates', 'duplicated', 'includes', 'partof', 'requires', 'required'
    pub relation_type: String,
    /// Optional description of the relation
    pub description: Option<String>,
    /// Optional lag in working days (for precedes/follows)
    pub lag: Option<i32>,
}

/// Parameters for deleting a work package relation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DeleteRelationParams {
    /// Numeric Relation ID
    pub id: i64,
}

/// Parameters for updating an existing time entry
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateTimeEntryParams {
    /// ID of the time entry to update
    pub id: i64,
    /// New duration, e.g. "2h", "1h30m", "PT2H"
    pub hours: Option<String>,
    /// New comment text
    pub comment: Option<String>,
    /// New date spent on (YYYY-MM-DD)
    pub spent_on: Option<String>,
    /// New activity ID
    pub activity_id: Option<i64>,
    /// New work package ID
    pub work_package_id: Option<i64>,
    /// New user ID
    pub user_id: Option<i64>,
}

/// Parameters for deleting a time entry
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DeleteTimeEntryParams {
    /// Numeric Time Entry ID
    pub id: i64,
}
