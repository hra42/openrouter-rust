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
    #[serde(default)]
    pub data: Vec<Model>,
}

/// A single model available on OpenRouter.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub canonical_slug: Option<String>,
    #[serde(default)]
    pub created: Option<f64>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub context_length: Option<f64>,
    #[serde(default)]
    pub hugging_face_id: Option<String>,
    #[serde(default)]
    pub architecture: ModelArchitecture,
    #[serde(default)]
    pub top_provider: ModelTopProvider,
    #[serde(default)]
    pub per_request_limits: Option<ModelPerRequestLimits>,
    #[serde(default)]
    pub supported_parameters: Vec<String>,
    #[serde(default)]
    pub default_parameters: Option<ModelDefaultParameters>,
    #[serde(default)]
    pub pricing: ModelPricing,
    #[serde(default)]
    pub expiration_date: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelArchitecture {
    #[serde(default)]
    pub input_modalities: Vec<String>,
    #[serde(default)]
    pub output_modalities: Vec<String>,
    #[serde(default)]
    pub tokenizer: String,
    #[serde(default)]
    pub instruct_type: Option<String>,
    #[serde(default)]
    pub modality: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelTopProvider {
    #[serde(default)]
    pub context_length: Option<f64>,
    #[serde(default)]
    pub max_completion_tokens: Option<f64>,
    #[serde(default)]
    pub is_moderated: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelPerRequestLimits {
    #[serde(default)]
    pub prompt_tokens: Option<f64>,
    #[serde(default)]
    pub completion_tokens: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelDefaultParameters {
    #[serde(default)]
    pub temperature: Option<f64>,
    #[serde(default)]
    pub top_p: Option<f64>,
    #[serde(default)]
    pub frequency_penalty: Option<f64>,
}

/// Pricing expressed as decimal strings (USD per token / per request / per image).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelPricing {
    #[serde(default)]
    pub prompt: String,
    #[serde(default)]
    pub completion: String,
    #[serde(default)]
    pub image: String,
    #[serde(default)]
    pub request: String,
    #[serde(default)]
    pub input_cache_read: Option<String>,
    #[serde(default)]
    pub input_cache_write: Option<String>,
    #[serde(default)]
    pub web_search: String,
    #[serde(default)]
    pub internal_reasoning: String,
}

/// Response from `GET /models/{author}/{slug}/endpoints`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelEndpointsResponse {
    #[serde(default)]
    pub data: ModelEndpointsData,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelEndpointsData {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub created: Option<f64>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub architecture: ModelEndpointsArchitecture,
    #[serde(default)]
    pub endpoints: Vec<ModelEndpoint>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelEndpointsArchitecture {
    #[serde(default)]
    pub tokenizer: Option<String>,
    #[serde(default)]
    pub instruct_type: Option<String>,
    #[serde(default)]
    pub input_modalities: Vec<String>,
    #[serde(default)]
    pub output_modalities: Vec<String>,
}

/// A single provider endpoint for a model.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelEndpoint {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub context_length: f64,
    #[serde(default)]
    pub pricing: ModelEndpointPricing,
    #[serde(default)]
    pub provider_name: String,
    #[serde(default)]
    pub quantization: Option<String>,
    #[serde(default)]
    pub max_completion_tokens: Option<f64>,
    #[serde(default)]
    pub max_prompt_tokens: Option<f64>,
    #[serde(default)]
    pub supported_parameters: Vec<String>,
    #[serde(default)]
    pub status: f64,
    #[serde(default)]
    pub uptime_last_30m: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelEndpointPricing {
    #[serde(default)]
    pub request: String,
    #[serde(default)]
    pub image: String,
    #[serde(default)]
    pub prompt: String,
    #[serde(default)]
    pub completion: String,
}

/// Response from `GET /providers`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProvidersResponse {
    #[serde(default)]
    pub data: Vec<ProviderInfo>,
}

/// Information about a provider available on OpenRouter.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProviderInfo {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub privacy_policy_url: Option<String>,
    #[serde(default)]
    pub terms_of_service_url: Option<String>,
    #[serde(default)]
    pub status_page_url: Option<String>,
}
