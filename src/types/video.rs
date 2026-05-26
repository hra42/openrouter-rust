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
    #[serde(rename = "16:9")]
    R16x9,
    #[serde(rename = "9:16")]
    R9x16,
    #[serde(rename = "1:1")]
    R1x1,
    #[serde(rename = "4:3")]
    R4x3,
    #[serde(rename = "3:4")]
    R3x4,
    #[serde(rename = "21:9")]
    R21x9,
    #[serde(rename = "9:21")]
    R9x21,
}

/// Supported output resolutions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoResolution {
    #[serde(rename = "480p")]
    P480,
    #[serde(rename = "720p")]
    P720,
    #[serde(rename = "1080p")]
    P1080,
    #[serde(rename = "1K")]
    K1,
    #[serde(rename = "2K")]
    K2,
    #[serde(rename = "4K")]
    K4,
}

/// Frame role for a supplied reference image.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VideoFrameType {
    FirstFrame,
    LastFrame,
}

/// Lifecycle status of a video generation job. Terminal values:
/// [`VideoStatus::Completed`], [`VideoStatus::Failed`],
/// [`VideoStatus::Cancelled`], [`VideoStatus::Expired`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VideoStatus {
    #[default]
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
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
    pub url: String,
}

/// Reference image used to guide generation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VideoContentPartImage {
    pub image_url: VideoImageUrl,
    /// Always `"image_url"`.
    #[serde(rename = "type")]
    pub kind: String,
}

impl VideoContentPartImage {
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
    pub image_url: VideoImageUrl,
    /// Always `"image_url"`.
    #[serde(rename = "type")]
    pub kind: String,
    pub frame_type: VideoFrameType,
}

impl VideoFrameImage {
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
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub options: Option<BTreeMap<String, BTreeMap<String, Value>>>,
}

/// Request body for [`crate::Client::create_video`].
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VideoGenerationRequest {
    pub model: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub aspect_ratio: Option<VideoAspectRatio>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub callback_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub duration: Option<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub frame_images: Vec<VideoFrameImage>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub generate_audio: Option<bool>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub input_references: Vec<VideoContentPartImage>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub provider: Option<VideoProvider>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub resolution: Option<VideoResolution>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub seed: Option<i64>,
    /// Exact pixel dimensions as `"WIDTHxHEIGHT"`.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub size: Option<String>,
}

/// Cost / BYOK information reported on completion.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VideoGenerationUsage {
    #[serde(default)]
    pub cost: Option<f64>,
    #[serde(default)]
    pub is_byok: bool,
}

/// Response from `POST /videos` and `GET /videos/{job_id}`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VideoGenerationResponse {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub polling_url: String,
    pub status: VideoStatus,
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub generation_id: String,
    #[serde(default)]
    pub unsigned_urls: Vec<String>,
    #[serde(default)]
    pub usage: Option<VideoGenerationUsage>,
}

/// Response from `GET /videos/{job_id}/content`: raw video bytes + the
/// upstream `Content-Type` (typically `application/octet-stream`).
#[derive(Clone, Debug)]
pub struct VideoContentResponse {
    pub content: Bytes,
    pub content_type: Option<String>,
}

/// A single video-generation model from `GET /videos/models`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VideoModel {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub canonical_slug: String,
    #[serde(default)]
    pub created: i64,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub hugging_face_id: Option<String>,
    #[serde(default)]
    pub allowed_passthrough_parameters: Vec<String>,
    #[serde(default)]
    pub generate_audio: Option<bool>,
    #[serde(default)]
    pub seed: Option<bool>,
    #[serde(default)]
    pub pricing_skus: BTreeMap<String, String>,
    #[serde(default)]
    pub supported_aspect_ratios: Vec<VideoAspectRatio>,
    #[serde(default)]
    pub supported_durations: Vec<u32>,
    #[serde(default)]
    pub supported_frame_images: Vec<VideoFrameType>,
    #[serde(default)]
    pub supported_resolutions: Vec<VideoResolution>,
    #[serde(default)]
    pub supported_sizes: Vec<String>,
}

/// Response from `GET /videos/models`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VideoModelsResponse {
    #[serde(default)]
    pub data: Vec<VideoModel>,
}
