//! Cloned-`Client` concurrency tests.
//!
//! `Client` is `Clone`-cheap (`Arc<ClientInner>`); these tests assert that
//! multiple cloned handles share the same underlying `reqwest::Client` and
//! can drive concurrent requests from independent tasks without interference.

use std::time::Duration;

use openrouter::{ChatCompletionRequest, Client, Error, Message};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client_for(server: &MockServer) -> Client {
    Client::builder()
        .api_key("sk-test")
        .base_url(server.uri())
        .unwrap()
        .retry(2, Duration::from_millis(1))
        .build()
        .unwrap()
}

#[tokio::test]
async fn cloned_clients_share_http_pool() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id":"g","model":"x/y","choices":[]
        })))
        .mount(&server)
        .await;

    let c = client_for(&server);
    let a = c.clone();
    let b = c.clone();

    // Two cheap clones should point at the same `reqwest::Client` instance.
    let a_ptr = a.http() as *const _;
    let b_ptr = b.http() as *const _;
    assert_eq!(a_ptr, b_ptr, "cloned clients must share the http pool");
}

#[tokio::test]
async fn many_cloned_clients_drive_concurrent_requests() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id":"g","model":"x/y","choices":[]
        })))
        .expect(16)
        .mount(&server)
        .await;

    let c = client_for(&server);
    let mut handles = Vec::with_capacity(16);
    for i in 0..16u32 {
        let cc = c.clone();
        handles.push(tokio::spawn(async move {
            let req = ChatCompletionRequest {
                model: "x/y".into(),
                messages: vec![Message::user(format!("hi-{i}"))],
                ..Default::default()
            };
            cc.chat_complete(req).await
        }));
    }
    for h in handles {
        let r = h.await.expect("task did not panic");
        assert!(r.is_ok(), "every concurrent request must succeed");
    }
}

#[tokio::test(start_paused = true)]
async fn concurrent_requests_each_observe_retry_independently() {
    // First request gets one 429 then succeeds; a second concurrent request
    // also hits the 429-once path. Each task must drive the retry independently.
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "0")
                .set_body_json(serde_json::json!({"error":{"message":"slow down"}})),
        )
        .up_to_n_times(2)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id":"g","model":"x/y","choices":[]
        })))
        .mount(&server)
        .await;

    let c = client_for(&server);
    let c1 = c.clone();
    let c2 = c.clone();
    let req = || ChatCompletionRequest {
        model: "x/y".into(),
        messages: vec![Message::user("hi")],
        ..Default::default()
    };

    let h1 = tokio::spawn(async move { c1.chat_complete(req()).await });
    let h2 = tokio::spawn(async move { c2.chat_complete(req()).await });

    let (r1, r2) = tokio::join!(h1, h2);
    assert!(r1.unwrap().is_ok());
    assert!(r2.unwrap().is_ok());
}

#[tokio::test(start_paused = true)]
async fn concurrent_503s_each_exhaust_retries() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(503).set_body_string("nope"))
        .mount(&server)
        .await;

    let c = client_for(&server);
    let c1 = c.clone();
    let c2 = c.clone();
    let req = || ChatCompletionRequest {
        model: "x/y".into(),
        messages: vec![Message::user("hi")],
        ..Default::default()
    };
    let h1 = tokio::spawn(async move { c1.chat_complete(req()).await });
    let h2 = tokio::spawn(async move { c2.chat_complete(req()).await });
    let (r1, r2) = tokio::join!(h1, h2);
    for r in [r1.unwrap(), r2.unwrap()] {
        match r.unwrap_err() {
            Error::RetryExhausted { source, .. } => {
                assert!(matches!(*source, Error::Api { status: 503, .. }));
            }
            other => panic!("expected RetryExhausted, got {other:?}"),
        }
    }
}
