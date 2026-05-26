//! Types for the text-to-speech endpoint (`POST /audio/speech`).
//!
//! Shapes mirror the Go SDK (`speech_models.go`). Named `audio_speech`
//! rather than `audio` so it doesn't collide with the audio-input helpers
//! in [`crate::types::multimodal`].

use std::collections::BTreeMap;

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Audio output format requested from the TTS endpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpeechFormat {
    /// MP3 audio bytes.
    Mp3,
    /// Raw PCM samples (defaults upstream when no format is requested).
    Pcm,
}

impl SpeechFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            SpeechFormat::Mp3 => "mp3",
            SpeechFormat::Pcm => "pcm",
        }
    }
}

/// Request body for [`crate::Client::create_speech`].
///
/// `input`, `model`, and `voice` are required. `response_format` defaults
/// to PCM upstream when left unset.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SpeechRequest {
    /// Text to synthesize.
    pub input: String,
    /// TTS model identifier.
    pub model: String,
    /// Provider-specific voice identifier (e.g. `alloy`, `nova`).
    pub voice: String,
    /// Output format. `None` defers to the provider default (PCM).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub response_format: Option<SpeechFormat>,
    /// Playback speed multiplier (only honored by providers that support
    /// it, e.g. OpenAI TTS).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub speed: Option<f64>,
    /// Provider-specific passthrough configuration.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub provider: Option<SpeechProvider>,
}

/// Provider-specific passthrough configuration for a TTS request.
///
/// `options` is keyed by provider slug; the map for the chosen provider
/// is spread into the upstream request body.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SpeechProvider {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub options: Option<BTreeMap<String, BTreeMap<String, Value>>>,
}

/// Result of a TTS request: the raw audio bytes plus the upstream
/// `Content-Type` and the resolved format (echoes the requested format,
/// or PCM when none was requested).
#[derive(Clone, Debug)]
pub struct SpeechResponse {
    pub audio: Bytes,
    pub content_type: Option<String>,
    pub format: SpeechFormat,
}
