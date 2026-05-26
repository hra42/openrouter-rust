//! Wiremock tests for POST /auth/keys (HRA-150).

use std::time::Duration;

use openrouter::oauth::{CodeChallengeMethod, ExchangeAuthCodeRequest};
use openrouter::{Client, Error};
use pretty_assertions::assert_eq;
use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client_for(server: &MockServer) -> Client {
    Client::builder()
        .api_key("sk-test")
        .base_url(server.uri())
        .unwrap()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap()
}

#[tokio::test]
async fn exchange_auth_code_with_verifier_returns_key() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/auth/keys"))
        .and(body_json(json!({
            "code": "abc123",
            "code_verifier": "ver-001",
            "code_challenge_method": "S256",
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"key": "sk-new-key", "user_id": "usr_1"})),
        )
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client
        .exchange_auth_code(&ExchangeAuthCodeRequest {
            code: "abc123".into(),
            code_verifier: Some("ver-001".into()),
            code_challenge_method: Some(CodeChallengeMethod::S256),
        })
        .await
        .unwrap();
    assert_eq!(resp.key, "sk-new-key");
    assert_eq!(resp.user_id.as_deref(), Some("usr_1"));
}

#[tokio::test]
async fn exchange_auth_code_without_verifier() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/auth/keys"))
        .and(body_json(json!({"code": "abc"})))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"key": "sk-x", "user_id": null})),
        )
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client
        .exchange_auth_code(&ExchangeAuthCodeRequest {
            code: "abc".into(),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(resp.key, "sk-x");
    assert!(resp.user_id.is_none());
}

#[tokio::test]
async fn exchange_auth_code_requires_code() {
    let client = Client::new("sk").unwrap();
    let err = client
        .exchange_auth_code(&ExchangeAuthCodeRequest::default())
        .await
        .unwrap_err();
    assert!(matches!(err, Error::InvalidInput(_)));
}
