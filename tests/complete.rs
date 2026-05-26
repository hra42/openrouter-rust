//! HTTP-level integration tests for the legacy `Client::complete` endpoint.

use std::time::Duration;

use openrouter::{Client, CompletionRequest, Provider};
use pretty_assertions::assert_eq;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client_for(server: &MockServer) -> Client {
    Client::builder()
        .api_key("sk-test")
        .base_url(server.uri())
        .unwrap()
        .retry(2, Duration::from_millis(1))
        .build()
        .unwrap()
}

#[tokio::test]
async fn complete_legacy_happy_path() {
    let server = MockServer::start().await;
    let expected_body = serde_json::json!({
        "model":"openai/gpt-3.5-turbo-instruct",
        "prompt":"once upon a time",
        "stream": false
    });
    let resp_body = serde_json::json!({
        "id":"cmp-1",
        "model":"openai/gpt-3.5-turbo-instruct",
        "choices":[{
            "index":0,
            "text":" there was a frog",
            "finish_reason":"length"
        }],
        "usage":{"prompt_tokens":4,"completion_tokens":5,"total_tokens":9}
    });
    Mock::given(method("POST"))
        .and(path("/completions"))
        .and(body_json(&expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_json(&resp_body))
        .mount(&server)
        .await;

    let c = client_for(&server);
    let req = CompletionRequest {
        model: "openai/gpt-3.5-turbo-instruct".into(),
        prompt: "once upon a time".into(),
        ..Default::default()
    };
    let resp = c.complete(req).await.unwrap();
    assert_eq!(resp.id.as_deref(), Some("cmp-1"));
    assert_eq!(resp.choices.len(), 1);
    assert_eq!(resp.choices[0].text, " there was a frog");
}

#[tokio::test]
async fn complete_forces_stream_false() {
    let server = MockServer::start().await;
    let expected_body = serde_json::json!({
        "model":"m",
        "prompt":"p",
        "stream": false
    });
    Mock::given(method("POST"))
        .and(path("/completions"))
        .and(body_json(&expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id":"x","model":"m","choices":[]
        })))
        .mount(&server)
        .await;

    let c = client_for(&server);
    let req = CompletionRequest {
        model: "m".into(),
        prompt: "p".into(),
        stream: Some(true),
        ..Default::default()
    };
    c.complete(req).await.unwrap();
}

#[tokio::test]
async fn complete_serializes_provider_routing() {
    let server = MockServer::start().await;
    let expected_body = serde_json::json!({
        "model":"m","prompt":"p","stream":false,
        "provider":{"order":["openai"]}
    });
    Mock::given(method("POST"))
        .and(path("/completions"))
        .and(body_json(&expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id":"x","model":"m","choices":[]
        })))
        .mount(&server)
        .await;
    let c = client_for(&server);
    let req = CompletionRequest {
        model: "m".into(),
        prompt: "p".into(),
        provider: Some(Provider {
            order: Some(vec!["openai".into()]),
            ..Default::default()
        }),
        ..Default::default()
    };
    c.complete(req).await.unwrap();
}
