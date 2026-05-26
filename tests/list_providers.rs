//! Wiremock test for `Client::list_providers` (HRA-138).

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
async fn list_providers_parses_typical_payload() {
    let server = MockServer::start().await;
    let body = r#"{
        "data": [
            {
                "name": "Google",
                "slug": "google",
                "privacy_policy_url": "https://policies.google.com/privacy",
                "terms_of_service_url": "https://policies.google.com/terms",
                "status_page_url": "https://status.cloud.google.com/"
            },
            {
                "name": "Anthropic",
                "slug": "anthropic",
                "privacy_policy_url": null,
                "terms_of_service_url": null,
                "status_page_url": null
            }
        ]
    }"#;
    Mock::given(method("GET"))
        .and(path("/providers"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client.list_providers().await.unwrap();
    assert_eq!(resp.data.len(), 2);
    assert_eq!(resp.data[0].slug, "google");
    assert_eq!(
        resp.data[0].privacy_policy_url.as_deref(),
        Some("https://policies.google.com/privacy")
    );
    assert_eq!(resp.data[1].slug, "anthropic");
    assert!(resp.data[1].status_page_url.is_none());
}
