pub mod tools;

use crate::client::OpenProjectClient;
use crate::client::filters::WorkPackageFilterBuilder;
use crate::client::models::{
    CreateTimeEntryRequest, CreateWorkPackageRequest, UpdateWorkPackageRequest,
};
use rmcp::{
    ErrorData, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ContentBlock},
    tool, tool_handler, tool_router,
};
use tools::{
    AddProjectMemberParams, CreateWorkPackageParams, GetCurrentUserParams, GetProjectParams,
    GetSprintParams, GetWorkPackageParams, ListProjectAssigneesParams, ListProjectsParams,
    ListSprintsParams, ListStatusesParams, ListTimeEntryActivitiesParams, ListTypesParams,
    LogTimeParams, SearchWorkPackagesParams, UpdateWorkPackageParams,
};

#[derive(Clone)]
pub struct OpenProjectServer {
    pub client: OpenProjectClient,
    tool_router: ToolRouter<Self>,
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for OpenProjectServer {}

#[tool_router(router = tool_router)]
impl OpenProjectServer {
    pub fn new(client: OpenProjectClient) -> Self {
        Self {
            client,
            tool_router: Self::tool_router(),
        }
    }

    /// Access the internal tool router
    pub fn router(&self) -> &ToolRouter<Self> {
        &self.tool_router
    }

    #[tool(
        name = "list_projects",
        description = "List accessible projects in OpenProject with pagination support"
    )]
    pub async fn list_projects(
        &self,
        Parameters(params): Parameters<ListProjectsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let page = params.page.unwrap_or(1);
        let page_size = params.page_size.unwrap_or(20);

        match self.client.list_projects(page, page_size).await {
            Ok(projects) => Ok(CallToolResult::structured(
                serde_json::to_value(projects).unwrap_or_default(),
            )),
            Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to list projects: {err}"
            ))])),
        }
    }

    #[tool(
        name = "get_project",
        description = "Retrieve details and description of a specific project by numeric ID or string identifier"
    )]
    pub async fn get_project(
        &self,
        Parameters(params): Parameters<GetProjectParams>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.client.get_project(&params.id_or_identifier).await {
            Ok(project) => Ok(CallToolResult::structured(
                serde_json::to_value(project).unwrap_or_default(),
            )),
            Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to get project '{}': {err}",
                params.id_or_identifier
            ))])),
        }
    }

    #[tool(
        name = "search_work_packages",
        description = "Search, filter, and list work packages (tasks, bugs, milestones) with optional text query, project ID, type, and status"
    )]
    pub async fn search_work_packages(
        &self,
        Parameters(params): Parameters<SearchWorkPackagesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let filter = WorkPackageFilterBuilder::new()
            .with_query(params.query.as_deref())
            .with_project_id(params.project_id)
            .with_type_id(params.type_id)
            .with_status(params.status.as_deref())
            .with_assignee_id(params.assignee_id)
            .with_sprint_id(params.sprint_id);

        let page = params.page.unwrap_or(1);
        let page_size = params.page_size.unwrap_or(20);

        match self
            .client
            .search_work_packages(&filter, page, page_size)
            .await
        {
            Ok(wps) => Ok(CallToolResult::structured(
                serde_json::to_value(wps).unwrap_or_default(),
            )),
            Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to search work packages: {err}"
            ))])),
        }
    }

    #[tool(
        name = "get_work_package",
        description = "Retrieve full details of a specific work package by ID (including subject, description, status, type, priority, assignee, dates, custom fields)"
    )]
    pub async fn get_work_package(
        &self,
        Parameters(params): Parameters<GetWorkPackageParams>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.client.get_work_package(params.id).await {
            Ok(wp) => Ok(CallToolResult::structured(
                serde_json::to_value(wp).unwrap_or_default(),
            )),
            Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to get work package #{}: {err}",
                params.id
            ))])),
        }
    }

    #[tool(
        name = "create_work_package",
        description = "Create a new work package in a project. Requires project_id, type_id, and subject. Use list_types to find available types."
    )]
    pub async fn create_work_package(
        &self,
        Parameters(params): Parameters<CreateWorkPackageParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let req = CreateWorkPackageRequest {
            project_id: params.project_id,
            type_id: params.type_id,
            subject: params.subject,
            description: params.description,
            status_id: params.status_id,
            priority_id: params.priority_id,
            assignee_id: params.assignee_id,
            parent_id: params.parent_id,
            sprint_id: params.sprint_id,
            start_date: params.start_date,
            due_date: params.due_date,
            estimated_time: params.estimated_time,
            remaining_time: params.remaining_time,
            percentage_done: params.percentage_done,
            custom_fields: params.custom_fields,
        };

        match self.client.create_work_package(req).await {
            Ok(wp) => Ok(CallToolResult::structured(
                serde_json::to_value(wp).unwrap_or_default(),
            )),
            Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to create work package: {err}"
            ))])),
        }
    }

    #[tool(
        name = "update_work_package",
        description = "Update an existing work package. If lock_version is omitted, it is fetched automatically."
    )]
    pub async fn update_work_package(
        &self,
        Parameters(params): Parameters<UpdateWorkPackageParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let wp_id = params.id;
        let req = UpdateWorkPackageRequest {
            id: params.id,
            lock_version: params.lock_version,
            subject: params.subject,
            description: params.description,
            type_id: params.type_id,
            status_id: params.status_id,
            priority_id: params.priority_id,
            assignee_id: params.assignee_id,
            parent_id: params.parent_id,
            sprint_id: params.sprint_id,
            start_date: params.start_date,
            due_date: params.due_date,
            estimated_time: params.estimated_time,
            remaining_time: params.remaining_time,
            percentage_done: params.percentage_done,
            custom_fields: params.custom_fields,
        };

        match self.client.update_work_package(req).await {
            Ok(wp) => Ok(CallToolResult::structured(
                serde_json::to_value(wp).unwrap_or_default(),
            )),
            Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to update work package #{wp_id}: {err}"
            ))])),
        }
    }

    #[tool(
        name = "list_types",
        description = "List all work package types available in OpenProject (e.g. Task, Bug, Milestone, Feature)"
    )]
    pub async fn list_types(
        &self,
        _params: Parameters<ListTypesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.client.list_types().await {
            Ok(types) => Ok(CallToolResult::structured(
                serde_json::to_value(types).unwrap_or_default(),
            )),
            Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to list types: {err}"
            ))])),
        }
    }

    #[tool(
        name = "list_statuses",
        description = "List all work package statuses available in OpenProject (e.g. New, In Progress, Closed)"
    )]
    pub async fn list_statuses(
        &self,
        _params: Parameters<ListStatusesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.client.list_statuses().await {
            Ok(statuses) => Ok(CallToolResult::structured(
                serde_json::to_value(statuses).unwrap_or_default(),
            )),
            Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to list statuses: {err}"
            ))])),
        }
    }

    #[tool(
        name = "get_current_user",
        description = "Retrieve information about the currently authenticated user (associated with the API key), including ID, name, email, login, and admin status"
    )]
    pub async fn get_current_user(
        &self,
        _params: Parameters<GetCurrentUserParams>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.client.get_current_user().await {
            Ok(user) => Ok(CallToolResult::structured(
                serde_json::to_value(user).unwrap_or_default(),
            )),
            Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to get current user: {err}"
            ))])),
        }
    }

    #[tool(
        name = "list_project_assignees",
        description = "List all users eligible to be assigned to work packages in a specific project"
    )]
    pub async fn list_project_assignees(
        &self,
        Parameters(params): Parameters<ListProjectAssigneesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.client.list_project_assignees(params.project_id).await {
            Ok(users) => Ok(CallToolResult::structured(
                serde_json::to_value(users).unwrap_or_default(),
            )),
            Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to list assignees for project #{}: {err}",
                params.project_id
            ))])),
        }
    }

    #[tool(
        name = "add_project_member",
        description = "Add a user as a member of a project with a role (default role_id: 9 for 'Member'), enabling them to be assigned to work packages in that project"
    )]
    pub async fn add_project_member(
        &self,
        Parameters(params): Parameters<AddProjectMemberParams>,
    ) -> Result<CallToolResult, ErrorData> {
        match self
            .client
            .add_project_member(params.project_id, params.user_id, params.role_id)
            .await
        {
            Ok(membership) => Ok(CallToolResult::structured(
                serde_json::to_value(membership).unwrap_or_default(),
            )),
            Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to add user #{} to project #{}: {err}",
                params.user_id, params.project_id
            ))])),
        }
    }

    #[tool(
        name = "list_time_entry_activities",
        description = "List available activities for logging spent time (e.g. Development, Specification, Testing) for a project or work package"
    )]
    pub async fn list_time_entry_activities(
        &self,
        Parameters(params): Parameters<ListTimeEntryActivitiesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        match self
            .client
            .list_time_entry_activities(params.project_id, params.work_package_id)
            .await
        {
            Ok(activities) => Ok(CallToolResult::structured(
                serde_json::to_value(activities).unwrap_or_default(),
            )),
            Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to list time entry activities: {err}"
            ))])),
        }
    }

    #[tool(
        name = "log_time",
        description = "Log spent time on a work package. Accepts friendly duration formats (e.g. '2h', '1.5h', '30m', '02:30', or 'PT2H'). OpenProject automatically recalculates the total spentTime."
    )]
    pub async fn log_time(
        &self,
        Parameters(params): Parameters<LogTimeParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let wp_id = params.work_package_id;
        let req = CreateTimeEntryRequest {
            work_package_id: params.work_package_id,
            hours: params.hours,
            comment: params.comment,
            spent_on: params.spent_on,
            activity_id: params.activity_id,
            user_id: params.user_id,
        };

        match self.client.log_time(req).await {
            Ok(entry) => Ok(CallToolResult::structured(
                serde_json::to_value(entry).unwrap_or_default(),
            )),
            Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to log time on work package #{wp_id}: {err}"
            ))])),
        }
    }

    #[tool(
        name = "list_sprints",
        description = "List sprints, optionally filtered by project ID. Returns sprint details including ID, name, status, dates, and defining project."
    )]
    pub async fn list_sprints(
        &self,
        Parameters(params): Parameters<ListSprintsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.client.list_sprints(params.project_id).await {
            Ok(sprints) => Ok(CallToolResult::structured(
                serde_json::to_value(sprints).unwrap_or_default(),
            )),
            Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to list sprints: {err}"
            ))])),
        }
    }

    #[tool(
        name = "get_sprint",
        description = "Retrieve details of a specific sprint by ID (including name, status, dates, and defining project)"
    )]
    pub async fn get_sprint(
        &self,
        Parameters(params): Parameters<GetSprintParams>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.client.get_sprint(params.id).await {
            Ok(sprint) => Ok(CallToolResult::structured(
                serde_json::to_value(sprint).unwrap_or_default(),
            )),
            Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to get sprint #{}: {err}",
                params.id
            ))])),
        }
    }
}
