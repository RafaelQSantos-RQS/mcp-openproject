use mcp_openproject::client::OpenProjectClient;
use mcp_openproject::config::Config;
use mcp_openproject::server::OpenProjectServer;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ContentBlock;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[test]
fn test_server_registers_all_8_tools() {
    let config = Config::new("https://example.com".into(), "test-key".into()).unwrap();
    let client = OpenProjectClient::new(config).unwrap();
    let server = OpenProjectServer::new(client);

    let tools = server.router().list_all();
    assert_eq!(tools.len(), 8);

    let names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
    assert!(names.contains(&"list_projects"));
    assert!(names.contains(&"get_project"));
    assert!(names.contains(&"search_work_packages"));
    assert!(names.contains(&"get_work_package"));
    assert!(names.contains(&"create_work_package"));
    assert!(names.contains(&"update_work_package"));
    assert!(names.contains(&"list_types"));
    assert!(names.contains(&"list_statuses"));
}

#[tokio::test]
async fn test_server_list_types_tool() {
    let mock_server = MockServer::start().await;

    let types_body = json!({
        "_type": "Collection",
        "total": 2,
        "count": 2,
        "_embedded": {
            "elements": [
                {
                    "id": 1,
                    "name": "Task",
                    "color": "#0000ff",
                    "isDefault": true,
                    "isMilestone": false
                },
                {
                    "id": 2,
                    "name": "Bug",
                    "color": "#ff0000",
                    "isDefault": false,
                    "isMilestone": false
                }
            ]
        }
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/types"))
        .respond_with(ResponseTemplate::new(200).set_body_json(types_body))
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), "token".into()).unwrap();
    let client = OpenProjectClient::new(config).unwrap();
    let server = OpenProjectServer::new(client);

    let result = server
        .list_types(Parameters(Default::default()))
        .await
        .unwrap();

    assert_eq!(result.is_error, Some(false));
    assert!(result.structured_content.is_some());
    let json_val = result.structured_content.unwrap();
    assert_eq!(json_val[0]["name"], "Task");
    assert_eq!(json_val[1]["name"], "Bug");
}

#[tokio::test]
async fn test_server_error_translates_to_tool_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/work_packages/999"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Work package not found"))
        .mount(&mock_server)
        .await;

    let config = Config::new(mock_server.uri(), "token".into()).unwrap();
    let client = OpenProjectClient::new(config).unwrap();
    let server = OpenProjectServer::new(client);

    let result = server
        .get_work_package(Parameters(
            mcp_openproject::server::tools::GetWorkPackageParams { id: 999 },
        ))
        .await
        .unwrap();

    assert_eq!(result.is_error, Some(true));
    match &result.content[0] {
        ContentBlock::Text(t) => {
            assert!(t.text.contains("Failed to get work package #999"));
        }
        _ => panic!("Expected text content"),
    }
}
