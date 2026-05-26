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
    /// Reset every day at midnight UTC.
    Daily,
    /// Reset weekly.
    Weekly,
    /// Reset monthly.
    Monthly,
}

/// A guardrail configuration controlling spending, model access, and data
/// policies.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Guardrail {
    /// Stable guardrail identifier.
    #[serde(default)]
    pub id: String,
    /// Human-readable name.
    #[serde(default)]
    pub name: String,
    /// Free-form description.
    #[serde(default)]
    pub description: Option<String>,
    /// Spend cap in USD, evaluated against [`Self::reset_interval`].
    #[serde(default)]
    pub limit_usd: Option<f64>,
    /// Reset cadence for [`Self::limit_usd`].
    #[serde(default)]
    pub reset_interval: Option<ResetInterval>,
    /// Allowlist of provider slugs; empty means "all providers".
    #[serde(default)]
    pub allowed_providers: Vec<String>,
    /// Allowlist of model ids; empty means "all models".
    #[serde(default)]
    pub allowed_models: Vec<String>,
    /// When true, require ZDR-certified endpoints.
    #[serde(default)]
    pub enforce_zdr: Option<bool>,
    /// Creation timestamp.
    #[serde(default)]
    pub created_at: String,
    /// Last-update timestamp.
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Optional query parameters for the guardrails listing endpoints (also
/// reused for the key/member assignment listings).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ListGuardrailsOptions {
    /// Skip this many rows before returning results.
    pub offset: Option<u32>,
    /// Cap on the number of rows returned.
    pub limit: Option<u32>,
}

impl ListGuardrailsOptions {
    /// Construct an empty options struct.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder: set [`Self::offset`].
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Builder: set [`Self::limit`].
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
    /// The guardrail rows returned by the server.
    #[serde(default)]
    pub data: Vec<Guardrail>,
    /// Total row count, ignoring pagination.
    #[serde(default)]
    pub total_count: u64,
}

/// Request body for [`crate::Client::create_guardrail`]. `name` is required.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateGuardrailRequest {
    /// Display name for the new guardrail.
    pub name: String,
    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub description: Option<String>,
    /// Optional spend limit in USD.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub limit_usd: Option<f64>,
    /// Reset cadence for [`Self::limit_usd`].
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub reset_interval: Option<ResetInterval>,
    /// Provider allowlist; empty means "all".
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub allowed_providers: Vec<String>,
    /// Model allowlist; empty means "all".
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub allowed_models: Vec<String>,
    /// Require ZDR-certified endpoints.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub enforce_zdr: Option<bool>,
}

/// Partial-update body for [`crate::Client::update_guardrail`]. All fields
/// are optional; only set fields are sent.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UpdateGuardrailRequest {
    /// New name.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub name: Option<String>,
    /// New description.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub description: Option<String>,
    /// New spend limit in USD.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub limit_usd: Option<f64>,
    /// New reset cadence.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub reset_interval: Option<ResetInterval>,
    /// New provider allowlist.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub allowed_providers: Vec<String>,
    /// New model allowlist.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub allowed_models: Vec<String>,
    /// New ZDR enforcement flag.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub enforce_zdr: Option<bool>,
}

/// Response from `DELETE /guardrails/{id}`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteGuardrailResponse {
    /// True when the guardrail existed and was deleted.
    #[serde(default)]
    pub deleted: bool,
}

/// An API key assignment to a guardrail.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GuardrailKeyAssignment {
    /// Assignment identifier.
    #[serde(default)]
    pub id: String,
    /// Hash of the assigned API key.
    #[serde(default)]
    pub key_hash: String,
    /// Organization that owns the assignment.
    #[serde(default)]
    pub organization_id: String,
    /// Guardrail the key is assigned to.
    #[serde(default)]
    pub guardrail_id: String,
    /// User id of whoever created the assignment, when known.
    #[serde(default)]
    pub assigned_by: Option<String>,
    /// Creation timestamp.
    #[serde(default)]
    pub created_at: String,
}

/// Response from key-assignment listings.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ListGuardrailKeyAssignmentsResponse {
    /// Assignment rows returned by the server.
    #[serde(default)]
    pub data: Vec<GuardrailKeyAssignment>,
    /// Total row count, ignoring pagination.
    #[serde(default)]
    pub total_count: u64,
}

/// Request body for [`crate::Client::assign_keys_to_guardrail`] and
/// [`crate::Client::unassign_keys_from_guardrail`].
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AssignKeysRequest {
    /// Hashes of the keys to assign or unassign.
    pub key_hashes: Vec<String>,
}

/// Response from [`crate::Client::assign_keys_to_guardrail`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssignKeysResponse {
    /// Number of keys whose assignment changed.
    #[serde(default)]
    pub assigned_count: u64,
}

/// A member assignment to a guardrail.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GuardrailMemberAssignment {
    /// Assignment identifier.
    #[serde(default)]
    pub id: String,
    /// User id of the assigned member.
    #[serde(default)]
    pub user_id: String,
    /// Organization that owns the assignment.
    #[serde(default)]
    pub organization_id: String,
    /// Guardrail the member is assigned to.
    #[serde(default)]
    pub guardrail_id: String,
    /// User id of whoever created the assignment, when known.
    #[serde(default)]
    pub assigned_by: Option<String>,
    /// Creation timestamp.
    #[serde(default)]
    pub created_at: String,
}

/// Response from member-assignment listings.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ListGuardrailMemberAssignmentsResponse {
    /// Assignment rows returned by the server.
    #[serde(default)]
    pub data: Vec<GuardrailMemberAssignment>,
    /// Total row count, ignoring pagination.
    #[serde(default)]
    pub total_count: u64,
}

/// Request body for [`crate::Client::assign_members_to_guardrail`] and
/// [`crate::Client::unassign_members_from_guardrail`].
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AssignMembersRequest {
    /// User ids to assign or unassign.
    pub member_user_ids: Vec<String>,
}

/// Response from [`crate::Client::assign_members_to_guardrail`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssignMembersResponse {
    /// Number of members whose assignment changed.
    #[serde(default)]
    pub assigned_count: u64,
}

// ---- ZDR endpoints (`GET /endpoints/zdr`) ----

/// Latency or throughput percentile statistics for a public endpoint.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PercentileStats {
    /// 50th percentile.
    #[serde(default)]
    pub p50: f64,
    /// 75th percentile.
    #[serde(default)]
    pub p75: f64,
    /// 90th percentile.
    #[serde(default)]
    pub p90: f64,
    /// 99th percentile.
    #[serde(default)]
    pub p99: f64,
}

/// Pricing information for a public endpoint (used by the ZDR listing).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PublicEndpointPricing {
    /// Per-prompt-token cost as a decimal string in USD.
    #[serde(default)]
    pub prompt: String,
    /// Per-completion-token cost as a decimal string in USD.
    #[serde(default)]
    pub completion: String,
    /// Per-request flat fee, when applicable.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub request: String,
    /// Per-image cost, when applicable.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub image: String,
    /// Per-image-input-token cost, when applicable.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub image_token: String,
    /// Per-image-output cost, when applicable.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub image_output: String,
    /// Per-second-of-audio-input cost, when applicable.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub audio: String,
    /// Per-second-of-audio-output cost, when applicable.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub audio_output: String,
    /// Audio-input cache pricing.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub input_audio_cache: String,
    /// Web-search invocation pricing.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub web_search: String,
    /// Per-internal-reasoning-token cost, when applicable.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub internal_reasoning: String,
    /// Cached-input read pricing.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub input_cache_read: String,
    /// Cached-input write pricing.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub input_cache_write: String,
    /// Discount as a multiplier (e.g. 0.5 = 50% off list).
    #[serde(default)]
    pub discount: f64,
}

/// A single endpoint from the ZDR endpoints listing.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PublicEndpoint {
    /// Endpoint display name.
    #[serde(default)]
    pub name: String,
    /// Model id this endpoint serves.
    #[serde(default)]
    pub model_id: String,
    /// Model display name.
    #[serde(default)]
    pub model_name: String,
    /// Context window length, in tokens.
    #[serde(default)]
    pub context_length: f64,
    /// Pricing information.
    #[serde(default)]
    pub pricing: PublicEndpointPricing,
    /// Provider serving this endpoint.
    #[serde(default)]
    pub provider_name: String,
    /// Optional provider-specific tag (e.g. region).
    #[serde(default)]
    pub tag: Option<String>,
    /// Weight quantization label.
    #[serde(default)]
    pub quantization: Option<String>,
    /// Maximum completion tokens supported, if advertised.
    #[serde(default)]
    pub max_completion_tokens: Option<f64>,
    /// Maximum prompt tokens supported, if advertised.
    #[serde(default)]
    pub max_prompt_tokens: Option<f64>,
    /// Provider-advertised supported parameter names.
    #[serde(default)]
    pub supported_parameters: Vec<String>,
    /// Operational status (0 = healthy, non-zero = degraded).
    #[serde(default)]
    pub status: f64,
    /// Rolling 30-minute uptime ratio.
    #[serde(default)]
    pub uptime_last_30m: Option<f64>,
    /// Whether the endpoint reports implicit cache support.
    #[serde(default)]
    pub supports_implicit_caching: Option<bool>,
    /// Recent latency percentiles.
    #[serde(default)]
    pub latency_last_30m: Option<PercentileStats>,
    /// Recent throughput percentiles.
    #[serde(default)]
    pub throughput_last_30m: Option<PercentileStats>,
}

/// Response from `GET /endpoints/zdr`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ZdrEndpointsResponse {
    /// The ZDR-certified endpoint rows.
    #[serde(default)]
    pub data: Vec<PublicEndpoint>,
}
