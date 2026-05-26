//! Types for the guardrails endpoints (`/guardrails`) and ZDR endpoint
//! listing (`/endpoints/zdr`).
//!
//! Shapes mirror the Go SDK (`guardrails_models.go`, `metadata_models.go`).
//! All endpoints in this module require a **provisioning (management) API
//! key**.

use serde::{Deserialize, Serialize};

/// Reset interval for a guardrail's spend budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResetInterval {
    Daily,
    Weekly,
    Monthly,
}

/// A guardrail configuration controlling spending, model access, and data
/// policies.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Guardrail {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub limit_usd: Option<f64>,
    #[serde(default)]
    pub reset_interval: Option<ResetInterval>,
    #[serde(default)]
    pub allowed_providers: Vec<String>,
    #[serde(default)]
    pub allowed_models: Vec<String>,
    #[serde(default)]
    pub enforce_zdr: Option<bool>,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Optional query parameters for the guardrails listing endpoints (also
/// reused for the key/member assignment listings).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ListGuardrailsOptions {
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

impl ListGuardrailsOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub(crate) fn to_query(self) -> Vec<(&'static str, String)> {
        let mut q = Vec::new();
        if let Some(o) = self.offset {
            q.push(("offset", o.to_string()));
        }
        if let Some(l) = self.limit {
            q.push(("limit", l.to_string()));
        }
        q
    }
}

/// Response from `GET /guardrails`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ListGuardrailsResponse {
    #[serde(default)]
    pub data: Vec<Guardrail>,
    #[serde(default)]
    pub total_count: u64,
}

/// Request body for [`crate::Client::create_guardrail`]. `name` is required.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateGuardrailRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub limit_usd: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub reset_interval: Option<ResetInterval>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub allowed_providers: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub allowed_models: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub enforce_zdr: Option<bool>,
}

/// Partial-update body for [`crate::Client::update_guardrail`]. All fields
/// are optional; only set fields are sent.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UpdateGuardrailRequest {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub limit_usd: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub reset_interval: Option<ResetInterval>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub allowed_providers: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub allowed_models: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub enforce_zdr: Option<bool>,
}

/// Response from `DELETE /guardrails/{id}`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteGuardrailResponse {
    #[serde(default)]
    pub deleted: bool,
}

/// An API key assignment to a guardrail.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GuardrailKeyAssignment {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub key_hash: String,
    #[serde(default)]
    pub organization_id: String,
    #[serde(default)]
    pub guardrail_id: String,
    #[serde(default)]
    pub assigned_by: Option<String>,
    #[serde(default)]
    pub created_at: String,
}

/// Response from key-assignment listings.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ListGuardrailKeyAssignmentsResponse {
    #[serde(default)]
    pub data: Vec<GuardrailKeyAssignment>,
    #[serde(default)]
    pub total_count: u64,
}

/// Request body for [`crate::Client::assign_keys_to_guardrail`] and
/// [`crate::Client::unassign_keys_from_guardrail`].
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AssignKeysRequest {
    pub key_hashes: Vec<String>,
}

/// Response from [`crate::Client::assign_keys_to_guardrail`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssignKeysResponse {
    #[serde(default)]
    pub assigned_count: u64,
}

/// A member assignment to a guardrail.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GuardrailMemberAssignment {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub user_id: String,
    #[serde(default)]
    pub organization_id: String,
    #[serde(default)]
    pub guardrail_id: String,
    #[serde(default)]
    pub assigned_by: Option<String>,
    #[serde(default)]
    pub created_at: String,
}

/// Response from member-assignment listings.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ListGuardrailMemberAssignmentsResponse {
    #[serde(default)]
    pub data: Vec<GuardrailMemberAssignment>,
    #[serde(default)]
    pub total_count: u64,
}

/// Request body for [`crate::Client::assign_members_to_guardrail`] and
/// [`crate::Client::unassign_members_from_guardrail`].
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AssignMembersRequest {
    pub member_user_ids: Vec<String>,
}

/// Response from [`crate::Client::assign_members_to_guardrail`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssignMembersResponse {
    #[serde(default)]
    pub assigned_count: u64,
}

// ---- ZDR endpoints (`GET /endpoints/zdr`) ----

/// Latency or throughput percentile statistics for a public endpoint.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PercentileStats {
    #[serde(default)]
    pub p50: f64,
    #[serde(default)]
    pub p75: f64,
    #[serde(default)]
    pub p90: f64,
    #[serde(default)]
    pub p99: f64,
}

/// Pricing information for a public endpoint (used by the ZDR listing).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PublicEndpointPricing {
    #[serde(default)]
    pub prompt: String,
    #[serde(default)]
    pub completion: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub request: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub image: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub image_token: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub image_output: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub audio: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub audio_output: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub input_audio_cache: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub web_search: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub internal_reasoning: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub input_cache_read: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub input_cache_write: String,
    #[serde(default)]
    pub discount: f64,
}

/// A single endpoint from the ZDR endpoints listing.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PublicEndpoint {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub model_id: String,
    #[serde(default)]
    pub model_name: String,
    #[serde(default)]
    pub context_length: f64,
    #[serde(default)]
    pub pricing: PublicEndpointPricing,
    #[serde(default)]
    pub provider_name: String,
    #[serde(default)]
    pub tag: Option<String>,
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
    #[serde(default)]
    pub supports_implicit_caching: Option<bool>,
    #[serde(default)]
    pub latency_last_30m: Option<PercentileStats>,
    #[serde(default)]
    pub throughput_last_30m: Option<PercentileStats>,
}

/// Response from `GET /endpoints/zdr`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ZdrEndpointsResponse {
    #[serde(default)]
    pub data: Vec<PublicEndpoint>,
}
