//! Wiremock tests for `POST /rerank` (HRA-146).

use std::time::Duration;

use openrouter::{Client, Error, Provider, RerankRequest};
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
async fn rerank_round_trip() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/rerank"))
        .and(header("authorization", "Bearer sk-test"))
        .and(body_json(json!({
            "model": "cohere/rerank-v3.5",
            "query": "How do I bake bread?",
            "documents": ["bread recipe", "car repair", "yeast"],
            "top_n": 2,
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "orid_01",
            "model": "cohere/rerank-v3.5",
            "provider": "cohere",
            "results": [
                {"index": 0, "relevance_score": 0.92, "document": {"text": "bread recipe"}},
                {"index": 2, "relevance_score": 0.41, "document": {"text": "yeast"}},
            ],
            "usage": {"total_tokens": 0, "search_units": 1}
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client
        .rerank(&RerankRequest {
            model: "cohere/rerank-v3.5".into(),
            query: "How do I bake bread?".into(),
            documents: vec!["bread recipe".into(), "car repair".into(), "yeast".into()],
            top_n: Some(2),
            provider: None,
        })
        .await
        .unwrap();
    assert_eq!(resp.results.len(), 2);
    assert_eq!(resp.results[0].index, 0);
    assert!(resp.results[0].relevance_score > resp.results[1].relevance_score);
    let usage = resp.usage.expect("usage present");
    assert_eq!(usage.search_units, 1);
}

#[tokio::test]
async fn rerank_with_provider_routing() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/rerank"))
        .and(body_json(json!({
            "model": "cohere/rerank-v3.5",
            "query": "q",
            "documents": ["d"],
            "provider": {"only": ["cohere"]},
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "orid_02",
            "model": "cohere/rerank-v3.5",
            "provider": "cohere",
            "results": []
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    client
        .rerank(&RerankRequest {
            model: "cohere/rerank-v3.5".into(),
            query: "q".into(),
            documents: vec!["d".into()],
            provider: Some(Provider {
                only: Some(vec!["cohere".into()]),
                ..Provider::default()
            }),
            ..Default::default()
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn rerank_validates_required_fields() {
    let client = Client::new("sk").unwrap();
    let err = client
        .rerank(&RerankRequest {
            model: "".into(),
            query: "q".into(),
            documents: vec!["d".into()],
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert!(matches!(err, Error::InvalidInput(_)));

    let err = client
        .rerank(&RerankRequest {
            model: "m".into(),
            query: "".into(),
            documents: vec!["d".into()],
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert!(matches!(err, Error::InvalidInput(_)));

    let err = client
        .rerank(&RerankRequest {
            model: "m".into(),
            query: "q".into(),
            documents: vec![],
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert!(matches!(err, Error::InvalidInput(_)));
}
