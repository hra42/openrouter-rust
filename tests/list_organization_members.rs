//! Wiremock tests for `GET /organization/members` (HRA-144).

use std::time::Duration;

use openrouter::{Client, ListOrganizationMembersOptions, OrganizationMemberRole};
use pretty_assertions::assert_eq;
use serde_json::json;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client_for(server: &MockServer) -> Client {
    Client::builder()
        .api_key("sk-prov-test")
        .base_url(server.uri())
        .unwrap()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap()
}

#[tokio::test]
async fn list_org_members_passes_pagination_query() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/organization/members"))
        .and(query_param("offset", "5"))
        .and(query_param("limit", "2"))
        .and(header("authorization", "Bearer sk-prov-test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {
                    "id": "usr_1",
                    "email": "alice@example.com",
                    "first_name": "Alice",
                    "last_name": null,
                    "role": "org:admin",
                },
                {
                    "id": "usr_2",
                    "email": "bob@example.com",
                    "first_name": null,
                    "last_name": "Smith",
                    "role": "org:member",
                }
            ],
            "total_count": 17,
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let opts = ListOrganizationMembersOptions::new().offset(5).limit(2);
    let resp = client.list_organization_members(Some(&opts)).await.unwrap();
    assert_eq!(resp.data.len(), 2);
    assert_eq!(resp.total_count, 17);
    assert_eq!(resp.data[0].role, OrganizationMemberRole::Admin);
    assert_eq!(resp.data[1].role, OrganizationMemberRole::Member);
    assert_eq!(resp.data[0].first_name.as_deref(), Some("Alice"));
    assert!(resp.data[0].last_name.is_none());
}

#[tokio::test]
async fn list_org_members_no_options_no_query() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/organization/members"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"data": [], "total_count": 0})),
        )
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client.list_organization_members(None).await.unwrap();
    assert!(resp.data.is_empty());
    assert_eq!(resp.total_count, 0);
}
