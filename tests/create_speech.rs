//! Wiremock tests for `POST /audio/speech` (HRA-147).

use std::time::Duration;

use openrouter::{Client, Error, SpeechFormat, SpeechRequest};
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
async fn create_speech_returns_audio_bytes_and_content_type() {
    let server = MockServer::start().await;
    let audio_payload = b"FAKE-MP3-BYTES".to_vec();
    Mock::given(method("POST"))
        .and(path("/audio/speech"))
        .and(header("authorization", "Bearer sk-test"))
        .and(body_json(json!({
            "input": "Hello world",
            "model": "openai/tts-1",
            "voice": "alloy",
            "response_format": "mp3",
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(audio_payload.clone())
                .insert_header("content-type", "audio/mpeg"),
        )
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client
        .create_speech(&SpeechRequest {
            input: "Hello world".into(),
            model: "openai/tts-1".into(),
            voice: "alloy".into(),
            response_format: Some(SpeechFormat::Mp3),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(&resp.audio[..], audio_payload.as_slice());
    assert_eq!(resp.content_type.as_deref(), Some("audio/mpeg"));
    assert_eq!(resp.format, SpeechFormat::Mp3);
}

#[tokio::test]
async fn create_speech_defaults_format_to_pcm_when_unset() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/audio/speech"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![0u8; 8]))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client
        .create_speech(&SpeechRequest {
            input: "Hi".into(),
            model: "m".into(),
            voice: "v".into(),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(resp.format, SpeechFormat::Pcm);
}

#[tokio::test]
async fn create_speech_validates_inputs() {
    let client = Client::new("sk").unwrap();
    let err = client
        .create_speech(&SpeechRequest {
            input: "".into(),
            model: "m".into(),
            voice: "v".into(),
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert!(matches!(err, Error::InvalidInput(_)));
    let err = client
        .create_speech(&SpeechRequest {
            input: "i".into(),
            model: "".into(),
            voice: "v".into(),
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert!(matches!(err, Error::InvalidInput(_)));
    let err = client
        .create_speech(&SpeechRequest {
            input: "i".into(),
            model: "m".into(),
            voice: "".into(),
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert!(matches!(err, Error::InvalidInput(_)));
}
