//! Wiremock tests for `Client::list_models` (HRA-136).
//!
//! Also exercises the shared GET plumbing in `src/request.rs`: query-pair
//! encoding, auth header propagation, and `Error::from_response_body` decoding
//! for non-2xx responses.

use std::time::Duration;

use openrouter::{Client, Error};
use pretty_assertions::assert_eq;
use wiremock::matchers::{header, method, path, query_param};
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
async fn list_models_sends_get_with_query_and_auth() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/models"))
        .and(query_param("category", "programming"))
        .and(header("authorization", "Bearer sk-test"))
        .and(header("accept", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"data":[]}"#))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let opts = openrouter::ListModelsOptions::new().category("programming");
    let resp = client.list_models(Some(&opts)).await.unwrap();
    assert_eq!(resp.data.len(), 0);
}

#[tokio::test]
async fn list_models_parses_typical_payload() {
    let server = MockServer::start().await;
    let body = r#"{
        "data": [
            {
                "id": "google/gemini-3.1-flash-lite",
                "name": "Gemini 3.1 Flash Lite",
                "canonical_slug": "gemini-3-1-flash-lite",
                "created": 1730000000.0,
                "description": "Fast and cheap.",
                "context_length": 1048576,
                "architecture": {
                    "input_modalities": ["text", "image"],
                    "output_modalities": ["text"],
                    "tokenizer": "Gemini",
                    "instruct_type": null,
                    "modality": "text->text"
                },
                "top_provider": {
                    "context_length": 1048576,
                    "max_completion_tokens": 65536,
                    "is_moderated": false
                },
                "supported_parameters": ["temperature", "tools"],
                "pricing": {
                    "prompt": "0.0000001",
                    "completion": "0.0000004",
                    "image": "0",
                    "request": "0",
                    "input_cache_read": "0.00000003",
                    "input_cache_write": null,
                    "web_search": "0",
                    "internal_reasoning": "0"
                }
            }
        ]
    }"#;
    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client.list_models(None).await.unwrap();
    assert_eq!(resp.data.len(), 1);
    let m = &resp.data[0];
    assert_eq!(m.id, "google/gemini-3.1-flash-lite");
    assert_eq!(m.context_length, Some(1_048_576.0));
    assert_eq!(m.architecture.input_modalities, vec!["text", "image"]);
    assert_eq!(m.top_provider.max_completion_tokens, Some(65_536.0));
    assert_eq!(m.pricing.input_cache_read.as_deref(), Some("0.00000003"));
    assert!(m.supported_parameters.contains(&"tools".to_string()));
}

#[tokio::test]
async fn list_models_surfaces_api_error_body() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(
            ResponseTemplate::new(401)
                .set_body_string(r#"{"error":{"code":401,"message":"missing key"}}"#),
        )
        .mount(&server)
        .await;

    let client = client_for(&server);
    let err = client.list_models(None).await.unwrap_err();
    match err {
        Error::Api {
            status,
            code,
            message,
            ..
        } => {
            assert_eq!(status, 401);
            assert_eq!(code.as_deref(), Some("401"));
            assert_eq!(message, "missing key");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}
