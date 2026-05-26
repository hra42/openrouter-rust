//! Types for the organization endpoints (`/organization/members`).
//!
//! Shapes mirror the Go SDK (`organization_models.go`). These endpoints
//! require a **provisioning (management) API key**.

use serde::{Deserialize, Serialize};

/// Optional query parameters for [`crate::Client::list_organization_members`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ListOrganizationMembersOptions {
    /// Skip this many rows before returning results.
    pub offset: Option<u32>,
    /// Cap on the number of rows returned.
    pub limit: Option<u32>,
}

impl ListOrganizationMembersOptions {
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

/// Role of a member within the organization. Upstream uses prefixed
/// values (`"org:admin"` / `"org:member"`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrganizationMemberRole {
    /// Organization administrator.
    #[serde(rename = "org:admin")]
    Admin,
    /// Regular organization member.
    #[serde(rename = "org:member")]
    Member,
}

/// A single member of an organization.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrganizationMember {
    /// Stable user identifier.
    #[serde(default)]
    pub id: String,
    /// Member email address.
    #[serde(default)]
    pub email: String,
    /// Optional first name.
    #[serde(default)]
    pub first_name: Option<String>,
    /// Optional last name.
    #[serde(default)]
    pub last_name: Option<String>,
    /// Role within the organization.
    pub role: OrganizationMemberRole,
}

/// Response from `GET /organization/members`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ListOrganizationMembersResponse {
    /// Member rows returned by the server.
    #[serde(default)]
    pub data: Vec<OrganizationMember>,
    /// Total row count, ignoring pagination.
    #[serde(default)]
    pub total_count: u64,
}
