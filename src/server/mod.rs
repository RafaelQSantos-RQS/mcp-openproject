pub mod tools;

use crate::client::OpenProjectClient;
use crate::client::filters::{TimeEntryFilterBuilder, WorkPackageFilterBuilder};
use crate::client::models::{
    CreateRelationRequest, CreateTimeEntryRequest, CreateVersionRequest, CreateWorkPackageRequest,
    UpdateProjectRequest, UpdateTimeEntryRequest, UpdateWorkPackageRequest,
};
use rmcp::{
    ErrorData, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ContentBlock},
    tool, tool_handler, tool_router,
};
use tools::{
    AddProjectMemberParams, AddWorkPackageCommentParams, CreateRelationParams, CreateVersionParams,
    CreateWorkPackageParams, DeleteRelationParams, DeleteTimeEntryParams, GetCurrentUserParams,
    GetCustomOptionParams, GetProjectParams, GetSprintParams, GetVersionParams,
    GetWorkPackageParams, GetWorkPackageSchemaParams, ListPrioritiesParams,
    ListProjectAssigneesParams, ListProjectsParams, ListRelationsParams, ListSprintsParams,
    ListStatusesParams, ListTimeEntriesParams, ListTimeEntryActivitiesParams, ListTypesParams,
    ListVersionsParams, ListWorkPackageActivitiesParams, LogTimeParams, SearchWorkPackagesParams,
    UpdateProjectParams, UpdateTimeEntryParams, UpdateWorkPackageParams,
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
        name = "update_project",
        description = "Update an existing project in OpenProject. Allows modifying description, project health status (e.g. 'on_track', 'at_risk', 'off_track', 'not_started', 'finished', 'discontinued', or 'none' to clear), status explanation, name, and visibility."
    )]
    pub async fn update_project(
        &self,
        Parameters(params): Parameters<UpdateProjectParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let id = params.id_or_identifier.clone();
        let req = UpdateProjectRequest {
            id_or_identifier: params.id_or_identifier,
            name: params.name,
            description: params.description,
            status: params.status,
            status_explanation: params.status_explanation,
            public: params.public,
        };

        match self.client.update_project(req).await {
            Ok(project) => Ok(CallToolResult::structured(
                serde_json::to_value(project).unwrap_or_default(),
            )),
            Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to update project '{id}': {err}"
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
            .with_sprint_id(params.sprint_id)
            .with_version_id(params.version_id)
            .with_due_date_range(
                params.due_date_from.as_deref(),
                params.due_date_to.as_deref(),
            )
            .with_updated_at_range(
                params.updated_after.as_deref(),
                params.updated_before.as_deref(),
            );

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
            version_id: params.version_id,
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
            version_id: params.version_id,
            start_date: params.start_date,
            due_date: params.due_date,
            estimated_time: params.estimated_time,
            remaining_time: params.remaining_time,
            percentage_done: params.percentage_done,
            comment: params.comment,
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

    #[tool(
        name = "create_version",
        description = "Create a new version / milestone in an OpenProject project. Requires project_id and name."
    )]
    pub async fn create_version(
        &self,
        Parameters(params): Parameters<CreateVersionParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let req = CreateVersionRequest {
            project_id: params.project_id,
            name: params.name,
            description: params.description,
            start_date: params.start_date,
            end_date: params.end_date,
            status: params.status,
            sharing: params.sharing,
        };

        match self.client.create_version(req).await {
            Ok(version) => Ok(CallToolResult::structured(
                serde_json::to_value(version).unwrap_or_default(),
            )),
            Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to create version: {err}"
            ))])),
        }
    }

    #[tool(
        name = "list_versions",
        description = "List versions / milestones, optionally filtered by project ID. If project_id is omitted, lists all versions visible to the user."
    )]
    pub async fn list_versions(
        &self,
        Parameters(params): Parameters<ListVersionsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.client.list_versions(params.project_id).await {
            Ok(versions) => Ok(CallToolResult::structured(
                serde_json::to_value(versions).unwrap_or_default(),
            )),
            Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to list versions: {err}"
            ))])),
        }
    }

    #[tool(
        name = "get_version",
        description = "Retrieve details of a specific version / milestone by ID (including name, status, dates, sharing, and defining project)"
    )]
    pub async fn get_version(
        &self,
        Parameters(params): Parameters<GetVersionParams>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.client.get_version(params.id).await {
            Ok(version) => Ok(CallToolResult::structured(
                serde_json::to_value(version).unwrap_or_default(),
            )),
            Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to get version #{}: {err}",
                params.id
            ))])),
        }
    }

    #[tool(
        name = "get_work_package_schema",
        description = "Inspect active custom fields (e.g. customField11), their types, required flags, location (_links vs root), and allowed CustomOption values (id, value, href) for a project and work package type"
    )]
    pub async fn get_work_package_schema(
        &self,
        Parameters(params): Parameters<GetWorkPackageSchemaParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let include_standard = params.include_standard_fields.unwrap_or(false);
        match self
            .client
            .get_work_package_schema(params.project_id, params.type_id, include_standard)
            .await
        {
            Ok(schema) => Ok(CallToolResult::structured(
                serde_json::to_value(schema).unwrap_or_default(),
            )),
            Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to inspect work package schema for project #{}: {err}",
                params.project_id
            ))])),
        }
    }

    #[tool(
        name = "get_custom_option",
        description = "Retrieve details of a specific CustomOption by numeric ID (returns id, value, and href)"
    )]
    pub async fn get_custom_option(
        &self,
        Parameters(params): Parameters<GetCustomOptionParams>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.client.get_custom_option(params.id).await {
            Ok(option) => Ok(CallToolResult::structured(
                serde_json::to_value(option).unwrap_or_default(),
            )),
            Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to get custom option #{}: {err}",
                params.id
            ))])),
        }
    }

    #[tool(
        name = "list_priorities",
        description = "List all work package priorities available in OpenProject (e.g. Low, Normal, High, Immediate) with their IDs"
    )]
    pub async fn list_priorities(
        &self,
        _params: Parameters<ListPrioritiesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.client.list_priorities().await {
            Ok(priorities) => Ok(CallToolResult::structured(
                serde_json::to_value(priorities).unwrap_or_default(),
            )),
            Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to list priorities: {err}"
            ))])),
        }
    }

    #[tool(
        name = "add_work_package_comment",
        description = "Add a comment / activity note to a work package without modifying its description or other fields"
    )]
    pub async fn add_work_package_comment(
        &self,
        Parameters(params): Parameters<AddWorkPackageCommentParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let wp_id = params.work_package_id;
        let internal = params.internal.unwrap_or(false);
        match self
            .client
            .add_work_package_comment(wp_id, &params.comment, internal)
            .await
        {
            Ok(activity) => Ok(CallToolResult::structured(
                serde_json::to_value(activity).unwrap_or_default(),
            )),
            Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to add comment to work package #{wp_id}: {err}"
            ))])),
        }
    }

    #[tool(
        name = "list_work_package_activities",
        description = "List the activity history (comments and field change details) of a work package"
    )]
    pub async fn list_work_package_activities(
        &self,
        Parameters(params): Parameters<ListWorkPackageActivitiesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let wp_id = params.work_package_id;
        match self.client.list_work_package_activities(wp_id).await {
            Ok(activities) => Ok(CallToolResult::structured(
                serde_json::to_value(activities).unwrap_or_default(),
            )),
            Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to list activities for work package #{wp_id}: {err}"
            ))])),
        }
    }

    #[tool(
        name = "list_time_entries",
        description = "List and filter logged time entries by work package, project, user, or spent_on date range with pagination"
    )]
    pub async fn list_time_entries(
        &self,
        Parameters(params): Parameters<ListTimeEntriesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let filter = TimeEntryFilterBuilder::new()
            .with_work_package_id(params.work_package_id)
            .with_project_id(params.project_id)
            .with_user_id(params.user_id)
            .with_spent_on_range(params.spent_from.as_deref(), params.spent_to.as_deref());

        let page = params.page.unwrap_or(1);
        let page_size = params.page_size.unwrap_or(20);

        match self
            .client
            .list_time_entries(&filter, page, page_size)
            .await
        {
            Ok(entries) => Ok(CallToolResult::structured(
                serde_json::to_value(entries).unwrap_or_default(),
            )),
            Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to list time entries: {err}"
            ))])),
        }
    }

    #[tool(
        name = "list_relations",
        description = "List dependencies and relations (blocks, blocked, precedes, follows, relates, duplicates, etc.) for a work package"
    )]
    pub async fn list_relations(
        &self,
        Parameters(params): Parameters<ListRelationsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let wp_id = params.work_package_id;
        match self.client.list_relations(wp_id).await {
            Ok(relations) => Ok(CallToolResult::structured(
                serde_json::to_value(relations).unwrap_or_default(),
            )),
            Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to list relations for work package #{wp_id}: {err}"
            ))])),
        }
    }

    #[tool(
        name = "create_relation",
        description = "Create a dependency / relation between two work packages (e.g. blocks, blocked_by, precedes, follows, relates, duplicates)"
    )]
    pub async fn create_relation(
        &self,
        Parameters(params): Parameters<CreateRelationParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let req = CreateRelationRequest {
            from_id: params.from_id,
            to_id: params.to_id,
            relation_type: params.relation_type,
            description: params.description,
            lag: params.lag,
        };

        match self.client.create_relation(req).await {
            Ok(relation) => Ok(CallToolResult::structured(
                serde_json::to_value(relation).unwrap_or_default(),
            )),
            Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to create relation: {err}"
            ))])),
        }
    }

    #[tool(
        name = "delete_relation",
        description = "Delete a work package relation / dependency by its relation ID"
    )]
    pub async fn delete_relation(
        &self,
        Parameters(params): Parameters<DeleteRelationParams>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.client.delete_relation(params.id).await {
            Ok(()) => Ok(CallToolResult::structured(serde_json::json!({
                "deleted": true,
                "id": params.id
            }))),
            Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to delete relation #{}: {err}",
                params.id
            ))])),
        }
    }

    #[tool(
        name = "update_time_entry",
        description = "Update an existing time entry (hours, comment, date, activity, work package or user) by its ID"
    )]
    pub async fn update_time_entry(
        &self,
        Parameters(params): Parameters<UpdateTimeEntryParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let req = UpdateTimeEntryRequest {
            id: params.id,
            hours: params.hours,
            comment: params.comment,
            spent_on: params.spent_on,
            activity_id: params.activity_id,
            work_package_id: params.work_package_id,
            user_id: params.user_id,
        };
        match self.client.update_time_entry(req).await {
            Ok(entry) => Ok(CallToolResult::structured(
                serde_json::to_value(entry).unwrap(),
            )),
            Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to update time entry #{}: {err}",
                params.id
            ))])),
        }
    }

    #[tool(
        name = "delete_time_entry",
        description = "Delete a time entry by its ID"
    )]
    pub async fn delete_time_entry(
        &self,
        Parameters(params): Parameters<DeleteTimeEntryParams>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.client.delete_time_entry(params.id).await {
            Ok(()) => Ok(CallToolResult::structured(serde_json::json!({
                "deleted": true,
                "id": params.id
            }))),
            Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Failed to delete time entry #{}: {err}",
                params.id
            ))])),
        }
    }
}
