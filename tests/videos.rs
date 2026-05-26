//! Wiremock tests for the video generation endpoints (HRA-148).

use std::time::Duration;

use openrouter::{
    Client, Error, VideoAspectRatio, VideoGenerationRequest, VideoResolution, VideoStatus,
};
use pretty_assertions::assert_eq;
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
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
async fn create_video_round_trip() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/videos"))
        .and(body_json(json!({
            "model": "google/veo",
            "prompt": "A cat surfing",
            "aspect_ratio": "16:9",
            "resolution": "720p",
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "vjob_01",
            "polling_url": "https://or.example/videos/vjob_01",
            "status": "pending",
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client
        .create_video(&VideoGenerationRequest {
            model: "google/veo".into(),
            prompt: "A cat surfing".into(),
            aspect_ratio: Some(VideoAspectRatio::R16x9),
            resolution: Some(VideoResolution::P720),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(resp.id, "vjob_01");
    assert_eq!(resp.status, VideoStatus::Pending);
}

#[tokio::test]
async fn create_video_validates_inputs() {
    let client = Client::new("sk").unwrap();
    let err = client
        .create_video(&VideoGenerationRequest {
            model: "".into(),
            prompt: "p".into(),
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert!(matches!(err, Error::InvalidInput(_)));
    let err = client
        .create_video(&VideoGenerationRequest {
            model: "m".into(),
            prompt: "".into(),
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert!(matches!(err, Error::InvalidInput(_)));
}

#[tokio::test]
async fn get_video_returns_status() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/videos/vjob_01"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "vjob_01",
            "polling_url": "https://or.example/videos/vjob_01",
            "status": "completed",
            "generation_id": "gen_99",
            "unsigned_urls": ["https://or.example/v.mp4"],
            "usage": {"cost": 0.42, "is_byok": false}
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client.get_video("vjob_01").await.unwrap();
    assert_eq!(resp.status, VideoStatus::Completed);
    assert!(resp.status.is_terminal());
    assert_eq!(resp.unsigned_urls.len(), 1);
    assert_eq!(resp.usage.unwrap().cost, Some(0.42));
}

#[tokio::test]
async fn get_video_content_default_index_omits_query() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/videos/vjob_01/content"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(b"VIDEO-BYTES".to_vec())
                .insert_header("content-type", "video/mp4"),
        )
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client.get_video_content("vjob_01", 0).await.unwrap();
    assert_eq!(&resp.content[..], b"VIDEO-BYTES");
    assert_eq!(resp.content_type.as_deref(), Some("video/mp4"));
}

#[tokio::test]
async fn get_video_content_nonzero_index_sends_query() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/videos/vjob_01/content"))
        .and(query_param("index", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![0u8; 4]))
        .mount(&server)
        .await;

    let client = client_for(&server);
    client.get_video_content("vjob_01", 2).await.unwrap();
}

#[tokio::test]
async fn list_video_models() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/videos/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [{
                "id": "google/veo",
                "name": "Veo",
                "canonical_slug": "google/veo",
                "created": 1717000000,
                "description": "",
                "allowed_passthrough_parameters": [],
                "supported_aspect_ratios": ["16:9", "9:16"],
                "supported_durations": [4, 8],
                "supported_frame_images": ["first_frame"],
                "supported_resolutions": ["720p", "1080p"],
                "supported_sizes": ["1280x720"],
                "pricing_skus": {"per_second": "0.10"},
            }]
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client.list_video_models().await.unwrap();
    assert_eq!(resp.data.len(), 1);
    assert_eq!(resp.data[0].id, "google/veo");
    assert_eq!(resp.data[0].supported_aspect_ratios.len(), 2);
}

#[tokio::test]
async fn wait_for_video_polls_until_terminal() {
    let server = MockServer::start().await;
    // First call returns in_progress; second call returns completed.
    Mock::given(method("GET"))
        .and(path("/videos/vjob_01"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "vjob_01",
            "polling_url": "u",
            "status": "in_progress",
        })))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/videos/vjob_01"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "vjob_01",
            "polling_url": "u",
            "status": "completed",
            "generation_id": "gen_1",
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client
        .wait_for_video("vjob_01", Duration::from_millis(5))
        .await
        .unwrap();
    assert_eq!(resp.status, VideoStatus::Completed);
}
