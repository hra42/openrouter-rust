//! Wiremock tests for `Client::list_model_endpoints` (HRA-137).

use std::time::Duration;

use openrouter::{Client, Error};
use pretty_assertions::assert_eq;
use wiremock::matchers::{method, path};
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
async fn list_model_endpoints_parses_typical_payload() {
    let server = MockServer::start().await;
    let body = r#"{
        "data": {
            "id": "google/gemini-3.1-flash-lite",
            "name": "Gemini 3.1 Flash Lite",
            "created": 1730000000.0,
            "description": "Fast.",
            "architecture": {
                "tokenizer": "Gemini",
                "instruct_type": null,
                "input_modalities": ["text"],
                "output_modalities": ["text"]
            },
            "endpoints": [
                {
                    "name": "Google | gemini-3.1-flash-lite",
                    "context_length": 1048576,
                    "pricing": {
                        "request": "0",
                        "image": "0",
                        "prompt": "0.0000001",
                        "completion": "0.0000004"
                    },
                    "provider_name": "Google",
                    "quantization": null,
                    "max_completion_tokens": 65536,
                    "max_prompt_tokens": null,
                    "supported_parameters": ["temperature"],
                    "status": 1,
                    "uptime_last_30m": 0.998
                }
            ]
        }
    }"#;
    Mock::given(method("GET"))
        .and(path("/models/google/gemini-3.1-flash-lite/endpoints"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client
        .list_model_endpoints("google", "gemini-3.1-flash-lite")
        .await
        .unwrap();
    assert_eq!(resp.data.id, "google/gemini-3.1-flash-lite");
    assert_eq!(resp.data.endpoints.len(), 1);
    let ep = &resp.data.endpoints[0];
    assert_eq!(ep.provider_name, "Google");
    assert_eq!(ep.pricing.prompt, "0.0000001");
    assert_eq!(ep.uptime_last_30m, Some(0.998));
}

#[tokio::test]
async fn list_model_endpoints_rejects_empty_author_or_slug() {
    let client = Client::builder().api_key("sk-test").build().unwrap();
    assert!(matches!(
        client.list_model_endpoints("", "x").await,
        Err(Error::InvalidInput(_))
    ));
    assert!(matches!(
        client.list_model_endpoints("x", "").await,
        Err(Error::InvalidInput(_))
    ));
}
