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
    /// Start date in ISO format YYYY-MM-DD
    pub start_date: Option<String>,
    /// Due date in ISO format YYYY-MM-DD
    pub due_date: Option<String>,
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
    /// Start date in ISO format YYYY-MM-DD
    pub start_date: Option<String>,
    /// Due date in ISO format YYYY-MM-DD
    pub due_date: Option<String>,
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
