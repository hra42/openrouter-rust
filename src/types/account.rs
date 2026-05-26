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

/// Rate limit applied to an API key.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct KeyRateLimit {
    /// Window (e.g. `"10s"`).
    #[serde(default)]
    pub interval: String,
    #[serde(default)]
    pub requests: f64,
}
