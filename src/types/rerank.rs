//! Types for the rerank endpoint (`POST /rerank`).
//!
//! Shapes mirror the Go SDK (`rerank_models.go`).

use serde::{Deserialize, Serialize};

use super::common::Provider;

/// Request body for [`crate::Client::rerank`].
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RerankRequest {
    /// Rerank model identifier (e.g. `cohere/rerank-v3.5`).
    pub model: String,
    /// Search query used to rank `documents`.
    pub query: String,
    /// Candidate documents to rerank.
    pub documents: Vec<String>,
    /// Return only the top `top_n` documents.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub top_n: Option<u32>,
    /// Provider routing parameters (shared with chat/completions).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub provider: Option<Provider>,
}

/// Response body from `/rerank`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RerankResponse {
    /// Unique identifier for the response (ORID format).
    #[serde(default)]
    pub id: String,
    /// Model that served the request.
    #[serde(default)]
    pub model: String,
    /// Provider that served the request.
    #[serde(default)]
    pub provider: String,
    /// Ranked results, ordered by descending relevance.
    #[serde(default)]
    pub results: Vec<RerankResult>,
    /// Usage statistics (when reported by the provider).
    #[serde(default)]
    pub usage: Option<RerankUsage>,
}

/// A single ranked result.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RerankResult {
    /// Index of the document in the original `documents` array.
    #[serde(default)]
    pub index: u32,
    /// Relevance score (provider-defined scale).
    #[serde(default)]
    pub relevance_score: f64,
    /// Echoed document text.
    #[serde(default)]
    pub document: RerankDocument,
}

/// The echoed document text inside a [`RerankResult`].
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RerankDocument {
    #[serde(default)]
    pub text: String,
}

/// Usage statistics for a rerank request.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RerankUsage {
    #[serde(default)]
    pub total_tokens: u64,
    /// Cohere-style billing unit (per-document cost).
    #[serde(default)]
    pub search_units: u64,
    /// Cost in credits.
    #[serde(default)]
    pub cost: Option<f64>,
}
