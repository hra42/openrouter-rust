//! Response types for the discovery endpoints (`/models`, `/models/{author}/{slug}/endpoints`,
//! `/providers`).
//!
//! Shapes mirror the Go SDK (`metadata_models.go`) one-for-one so behavior stays
//! aligned across the two ports.

use serde::{Deserialize, Serialize};

/// Optional query parameters for [`crate::Client::list_models`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ListModelsOptions {
    /// Filters models by category (e.g. `"programming"`). Results are sorted
    /// from most to least used.
    pub category: Option<String>,
    /// Filters models by supported parameter (e.g. `"tools"`, `"temperature"`).
    pub supported_parameters: Option<String>,
}

impl ListModelsOptions {
    /// Start a new, empty options builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the `category` filter.
    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Set the `supported_parameters` filter.
    pub fn supported_parameters(mut self, value: impl Into<String>) -> Self {
        self.supported_parameters = Some(value.into());
        self
    }

    pub(crate) fn to_query(&self) -> Vec<(&'static str, String)> {
        let mut q = Vec::new();
        if let Some(c) = &self.category {
            q.push(("category", c.clone()));
        }
        if let Some(s) = &self.supported_parameters {
            q.push(("supported_parameters", s.clone()));
        }
        q
    }
}

/// Response from `GET /models`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelsResponse {
    /// Model rows.
    #[serde(default)]
    pub data: Vec<Model>,
}

/// A single model available on OpenRouter.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Model {
    /// Model id (e.g. `google/gemini-3.1-flash-lite`).
    pub id: String,
    /// Display name.
    #[serde(default)]
    pub name: String,
    /// Stable canonical slug (immutable across renames).
    #[serde(default)]
    pub canonical_slug: Option<String>,
    /// Unix-seconds creation timestamp.
    #[serde(default)]
    pub created: Option<f64>,
    /// Long description.
    #[serde(default)]
    pub description: String,
    /// Maximum context window in tokens.
    #[serde(default)]
    pub context_length: Option<f64>,
    /// Hugging Face model id, when published.
    #[serde(default)]
    pub hugging_face_id: Option<String>,
    /// Model architecture summary.
    #[serde(default)]
    pub architecture: ModelArchitecture,
    /// Description of the top provider serving this model.
    #[serde(default)]
    pub top_provider: ModelTopProvider,
    /// Per-request token limits, when published.
    #[serde(default)]
    pub per_request_limits: Option<ModelPerRequestLimits>,
    /// Supported parameter names.
    #[serde(default)]
    pub supported_parameters: Vec<String>,
    /// Default sampling parameters published by the provider.
    #[serde(default)]
    pub default_parameters: Option<ModelDefaultParameters>,
    /// Pricing breakdown.
    #[serde(default)]
    pub pricing: ModelPricing,
    /// Model retirement date, when scheduled.
    #[serde(default)]
    pub expiration_date: Option<String>,
}

/// Architecture summary for a model.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelArchitecture {
    /// Input modalities (`text`, `image`, ...).
    #[serde(default)]
    pub input_modalities: Vec<String>,
    /// Output modalities.
    #[serde(default)]
    pub output_modalities: Vec<String>,
    /// Tokenizer name.
    #[serde(default)]
    pub tokenizer: String,
    /// Instruction-tuning family.
    #[serde(default)]
    pub instruct_type: Option<String>,
    /// Combined modality string for legacy clients.
    #[serde(default)]
    pub modality: Option<String>,
}

/// Top-provider summary for a model.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelTopProvider {
    /// Context length advertised by the top provider.
    #[serde(default)]
    pub context_length: Option<f64>,
    /// Max completion tokens.
    #[serde(default)]
    pub max_completion_tokens: Option<f64>,
    /// True if the top provider applies content moderation.
    #[serde(default)]
    pub is_moderated: bool,
}

/// Per-request token limits.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelPerRequestLimits {
    /// Max prompt tokens per request.
    #[serde(default)]
    pub prompt_tokens: Option<f64>,
    /// Max completion tokens per request.
    #[serde(default)]
    pub completion_tokens: Option<f64>,
}

/// Default sampling parameters published by the provider.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelDefaultParameters {
    /// Default temperature.
    #[serde(default)]
    pub temperature: Option<f64>,
    /// Default top-p.
    #[serde(default)]
    pub top_p: Option<f64>,
    /// Default frequency penalty.
    #[serde(default)]
    pub frequency_penalty: Option<f64>,
}

/// Pricing expressed as decimal strings (USD per token / per request / per image).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelPricing {
    /// Per-prompt-token cost.
    #[serde(default)]
    pub prompt: String,
    /// Per-completion-token cost.
    #[serde(default)]
    pub completion: String,
    /// Per-image cost.
    #[serde(default)]
    pub image: String,
    /// Per-request flat fee.
    #[serde(default)]
    pub request: String,
    /// Cached-input read cost.
    #[serde(default)]
    pub input_cache_read: Option<String>,
    /// Cached-input write cost.
    #[serde(default)]
    pub input_cache_write: Option<String>,
    /// Web-search invocation cost.
    #[serde(default)]
    pub web_search: String,
    /// Internal reasoning token cost.
    #[serde(default)]
    pub internal_reasoning: String,
}

/// Response from `GET /models/{author}/{slug}/endpoints`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelEndpointsResponse {
    /// Model + endpoint payload.
    #[serde(default)]
    pub data: ModelEndpointsData,
}

/// Body of [`ModelEndpointsResponse`].
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelEndpointsData {
    /// Model id.
    #[serde(default)]
    pub id: String,
    /// Display name.
    #[serde(default)]
    pub name: String,
    /// Unix-seconds creation timestamp.
    #[serde(default)]
    pub created: Option<f64>,
    /// Long description.
    #[serde(default)]
    pub description: String,
    /// Architecture summary.
    #[serde(default)]
    pub architecture: ModelEndpointsArchitecture,
    /// Endpoint rows.
    #[serde(default)]
    pub endpoints: Vec<ModelEndpoint>,
}

/// Architecture summary specific to the endpoints listing.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelEndpointsArchitecture {
    /// Tokenizer name.
    #[serde(default)]
    pub tokenizer: Option<String>,
    /// Instruction-tuning family.
    #[serde(default)]
    pub instruct_type: Option<String>,
    /// Input modalities.
    #[serde(default)]
    pub input_modalities: Vec<String>,
    /// Output modalities.
    #[serde(default)]
    pub output_modalities: Vec<String>,
}

/// A single provider endpoint for a model.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelEndpoint {
    /// Endpoint display name.
    #[serde(default)]
    pub name: String,
    /// Context window in tokens.
    #[serde(default)]
    pub context_length: f64,
    /// Pricing information.
    #[serde(default)]
    pub pricing: ModelEndpointPricing,
    /// Provider serving this endpoint.
    #[serde(default)]
    pub provider_name: String,
    /// Weight quantization label.
    #[serde(default)]
    pub quantization: Option<String>,
    /// Maximum completion tokens supported.
    #[serde(default)]
    pub max_completion_tokens: Option<f64>,
    /// Maximum prompt tokens supported.
    #[serde(default)]
    pub max_prompt_tokens: Option<f64>,
    /// Provider-advertised supported parameter names.
    #[serde(default)]
    pub supported_parameters: Vec<String>,
    /// Operational status (0 = healthy).
    #[serde(default)]
    pub status: f64,
    /// Rolling 30-minute uptime ratio.
    #[serde(default)]
    pub uptime_last_30m: Option<f64>,
}

/// Endpoint-specific pricing.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelEndpointPricing {
    /// Per-request flat fee.
    #[serde(default)]
    pub request: String,
    /// Per-image cost.
    #[serde(default)]
    pub image: String,
    /// Per-prompt-token cost.
    #[serde(default)]
    pub prompt: String,
    /// Per-completion-token cost.
    #[serde(default)]
    pub completion: String,
}

/// Response from `GET /providers`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProvidersResponse {
    /// Provider rows.
    #[serde(default)]
    pub data: Vec<ProviderInfo>,
}

/// Information about a provider available on OpenRouter.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProviderInfo {
    /// Provider name.
    #[serde(default)]
    pub name: String,
    /// Provider slug used in routing parameters.
    #[serde(default)]
    pub slug: String,
    /// Privacy-policy URL.
    #[serde(default)]
    pub privacy_policy_url: Option<String>,
    /// Terms-of-service URL.
    #[serde(default)]
    pub terms_of_service_url: Option<String>,
    /// Public status-page URL.
    #[serde(default)]
    pub status_page_url: Option<String>,
}
