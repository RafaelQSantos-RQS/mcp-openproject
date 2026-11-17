use mcp_openproject::client::OpenProjectClient;
use mcp_openproject::client::error::OpenProjectError;
use mcp_openproject::client::models::{CreateWorkPackageRequest, UpdateWorkPackageRequest};
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
