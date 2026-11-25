use mcp_openproject::client::OpenProjectClient;
use mcp_openproject::client::error::OpenProjectError;
use mcp_openproject::client::filters::TimeEntryFilterBuilder;
use mcp_openproject::client::models::{
    CreateRelationRequest, CreateVersionRequest, CreateWorkPackageRequest, UpdateProjectRequest,
    UpdateWorkPackageRequest,
};
use mcp_openproject::config::Config;
use serde_json::json;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_mock_list_projects() {
    let mock_server = MockServer::start().await;

    let response_body = json!({
        "_type": "Collection",
        "total": 1,
        "count": 1,
        "pageSize": 20,
        "offset": 1,
        "_embedded": {
            "elements": [
                {
                    "id": 10,
                    "identifier": "demo-proj",
                    "name": "Demo Project",
                    "active": true,
                    "public": true,
                    "description": {
                        "raw": "A sample project"
                    }
                }
            ]
        }
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/projects"))
        .and(query_param("offset", "1"))
        .and(query_param("pageSize", "20"))
        .and(header("authorization", "Basic YXBpa2V5OnRlc3QtdG9rZW4="))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), "test-token".into()).unwrap();
    let client = OpenProjectClient::new(config).unwrap();

    let res = client.list_projects(1, 20).await.unwrap();
    assert_eq!(res.total, 1);
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].id, 10);
    assert_eq!(res.items[0].name, "Demo Project");
    assert_eq!(res.items[0].description, Some("A sample project".into()));
}

#[tokio::test]
async fn test_mock_get_work_package() {
    let mock_server = MockServer::start().await;

    let response_body = json!({
        "id": 100,
        "lockVersion": 3,
        "subject": "Fix login bug",
        "description": {
            "raw": "Users cannot log in via SSO"
        },
        "startDate": "2026-11-01",
        "dueDate": "2026-11-20",
        "_links": {
            "project": { "href": "/api/v3/projects/10", "title": "Demo Project" },
            "type": { "href": "/api/v3/types/1", "title": "Bug" },
            "status": { "href": "/api/v3/statuses/2", "title": "In Progress" },
            "priority": { "href": "/api/v3/priorities/7", "title": "High" },
            "assignee": { "href": "/api/v3/users/42", "title": "Jane Doe" }
        },
        "customField1": "Critical System"
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/work_packages/100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), "test-token".into()).unwrap();
    let client = OpenProjectClient::new(config).unwrap();

    let wp = client.get_work_package(100).await.unwrap();
    assert_eq!(wp.id, 100);
    assert_eq!(wp.lock_version, 3);
    assert_eq!(wp.subject, "Fix login bug");
    assert_eq!(wp.type_name, Some("Bug".into()));
    assert_eq!(wp.project_id, Some(10));
    assert_eq!(wp.status_name, Some("In Progress".into()));
    assert_eq!(
        wp.custom_fields.get("customField1"),
        Some(&json!("Critical System"))
    );
}

#[tokio::test]
async fn test_mock_update_work_package_auto_fetch_lock_version() {
    let mock_server = MockServer::start().await;

    // 1. GET work package returns lockVersion = 5
    let get_wp_body = json!({
        "id": 100,
        "lockVersion": 5,
        "subject": "Original Subject",
        "_links": {
            "type": { "href": "/api/v3/types/1", "title": "Bug" }
        }
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/work_packages/100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(get_wp_body))
        .mount(&mock_server)
        .await;

    // 2. PATCH work package expects lockVersion = 5 in payload
    let patch_wp_body = json!({
        "id": 100,
        "lockVersion": 6,
        "subject": "Updated Subject",
        "_links": {
            "type": { "href": "/api/v3/types/1", "title": "Bug" }
        }
    });

    Mock::given(method("PATCH"))
        .and(path("/api/v3/work_packages/100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(patch_wp_body))
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), "test-token".into()).unwrap();
    let client = OpenProjectClient::new(config).unwrap();

    // Call update without lock_version (auto-fetch)
    let updated = client
        .update_work_package(UpdateWorkPackageRequest {
            id: 100,
            subject: Some("Updated Subject".into()),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(updated.id, 100);
    assert_eq!(updated.lock_version, 6);
    assert_eq!(updated.subject, "Updated Subject");
}

#[tokio::test]
async fn test_mock_api_error_handling() {
    let mock_server = MockServer::start().await;

    let error_body = json!({
        "_type": "Error",
        "errorIdentifier": "urn:openproject-org:api:v3:errors:PropertyConstraintViolation",
        "message": "Subject can't be blank.",
        "_embedded": {
            "errors": [
                {
                    "message": "Subject is required",
                    "errorIdentifier": "urn:openproject-org:api:v3:errors:PropertyConstraintViolation"
                }
            ]
        }
    });

    Mock::given(method("POST"))
        .and(path("/api/v3/projects/1/work_packages"))
        .respond_with(ResponseTemplate::new(422).set_body_json(error_body))
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), "test-token".into()).unwrap();
    let client = OpenProjectClient::new(config).unwrap();

    let err = client
        .create_work_package(CreateWorkPackageRequest {
            project_id: 1,
            type_id: 1,
            subject: "".into(),
            ..Default::default()
        })
        .await
        .unwrap_err();

    assert!(matches!(err, OpenProjectError::Validation(_)));
    assert!(err.to_string().contains("Subject can't be blank."));
}

#[tokio::test]
async fn test_mock_get_work_package_with_array_links_and_formattable_custom_fields() {
    let mock_server = MockServer::start().await;

    let response_body = json!({
        "id": 42,
        "lockVersion": 1,
        "subject": "Task with complex links",
        "description": "Simple string description",
        "_links": {
            "project": { "href": "/api/v3/projects/9", "title": "Main Project" },
            "targetVersions": [
                { "href": "/api/v3/versions/5", "title": "v1.0" }
            ],
            "ancestors": [],
            "customField11": []
        },
        "customField4": {
            "format": "markdown",
            "raw": "Formatted custom text",
            "html": "<p>Formatted custom text</p>"
        }
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/work_packages/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), "token".into()).unwrap();
    let client = OpenProjectClient::new(config).unwrap();

    let wp = client.get_work_package(42).await.unwrap();
    assert_eq!(wp.id, 42);
    assert_eq!(wp.project_id, Some(9));
    assert_eq!(wp.project_name, Some("Main Project".into()));
    assert_eq!(wp.description, Some("Simple string description".into()));
    assert_eq!(
        wp.custom_fields.get("customField4"),
        Some(&json!("Formatted custom text"))
    );
}

#[test]
fn test_real_sample_wps_deserialize() {
    if let Ok(raw) = std::fs::read_to_string("target/sample_wps.json") {
        let col: mcp_openproject::client::models::HalCollection<
            mcp_openproject::client::models::HalWorkPackage,
        > = serde_json::from_str(&raw).expect("failed to deserialize real sample wps");
        assert!(!col.embedded.elements.is_empty());
    }
}

#[tokio::test]
async fn test_mock_create_work_package_with_parent() {
    let mock_server = MockServer::start().await;

    let response_body = json!({
        "id": 101,
        "lockVersion": 1,
        "subject": "Child subtask",
        "_links": {
            "project": { "href": "/api/v3/projects/1", "title": "Project 1" },
            "type": { "href": "/api/v3/types/1", "title": "Task" },
            "parent": { "href": "/api/v3/work_packages/50", "title": "Parent Task" }
        }
    });

    Mock::given(method("POST"))
        .and(path("/api/v3/projects/1/work_packages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), "token".into()).unwrap();
    let client = OpenProjectClient::new(config).unwrap();

    let created = client
        .create_work_package(CreateWorkPackageRequest {
            project_id: 1,
            type_id: 1,
            subject: "Child subtask".into(),
            parent_id: Some(50),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(created.id, 101);
    assert_eq!(created.parent_id, Some(50));
    assert_eq!(created.subject, "Child subtask");
}

#[tokio::test]
async fn test_mock_get_current_user() {
    let mock_server = MockServer::start().await;

    let response_body = json!({
        "_type": "User",
        "id": 8,
        "name": "Rafael Queiroz Santos",
        "login": "rafael_qsantos",
        "admin": true,
        "email": "rafael.santos@fieb.org.br",
        "status": "active"
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/users/me"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), "token".into()).unwrap();
    let client = OpenProjectClient::new(config).unwrap();

    let user = client.get_current_user().await.unwrap();
    assert_eq!(user.id, 8);
    assert_eq!(user.name, "Rafael Queiroz Santos");
    assert_eq!(user.login, Some("rafael_qsantos".into()));
    assert_eq!(user.email, Some("rafael.santos@fieb.org.br".into()));
    assert!(user.admin);
    assert_eq!(user.status, Some("active".into()));
}

#[tokio::test]
async fn test_mock_list_project_assignees() {
    let mock_server = MockServer::start().await;

    let response_body = json!({
        "_type": "Collection",
        "total": 2,
        "count": 2,
        "_embedded": {
            "elements": [
                {
                    "id": 8,
                    "name": "Rafael Queiroz Santos",
                    "email": "rafael.santos@fieb.org.br",
                    "login": "rafael_qsantos",
                    "admin": true,
                    "status": "active"
                },
                {
                    "id": 15,
                    "name": "Alice Smith",
                    "email": "alice@example.com",
                    "login": "asmith",
                    "admin": false,
                    "status": "active"
                }
            ]
        }
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/projects/10/available_assignees"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), "token".into()).unwrap();
    let client = OpenProjectClient::new(config).unwrap();

    let assignees = client.list_project_assignees(10).await.unwrap();
    assert_eq!(assignees.len(), 2);
    assert_eq!(assignees[0].id, 8);
    assert_eq!(assignees[0].name, "Rafael Queiroz Santos");
    assert_eq!(assignees[1].id, 15);
    assert_eq!(assignees[1].name, "Alice Smith");
}

#[tokio::test]
async fn test_mock_add_project_member() {
    let mock_server = MockServer::start().await;

    let response_body = json!({
        "_type": "Membership",
        "id": 45,
        "_links": {
            "project": { "href": "/api/v3/projects/10", "title": "Demo Project" },
            "principal": { "href": "/api/v3/users/8", "title": "Rafael Queiroz Santos" },
            "roles": [
                { "href": "/api/v3/roles/9", "title": "Member" }
            ]
        }
    });

    Mock::given(method("POST"))
        .and(path("/api/v3/memberships"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), "token".into()).unwrap();
    let client = OpenProjectClient::new(config).unwrap();

    let membership = client.add_project_member(10, 8, Some(9)).await.unwrap();
    assert_eq!(membership.id, 45);
    assert_eq!(membership.project_id, Some(10));
    assert_eq!(membership.project_name, Some("Demo Project".into()));
    assert_eq!(membership.user_id, Some(8));
    assert_eq!(membership.user_name, Some("Rafael Queiroz Santos".into()));
    assert_eq!(membership.roles, vec!["Member".to_string()]);
}

#[tokio::test]
async fn test_mock_create_work_package_with_time_and_percentage() {
    let mock_server = MockServer::start().await;

    let response_body = json!({
        "id": 102,
        "lockVersion": 1,
        "subject": "Task with time and percentage",
        "estimatedTime": "PT4H",
        "remainingTime": "PT2H30M",
        "spentTime": "PT1H",
        "percentageDone": 25,
        "_links": {
            "project": { "href": "/api/v3/projects/1", "title": "Project 1" },
            "type": { "href": "/api/v3/types/1", "title": "Task" }
        }
    });

    Mock::given(method("POST"))
        .and(path("/api/v3/projects/1/work_packages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), "token".into()).unwrap();
    let client = OpenProjectClient::new(config).unwrap();

    let created = client
        .create_work_package(CreateWorkPackageRequest {
            project_id: 1,
            type_id: 1,
            subject: "Task with time and percentage".into(),
            estimated_time: Some("4h".into()),
            remaining_time: Some("2.5h".into()),
            percentage_done: Some(25),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(created.id, 102);
    assert_eq!(created.estimated_time, Some("PT4H".into()));
    assert_eq!(created.remaining_time, Some("PT2H30M".into()));
    assert_eq!(created.spent_time, Some("PT1H".into()));
    assert_eq!(created.percentage_done, Some(25));
}

#[tokio::test]
async fn test_mock_list_time_entry_activities() {
    let mock_server = MockServer::start().await;

    let form_body = json!({
        "_type": "TimeEntryForm",
        "_embedded": {
            "schema": {
                "activity": {
                    "_embedded": {
                        "allowedValues": [
                            {
                                "id": 3,
                                "name": "Development",
                                "default": true
                            },
                            {
                                "id": 4,
                                "name": "Testing",
                                "default": false
                            }
                        ]
                    }
                }
            }
        }
    });

    Mock::given(method("POST"))
        .and(path("/api/v3/time_entries/form"))
        .respond_with(ResponseTemplate::new(200).set_body_json(form_body))
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), "token".into()).unwrap();
    let client = OpenProjectClient::new(config).unwrap();

    let activities = client
        .list_time_entry_activities(Some(10), None)
        .await
        .unwrap();
    assert_eq!(activities.len(), 2);
    assert_eq!(activities[0].id, 3);
    assert_eq!(activities[0].name, "Development");
    assert!(activities[0].is_default);
    assert_eq!(activities[1].id, 4);
    assert_eq!(activities[1].name, "Testing");
    assert!(!activities[1].is_default);
}

#[tokio::test]
async fn test_mock_log_time() {
    let mock_server = MockServer::start().await;

    let response_body = json!({
        "_type": "TimeEntry",
        "id": 500,
        "hours": "PT2H",
        "spentOn": "2026-11-17",
        "comment": {
            "raw": "Implemented feature"
        },
        "_links": {
            "workPackage": { "href": "/api/v3/work_packages/102", "title": "My Task" },
            "activity": { "href": "/api/v3/time_entries/activities/3", "title": "Development" }
        }
    });

    Mock::given(method("POST"))
        .and(path("/api/v3/time_entries"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), "token".into()).unwrap();
    let client = OpenProjectClient::new(config).unwrap();

    let entry = client
        .log_time(mcp_openproject::client::models::CreateTimeEntryRequest {
            work_package_id: 102,
            hours: "2h".into(),
            comment: Some("Implemented feature".into()),
            spent_on: Some("2026-11-17".into()),
            activity_id: Some(3),
            user_id: None,
        })
        .await
        .unwrap();

    assert_eq!(entry.id, 500);
    assert_eq!(entry.hours, "PT2H");
    assert_eq!(entry.spent_on, "2026-11-17");
    assert_eq!(entry.comment, Some("Implemented feature".into()));
    assert_eq!(entry.work_package_id, Some(102));
    assert_eq!(entry.activity_id, Some(3));
}

#[tokio::test]
async fn test_mock_list_sprints() {
    let mock_server = MockServer::start().await;

    let response_body = json!({
        "_type": "Collection",
        "total": 1,
        "count": 1,
        "_embedded": {
            "elements": [
                {
                    "_type": "Sprint",
                    "id": 75,
                    "name": "Sprint 1",
                    "startDate": "2026-11-18",
                    "finishDate": "2026-12-02",
                    "_links": {
                        "status": {
                            "href": "urn:openproject-org:api:v3:sprints:status:in_planning",
                            "title": "Em planejamento"
                        },
                        "definingWorkspace": {
                            "href": "/api/v3/projects/118",
                            "title": "Status Report"
                        }
                    }
                }
            ]
        }
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/sprints"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), "token".into()).unwrap();
    let client = OpenProjectClient::new(config).unwrap();

    let sprints = client.list_sprints(None).await.unwrap();
    assert_eq!(sprints.len(), 1);
    assert_eq!(sprints[0].id, 75);
    assert_eq!(sprints[0].name, "Sprint 1");
    assert_eq!(sprints[0].status, Some("Em planejamento".into()));
    assert_eq!(sprints[0].project_id, Some(118));
    assert_eq!(sprints[0].project_name, Some("Status Report".into()));
    assert_eq!(sprints[0].start_date, Some("2026-11-18".into()));
}

#[tokio::test]
async fn test_mock_get_sprint() {
    let mock_server = MockServer::start().await;

    let response_body = json!({
        "_type": "Sprint",
        "id": 75,
        "name": "Sprint 1",
        "startDate": "2026-11-18",
        "_links": {
            "status": {
                "href": "urn:openproject-org:api:v3:sprints:status:active",
                "title": "Ativo"
            },
            "definingWorkspace": {
                "href": "/api/v3/projects/118",
                "title": "Status Report"
            }
        }
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/sprints/75"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), "token".into()).unwrap();
    let client = OpenProjectClient::new(config).unwrap();

    let sprint = client.get_sprint(75).await.unwrap();
    assert_eq!(sprint.id, 75);
    assert_eq!(sprint.name, "Sprint 1");
    assert_eq!(sprint.status, Some("Ativo".into()));
    assert_eq!(sprint.project_id, Some(118));
}

#[tokio::test]
async fn test_mock_create_work_package_with_sprint() {
    let mock_server = MockServer::start().await;

    let response_body = json!({
        "id": 105,
        "lockVersion": 1,
        "subject": "Task in Sprint 1",
        "_links": {
            "project": { "href": "/api/v3/projects/118", "title": "Status Report" },
            "type": { "href": "/api/v3/types/1", "title": "Task" },
            "sprint": { "href": "/api/v3/sprints/75", "title": "Sprint 1" }
        }
    });

    Mock::given(method("POST"))
        .and(path("/api/v3/projects/118/work_packages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), "token".into()).unwrap();
    let client = OpenProjectClient::new(config).unwrap();

    let created = client
        .create_work_package(mcp_openproject::client::models::CreateWorkPackageRequest {
            project_id: 118,
            type_id: 1,
            subject: "Task in Sprint 1".into(),
            sprint_id: Some(75),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(created.id, 105);
    assert_eq!(created.sprint_id, Some(75));
    assert_eq!(created.sprint_name, Some("Sprint 1".into()));
}

#[tokio::test]
async fn test_mock_create_version() {
    let mock_server = MockServer::start().await;

    let response_body = json!({
        "_type": "Version",
        "id": 12,
        "name": "v1.0.0",
        "description": {
            "format": "markdown",
            "raw": "First stable release"
        },
        "startDate": "2026-12-01",
        "endDate": "2026-12-31",
        "status": "open",
        "sharing": "none",
        "createdAt": "2026-11-19T20:00:00Z",
        "updatedAt": "2026-11-19T20:00:00Z",
        "_links": {
            "definingProject": {
                "href": "/api/v3/projects/117",
                "title": "Sistema de Gestão"
            }
        }
    });

    Mock::given(method("POST"))
        .and(path("/api/v3/versions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), "token".into()).unwrap();
    let client = OpenProjectClient::new(config).unwrap();

    let created = client
        .create_version(CreateVersionRequest {
            project_id: 117,
            name: "v1.0.0".into(),
            description: Some("First stable release".into()),
            start_date: Some("2026-12-01".into()),
            end_date: Some("2026-12-31".into()),
            status: Some("open".into()),
            sharing: Some("none".into()),
        })
        .await
        .unwrap();

    assert_eq!(created.id, 12);
    assert_eq!(created.name, "v1.0.0");
    assert_eq!(created.description, Some("First stable release".into()));
    assert_eq!(created.project_id, Some(117));
    assert_eq!(created.project_name, Some("Sistema de Gestão".into()));
    assert_eq!(created.status, Some("open".into()));
    assert_eq!(created.start_date, Some("2026-12-01".into()));
    assert_eq!(created.end_date, Some("2026-12-31".into()));
}

#[tokio::test]
async fn test_mock_list_versions() {
    let mock_server = MockServer::start().await;

    let response_body = json!({
        "_type": "Collection",
        "total": 1,
        "count": 1,
        "_embedded": {
            "elements": [
                {
                    "_type": "Version",
                    "id": 111,
                    "name": "Backlog",
                    "description": {
                        "raw": "Grupo de pacotes"
                    },
                    "status": "open",
                    "sharing": "none",
                    "_links": {
                        "definingProject": {
                            "href": "/api/v3/projects/117",
                            "title": "Sistema de Gestão"
                        }
                    }
                }
            ]
        }
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/projects/117/versions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), "token".into()).unwrap();
    let client = OpenProjectClient::new(config).unwrap();

    let versions = client.list_versions(Some(117)).await.unwrap();
    assert_eq!(versions.len(), 1);
    assert_eq!(versions[0].id, 111);
    assert_eq!(versions[0].name, "Backlog");
    assert_eq!(versions[0].project_id, Some(117));
}

#[tokio::test]
async fn test_mock_get_version() {
    let mock_server = MockServer::start().await;

    let response_body = json!({
        "_type": "Version",
        "id": 111,
        "name": "Backlog",
        "status": "open",
        "_links": {
            "definingProject": {
                "href": "/api/v3/projects/117",
                "title": "Sistema de Gestão"
            }
        }
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/versions/111"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), "token".into()).unwrap();
    let client = OpenProjectClient::new(config).unwrap();

    let version = client.get_version(111).await.unwrap();
    assert_eq!(version.id, 111);
    assert_eq!(version.name, "Backlog");
    assert_eq!(version.project_id, Some(117));
}

#[tokio::test]
async fn test_mock_create_work_package_with_version() {
    let mock_server = MockServer::start().await;

    let response_body = json!({
        "id": 106,
        "lockVersion": 1,
        "subject": "Task in Version 1.0",
        "_links": {
            "project": { "href": "/api/v3/projects/117", "title": "Sistema de Gestão" },
            "type": { "href": "/api/v3/types/1", "title": "Task" },
            "version": { "href": "/api/v3/versions/111", "title": "Backlog" }
        }
    });

    Mock::given(method("POST"))
        .and(path("/api/v3/projects/117/work_packages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), "token".into()).unwrap();
    let client = OpenProjectClient::new(config).unwrap();

    let created = client
        .create_work_package(CreateWorkPackageRequest {
            project_id: 117,
            type_id: 1,
            subject: "Task in Version 1.0".into(),
            version_id: Some(111),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(created.id, 106);
    assert_eq!(created.version_id, Some(111));
    assert_eq!(created.version_name, Some("Backlog".into()));
}

#[tokio::test]
async fn test_mock_update_project() {
    let mock_server = MockServer::start().await;

    let response_body = json!({
        "_type": "Project",
        "id": 117,
        "identifier": "sistema-gestao",
        "name": "Sistema de Gestão",
        "active": true,
        "public": false,
        "description": {
            "format": "markdown",
            "raw": "Nova descrição detalhada do projeto"
        },
        "statusExplanation": {
            "format": "markdown",
            "raw": "Projeto avançando conforme esperado na sprint atual"
        },
        "_links": {
            "status": {
                "href": "/api/v3/project_statuses/on_track",
                "title": "No cronograma"
            }
        }
    });

    Mock::given(method("PATCH"))
        .and(path("/api/v3/projects/117"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), "token".into()).unwrap();
    let client = OpenProjectClient::new(config).unwrap();

    let updated = client
        .update_project(UpdateProjectRequest {
            id_or_identifier: "117".into(),
            description: Some("Nova descrição detalhada do projeto".into()),
            status: Some("on_track".into()),
            status_explanation: Some("Projeto avançando conforme esperado na sprint atual".into()),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(updated.id, 117);
    assert_eq!(
        updated.description,
        Some("Nova descrição detalhada do projeto".into())
    );
    assert_eq!(updated.status, Some("on_track".into()));
    assert_eq!(updated.status_name, Some("No cronograma".into()));
    assert_eq!(
        updated.status_explanation,
        Some("Projeto avançando conforme esperado na sprint atual".into())
    );
}

#[tokio::test]
async fn test_mock_get_work_package_schema() {
    let mock_server = MockServer::start().await;

    let form_body = json!({
        "_type": "Form",
        "_embedded": {
            "schema": {
                "_type": "Schema",
                "subject": {
                    "type": "String",
                    "name": "Subject",
                    "required": true,
                    "hasDefault": false,
                    "writable": true
                },
                "customField12": {
                    "type": "Boolean",
                    "name": "Acelerado por IA",
                    "required": false,
                    "hasDefault": false,
                    "writable": true
                },
                "customField11": {
                    "type": "[]CustomOption",
                    "name": "Área técnica",
                    "required": false,
                    "hasDefault": false,
                    "writable": true,
                    "location": "_links",
                    "_embedded": {
                        "allowedValues": [
                            {
                                "_type": "CustomOption",
                                "id": 25,
                                "value": "Backend",
                                "_links": {
                                    "self": { "href": "/api/v3/custom_options/25", "title": "Backend" }
                                }
                            },
                            {
                                "_type": "CustomOption",
                                "id": 26,
                                "value": "DevOps",
                                "_links": {
                                    "self": { "href": "/api/v3/custom_options/26", "title": "DevOps" }
                                }
                            }
                        ]
                    }
                }
            }
        }
    });

    Mock::given(method("POST"))
        .and(path("/api/v3/projects/80/work_packages/form"))
        .respond_with(ResponseTemplate::new(200).set_body_json(form_body))
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), "token".into()).unwrap();
    let client = OpenProjectClient::new(config).unwrap();

    let schema = client
        .get_work_package_schema(80, Some(1), false)
        .await
        .unwrap();

    assert_eq!(schema.project_id, 80);
    assert_eq!(schema.type_id, Some(1));
    assert!(schema.standard_fields.is_empty());
    assert_eq!(schema.custom_fields.len(), 2);

    let cf11 = &schema.custom_fields[0];
    assert_eq!(cf11.key, "customField11");
    assert_eq!(cf11.name, "Área técnica");
    assert_eq!(cf11.field_type, "[]CustomOption");
    assert_eq!(cf11.location, "_links");
    assert_eq!(cf11.allowed_values.len(), 2);
    assert_eq!(cf11.allowed_values[0].id, Some(25));
    assert_eq!(cf11.allowed_values[0].value, "Backend");
    assert_eq!(cf11.allowed_values[0].href, "/api/v3/custom_options/25");

    let cf12 = &schema.custom_fields[1];
    assert_eq!(cf12.key, "customField12");
    assert_eq!(cf12.location, "root");
}

#[tokio::test]
async fn test_mock_get_custom_option() {
    let mock_server = MockServer::start().await;

    let response_body = json!({
        "_type": "CustomOption",
        "id": 25,
        "value": "Backend",
        "_links": {
            "self": {
                "href": "/api/v3/custom_options/25",
                "title": "Backend"
            }
        }
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/custom_options/25"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), "token".into()).unwrap();
    let client = OpenProjectClient::new(config).unwrap();

    let opt = client.get_custom_option(25).await.unwrap();
    assert_eq!(opt.id, Some(25));
    assert_eq!(opt.value, "Backend");
    assert_eq!(opt.href, "/api/v3/custom_options/25");
}

#[tokio::test]
async fn test_mock_create_work_package_with_custom_option_links() {
    let mock_server = MockServer::start().await;

    let expected_body = json!({
        "subject": "Implementar API",
        "customField12": true,
        "_links": {
            "type": { "href": "/api/v3/types/1" },
            "customField11": [
                { "href": "/api/v3/custom_options/25" },
                { "href": "/api/v3/custom_options/26" }
            ]
        }
    });

    let response_body = json!({
        "id": 200,
        "lockVersion": 1,
        "subject": "Implementar API",
        "customField12": true,
        "_links": {
            "project": { "href": "/api/v3/projects/80", "title": "Nova Solução PPM" },
            "type": { "href": "/api/v3/types/1", "title": "Task" },
            "customField11": [
                { "href": "/api/v3/custom_options/25", "title": "Backend" },
                { "href": "/api/v3/custom_options/26", "title": "DevOps" }
            ]
        }
    });

    Mock::given(method("POST"))
        .and(path("/api/v3/projects/80/work_packages"))
        .and(wiremock::matchers::body_json(&expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), "token".into()).unwrap();
    let client = OpenProjectClient::new(config).unwrap();

    let mut custom_fields = std::collections::HashMap::new();
    custom_fields.insert("customField12".into(), json!(true));
    custom_fields.insert(
        "customField11".into(),
        json!([
            { "href": "/api/v3/custom_options/25" },
            { "href": "/api/v3/custom_options/26" }
        ]),
    );

    let created = client
        .create_work_package(CreateWorkPackageRequest {
            project_id: 80,
            type_id: 1,
            subject: "Implementar API".into(),
            custom_fields: Some(custom_fields),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(created.id, 200);
    assert_eq!(
        created.custom_fields.get("customField12"),
        Some(&json!(true))
    );
    assert_eq!(
        created.custom_fields.get("customField11"),
        Some(&json!(["Backend", "DevOps"]))
    );
}

#[tokio::test]
async fn test_mock_list_priorities() {
    let mock_server = MockServer::start().await;

    let response_body = json!({
        "_type": "Collection",
        "total": 2,
        "count": 2,
        "_embedded": {
            "elements": [
                { "id": 7, "name": "Baixa", "position": 1, "isDefault": false, "isActive": true },
                { "id": 8, "name": "Normal", "position": 2, "isDefault": true, "isActive": true }
            ]
        }
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/priorities"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), "token".into()).unwrap();
    let client = OpenProjectClient::new(config).unwrap();

    let priorities = client.list_priorities().await.unwrap();
    assert_eq!(priorities.len(), 2);
    assert_eq!(priorities[0].id, 7);
    assert_eq!(priorities[0].name, "Baixa");
    assert!(!priorities[0].is_default);
    assert_eq!(priorities[1].id, 8);
    assert!(priorities[1].is_default);
}

#[tokio::test]
async fn test_mock_add_and_list_work_package_activities() {
    let mock_server = MockServer::start().await;

    let created_activity = json!({
        "_type": "Activity::Comment",
        "id": 999,
        "comment": { "format": "markdown", "raw": "Aguardando revisão do time de DevOps" },
        "details": [],
        "version": 3,
        "internal": false,
        "createdAt": "2026-11-25T16:20:00Z",
        "_links": {
            "user": { "href": "/api/v3/users/8", "title": "Rafael" }
        }
    });

    Mock::given(method("POST"))
        .and(path("/api/v3/work_packages/5454/activities"))
        .respond_with(ResponseTemplate::new(200).set_body_json(created_activity.clone()))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v3/work_packages/5454/activities"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "_type": "Collection",
            "total": 1,
            "count": 1,
            "_embedded": { "elements": [created_activity] }
        })))
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), "token".into()).unwrap();
    let client = OpenProjectClient::new(config).unwrap();

    let added = client
        .add_work_package_comment(5454, "Aguardando revisão do time de DevOps", false)
        .await
        .unwrap();
    assert_eq!(added.id, 999);
    assert_eq!(
        added.comment,
        Some("Aguardando revisão do time de DevOps".into())
    );
    assert_eq!(added.user_id, Some(8));
    assert_eq!(added.user_name, Some("Rafael".into()));

    let list = client.list_work_package_activities(5454).await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, 999);
}

#[tokio::test]
async fn test_mock_list_time_entries() {
    let mock_server = MockServer::start().await;

    let response_body = json!({
        "_type": "Collection",
        "total": 1,
        "count": 1,
        "pageSize": 20,
        "offset": 1,
        "_embedded": {
            "elements": [
                {
                    "_type": "TimeEntry",
                    "id": 321,
                    "hours": "PT2H",
                    "spentOn": "2026-11-25",
                    "comment": { "raw": "Implementação MCP" },
                    "_links": {
                        "project": { "href": "/api/v3/projects/80", "title": "Nova Solução PPM" },
                        "workPackage": { "href": "/api/v3/work_packages/5454", "title": "API v3" },
                        "user": { "href": "/api/v3/users/8", "title": "Rafael" },
                        "activity": { "href": "/api/v3/time_entries/activities/3", "title": "Development" }
                    }
                }
            ]
        }
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/time_entries"))
        .and(query_param(
            "filters",
            "[{\"entity\":{\"operator\":\"=\",\"values\":[\"5454\"]}}]",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), "token".into()).unwrap();
    let client = OpenProjectClient::new(config).unwrap();

    let filter = TimeEntryFilterBuilder::new().with_work_package_id(Some(5454));
    let res = client.list_time_entries(&filter, 1, 20).await.unwrap();
    assert_eq!(res.total, 1);
    assert_eq!(res.items[0].id, 321);
    assert_eq!(res.items[0].hours, "PT2H");
    assert_eq!(res.items[0].work_package_id, Some(5454));
}

#[tokio::test]
async fn test_mock_relations_lifecycle() {
    let mock_server = MockServer::start().await;

    let relation_json = json!({
        "_type": "Relation",
        "id": 77,
        "type": "blocked",
        "reverseType": "blocks",
        "description": "Frontend aguarda endpoint",
        "_links": {
            "from": { "href": "/api/v3/work_packages/10", "title": "Tela Login" },
            "to": { "href": "/api/v3/work_packages/11", "title": "Auth API" }
        }
    });

    Mock::given(method("POST"))
        .and(path("/api/v3/work_packages/10/relations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(relation_json.clone()))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v3/work_packages/10/relations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "_type": "Collection",
            "total": 1,
            "count": 1,
            "_embedded": { "elements": [relation_json] }
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/api/v3/relations/77"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), "token".into()).unwrap();
    let client = OpenProjectClient::new(config).unwrap();

    let created = client
        .create_relation(CreateRelationRequest {
            from_id: 10,
            to_id: 11,
            relation_type: "blocked_by".into(),
            description: Some("Frontend aguarda endpoint".into()),
            lag: None,
        })
        .await
        .unwrap();
    assert_eq!(created.id, 77);
    assert_eq!(created.relation_type, "blocked");
    assert_eq!(created.from_id, Some(10));
    assert_eq!(created.to_id, Some(11));

    let list = client.list_relations(10).await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, 77);

    client.delete_relation(77).await.unwrap();
}
