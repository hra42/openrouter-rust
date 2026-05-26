//! Types for the organization endpoints (`/organization/members`).
//!
//! Shapes mirror the Go SDK (`organization_models.go`). These endpoints
//! require a **provisioning (management) API key**.

use serde::{Deserialize, Serialize};

/// Optional query parameters for [`crate::Client::list_organization_members`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ListOrganizationMembersOptions {
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

impl ListOrganizationMembersOptions {
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

/// Role of a member within the organization. Upstream uses prefixed
/// values (`"org:admin"` / `"org:member"`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrganizationMemberRole {
    #[serde(rename = "org:admin")]
    Admin,
    #[serde(rename = "org:member")]
    Member,
}

/// A single member of an organization.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrganizationMember {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    pub role: OrganizationMemberRole,
}

/// Response from `GET /organization/members`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ListOrganizationMembersResponse {
    #[serde(default)]
    pub data: Vec<OrganizationMember>,
    #[serde(default)]
    pub total_count: u64,
}
