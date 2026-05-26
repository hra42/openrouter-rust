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
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub created_by: Option<String>,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub default_text_model: Option<String>,
    #[serde(default)]
    pub default_image_model: Option<String>,
    #[serde(default)]
    pub default_provider_sort: Option<String>,
    #[serde(default)]
    pub io_logging_api_key_ids: Option<Vec<i64>>,
    #[serde(default)]
    pub io_logging_sampling_rate: f64,
    #[serde(default)]
    pub is_data_discount_logging_enabled: bool,
    #[serde(default)]
    pub is_observability_broadcast_enabled: bool,
    #[serde(default)]
    pub is_observability_io_logging_enabled: bool,
}

/// Optional query parameters for [`crate::Client::list_workspaces`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ListWorkspacesOptions {
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

impl ListWorkspacesOptions {
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

/// Response from `GET /workspaces`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ListWorkspacesResponse {
    #[serde(default)]
    pub data: Vec<Workspace>,
    #[serde(default)]
    pub total_count: u64,
}

/// Request body for [`crate::Client::create_workspace`].
///
/// `name` and `slug` are required; all other fields are optional.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateWorkspaceRequest {
    pub name: String,
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub default_text_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub default_image_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub default_provider_sort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub io_logging_api_key_ids: Option<Vec<i64>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub io_logging_sampling_rate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub is_data_discount_logging_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub is_observability_broadcast_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub is_observability_io_logging_enabled: Option<bool>,
}

/// Partial-update body for [`crate::Client::update_workspace`]. Only the
/// fields you set are sent.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UpdateWorkspaceRequest {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub default_text_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub default_image_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub default_provider_sort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub io_logging_api_key_ids: Option<Vec<i64>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub io_logging_sampling_rate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub is_data_discount_logging_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub is_observability_broadcast_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub is_observability_io_logging_enabled: Option<bool>,
}

/// Response from `GET /workspaces/{id_or_slug}`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GetWorkspaceResponse {
    #[serde(default)]
    pub data: Workspace,
}

/// Response from `POST /workspaces`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateWorkspaceResponse {
    #[serde(default)]
    pub data: Workspace,
}

/// Response from `PATCH /workspaces/{id_or_slug}`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UpdateWorkspaceResponse {
    #[serde(default)]
    pub data: Workspace,
}

/// Response from `DELETE /workspaces/{id_or_slug}`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteWorkspaceResponse {
    #[serde(default)]
    pub deleted: bool,
}

/// Role of a member within a workspace.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkspaceMemberRole {
    Admin,
    Member,
}

/// A single workspace membership.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceMember {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub workspace_id: String,
    #[serde(default)]
    pub user_id: String,
    pub role: WorkspaceMemberRole,
    #[serde(default)]
    pub created_at: String,
}

/// Response from `POST /workspaces/{id_or_slug}/members/add`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BulkAddWorkspaceMembersResponse {
    #[serde(default)]
    pub added_count: u64,
    #[serde(default)]
    pub data: Vec<WorkspaceMember>,
}

/// Response from `POST /workspaces/{id_or_slug}/members/remove`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BulkRemoveWorkspaceMembersResponse {
    #[serde(default)]
    pub removed_count: u64,
}

/// Internal body shape for bulk add/remove requests.
#[derive(Debug, Serialize)]
pub(crate) struct BulkWorkspaceMembersRequest<'a> {
    pub user_ids: &'a [String],
}
