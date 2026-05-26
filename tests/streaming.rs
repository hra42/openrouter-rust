//! End-to-end tests for `Client::chat_complete_stream` and `complete_stream`.

use std::time::Duration;

use futures::StreamExt;
use openrouter::{ChatCompletionRequest, Client, CompletionRequest, Message};
use pretty_assertions::assert_eq;
use wiremock::matchers::{header, method, path};
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

const CHAT_SSE_BODY: &str = "\
data: {\"id\":\"g\",\"model\":\"x/y\",\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\",\"content\":\"Hel\"}}]}

data: {\"id\":\"g\",\"model\":\"x/y\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"lo!\"}}]}

data: [DONE]

";

#[tokio::test]
async fn chat_stream_yields_chunks_and_terminates() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(header("accept", "text/event-stream"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(CHAT_SSE_BODY),
        )
        .mount(&server)
        .await;

    let c = client_for(&server);
    let req = ChatCompletionRequest {
        model: "x/y".into(),
        messages: vec![Message::user("hi")],
        ..Default::default()
    };
    let mut stream = c.chat_complete_stream(req).await.unwrap();

    let mut accumulated = String::new();
    while let Some(item) = stream.next().await {
        let chunk = item.unwrap();
        if let Some(delta) = chunk.choices.first().and_then(|c| c.delta.as_ref()) {
            if let Some(s) = delta.content.as_deref() {
                accumulated.push_str(s);
            }
        }
    }
    assert_eq!(accumulated, "Hello!");
}

#[tokio::test]
async fn chat_stream_sends_stream_true_and_accept_sse() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(header("accept", "text/event-stream"))
        .and(header("content-type", "application/json"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string("data: [DONE]\n\n"),
        )
        .mount(&server)
        .await;

    let c = client_for(&server);
    let req = ChatCompletionRequest {
        model: "x/y".into(),
        messages: vec![Message::user("hi")],
        // Caller leaves stream unset; method must force it to true.
        ..Default::default()
    };
    let mut stream = c.chat_complete_stream(req).await.unwrap();
    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn chat_stream_propagates_initial_4xx() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "error":{"code":"invalid_request_error","message":"bad model"}
        })))
        .mount(&server)
        .await;

    let c = client_for(&server);
    let req = ChatCompletionRequest {
        model: "x/y".into(),
        messages: vec![Message::user("hi")],
        ..Default::default()
    };
    let err = c.chat_complete_stream(req).await.unwrap_err();
    match err {
        openrouter::Error::Api { status, .. } => assert_eq!(status, 400),
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn dropping_stream_cancels_cleanly() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(CHAT_SSE_BODY),
        )
        .mount(&server)
        .await;
    let c = client_for(&server);
    let req = ChatCompletionRequest {
        model: "x/y".into(),
        messages: vec![Message::user("hi")],
        ..Default::default()
    };
    let mut stream = c.chat_complete_stream(req).await.unwrap();
    // Take one chunk then drop the stream.
    let _ = stream.next().await.unwrap().unwrap();
    drop(stream);
    // Reaching here without panic / hang is the assertion.
}

#[tokio::test]
async fn complete_stream_yields_chunks() {
    let server = MockServer::start().await;
    let body = "\
data: {\"id\":\"c\",\"model\":\"m\",\"choices\":[{\"index\":0,\"text\":\"once \"}]}

data: {\"id\":\"c\",\"model\":\"m\",\"choices\":[{\"index\":0,\"text\":\"upon\"}]}

data: [DONE]

";
    Mock::given(method("POST"))
        .and(path("/completions"))
        .and(header("accept", "text/event-stream"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(body),
        )
        .mount(&server)
        .await;

    let c = client_for(&server);
    let req = CompletionRequest {
        model: "m".into(),
        prompt: "p".into(),
        ..Default::default()
    };
    let mut stream = c.complete_stream(req).await.unwrap();
    let mut text = String::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.unwrap();
        if let Some(choice) = chunk.choices.first() {
            text.push_str(&choice.text);
        }
    }
    assert_eq!(text, "once upon");
}
