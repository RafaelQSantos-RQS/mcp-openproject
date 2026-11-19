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
}
