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

/// Rate limit applied to an API key.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct KeyRateLimit {
    /// Window (e.g. `"10s"`).
    #[serde(default)]
    pub interval: String,
    #[serde(default)]
    pub requests: f64,
}
