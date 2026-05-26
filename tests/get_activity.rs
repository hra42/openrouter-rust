//! Wiremock test for `Client::get_activity` (HRA-140).

use std::time::Duration;

use openrouter::{ActivityOptions, Client};
use pretty_assertions::assert_eq;
use wiremock::matchers::{method, path, query_param};
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
async fn get_activity_with_date_filter_parses_payload() {
    let server = MockServer::start().await;
    let body = r#"{
        "data": [
            {
                "date": "2026-05-25 00:00:00",
                "model": "google/gemini-3.1-flash-lite",
                "model_permaslug": "google-gemini-3-1-flash-lite",
                "endpoint_id": "ep_123",
                "provider_name": "Google",
                "usage": 0.0123,
                "byok_usage_inference": 0.0,
                "requests": 5.0,
                "prompt_tokens": 1024.0,
                "completion_tokens": 256.0,
                "reasoning_tokens": 0.0
            }
        ]
    }"#;
    Mock::given(method("GET"))
        .and(path("/activity"))
        .and(query_param("date", "2026-05-25"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let opts = ActivityOptions::new().date("2026-05-25");
    let resp = client.get_activity(Some(&opts)).await.unwrap();
    assert_eq!(resp.data.len(), 1);
    let row = &resp.data[0];
    assert_eq!(row.model, "google/gemini-3.1-flash-lite");
    assert_eq!(row.provider_name, "Google");
    assert_eq!(row.requests, 5.0);
    assert_eq!(row.prompt_tokens, 1024.0);
}

#[tokio::test]
async fn get_activity_without_options_sends_no_query() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/activity"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"data":[]}"#))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client.get_activity(None).await.unwrap();
    assert!(resp.data.is_empty());
}
