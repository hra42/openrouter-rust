//! Types for the asynchronous video-generation endpoints
//! (`POST /videos`, `GET /videos/{job_id}`, `GET /videos/{job_id}/content`,
//! `GET /videos/models`).
//!
//! Shapes mirror the Go SDK (`videos_models.go`).

use std::collections::BTreeMap;

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Supported aspect ratios for video generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoAspectRatio {
    /// 16:9 widescreen.
    #[serde(rename = "16:9")]
    R16x9,
    /// 9:16 vertical.
    #[serde(rename = "9:16")]
    R9x16,
    /// 1:1 square.
    #[serde(rename = "1:1")]
    R1x1,
    /// 4:3 traditional.
    #[serde(rename = "4:3")]
    R4x3,
    /// 3:4 portrait.
    #[serde(rename = "3:4")]
    R3x4,
    /// 21:9 ultra-wide.
    #[serde(rename = "21:9")]
    R21x9,
    /// 9:21 ultra-tall.
    #[serde(rename = "9:21")]
    R9x21,
}

/// Supported output resolutions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoResolution {
    /// 480p.
    #[serde(rename = "480p")]
    P480,
    /// 720p.
    #[serde(rename = "720p")]
    P720,
    /// 1080p.
    #[serde(rename = "1080p")]
    P1080,
    /// 1K.
    #[serde(rename = "1K")]
    K1,
    /// 2K.
    #[serde(rename = "2K")]
    K2,
    /// 4K.
    #[serde(rename = "4K")]
    K4,
}

/// Frame role for a supplied reference image.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VideoFrameType {
    /// Use the image as the first frame.
    FirstFrame,
    /// Use the image as the last frame.
    LastFrame,
}

/// Lifecycle status of a video generation job. Terminal values:
/// [`VideoStatus::Completed`], [`VideoStatus::Failed`],
/// [`VideoStatus::Cancelled`], [`VideoStatus::Expired`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VideoStatus {
    /// Queued, not yet started.
    #[default]
    Pending,
    /// Currently generating.
    InProgress,
    /// Successfully completed.
    Completed,
    /// Generation failed.
    Failed,
    /// Cancelled by the caller.
    Cancelled,
    /// Output expired before retrieval.
    Expired,
}

impl VideoStatus {
    /// `true` once the job will never advance further.
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            VideoStatus::Completed
                | VideoStatus::Failed
                | VideoStatus::Cancelled
                | VideoStatus::Expired
        )
    }
}

/// A URL wrapper used in image inputs.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VideoImageUrl {
    /// HTTPS URL or data URL pointing at the image bytes.
    pub url: String,
}

/// Reference image used to guide generation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VideoContentPartImage {
    /// Image reference.
    pub image_url: VideoImageUrl,
    /// Always `"image_url"`.
    #[serde(rename = "type")]
    pub kind: String,
}

impl VideoContentPartImage {
    /// Construct a reference image from a URL.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            image_url: VideoImageUrl { url: url.into() },
            kind: "image_url".into(),
        }
    }
}

/// First- or last-frame reference image.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VideoFrameImage {
    /// Image reference.
    pub image_url: VideoImageUrl,
    /// Always `"image_url"`.
    #[serde(rename = "type")]
    pub kind: String,
    /// Which frame this image represents.
    pub frame_type: VideoFrameType,
}

impl VideoFrameImage {
    /// Construct a frame-image reference.
    pub fn new(url: impl Into<String>, frame_type: VideoFrameType) -> Self {
        Self {
            image_url: VideoImageUrl { url: url.into() },
            kind: "image_url".into(),
            frame_type,
        }
    }
}

/// Provider-specific passthrough options for video generation.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VideoProvider {
    /// Provider-keyed options. The map under the chosen provider's slug
    /// is spread into the upstream request body.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub options: Option<BTreeMap<String, BTreeMap<String, Value>>>,
}

/// Request body for [`crate::Client::create_video`].
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VideoGenerationRequest {
    /// Video model id.
    pub model: String,
    /// Prompt describing the desired video.
    pub prompt: String,
    /// Output aspect ratio.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub aspect_ratio: Option<VideoAspectRatio>,
    /// Optional webhook URL invoked when the job completes.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub callback_url: Option<String>,
    /// Output duration in seconds.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub duration: Option<u32>,
    /// First/last-frame reference images.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub frame_images: Vec<VideoFrameImage>,
    /// Whether to generate an audio track.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub generate_audio: Option<bool>,
    /// Free-form reference images.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub input_references: Vec<VideoContentPartImage>,
    /// Provider-specific passthrough options.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub provider: Option<VideoProvider>,
    /// Output resolution.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub resolution: Option<VideoResolution>,
    /// Sampling seed for reproducibility.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub seed: Option<i64>,
    /// Exact pixel dimensions as `"WIDTHxHEIGHT"`.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub size: Option<String>,
}

/// Cost / BYOK information reported on completion.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VideoGenerationUsage {
    /// USD cost of the request.
    #[serde(default)]
    pub cost: Option<f64>,
    /// True when served via a BYOK provider key.
    #[serde(default)]
    pub is_byok: bool,
}

/// Response from `POST /videos` and `GET /videos/{job_id}`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VideoGenerationResponse {
    /// Job identifier.
    #[serde(default)]
    pub id: String,
    /// URL to poll for status updates.
    #[serde(default)]
    pub polling_url: String,
    /// Lifecycle status.
    pub status: VideoStatus,
    /// Error message, when [`Self::status`] is [`VideoStatus::Failed`].
    #[serde(default)]
    pub error: String,
    /// Underlying generation id (provider-side).
    #[serde(default)]
    pub generation_id: String,
    /// Direct (unsigned) URLs for downloading the result.
    #[serde(default)]
    pub unsigned_urls: Vec<String>,
    /// Cost / BYOK information, when reported.
    #[serde(default)]
    pub usage: Option<VideoGenerationUsage>,
}

/// Response from `GET /videos/{job_id}/content`: raw video bytes + the
/// upstream `Content-Type` (typically `application/octet-stream`).
#[derive(Clone, Debug)]
pub struct VideoContentResponse {
    /// Raw video bytes.
    pub content: Bytes,
    /// Upstream `Content-Type` header, when present.
    pub content_type: Option<String>,
}

/// A single video-generation model from `GET /videos/models`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VideoModel {
    /// Model id.
    #[serde(default)]
    pub id: String,
    /// Display name.
    #[serde(default)]
    pub name: String,
    /// Stable canonical slug.
    #[serde(default)]
    pub canonical_slug: String,
    /// Unix-seconds creation timestamp.
    #[serde(default)]
    pub created: i64,
    /// Long description.
    #[serde(default)]
    pub description: String,
    /// Hugging Face model id, when published.
    #[serde(default)]
    pub hugging_face_id: Option<String>,
    /// Allowed provider passthrough parameter names.
    #[serde(default)]
    pub allowed_passthrough_parameters: Vec<String>,
    /// Whether audio generation is supported.
    #[serde(default)]
    pub generate_audio: Option<bool>,
    /// Whether sampling seeds are supported.
    #[serde(default)]
    pub seed: Option<bool>,
    /// Pricing SKU table keyed by SKU name.
    #[serde(default)]
    pub pricing_skus: BTreeMap<String, String>,
    /// Supported aspect ratios.
    #[serde(default)]
    pub supported_aspect_ratios: Vec<VideoAspectRatio>,
    /// Supported durations (seconds).
    #[serde(default)]
    pub supported_durations: Vec<u32>,
    /// Supported frame-image roles.
    #[serde(default)]
    pub supported_frame_images: Vec<VideoFrameType>,
    /// Supported output resolutions.
    #[serde(default)]
    pub supported_resolutions: Vec<VideoResolution>,
    /// Supported exact pixel sizes.
    #[serde(default)]
    pub supported_sizes: Vec<String>,
}

/// Response from `GET /videos/models`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VideoModelsResponse {
    /// Model rows.
    #[serde(default)]
    pub data: Vec<VideoModel>,
}
