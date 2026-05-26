//! Cross-endpoint mapping of HTTP responses into [`openrouter::Error`] variants.
//!
//! The crate-internal `Error::from_response_body` is exercised through public
//! call sites; this file pins the externally-visible behavior so we notice if
//! the mapping drifts.

use std::time::Duration;

use openrouter::{ChatCompletionRequest, Client, Error, Message};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn no_retry_client(server: &MockServer) -> Client {
    Client::builder()
        .api_key("sk-test")
        .base_url(server.uri())
        .unwrap()
        .retry(0, Duration::from_millis(1))
        .build()
        .unwrap()
}

fn req() -> ChatCompletionRequest {
    ChatCompletionRequest {
        model: "x/y".into(),
        messages: vec![Message::user("hi")],
        ..Default::default()
    }
}

#[tokio::test]
async fn structured_envelope_with_metadata_and_provider() {
    let server = MockServer::start().await;
    let body = serde_json::json!({
        "error": {
            "code": "invalid_request_error",
            "message": "bad model",
            "metadata": {"raw": "foo"},
            "provider_name": "openai"
        }
    });
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(400).set_body_json(&body))
        .mount(&server)
        .await;

    let err = no_retry_client(&server)
        .chat_complete(req())
        .await
        .unwrap_err();
    match err {
        Error::Api {
            status,
            code,
            message,
            metadata,
            provider,
            ..
        } => {
            assert_eq!(status, 400);
            assert_eq!(code.as_deref(), Some("invalid_request_error"));
            assert_eq!(message, "bad model");
            assert!(metadata.is_some());
            assert_eq!(provider.as_deref(), Some("openai"));
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn numeric_code_is_stringified() {
    let server = MockServer::start().await;
    let body = serde_json::json!({"error": {"code": 429, "message": "too many"}});
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "1")
                .set_body_json(&body),
        )
        .mount(&server)
        .await;

    let err = no_retry_client(&server)
        .chat_complete(req())
        .await
        .unwrap_err();
    if let Error::Api { code, .. } = err {
        assert_eq!(code.as_deref(), Some("429"));
    } else {
        panic!("expected Error::Api");
    }
}

#[tokio::test]
async fn non_json_body_falls_back_to_raw_message() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(502).set_body_string("upstream gone"))
        .mount(&server)
        .await;

    let err = no_retry_client(&server)
        .chat_complete(req())
        .await
        .unwrap_err();
    match err {
        Error::Api {
            status,
            message,
            code,
            ..
        } => {
            assert_eq!(status, 502);
            assert_eq!(message, "upstream gone");
            assert!(code.is_none());
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn unauthorized_401_surfaces_as_api_error() {
    let server = MockServer::start().await;
    let body = serde_json::json!({"error": {"code": "unauthenticated", "message": "missing key"}});
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(401).set_body_json(&body))
        .mount(&server)
        .await;

    let err = no_retry_client(&server)
        .chat_complete(req())
        .await
        .unwrap_err();
    match err {
        Error::Api { status, code, .. } => {
            assert_eq!(status, 401);
            assert_eq!(code.as_deref(), Some("unauthenticated"));
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn malformed_success_body_returns_decode_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json at all"))
        .mount(&server)
        .await;

    let err = no_retry_client(&server)
        .chat_complete(req())
        .await
        .unwrap_err();
    assert!(
        matches!(err, Error::Decode(_)),
        "expected Error::Decode for a non-JSON 200 body"
    );
}

#[tokio::test]
async fn missing_field_for_empty_api_key() {
    let err = Client::builder().build().unwrap_err();
    assert!(matches!(err, Error::MissingField("api_key")));
}

#[tokio::test]
async fn invalid_input_for_bad_base_url() {
    let err = Client::builder()
        .api_key("sk-test")
        .base_url("not a url")
        .unwrap_err();
    assert!(matches!(err, Error::InvalidInput(_)));
}
