//! Wiremock tests for the API-key CRUD endpoints (HRA-142).

use std::time::Duration;

use openrouter::{Client, CreateKeyRequest, Error, ListKeysOptions, UpdateKeyRequest};
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

fn sample_key() -> serde_json::Value {
    json!({
        "name": "primary",
        "label": "primary",
        "limit": 100.0,
        "disabled": false,
        "created_at": "2026-05-01T00:00:00Z",
        "updated_at": "2026-05-25T00:00:00Z",
        "hash": "abc123"
    })
}

#[tokio::test]
async fn list_keys_passes_pagination_query() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/keys"))
        .and(query_param("offset", "10"))
        .and(query_param("include_disabled", "true"))
        .and(header("authorization", "Bearer sk-prov-test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": [sample_key()]})))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let opts = ListKeysOptions::new().offset(10).include_disabled(true);
    let resp = client.list_keys(Some(&opts)).await.unwrap();
    assert_eq!(resp.data.len(), 1);
    assert_eq!(resp.data[0].hash, "abc123");
}

#[tokio::test]
async fn get_key_by_hash_round_trip() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/keys/abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": sample_key()})))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client.get_key_by_hash("abc123").await.unwrap();
    assert_eq!(resp.data.label, "primary");
}

#[tokio::test]
async fn create_key_posts_body_and_returns_secret() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/keys"))
        .and(body_json(
            json!({"name": "new", "limit": 25.0, "include_byok_in_limit": true}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": sample_key(),
            "key": "sk-or-v1-newly-minted"
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let req = CreateKeyRequest {
        name: "new".into(),
        limit: Some(25.0),
        include_byok_in_limit: Some(true),
    };
    let resp = client.create_key(&req).await.unwrap();
    assert_eq!(resp.key.as_deref(), Some("sk-or-v1-newly-minted"));
    assert_eq!(resp.data.hash, "abc123");
}

#[tokio::test]
async fn update_key_patches_only_provided_fields() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/keys/abc123"))
        .and(body_json(json!({"name": "renamed"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": sample_key()})))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let req = UpdateKeyRequest {
        name: Some("renamed".into()),
        ..Default::default()
    };
    client.update_key("abc123", &req).await.unwrap();
}

#[tokio::test]
async fn delete_key_sends_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/keys/abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"success": true}})))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client.delete_key("abc123").await.unwrap();
    assert!(resp.data.success);
}

#[tokio::test]
async fn key_crud_rejects_empty_inputs() {
    let client = Client::builder().api_key("sk-test").build().unwrap();
    assert!(matches!(
        client.get_key_by_hash("").await,
        Err(Error::InvalidInput(_))
    ));
    assert!(matches!(
        client.delete_key("").await,
        Err(Error::InvalidInput(_))
    ));
    assert!(matches!(
        client.update_key("", &UpdateKeyRequest::default()).await,
        Err(Error::InvalidInput(_))
    ));
    assert!(matches!(
        client.create_key(&CreateKeyRequest::default()).await,
        Err(Error::InvalidInput(_))
    ));
}
