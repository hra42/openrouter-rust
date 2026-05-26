//! Response types for the account endpoints (`/credits`, `/key`, `/activity`,
//! `/keys` CRUD).
//!
//! Shapes mirror the Go SDK (`account_models.go`) one-for-one.

use serde::{Deserialize, Serialize};

/// Response from `GET /credits`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CreditsResponse {
    #[serde(default)]
    pub data: CreditsData,
}

/// Credit balance for the authenticated user.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CreditsData {
    #[serde(default)]
    pub total_credits: f64,
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
    #[serde(default)]
    pub data: KeyData,
}

/// Information about the current API key.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct KeyData {
    #[serde(default)]
    pub label: String,
    /// Configured spend limit. `None` if no limit is set.
    #[serde(default)]
    pub limit: Option<f64>,
    #[serde(default)]
    pub usage: f64,
    #[serde(default)]
    pub is_free_tier: bool,
    /// Remaining spend allowance. `None` when no limit is set.
    #[serde(default)]
    pub limit_remaining: Option<f64>,
    /// `true` if this is a provisioning key (can manage other keys).
    #[serde(default)]
    pub is_provisioning_key: bool,
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
    pub fn new() -> Self {
        Self::default()
    }

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
    #[serde(default)]
    pub data: Vec<ActivityData>,
}

/// One row of daily activity grouped by model endpoint.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ActivityData {
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub model_permaslug: String,
    #[serde(default)]
    pub endpoint_id: String,
    #[serde(default)]
    pub provider_name: String,
    #[serde(default)]
    pub usage: f64,
    #[serde(default)]
    pub byok_usage_inference: f64,
    #[serde(default)]
    pub requests: f64,
    #[serde(default)]
    pub prompt_tokens: f64,
    #[serde(default)]
    pub completion_tokens: f64,
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

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
    #[serde(default)]
    pub data: Vec<ApiKey>,
}

/// Metadata for a single API key as returned by the provisioning endpoints.
///
/// Note: the secret key value is only ever returned once, in
/// [`CreateKeyResponse::key`].
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ApiKey {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub limit: f64,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub created_at: String,
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
    #[serde(default)]
    pub data: ApiKey,
}

/// Partial-update request body for [`crate::Client::update_key`]. Only the
/// fields you set are sent.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UpdateKeyRequest {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub disabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub limit: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub include_byok_in_limit: Option<bool>,
}

/// Response from `PATCH /keys/{hash}`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UpdateKeyResponse {
    #[serde(default)]
    pub data: ApiKey,
}

/// Response from `DELETE /keys/{hash}`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DeleteKeyResponse {
    #[serde(default)]
    pub data: DeleteKeyData,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DeleteKeyData {
    #[serde(default)]
    pub success: bool,
}

/// Rate limit applied to an API key.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct KeyRateLimit {
    /// Window (e.g. `"10s"`).
    #[serde(default)]
    pub interval: String,
    #[serde(default)]
    pub requests: f64,
}
