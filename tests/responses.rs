//! Wiremock tests for the beta Responses API (HRA-151). Only built when
//! the `beta` cargo feature is enabled.

#![cfg(feature = "beta")]

use std::time::Duration;

use futures::StreamExt;
use openrouter::responses::{
    reasoning_effort, ResponsesInputItem, ResponsesRequest, ResponsesTool,
};
use openrouter::{Client, Error};
use pretty_assertions::assert_eq;
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path};
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
async fn create_response_string_input_round_trip() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/responses"))
        .and(header("authorization", "Bearer sk-test"))
        .and(body_json(json!({
            "model": "openai/o4-mini",
            "input": "Hello world",
            "stream": false,
            "max_output_tokens": 100,
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "resp_01",
            "object": "response",
            "created_at": 1700000000,
            "model": "openai/o4-mini",
            "output": [{
                "type": "message",
                "id": "msg_01",
                "role": "assistant",
                "content": [{"type": "output_text", "text": "hi back"}]
            }],
            "usage": {
                "input_tokens": 5,
                "output_tokens": 2,
                "total_tokens": 7,
            },
            "status": "completed"
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client
        .create_response(
            ResponsesRequest::new("openai/o4-mini")
                .input("Hello world")
                .max_output_tokens(100),
        )
        .await
        .unwrap();
    assert_eq!(resp.id, "resp_01");
    assert_eq!(resp.text_content(), "hi back");
    assert_eq!(resp.usage.total_tokens, 7);
}

#[tokio::test]
async fn create_response_structured_input() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/responses"))
        .and(body_json(json!({
            "model": "openai/o4-mini",
            "input": [
                {"type": "message", "role": "system",
                 "content": [{"type": "input_text", "text": "be brief"}]},
                {"type": "message", "role": "user",
                 "content": [{"type": "input_text", "text": "hi"}]}
            ],
            "stream": false,
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "r", "object": "response", "created_at": 0, "model": "x",
            "output": [], "usage": {"input_tokens":0,"output_tokens":0,"total_tokens":0},
            "status": "completed"
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    client
        .create_response(ResponsesRequest::new("openai/o4-mini").input(vec![
            ResponsesInputItem::system("be brief"),
            ResponsesInputItem::user("hi"),
        ]))
        .await
        .unwrap();
}

#[tokio::test]
async fn create_response_with_tools_and_reasoning() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/responses"))
        .and(body_json(json!({
            "model": "openai/o4-mini",
            "input": "weather please",
            "stream": false,
            "reasoning": {"effort": "high"},
            "tools": [{
                "type": "function",
                "name": "get_weather",
                "description": "look up weather",
                "strict": null,
                "parameters": {"type":"object","properties":{"city":{"type":"string"}}}
            }],
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"r","object":"response","created_at":0,"model":"x",
            "output":[{
                "type":"function_call","id":"fc_1","call_id":"call_1",
                "name":"get_weather","arguments":"{\"city\":\"berlin\"}"
            }],
            "usage":{"input_tokens":0,"output_tokens":0,"total_tokens":0},
            "status":"completed"
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client
        .create_response(
            ResponsesRequest::new("openai/o4-mini")
                .input("weather please")
                .reasoning_effort(reasoning_effort::HIGH)
                .tools([ResponsesTool::function(
                    "get_weather",
                    "look up weather",
                    json!({"type":"object","properties":{"city":{"type":"string"}}}),
                )]),
        )
        .await
        .unwrap();
    let calls = resp.function_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "get_weather");
}

#[tokio::test]
async fn validation_rejects_empty_text() {
    let client = Client::new("sk").unwrap();
    let err = client
        .create_response(ResponsesRequest::new("m").input(""))
        .await
        .unwrap_err();
    assert!(matches!(err, Error::InvalidInput(_)));
}

#[tokio::test]
async fn validation_rejects_bad_effort() {
    let client = Client::new("sk").unwrap();
    let mut req = ResponsesRequest::new("m").input("hi");
    req.reasoning = Some(openrouter::responses::ResponsesReasoning {
        effort: "absurd".into(),
    });
    let err = client.create_response(req).await.unwrap_err();
    assert!(matches!(err, Error::InvalidInput(_)));
}

#[tokio::test]
async fn create_response_stream_yields_chunks() {
    let server = MockServer::start().await;
    let sse_body = concat!(
        "data: {\"id\":\"r\",\"object\":\"response\",\"created_at\":0,\"model\":\"x\",\"output\":[{\"type\":\"message\",\"id\":\"m\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"he\"}]}],\"usage\":{\"input_tokens\":0,\"output_tokens\":0,\"total_tokens\":0},\"status\":\"in_progress\"}\n\n",
        "data: {\"id\":\"r\",\"object\":\"response\",\"created_at\":0,\"model\":\"x\",\"output\":[{\"type\":\"message\",\"id\":\"m\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"llo\"}]}],\"usage\":{\"input_tokens\":0,\"output_tokens\":2,\"total_tokens\":2},\"status\":\"completed\"}\n\n",
        "data: [DONE]\n\n",
    );
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(sse_body)
                .insert_header("content-type", "text/event-stream"),
        )
        .mount(&server)
        .await;

    let client = client_for(&server);
    let mut stream = client
        .create_response_stream(ResponsesRequest::new("x").input("hi"))
        .await
        .unwrap();
    let mut total = String::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.unwrap();
        total.push_str(chunk.text_content());
    }
    assert_eq!(total, "hello");
}
