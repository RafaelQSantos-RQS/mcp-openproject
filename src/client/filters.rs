use serde::Serialize;
use serde_json::json;

/// Filter condition for OpenProject API v3 queries
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FilterCondition {
    pub operator: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct WorkPackageFilterBuilder {
    filters: Vec<serde_json::Value>,
}

impl WorkPackageFilterBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Search for text in subject, ID, or description
    pub fn with_query(mut self, query: Option<&str>) -> Self {
        if let Some(q) = query {
            let trimmed = q.trim();
            if !trimmed.is_empty() {
                self.filters.push(json!({
                    "subjectOrId": {
                        "operator": "**",
                        "values": [trimmed]
                    }
                }));
            }
        }
        self
    }

    /// Filter by Project ID
    pub fn with_project_id(mut self, project_id: Option<i64>) -> Self {
        if let Some(id) = project_id {
            self.filters.push(json!({
                "project": {
                    "operator": "=",
                    "values": [id.to_string()]
                }
            }));
        }
        self
    }

    /// Filter by Type ID
    pub fn with_type_id(mut self, type_id: Option<i64>) -> Self {
        if let Some(id) = type_id {
            self.filters.push(json!({
                "type": {
                    "operator": "=",
                    "values": [id.to_string()]
                }
            }));
        }
        self
    }

    /// Filter by Status ID or open/closed status string ("open", "closed", or specific ID)
    pub fn with_status(mut self, status: Option<&str>) -> Self {
        if let Some(s) = status {
            let trimmed = s.trim().to_lowercase();
            if trimmed == "open" || trimmed == "o" {
                self.filters.push(json!({
                    "status": {
                        "operator": "o",
                        "values": []
                    }
                }));
            } else if trimmed == "closed" || trimmed == "c" {
                self.filters.push(json!({
                    "status": {
                        "operator": "c",
                        "values": []
                    }
                }));
            } else if !trimmed.is_empty() {
                // Specific status ID or name
                self.filters.push(json!({
                    "status": {
                        "operator": "=",
                        "values": [s.trim()]
                    }
                }));
            }
        }
        self
    }

    /// Filter by Assignee ID
    pub fn with_assignee_id(mut self, assignee_id: Option<i64>) -> Self {
        if let Some(id) = assignee_id {
            self.filters.push(json!({
                "assignee": {
                    "operator": "=",
                    "values": [id.to_string()]
                }
            }));
        }
        self
    }

    /// Filter by Sprint ID
    pub fn with_sprint_id(mut self, sprint_id: Option<i64>) -> Self {
        if let Some(id) = sprint_id {
            self.filters.push(json!({
                "sprint": {
                    "operator": "=",
                    "values": [id.to_string()]
                }
            }));
        }
        self
    }

    /// Filter by Version ID
    pub fn with_version_id(mut self, version_id: Option<i64>) -> Self {
        if let Some(id) = version_id {
            self.filters.push(json!({
                "version": {
                    "operator": "=",
                    "values": [id.to_string()]
                }
            }));
        }
        self
    }

    /// Filter by due date range (ISO YYYY-MM-DD)
    pub fn with_due_date_range(mut self, from: Option<&str>, to: Option<&str>) -> Self {
        push_date_range_filter(&mut self.filters, "dueDate", from, to);
        self
    }

    /// Filter by updated_at date range (ISO YYYY-MM-DD or YYYY-MM-DDTHH:MM:SSZ)
    pub fn with_updated_at_range(mut self, after: Option<&str>, before: Option<&str>) -> Self {
        push_date_range_filter(&mut self.filters, "updatedAt", after, before);
        self
    }

    /// Checks if any filters are set
    pub fn is_empty(&self) -> bool {
        self.filters.is_empty()
    }

    /// Serializes filters to the JSON string required by OpenProject API v3
    pub fn to_query_string(&self) -> Option<String> {
        if self.filters.is_empty() {
            None
        } else {
            serde_json::to_string(&self.filters).ok()
        }
    }
}

/// Shared helper to append OpenProject date/datetime range filters (`<>d`, supporting open bounds via `""`)
fn push_date_range_filter(
    filters: &mut Vec<serde_json::Value>,
    field: &str,
    from: Option<&str>,
    to: Option<&str>,
) {
    let from = from.map(str::trim).filter(|s| !s.is_empty());
    let to = to.map(str::trim).filter(|s| !s.is_empty());

    if from.is_some() || to.is_some() {
        filters.push(json!({
            field: {
                "operator": "<>d",
                "values": [from.unwrap_or(""), to.unwrap_or("")]
            }
        }));
    }
}

#[derive(Debug, Clone, Default)]
pub struct TimeEntryFilterBuilder {
    filters: Vec<serde_json::Value>,
}

impl TimeEntryFilterBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by Work Package ID (OpenProject time_entries API uses the `entity` filter key)
    pub fn with_work_package_id(mut self, work_package_id: Option<i64>) -> Self {
        if let Some(id) = work_package_id {
            self.filters.push(json!({
                "entity": {
                    "operator": "=",
                    "values": [id.to_string()]
                }
            }));
        }
        self
    }

    /// Filter by Project ID
    pub fn with_project_id(mut self, project_id: Option<i64>) -> Self {
        if let Some(id) = project_id {
            self.filters.push(json!({
                "project": {
                    "operator": "=",
                    "values": [id.to_string()]
                }
            }));
        }
        self
    }

    /// Filter by User ID
    pub fn with_user_id(mut self, user_id: Option<i64>) -> Self {
        if let Some(id) = user_id {
            self.filters.push(json!({
                "user": {
                    "operator": "=",
                    "values": [id.to_string()]
                }
            }));
        }
        self
    }

    /// Filter by spentOn date range (ISO YYYY-MM-DD)
    pub fn with_spent_on_range(mut self, from: Option<&str>, to: Option<&str>) -> Self {
        push_date_range_filter(&mut self.filters, "spentOn", from, to);
        self
    }

    /// Serializes filters to the JSON string required by OpenProject API v3
    pub fn to_query_string(&self) -> Option<String> {
        if self.filters.is_empty() {
            None
        } else {
            serde_json::to_string(&self.filters).ok()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_builder() {
        let builder = WorkPackageFilterBuilder::new();
        assert!(builder.is_empty());
        assert_eq!(builder.to_query_string(), None);
    }

    #[test]
    fn test_query_and_project() {
        let builder = WorkPackageFilterBuilder::new()
            .with_query(Some("fix bug"))
            .with_project_id(Some(42));

        let query = builder.to_query_string().unwrap();
        assert!(query.contains("\"subjectOrId\":{\"operator\":\"**\",\"values\":[\"fix bug\"]}"));
        assert!(query.contains("\"project\":{\"operator\":\"=\",\"values\":[\"42\"]}"));
    }

    #[test]
    fn test_open_status() {
        let builder = WorkPackageFilterBuilder::new().with_status(Some("open"));
        let query = builder.to_query_string().unwrap();
        assert!(query.contains("\"status\":{\"operator\":\"o\",\"values\":[]}"));
    }

    #[test]
    fn test_sprint_filter() {
        let builder = WorkPackageFilterBuilder::new().with_sprint_id(Some(75));
        let query = builder.to_query_string().unwrap();
        assert!(query.contains("\"sprint\":{\"operator\":\"=\",\"values\":[\"75\"]}"));
    }

    #[test]
    fn test_version_filter() {
        let builder = WorkPackageFilterBuilder::new().with_version_id(Some(111));
        let query = builder.to_query_string().unwrap();
        assert!(query.contains("\"version\":{\"operator\":\"=\",\"values\":[\"111\"]}"));
    }

    #[test]
    fn test_date_range_filters() {
        let builder = WorkPackageFilterBuilder::new()
            .with_due_date_range(Some("2026-11-01"), Some("2026-11-30"))
            .with_updated_at_range(Some("2026-11-20"), None);
        let query = builder.to_query_string().unwrap();
        assert!(query.contains(
            "\"dueDate\":{\"operator\":\"<>d\",\"values\":[\"2026-11-01\",\"2026-11-30\"]}"
        ));
        assert!(
            query.contains("\"updatedAt\":{\"operator\":\"<>d\",\"values\":[\"2026-11-20\",\"\"]}")
        );
    }

    #[test]
    fn test_time_entry_filter_builder() {
        let builder = TimeEntryFilterBuilder::new()
            .with_work_package_id(Some(5454))
            .with_user_id(Some(8))
            .with_spent_on_range(None, Some("2026-11-25"));
        let query = builder.to_query_string().unwrap();
        assert!(query.contains("\"entity\":{\"operator\":\"=\",\"values\":[\"5454\"]}"));
        assert!(query.contains("\"user\":{\"operator\":\"=\",\"values\":[\"8\"]}"));
        assert!(
            query.contains("\"spentOn\":{\"operator\":\"<>d\",\"values\":[\"\",\"2026-11-25\"]}")
        );
    }
}
