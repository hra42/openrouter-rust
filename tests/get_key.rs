//! Wiremock test for `Client::get_key` (HRA-141).

use std::time::Duration;

use openrouter::Client;
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
async fn get_key_parses_typical_payload() {
    let server = MockServer::start().await;
    let body = r#"{
        "data": {
            "label": "primary",
            "limit": 100.0,
            "usage": 3.5,
            "is_free_tier": false,
            "limit_remaining": 96.5,
            "is_provisioning_key": false,
            "rate_limit": {"interval": "10s", "requests": 20}
        }
    }"#;
    Mock::given(method("GET"))
        .and(path("/key"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client.get_key().await.unwrap();
    assert_eq!(resp.data.label, "primary");
    assert_eq!(resp.data.limit, Some(100.0));
    assert_eq!(resp.data.usage, 3.5);
    assert_eq!(resp.data.limit_remaining, Some(96.5));
    assert!(!resp.data.is_provisioning_key);
    let rate = resp.data.rate_limit.unwrap();
    assert_eq!(rate.interval, "10s");
    assert_eq!(rate.requests, 20.0);
}

#[tokio::test]
async fn get_key_handles_unlimited_key() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/key"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{"data":{"label":"prov","limit":null,"usage":0.0,"is_free_tier":false,"limit_remaining":null,"is_provisioning_key":true}}"#,
        ))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client.get_key().await.unwrap();
    assert!(resp.data.limit.is_none());
    assert!(resp.data.limit_remaining.is_none());
    assert!(resp.data.is_provisioning_key);
}
