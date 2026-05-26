//! Wiremock test for `Client::get_credits` (HRA-139).

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
async fn get_credits_parses_payload_and_computes_remaining() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/credits"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"{"data":{"total_credits":50.0,"total_usage":12.34}}"#),
        )
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client.get_credits().await.unwrap();
    assert_eq!(resp.data.total_credits, 50.0);
    assert_eq!(resp.data.total_usage, 12.34);
    assert!((resp.data.remaining() - 37.66).abs() < 1e-9);
}
