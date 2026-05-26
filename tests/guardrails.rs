//! Wiremock tests for the guardrails + ZDR endpoints (HRA-145).

use std::time::Duration;

use openrouter::{
    AssignKeysRequest, AssignMembersRequest, Client, CreateGuardrailRequest, Error,
    ListGuardrailsOptions, ResetInterval, UpdateGuardrailRequest,
};
use pretty_assertions::assert_eq;
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
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

fn sample_guardrail() -> serde_json::Value {
    json!({
        "id": "gr_01HZX",
        "name": "team-budget",
        "description": "monthly team cap",
        "limit_usd": 200.0,
        "reset_interval": "monthly",
        "allowed_providers": ["openai", "anthropic"],
        "allowed_models": [],
        "enforce_zdr": true,
        "created_at": "2026-05-01T00:00:00Z",
        "updated_at": null,
    })
}

#[tokio::test]
async fn list_guardrails_passes_pagination() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/guardrails"))
        .and(query_param("offset", "10"))
        .and(query_param("limit", "5"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"data": [sample_guardrail()], "total_count": 1})),
        )
        .mount(&server)
        .await;

    let client = client_for(&server);
    let opts = ListGuardrailsOptions::new().offset(10).limit(5);
    let resp = client.list_guardrails(Some(&opts)).await.unwrap();
    assert_eq!(resp.data.len(), 1);
    assert_eq!(resp.data[0].reset_interval, Some(ResetInterval::Monthly),);
}

#[tokio::test]
async fn create_guardrail_round_trip() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/guardrails"))
        .and(body_json(json!({
            "name": "team-budget",
            "limit_usd": 200.0,
            "reset_interval": "monthly",
            "enforce_zdr": true,
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_guardrail()))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client
        .create_guardrail(&CreateGuardrailRequest {
            name: "team-budget".into(),
            limit_usd: Some(200.0),
            reset_interval: Some(ResetInterval::Monthly),
            enforce_zdr: Some(true),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(resp.id, "gr_01HZX");
}

#[tokio::test]
async fn create_guardrail_requires_name() {
    let client = Client::new("sk").unwrap();
    let err = client
        .create_guardrail(&CreateGuardrailRequest::default())
        .await
        .unwrap_err();
    assert!(matches!(err, Error::InvalidInput(_)));
}

#[tokio::test]
async fn get_update_delete_round_trip() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/guardrails/gr_01HZX"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_guardrail()))
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/guardrails/gr_01HZX"))
        .and(body_json(json!({"description": "renamed"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_guardrail()))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/guardrails/gr_01HZX"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"deleted": true})))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let g = client.get_guardrail("gr_01HZX").await.unwrap();
    assert_eq!(g.name, "team-budget");
    client
        .update_guardrail(
            "gr_01HZX",
            &UpdateGuardrailRequest {
                description: Some("renamed".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let del = client.delete_guardrail("gr_01HZX").await.unwrap();
    assert!(del.deleted);
}

#[tokio::test]
async fn assign_and_unassign_keys() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/guardrails/gr_01HZX/key-assignments"))
        .and(body_json(json!({"key_hashes": ["h1", "h2"]})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"assigned_count": 2})))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/guardrails/gr_01HZX/key-assignments"))
        .and(body_json(json!({"key_hashes": ["h1"]})))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client
        .assign_keys_to_guardrail(
            "gr_01HZX",
            &AssignKeysRequest {
                key_hashes: vec!["h1".into(), "h2".into()],
            },
        )
        .await
        .unwrap();
    assert_eq!(resp.assigned_count, 2);
    client
        .unassign_keys_from_guardrail(
            "gr_01HZX",
            &AssignKeysRequest {
                key_hashes: vec!["h1".into()],
            },
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn list_key_assignments_for_guardrail() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/guardrails/gr_01HZX/key-assignments"))
        .and(query_param("limit", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [{
                "id": "ka_1",
                "key_hash": "h1",
                "organization_id": "org_1",
                "guardrail_id": "gr_01HZX",
                "assigned_by": "usr_admin",
                "created_at": "2026-05-26T00:00:00Z",
            }],
            "total_count": 1,
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let opts = ListGuardrailsOptions::new().limit(10);
    let resp = client
        .list_guardrail_key_assignments("gr_01HZX", Some(&opts))
        .await
        .unwrap();
    assert_eq!(resp.data[0].key_hash, "h1");
}

#[tokio::test]
async fn list_all_member_assignments() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/guardrails/member-assignments"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"data": [], "total_count": 0})),
        )
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client
        .list_all_guardrail_member_assignments(None)
        .await
        .unwrap();
    assert!(resp.data.is_empty());
}

#[tokio::test]
async fn assign_and_unassign_members() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/guardrails/gr_01HZX/member-assignments"))
        .and(body_json(json!({"member_user_ids": ["usr_1"]})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"assigned_count": 1})))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/guardrails/gr_01HZX/member-assignments"))
        .and(body_json(json!({"member_user_ids": ["usr_1"]})))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client
        .assign_members_to_guardrail(
            "gr_01HZX",
            &AssignMembersRequest {
                member_user_ids: vec!["usr_1".into()],
            },
        )
        .await
        .unwrap();
    assert_eq!(resp.assigned_count, 1);
    client
        .unassign_members_from_guardrail(
            "gr_01HZX",
            &AssignMembersRequest {
                member_user_ids: vec!["usr_1".into()],
            },
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn assignment_endpoints_reject_empty_inputs() {
    let client = Client::new("sk").unwrap();
    let err = client
        .assign_keys_to_guardrail("gr", &AssignKeysRequest { key_hashes: vec![] })
        .await
        .unwrap_err();
    assert!(matches!(err, Error::InvalidInput(_)));

    let err = client
        .assign_members_to_guardrail(
            "",
            &AssignMembersRequest {
                member_user_ids: vec!["usr_1".into()],
            },
        )
        .await
        .unwrap_err();
    assert!(matches!(err, Error::InvalidInput(_)));
}

#[tokio::test]
async fn list_zdr_endpoints() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/endpoints/zdr"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [{
                "name": "OpenAI GPT-5 (ZDR)",
                "model_id": "openai/gpt-5",
                "model_name": "GPT-5",
                "context_length": 200000,
                "pricing": {
                    "prompt": "0.000005",
                    "completion": "0.000015",
                },
                "provider_name": "openai",
                "tag": "stable",
                "quantization": null,
                "max_completion_tokens": 8192,
                "max_prompt_tokens": null,
                "supported_parameters": ["tools", "json_schema"],
                "status": 1,
                "uptime_last_30m": 0.999,
                "supports_implicit_caching": true,
                "latency_last_30m": {"p50": 200.0, "p75": 300.0, "p90": 500.0, "p99": 1200.0},
                "throughput_last_30m": null
            }]
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client.list_zdr_endpoints().await.unwrap();
    assert_eq!(resp.data.len(), 1);
    assert_eq!(resp.data[0].model_id, "openai/gpt-5");
    assert!(resp.data[0].latency_last_30m.is_some());
    assert!(resp.data[0].throughput_last_30m.is_none());
}
