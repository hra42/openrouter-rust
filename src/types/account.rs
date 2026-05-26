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
