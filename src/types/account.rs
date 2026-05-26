//! Response types for the account endpoints (`/credits`, `/key`, `/activity`,
//! `/keys` CRUD).
//!
//! Shapes mirror the Go SDK (`account_models.go`) one-for-one.

use serde::{Deserialize, Serialize};

/// Response from `GET /credits`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CreditsResponse {
    /// Credit-balance payload.
    #[serde(default)]
    pub data: CreditsData,
}

/// Credit balance for the authenticated user.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CreditsData {
    /// Total purchased credits in USD.
    #[serde(default)]
    pub total_credits: f64,
    /// Lifetime usage in USD.
    #[serde(default)]
    pub total_usage: f64,
}

impl CreditsData {
    /// Remaining balance (`total_credits - total_usage`).
    pub fn remaining(&self) -> f64 {
        self.total_credits - self.total_usage
    }
}

/// Response from `GET /key`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct KeyResponse {
    /// Key-info payload.
    #[serde(default)]
    pub data: KeyData,
}

/// Information about the current API key.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct KeyData {
    /// Display label for the key.
    #[serde(default)]
    pub label: String,
    /// Configured spend limit. `None` if no limit is set.
    #[serde(default)]
    pub limit: Option<f64>,
    /// Lifetime usage in USD.
    #[serde(default)]
    pub usage: f64,
    /// True if the key is on the free tier.
    #[serde(default)]
    pub is_free_tier: bool,
    /// Remaining spend allowance. `None` when no limit is set.
    #[serde(default)]
    pub limit_remaining: Option<f64>,
    /// `true` if this is a provisioning key (can manage other keys).
    #[serde(default)]
    pub is_provisioning_key: bool,
    /// Rate-limit applied to this key, when set.
    #[serde(default)]
    pub rate_limit: Option<KeyRateLimit>,
}

/// Optional query parameters for [`crate::Client::get_activity`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ActivityOptions {
    /// Single UTC date (`YYYY-MM-DD`) within the last 30 days. The API still
    /// returns the timestamped date string in the response.
    pub date: Option<String>,
}

impl ActivityOptions {
    /// Construct an empty options struct.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder: set [`Self::date`].
    pub fn date(mut self, date: impl Into<String>) -> Self {
        self.date = Some(date.into());
        self
    }

    pub(crate) fn to_query(&self) -> Vec<(&'static str, String)> {
        let mut q = Vec::new();
        if let Some(d) = &self.date {
            q.push(("date", d.clone()));
        }
        q
    }
}

/// Response from `GET /activity`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ActivityResponse {
    /// Per-row activity data.
    #[serde(default)]
    pub data: Vec<ActivityData>,
}

/// One row of daily activity grouped by model endpoint.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ActivityData {
    /// UTC date of the row.
    #[serde(default)]
    pub date: String,
    /// Model id.
    #[serde(default)]
    pub model: String,
    /// Stable model permaslug (immutable across renames).
    #[serde(default)]
    pub model_permaslug: String,
    /// Endpoint identifier within the model.
    #[serde(default)]
    pub endpoint_id: String,
    /// Provider that served the requests.
    #[serde(default)]
    pub provider_name: String,
    /// USD spent on this row.
    #[serde(default)]
    pub usage: f64,
    /// USD spent on BYOK inference (not deducted from credits).
    #[serde(default)]
    pub byok_usage_inference: f64,
    /// Request count.
    #[serde(default)]
    pub requests: f64,
    /// Prompt tokens.
    #[serde(default)]
    pub prompt_tokens: f64,
    /// Completion tokens.
    #[serde(default)]
    pub completion_tokens: f64,
    /// Reasoning tokens.
    #[serde(default)]
    pub reasoning_tokens: f64,
}

/// Optional query parameters for [`crate::Client::list_keys`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ListKeysOptions {
    /// Pagination offset.
    pub offset: Option<u32>,
    /// Include disabled keys in the response.
    pub include_disabled: Option<bool>,
}

impl ListKeysOptions {
    /// Construct an empty options struct.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder: set [`Self::offset`].
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Builder: set [`Self::include_disabled`].
    pub fn include_disabled(mut self, include: bool) -> Self {
        self.include_disabled = Some(include);
        self
    }

    pub(crate) fn to_query(self) -> Vec<(&'static str, String)> {
        let mut q = Vec::new();
        if let Some(o) = self.offset {
            q.push(("offset", o.to_string()));
        }
        if let Some(i) = self.include_disabled {
            q.push(("include_disabled", i.to_string()));
        }
        q
    }
}

/// Response from `GET /keys`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ListKeysResponse {
    /// Key rows returned by the server.
    #[serde(default)]
    pub data: Vec<ApiKey>,
}

/// Metadata for a single API key as returned by the provisioning endpoints.
///
/// Note: the secret key value is only ever returned once, in
/// [`CreateKeyResponse::key`].
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ApiKey {
    /// Display name.
    #[serde(default)]
    pub name: String,
    /// Internal label (often equals `name`).
    #[serde(default)]
    pub label: String,
    /// Spend limit in USD (0 means unlimited).
    #[serde(default)]
    pub limit: f64,
    /// True if the key is disabled.
    #[serde(default)]
    pub disabled: bool,
    /// Creation timestamp.
    #[serde(default)]
    pub created_at: String,
    /// Last-update timestamp.
    #[serde(default)]
    pub updated_at: String,
    /// Stable identifier for the key. Use this to address the key in
    /// [`crate::Client::get_key_by_hash`], [`crate::Client::update_key`], and
    /// [`crate::Client::delete_key`].
    #[serde(default)]
    pub hash: String,
}

/// Request body for [`crate::Client::create_key`].
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateKeyRequest {
    /// Required label / display name for the key.
    pub name: String,
    /// Optional credit limit (in dollars).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub limit: Option<f64>,
    /// When true, BYOK usage counts toward `limit`.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub include_byok_in_limit: Option<bool>,
}

/// Response from `POST /keys` — the only place the secret key is ever
/// returned. Store it immediately; it cannot be recovered later.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateKeyResponse {
    /// Key metadata.
    #[serde(default)]
    pub data: ApiKey,
    /// Secret API key value. **Returned only on creation.** `None` from any
    /// other endpoint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
}

/// Response from `GET /keys/{hash}`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GetKeyByHashResponse {
    /// Key metadata.
    #[serde(default)]
    pub data: ApiKey,
}

/// Partial-update request body for [`crate::Client::update_key`]. Only the
/// fields you set are sent.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UpdateKeyRequest {
    /// New display name.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub name: Option<String>,
    /// New disabled flag.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub disabled: Option<bool>,
    /// New spend limit.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub limit: Option<f64>,
    /// New BYOK-counts-toward-limit flag.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub include_byok_in_limit: Option<bool>,
}

/// Response from `PATCH /keys/{hash}`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UpdateKeyResponse {
    /// Updated key metadata.
    #[serde(default)]
    pub data: ApiKey,
}

/// Response from `DELETE /keys/{hash}`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DeleteKeyResponse {
    /// Deletion outcome.
    #[serde(default)]
    pub data: DeleteKeyData,
}

/// Deletion outcome carried by [`DeleteKeyResponse`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DeleteKeyData {
    /// True when the key existed and was deleted.
    #[serde(default)]
    pub success: bool,
}

/// Rate limit applied to an API key.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct KeyRateLimit {
    /// Window (e.g. `"10s"`).
    #[serde(default)]
    pub interval: String,
    /// Allowed requests per [`Self::interval`].
    #[serde(default)]
    pub requests: f64,
}
