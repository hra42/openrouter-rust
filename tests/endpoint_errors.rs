//! Per-endpoint error-path coverage.
//!
//! Each test stubs a single endpoint with a representative non-2xx response
//! and asserts the call surfaces an [`openrouter::Error::Api`] (or, for the
//! beta surfaces, the equivalent typed error). Happy-path coverage lives in
//! the per-endpoint test files; this file deliberately concentrates failure
//! modes in one place so a wire-format drift is easy to spot.

use std::time::Duration;

use openrouter::oauth::ExchangeAuthCodeRequest;
use openrouter::{
    ActivityOptions, Client, CompletionRequest, CreateGuardrailRequest, CreateKeyRequest,
    CreateWorkspaceRequest, Error, ListKeysOptions, ListOrganizationMembersOptions,
    ListWorkspacesOptions, RerankRequest, SpeechFormat, SpeechRequest, UpdateKeyRequest,
    UpdateWorkspaceRequest, VideoGenerationRequest,
};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client_for(server: &MockServer) -> Client {
    Client::builder()
        .api_key("sk-test")
        .base_url(server.uri())
        .unwrap()
        .retry(0, Duration::from_millis(1))
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap()
}

fn expect_api_error(err: Error, status: u16) {
    match err {
        Error::Api { status: s, .. } => assert_eq!(s, status),
        other => panic!("expected Error::Api({status}), got {other:?}"),
    }
}

fn err_body(code: &str, msg: &str) -> serde_json::Value {
    serde_json::json!({"error": {"code": code, "message": msg}})
}

async fn mount_error(server: &MockServer, http_method: &'static str, p: &'static str, status: u16) {
    Mock::given(method(http_method))
        .and(path(p))
        .respond_with(ResponseTemplate::new(status).set_body_json(err_body("e", "boom")))
        .mount(server)
        .await;
}

// ---- chat / completions ----

#[tokio::test]
async fn complete_400_surfaces_api_error() {
    let server = MockServer::start().await;
    mount_error(&server, "POST", "/completions", 400).await;
    let err = client_for(&server)
        .complete(CompletionRequest {
            model: "x/y".into(),
            prompt: "hi".into(),
            ..Default::default()
        })
        .await
        .unwrap_err();
    expect_api_error(err, 400);
}

// ---- discovery ----

#[tokio::test]
async fn list_providers_500_surfaces_api_error() {
    let server = MockServer::start().await;
    mount_error(&server, "GET", "/providers", 500).await;
    let err = client_for(&server).list_providers().await.unwrap_err();
    expect_api_error(err, 500);
}

#[tokio::test]
async fn list_model_endpoints_404_surfaces_api_error() {
    let server = MockServer::start().await;
    mount_error(&server, "GET", "/models/unknown/author/endpoints", 404).await;
    let err = client_for(&server)
        .list_model_endpoints("unknown", "author")
        .await
        .unwrap_err();
    expect_api_error(err, 404);
}

// ---- account ----

#[tokio::test]
async fn get_credits_401_surfaces_api_error() {
    let server = MockServer::start().await;
    mount_error(&server, "GET", "/credits", 401).await;
    let err = client_for(&server).get_credits().await.unwrap_err();
    expect_api_error(err, 401);
}

#[tokio::test]
async fn get_key_401_surfaces_api_error() {
    let server = MockServer::start().await;
    mount_error(&server, "GET", "/key", 401).await;
    let err = client_for(&server).get_key().await.unwrap_err();
    expect_api_error(err, 401);
}

#[tokio::test]
async fn get_activity_401_surfaces_api_error() {
    let server = MockServer::start().await;
    mount_error(&server, "GET", "/activity", 401).await;
    let err = client_for(&server)
        .get_activity(Some(&ActivityOptions::default()))
        .await
        .unwrap_err();
    expect_api_error(err, 401);
}

// ---- key CRUD ----

#[tokio::test]
async fn list_keys_403_surfaces_api_error() {
    let server = MockServer::start().await;
    mount_error(&server, "GET", "/keys", 403).await;
    let err = client_for(&server)
        .list_keys(Some(&ListKeysOptions::default()))
        .await
        .unwrap_err();
    expect_api_error(err, 403);
}

#[tokio::test]
async fn create_key_403_surfaces_api_error() {
    let server = MockServer::start().await;
    mount_error(&server, "POST", "/keys", 403).await;
    let err = client_for(&server)
        .create_key(&CreateKeyRequest {
            name: "n".into(),
            ..Default::default()
        })
        .await
        .unwrap_err();
    expect_api_error(err, 403);
}

#[tokio::test]
async fn update_key_404_surfaces_api_error() {
    let server = MockServer::start().await;
    mount_error(&server, "PATCH", "/keys/missing", 404).await;
    let err = client_for(&server)
        .update_key("missing", &UpdateKeyRequest::default())
        .await
        .unwrap_err();
    expect_api_error(err, 404);
}

#[tokio::test]
async fn delete_key_404_surfaces_api_error() {
    let server = MockServer::start().await;
    mount_error(&server, "DELETE", "/keys/missing", 404).await;
    let err = client_for(&server).delete_key("missing").await.unwrap_err();
    expect_api_error(err, 404);
}

// ---- rerank ----

#[tokio::test]
async fn rerank_400_surfaces_api_error() {
    let server = MockServer::start().await;
    mount_error(&server, "POST", "/rerank", 400).await;
    let err = client_for(&server)
        .rerank(&RerankRequest {
            model: "x/y".into(),
            query: "q".into(),
            documents: vec!["d".into()],
            ..Default::default()
        })
        .await
        .unwrap_err();
    expect_api_error(err, 400);
}

// ---- audio ----

#[tokio::test]
async fn create_speech_500_surfaces_api_error() {
    let server = MockServer::start().await;
    mount_error(&server, "POST", "/audio/speech", 500).await;
    let err = client_for(&server)
        .create_speech(&SpeechRequest {
            model: "x/y".into(),
            input: "hi".into(),
            voice: "alloy".into(),
            response_format: Some(SpeechFormat::Mp3),
            ..Default::default()
        })
        .await
        .unwrap_err();
    expect_api_error(err, 500);
}

// ---- video ----

#[tokio::test]
async fn create_video_400_surfaces_api_error() {
    let server = MockServer::start().await;
    mount_error(&server, "POST", "/videos", 400).await;
    let err = client_for(&server)
        .create_video(&VideoGenerationRequest {
            model: "x/y".into(),
            prompt: "p".into(),
            ..Default::default()
        })
        .await
        .unwrap_err();
    expect_api_error(err, 400);
}

#[tokio::test]
async fn get_video_404_surfaces_api_error() {
    let server = MockServer::start().await;
    mount_error(&server, "GET", "/videos/missing", 404).await;
    let err = client_for(&server).get_video("missing").await.unwrap_err();
    expect_api_error(err, 404);
}

#[tokio::test]
async fn list_video_models_500_surfaces_api_error() {
    let server = MockServer::start().await;
    mount_error(&server, "GET", "/videos/models", 500).await;
    let err = client_for(&server).list_video_models().await.unwrap_err();
    expect_api_error(err, 500);
}

// ---- workspaces ----

#[tokio::test]
async fn list_workspaces_403_surfaces_api_error() {
    let server = MockServer::start().await;
    mount_error(&server, "GET", "/workspaces", 403).await;
    let err = client_for(&server)
        .list_workspaces(Some(&ListWorkspacesOptions::default()))
        .await
        .unwrap_err();
    expect_api_error(err, 403);
}

#[tokio::test]
async fn create_workspace_409_surfaces_api_error() {
    let server = MockServer::start().await;
    mount_error(&server, "POST", "/workspaces", 409).await;
    let err = client_for(&server)
        .create_workspace(&CreateWorkspaceRequest {
            name: "n".into(),
            slug: "n".into(),
            ..Default::default()
        })
        .await
        .unwrap_err();
    expect_api_error(err, 409);
}

#[tokio::test]
async fn update_workspace_404_surfaces_api_error() {
    let server = MockServer::start().await;
    mount_error(&server, "PATCH", "/workspaces/missing", 404).await;
    let err = client_for(&server)
        .update_workspace("missing", &UpdateWorkspaceRequest::default())
        .await
        .unwrap_err();
    expect_api_error(err, 404);
}

// ---- organization members ----

#[tokio::test]
async fn list_org_members_401_surfaces_api_error() {
    let server = MockServer::start().await;
    mount_error(&server, "GET", "/organization/members", 401).await;
    let err = client_for(&server)
        .list_organization_members(Some(&ListOrganizationMembersOptions::default()))
        .await
        .unwrap_err();
    expect_api_error(err, 401);
}

// ---- guardrails ----

#[tokio::test]
async fn create_guardrail_403_surfaces_api_error() {
    let server = MockServer::start().await;
    mount_error(&server, "POST", "/guardrails", 403).await;
    let err = client_for(&server)
        .create_guardrail(&CreateGuardrailRequest {
            name: "g".into(),
            ..Default::default()
        })
        .await
        .unwrap_err();
    expect_api_error(err, 403);
}

#[tokio::test]
async fn delete_guardrail_404_surfaces_api_error() {
    let server = MockServer::start().await;
    mount_error(&server, "DELETE", "/guardrails/missing", 404).await;
    let err = client_for(&server)
        .delete_guardrail("missing")
        .await
        .unwrap_err();
    expect_api_error(err, 404);
}

// ---- oauth ----

#[tokio::test]
async fn exchange_auth_code_400_surfaces_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/auth/keys"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "error": {"code": "invalid_grant", "message": "code expired"}
        })))
        .mount(&server)
        .await;

    let err = client_for(&server)
        .exchange_auth_code(&ExchangeAuthCodeRequest {
            code: "abc".into(),
            ..Default::default()
        })
        .await
        .unwrap_err();
    match err {
        Error::Api { status, code, .. } => {
            assert_eq!(status, 400);
            assert_eq!(code.as_deref(), Some("invalid_grant"));
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}
