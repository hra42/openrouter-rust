//! HTTP-level integration tests for `Client::chat_complete`.
//!
//! Uses `wiremock` to stub OpenRouter responses. Retry/backoff tests use
//! `start_paused = true` so `tokio::time::sleep` is virtual.

use std::time::Duration;

use openrouter::{ChatCompletionRequest, Client, Error, Message};
use pretty_assertions::assert_eq;
use wiremock::matchers::{body_json, header, method, path};
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
async fn chat_complete_happy_path() {
    let server = MockServer::start().await;
    let body = serde_json::json!({
        "id": "gen-1",
        "object": "chat.completion",
        "created": 1700000000_u64,
        "model": "anthropic/claude-3-opus",
        "choices": [{
            "index": 0,
            "message": {"role":"assistant","content":"hi back"},
            "finish_reason":"stop"
        }],
        "usage": {"prompt_tokens":3,"completion_tokens":2,"total_tokens":5}
    });
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(header("authorization", "Bearer sk-test"))
        .and(header("content-type", "application/json"))
        .and(header("accept", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let c = client_for(&server);
    let req = ChatCompletionRequest {
        model: "anthropic/claude-3-opus".into(),
        messages: vec![Message::user("hi")],
        ..Default::default()
    };
    let resp = c.chat_complete(req).await.unwrap();
    assert_eq!(resp.id.as_deref(), Some("gen-1"));
    assert_eq!(resp.model, "anthropic/claude-3-opus");
    assert_eq!(resp.choices.len(), 1);
    assert_eq!(
        resp.choices[0].message.as_ref().unwrap().content_text(),
        Some("hi back")
    );
    assert_eq!(resp.usage.as_ref().unwrap().total_tokens, Some(5));
}

#[tokio::test]
async fn chat_complete_forces_stream_false() {
    let server = MockServer::start().await;
    let expected_body = serde_json::json!({
        "model":"x/y",
        "messages":[{"role":"user","content":"hi"}],
        "stream": false
    });
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(body_json(&expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id":"g","model":"x/y","choices":[]
        })))
        .mount(&server)
        .await;

    let c = client_for(&server);
    let req = ChatCompletionRequest {
        model: "x/y".into(),
        messages: vec![Message::user("hi")],
        stream: Some(true),
        ..Default::default()
    };
    c.chat_complete(req).await.unwrap();
}

#[tokio::test]
async fn chat_complete_attribution_headers() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(header("http-referer", "https://demo.example"))
        .and(header("x-title", "demo-app"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id":"g","model":"x/y","choices":[]
        })))
        .mount(&server)
        .await;

    let c = Client::builder()
        .api_key("sk-test")
        .base_url(server.uri())
        .unwrap()
        .app_name("demo-app")
        .referer("https://demo.example")
        .build()
        .unwrap();
    let req = ChatCompletionRequest {
        model: "x/y".into(),
        messages: vec![Message::user("hi")],
        ..Default::default()
    };
    c.chat_complete(req).await.unwrap();
}

#[tokio::test]
async fn chat_complete_400_is_not_retried() {
    let server = MockServer::start().await;
    let body = serde_json::json!({
        "error": {"code":"invalid_request_error","message":"bad model"}
    });
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(400).set_body_json(&body))
        .expect(1) // not retried
        .mount(&server)
        .await;

    let c = client_for(&server);
    let req = ChatCompletionRequest {
        model: "x/y".into(),
        messages: vec![Message::user("hi")],
        ..Default::default()
    };
    let err = c.chat_complete(req).await.unwrap_err();
    match err {
        Error::Api {
            status,
            code,
            message,
            ..
        } => {
            assert_eq!(status, 400);
            assert_eq!(code.as_deref(), Some("invalid_request_error"));
            assert_eq!(message, "bad model");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test(start_paused = true)]
async fn chat_complete_503_retries_then_exhausts() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(503).set_body_string("upstream gone"))
        .expect(3) // initial + 2 retries
        .mount(&server)
        .await;

    let c = client_for(&server);
    let req = ChatCompletionRequest {
        model: "x/y".into(),
        messages: vec![Message::user("hi")],
        ..Default::default()
    };
    let err = c.chat_complete(req).await.unwrap_err();
    match err {
        Error::RetryExhausted { attempts, source } => {
            assert_eq!(attempts, 3);
            assert!(matches!(*source, Error::Api { status: 503, .. }));
        }
        other => panic!("expected RetryExhausted, got {other:?}"),
    }
}

#[tokio::test(start_paused = true)]
async fn chat_complete_429_then_success() {
    let server = MockServer::start().await;
    let ok_body = serde_json::json!({"id":"g","model":"x/y","choices":[]});
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "1")
                .set_body_json(serde_json::json!({"error":{"message":"slow down"}})),
        )
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&ok_body))
        .mount(&server)
        .await;

    let c = client_for(&server);
    let req = ChatCompletionRequest {
        model: "x/y".into(),
        messages: vec![Message::user("hi")],
        ..Default::default()
    };
    let resp = c.chat_complete(req).await.unwrap();
    assert_eq!(resp.id.as_deref(), Some("g"));
}
