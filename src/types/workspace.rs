//! Types for the workspace management endpoints
//! (`/workspaces` CRUD + bulk-member add/remove).
//!
//! Shapes mirror the Go SDK (`workspaces_models.go`) one-for-one. All these
//! endpoints require a **provisioning (management) API key** — a regular
//! inference key returns 401.

use serde::{Deserialize, Serialize};

/// A workspace as returned by the workspaces endpoints.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Workspace {
    /// Stable workspace identifier.
    #[serde(default)]
    pub id: String,
    /// Display name.
    #[serde(default)]
    pub name: String,
    /// URL-safe slug.
    #[serde(default)]
    pub slug: String,
    /// Free-form description.
    #[serde(default)]
    pub description: Option<String>,
    /// User id of the workspace creator, when known.
    #[serde(default)]
    pub created_by: Option<String>,
    /// Creation timestamp.
    #[serde(default)]
    pub created_at: String,
    /// Last-update timestamp.
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Default text-model id.
    #[serde(default)]
    pub default_text_model: Option<String>,
    /// Default image-model id.
    #[serde(default)]
    pub default_image_model: Option<String>,
    /// Default provider sort (`throughput`, `price`, `latency`).
    #[serde(default)]
    pub default_provider_sort: Option<String>,
    /// API key ids whose IO is logged for this workspace.
    #[serde(default)]
    pub io_logging_api_key_ids: Option<Vec<i64>>,
    /// IO-logging sampling rate (0.0–1.0).
    #[serde(default)]
    pub io_logging_sampling_rate: f64,
    /// Enable data-discount logging.
    #[serde(default)]
    pub is_data_discount_logging_enabled: bool,
    /// Broadcast observability events.
    #[serde(default)]
    pub is_observability_broadcast_enabled: bool,
    /// Enable IO logging for observability.
    #[serde(default)]
    pub is_observability_io_logging_enabled: bool,
}

/// Optional query parameters for [`crate::Client::list_workspaces`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ListWorkspacesOptions {
    /// Skip this many rows before returning results.
    pub offset: Option<u32>,
    /// Cap on the number of rows returned.
    pub limit: Option<u32>,
}

impl ListWorkspacesOptions {
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

/// Response from `GET /workspaces`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ListWorkspacesResponse {
    /// Workspace rows.
    #[serde(default)]
    pub data: Vec<Workspace>,
    /// Total row count, ignoring pagination.
    #[serde(default)]
    pub total_count: u64,
}

/// Request body for [`crate::Client::create_workspace`].
///
/// `name` and `slug` are required; all other fields are optional.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateWorkspaceRequest {
    /// Display name.
    pub name: String,
    /// URL-safe slug.
    pub slug: String,
    /// Free-form description.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub description: Option<String>,
    /// Default text-model id.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub default_text_model: Option<String>,
    /// Default image-model id.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub default_image_model: Option<String>,
    /// Default provider sort.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub default_provider_sort: Option<String>,
    /// API key ids whose IO is logged.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub io_logging_api_key_ids: Option<Vec<i64>>,
    /// IO-logging sampling rate.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub io_logging_sampling_rate: Option<f64>,
    /// Enable data-discount logging.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub is_data_discount_logging_enabled: Option<bool>,
    /// Broadcast observability events.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub is_observability_broadcast_enabled: Option<bool>,
    /// Enable IO logging for observability.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub is_observability_io_logging_enabled: Option<bool>,
}

/// Partial-update body for [`crate::Client::update_workspace`]. Only the
/// fields you set are sent.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UpdateWorkspaceRequest {
    /// New display name.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub name: Option<String>,
    /// New URL-safe slug.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub slug: Option<String>,
    /// New description.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub description: Option<String>,
    /// New default text-model id.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub default_text_model: Option<String>,
    /// New default image-model id.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub default_image_model: Option<String>,
    /// New default provider sort.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub default_provider_sort: Option<String>,
    /// New IO-logging API key ids.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub io_logging_api_key_ids: Option<Vec<i64>>,
    /// New IO-logging sampling rate.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub io_logging_sampling_rate: Option<f64>,
    /// New data-discount logging flag.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub is_data_discount_logging_enabled: Option<bool>,
    /// New observability broadcast flag.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub is_observability_broadcast_enabled: Option<bool>,
    /// New observability IO-logging flag.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub is_observability_io_logging_enabled: Option<bool>,
}

/// Response from `GET /workspaces/{id_or_slug}`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GetWorkspaceResponse {
    /// Workspace payload.
    #[serde(default)]
    pub data: Workspace,
}

/// Response from `POST /workspaces`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateWorkspaceResponse {
    /// Newly-created workspace.
    #[serde(default)]
    pub data: Workspace,
}

/// Response from `PATCH /workspaces/{id_or_slug}`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UpdateWorkspaceResponse {
    /// Updated workspace.
    #[serde(default)]
    pub data: Workspace,
}

/// Response from `DELETE /workspaces/{id_or_slug}`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteWorkspaceResponse {
    /// True when the workspace existed and was deleted.
    #[serde(default)]
    pub deleted: bool,
}

/// Role of a member within a workspace.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkspaceMemberRole {
    /// Workspace administrator.
    Admin,
    /// Regular workspace member.
    Member,
}

/// A single workspace membership.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceMember {
    /// Membership identifier.
    #[serde(default)]
    pub id: String,
    /// Workspace this membership belongs to.
    #[serde(default)]
    pub workspace_id: String,
    /// User id of the member.
    #[serde(default)]
    pub user_id: String,
    /// Member role.
    pub role: WorkspaceMemberRole,
    /// Membership creation timestamp.
    #[serde(default)]
    pub created_at: String,
}

/// Response from `POST /workspaces/{id_or_slug}/members/add`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BulkAddWorkspaceMembersResponse {
    /// Number of members added.
    #[serde(default)]
    pub added_count: u64,
    /// Membership rows for the newly-added members.
    #[serde(default)]
    pub data: Vec<WorkspaceMember>,
}

/// Response from `POST /workspaces/{id_or_slug}/members/remove`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BulkRemoveWorkspaceMembersResponse {
    /// Number of members removed.
    #[serde(default)]
    pub removed_count: u64,
}

/// Internal body shape for bulk add/remove requests.
#[derive(Debug, Serialize)]
pub(crate) struct BulkWorkspaceMembersRequest<'a> {
    pub user_ids: &'a [String],
}
