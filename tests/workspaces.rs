//! Wiremock tests for the workspaces endpoints (HRA-143).

use std::time::Duration;

use openrouter::{
    Client, CreateWorkspaceRequest, Error, ListWorkspacesOptions, UpdateWorkspaceRequest,
    WorkspaceMemberRole,
};
use pretty_assertions::assert_eq;
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client_for(server: &MockServer) -> Client {
    Client::builder()
        .api_key("sk-prov-test")
        .base_url(server.uri())
        .unwrap()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap()
}

fn sample_workspace() -> serde_json::Value {
    json!({
        "id": "ws_01HZX",
        "name": "Engineering",
        "slug": "engineering",
        "description": null,
        "created_by": "usr_1",
        "created_at": "2026-05-01T00:00:00Z",
        "updated_at": null,
        "default_text_model": null,
        "default_image_model": null,
        "default_provider_sort": null,
        "io_logging_api_key_ids": null,
        "io_logging_sampling_rate": 0.0,
        "is_data_discount_logging_enabled": false,
        "is_observability_broadcast_enabled": false,
        "is_observability_io_logging_enabled": false,
    })
}

#[tokio::test]
async fn list_workspaces_passes_pagination_query() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/workspaces"))
        .and(query_param("offset", "20"))
        .and(query_param("limit", "5"))
        .and(header("authorization", "Bearer sk-prov-test"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"data": [sample_workspace()], "total_count": 1})),
        )
        .mount(&server)
        .await;

    let client = client_for(&server);
    let opts = ListWorkspacesOptions::new().offset(20).limit(5);
    let resp = client.list_workspaces(Some(&opts)).await.unwrap();
    assert_eq!(resp.data.len(), 1);
    assert_eq!(resp.total_count, 1);
    assert_eq!(resp.data[0].slug, "engineering");
}

#[tokio::test]
async fn list_workspaces_no_options_no_query() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/workspaces"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"data": [], "total_count": 0})),
        )
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client.list_workspaces(None).await.unwrap();
    assert!(resp.data.is_empty());
}

#[tokio::test]
async fn create_workspace_round_trip() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/workspaces"))
        .and(body_json(json!({
            "name": "Engineering",
            "slug": "engineering",
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": sample_workspace()})))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let req = CreateWorkspaceRequest {
        name: "Engineering".into(),
        slug: "engineering".into(),
        ..Default::default()
    };
    let resp = client.create_workspace(&req).await.unwrap();
    assert_eq!(resp.data.id, "ws_01HZX");
}

#[tokio::test]
async fn create_workspace_validates_name_and_slug() {
    let client = Client::new("sk").unwrap();
    let err = client
        .create_workspace(&CreateWorkspaceRequest {
            name: "".into(),
            slug: "x".into(),
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert!(matches!(err, Error::InvalidInput(_)));

    let err = client
        .create_workspace(&CreateWorkspaceRequest {
            name: "x".into(),
            slug: "".into(),
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert!(matches!(err, Error::InvalidInput(_)));
}

#[tokio::test]
async fn get_workspace_by_slug() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/workspaces/engineering"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": sample_workspace()})))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client.get_workspace("engineering").await.unwrap();
    assert_eq!(resp.data.name, "Engineering");
}

#[tokio::test]
async fn update_workspace_sends_only_set_fields() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/workspaces/engineering"))
        .and(body_json(json!({"description": "team workspace"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": sample_workspace()})))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let req = UpdateWorkspaceRequest {
        description: Some("team workspace".into()),
        ..Default::default()
    };
    client.update_workspace("engineering", &req).await.unwrap();
}

#[tokio::test]
async fn delete_workspace_returns_flag() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/workspaces/engineering"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"deleted": true})))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client.delete_workspace("engineering").await.unwrap();
    assert!(resp.deleted);
}

#[tokio::test]
async fn add_workspace_members_bulk() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/workspaces/engineering/members/add"))
        .and(body_json(json!({"user_ids": ["usr_a", "usr_b"]})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "added_count": 2,
            "data": [
                {
                    "id": "wsm_1",
                    "workspace_id": "ws_01HZX",
                    "user_id": "usr_a",
                    "role": "member",
                    "created_at": "2026-05-26T00:00:00Z",
                },
                {
                    "id": "wsm_2",
                    "workspace_id": "ws_01HZX",
                    "user_id": "usr_b",
                    "role": "admin",
                    "created_at": "2026-05-26T00:00:00Z",
                }
            ]
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client
        .add_workspace_members("engineering", &["usr_a".into(), "usr_b".into()])
        .await
        .unwrap();
    assert_eq!(resp.added_count, 2);
    assert_eq!(resp.data[0].role, WorkspaceMemberRole::Member);
    assert_eq!(resp.data[1].role, WorkspaceMemberRole::Admin);
}

#[tokio::test]
async fn remove_workspace_members_bulk() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/workspaces/engineering/members/remove"))
        .and(body_json(json!({"user_ids": ["usr_a"]})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"removed_count": 1})))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client
        .remove_workspace_members("engineering", &["usr_a".into()])
        .await
        .unwrap();
    assert_eq!(resp.removed_count, 1);
}

#[tokio::test]
async fn member_endpoints_reject_empty_inputs() {
    let client = Client::new("sk").unwrap();
    let err = client
        .add_workspace_members("", &["usr".into()])
        .await
        .unwrap_err();
    assert!(matches!(err, Error::InvalidInput(_)));

    let err = client
        .add_workspace_members("engineering", &[])
        .await
        .unwrap_err();
    assert!(matches!(err, Error::InvalidInput(_)));
}

#[tokio::test]
async fn surfaces_api_error_envelope() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/workspaces/default"))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "error": {
                "code": 409,
                "message": "default workspace cannot be deleted",
            }
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let err = client.delete_workspace("default").await.unwrap_err();
    match err {
        Error::Api {
            status, message, ..
        } => {
            assert_eq!(status, 409);
            assert!(message.contains("default workspace"));
        }
        other => panic!("expected Api error, got {other:?}"),
    }
}
