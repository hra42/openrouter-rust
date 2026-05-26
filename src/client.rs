//! `Client` and `ClientBuilder`.

use std::sync::Arc;
use std::time::Duration;

use url::Url;

use futures::FutureExt;

use crate::error::{Error, Result};
use crate::request;
use crate::retry::RetryConfig;
use crate::stream::EventStream;
use crate::types::{
    ActivityOptions, ActivityResponse, AssignKeysRequest, AssignKeysResponse, AssignMembersRequest,
    AssignMembersResponse, BulkAddWorkspaceMembersResponse, BulkRemoveWorkspaceMembersResponse,
    BulkWorkspaceMembersRequest, ChatCompletionRequest, ChatCompletionResponse, CompletionRequest,
    CompletionResponse, CreateGuardrailRequest, CreateKeyRequest, CreateKeyResponse,
    CreateWorkspaceRequest, CreateWorkspaceResponse, CreditsResponse, DeleteGuardrailResponse,
    DeleteKeyResponse, DeleteWorkspaceResponse, GetKeyByHashResponse, GetWorkspaceResponse,
    Guardrail, KeyResponse, ListGuardrailKeyAssignmentsResponse,
    ListGuardrailMemberAssignmentsResponse, ListGuardrailsOptions, ListGuardrailsResponse,
    ListKeysOptions, ListKeysResponse, ListModelsOptions, ListOrganizationMembersOptions,
    ListOrganizationMembersResponse, ListWorkspacesOptions, ListWorkspacesResponse,
    ModelEndpointsResponse, ModelsResponse, Provider, ProvidersResponse, RerankRequest,
    RerankResponse, UpdateGuardrailRequest, UpdateKeyRequest, UpdateKeyResponse,
    UpdateWorkspaceRequest, UpdateWorkspaceResponse, ZdrEndpointsResponse,
};

const DEFAULT_BASE_URL: &str = "https://openrouter.ai/api/v1/";

/// The OpenRouter client. Cheap to `Clone` (internally an `Arc`).
#[derive(Clone, Debug)]
pub struct Client {
    inner: Arc<ClientInner>,
}

#[derive(Debug)]
struct ClientInner {
    api_key: String,
    base_url: Url,
    http: reqwest::Client,
    retry: RetryConfig,
    app_name: Option<String>,
    referer: Option<String>,
}

impl Client {
    /// Start a new `ClientBuilder`.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// Build a client with only an API key, using all other defaults.
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::builder().api_key(api_key).build()
    }

    /// Configured API key.
    pub fn api_key(&self) -> &str {
        &self.inner.api_key
    }

    /// Configured base URL.
    pub fn base_url(&self) -> &Url {
        &self.inner.base_url
    }

    /// Underlying `reqwest::Client`.
    pub fn http(&self) -> &reqwest::Client {
        &self.inner.http
    }

    /// Active retry configuration.
    pub fn retry(&self) -> &RetryConfig {
        &self.inner.retry
    }

    /// Optional app-attribution name (sent as `X-Title` in Phase 2+).
    pub fn app_name(&self) -> Option<&str> {
        self.inner.app_name.as_deref()
    }

    /// Optional referer (sent as `HTTP-Referer` in Phase 2+).
    pub fn referer(&self) -> Option<&str> {
        self.inner.referer.as_deref()
    }

    /// Send a chat-completions request and decode the unary response.
    ///
    /// Retries transient failures per the client's [`RetryConfig`]. If `req.stream`
    /// is set, it is forced to `Some(false)` so a caller-set `stream: true` cannot
    /// subvert the unary endpoint.
    pub async fn chat_complete(
        &self,
        mut req: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        req.stream = Some(false);
        apply_model_suffix(&mut req.model, &mut req.provider);
        request::execute_json(self, "chat/completions", &req).await
    }

    /// Send a legacy text-completions request and decode the unary response.
    ///
    /// `req.stream` is forced to `Some(false)` for the same reason as
    /// [`Client::chat_complete`].
    pub async fn complete(&self, mut req: CompletionRequest) -> Result<CompletionResponse> {
        req.stream = Some(false);
        apply_model_suffix(&mut req.model, &mut req.provider);
        request::execute_json(self, "completions", &req).await
    }

    /// Open a streaming chat-completions request.
    ///
    /// Returns an [`EventStream<ChatCompletionResponse>`]; each yielded chunk
    /// carries `delta` (instead of `message`) on its `choices`. The stream
    /// terminates cleanly on the `[DONE]` SSE marker.
    ///
    /// Transient mid-stream disconnects (timeouts, 5xx, 429) trigger a
    /// reconnect with exponential backoff capped at `MAX_RECONNECT_BACKOFF`,
    /// re-sending the original request body. Dropping the returned stream
    /// cancels the underlying connection.
    pub async fn chat_complete_stream(
        &self,
        mut req: ChatCompletionRequest,
    ) -> Result<EventStream<ChatCompletionResponse>> {
        req.stream = Some(true);
        apply_model_suffix(&mut req.model, &mut req.provider);
        self.open_event_stream("chat/completions", &req).await
    }

    /// Open a streaming legacy completions request. Semantics mirror
    /// [`Client::chat_complete_stream`]; chunks deserialize into
    /// `CompletionResponse` with the streaming `text` delta on each choice.
    pub async fn complete_stream(
        &self,
        mut req: CompletionRequest,
    ) -> Result<EventStream<CompletionResponse>> {
        req.stream = Some(true);
        apply_model_suffix(&mut req.model, &mut req.provider);
        self.open_event_stream("completions", &req).await
    }

    /// List available models on OpenRouter.
    ///
    /// `GET /models`. Supports an optional category filter
    /// ([`ListModelsOptions::category`]) and supported-parameter filter.
    /// The `supported_parameters` field of each [`crate::Model`] is the union
    /// of parameters across all providers — a single provider may not offer
    /// every listed parameter.
    pub async fn list_models(&self, opts: Option<&ListModelsOptions>) -> Result<ModelsResponse> {
        let query = opts.map(ListModelsOptions::to_query).unwrap_or_default();
        request::execute_json_get(self, "models", &query).await
    }

    /// List per-provider endpoints for a single model.
    ///
    /// `GET /models/{author}/{slug}/endpoints`. The response includes pricing,
    /// status, context length, uptime, quantization, and supported parameters
    /// for every provider serving the model — useful for routing or price
    /// comparison.
    pub async fn list_model_endpoints(
        &self,
        author: &str,
        slug: &str,
    ) -> Result<ModelEndpointsResponse> {
        if author.is_empty() {
            return Err(Error::InvalidInput("author cannot be empty"));
        }
        if slug.is_empty() {
            return Err(Error::InvalidInput("slug cannot be empty"));
        }
        let path = format!(
            "models/{}/{}/endpoints",
            percent_encode_segment(author),
            percent_encode_segment(slug),
        );
        request::execute_json_get(self, &path, &[]).await
    }

    /// List all providers available through OpenRouter.
    ///
    /// `GET /providers`. Returns the provider name, slug, and policy /
    /// status-page URLs (when published).
    pub async fn list_providers(&self) -> Result<ProvidersResponse> {
        request::execute_json_get(self, "providers", &[]).await
    }

    /// Retrieve the authenticated user's purchased credits and total usage.
    ///
    /// `GET /credits`. Use [`crate::CreditsData::remaining`] for the available
    /// balance.
    pub async fn get_credits(&self) -> Result<CreditsResponse> {
        request::execute_json_get(self, "credits", &[]).await
    }

    /// Retrieve metadata about the currently authenticated API key.
    ///
    /// `GET /key`. Returns label, configured spend limit, current usage,
    /// remaining balance, free-tier flag, provisioning-key flag, and any
    /// configured rate limit.
    pub async fn get_key(&self) -> Result<KeyResponse> {
        request::execute_json_get(self, "key", &[]).await
    }

    /// Daily activity grouped by model endpoint for the last 30 completed UTC days.
    ///
    /// `GET /activity`. **Requires a provisioning key** — using a regular
    /// inference key returns 401. If [`ActivityOptions::date`] is set
    /// (`YYYY-MM-DD`), results are filtered to that single UTC day.
    ///
    /// When ingesting on a schedule, wait ~30 minutes past the UTC boundary
    /// before requesting the previous day: events are aggregated by request
    /// start time, and some reasoning models take a few minutes to complete.
    pub async fn get_activity(&self, opts: Option<&ActivityOptions>) -> Result<ActivityResponse> {
        let query = opts.map(ActivityOptions::to_query).unwrap_or_default();
        request::execute_json_get(self, "activity", &query).await
    }

    /// List all API keys on the account.
    ///
    /// `GET /keys`. **Requires a provisioning key.** Supports `offset` and
    /// `include_disabled` filters via [`ListKeysOptions`].
    pub async fn list_keys(&self, opts: Option<&ListKeysOptions>) -> Result<ListKeysResponse> {
        let query = opts
            .copied()
            .map(ListKeysOptions::to_query)
            .unwrap_or_default();
        request::execute_json_get(self, "keys", &query).await
    }

    /// Look up a single API key by its `hash` (returned from
    /// [`Client::list_keys`] or [`Client::create_key`]).
    ///
    /// `GET /keys/{hash}`. **Requires a provisioning key.**
    pub async fn get_key_by_hash(&self, hash: &str) -> Result<GetKeyByHashResponse> {
        if hash.is_empty() {
            return Err(Error::InvalidInput("hash cannot be empty"));
        }
        let path = format!("keys/{}", percent_encode_segment(hash));
        request::execute_json_get(self, &path, &[]).await
    }

    /// Create a new API key.
    ///
    /// `POST /keys`. **Requires a provisioning key.** The plaintext key is
    /// returned **only once** in [`CreateKeyResponse::key`] — store it
    /// immediately, it cannot be recovered later.
    pub async fn create_key(&self, req: &CreateKeyRequest) -> Result<CreateKeyResponse> {
        if req.name.is_empty() {
            return Err(Error::InvalidInput("name is required"));
        }
        request::execute_json(self, "keys", req).await
    }

    /// Update an existing API key by hash. Pass only the fields you want to
    /// change on [`UpdateKeyRequest`].
    ///
    /// `PATCH /keys/{hash}`. **Requires a provisioning key.**
    pub async fn update_key(
        &self,
        hash: &str,
        req: &UpdateKeyRequest,
    ) -> Result<UpdateKeyResponse> {
        if hash.is_empty() {
            return Err(Error::InvalidInput("hash cannot be empty"));
        }
        let path = format!("keys/{}", percent_encode_segment(hash));
        request::execute_json_method(self, reqwest::Method::PATCH, &path, Some(req)).await
    }

    /// Delete an API key by hash.
    ///
    /// `DELETE /keys/{hash}`. **Requires a provisioning key.** This operation
    /// is irreversible — the deleted key cannot be restored, and any clients
    /// still using it will immediately start receiving 401s.
    pub async fn delete_key(&self, hash: &str) -> Result<DeleteKeyResponse> {
        if hash.is_empty() {
            return Err(Error::InvalidInput("hash cannot be empty"));
        }
        let path = format!("keys/{}", percent_encode_segment(hash));
        request::execute_json_method::<(), _>(self, reqwest::Method::DELETE, &path, None).await
    }

    /// List guardrails for the organization.
    ///
    /// `GET /guardrails`. **Requires a provisioning key.**
    pub async fn list_guardrails(
        &self,
        opts: Option<&ListGuardrailsOptions>,
    ) -> Result<ListGuardrailsResponse> {
        let query = opts
            .copied()
            .map(ListGuardrailsOptions::to_query)
            .unwrap_or_default();
        request::execute_json_get(self, "guardrails", &query).await
    }

    /// Create a new guardrail. `name` is required.
    ///
    /// `POST /guardrails`. **Requires a provisioning key.**
    pub async fn create_guardrail(&self, req: &CreateGuardrailRequest) -> Result<Guardrail> {
        if req.name.is_empty() {
            return Err(Error::InvalidInput("name is required"));
        }
        request::execute_json(self, "guardrails", req).await
    }

    /// Fetch a single guardrail by ID.
    ///
    /// `GET /guardrails/{id}`. **Requires a provisioning key.**
    pub async fn get_guardrail(&self, id: &str) -> Result<Guardrail> {
        if id.is_empty() {
            return Err(Error::InvalidInput("id cannot be empty"));
        }
        let path = format!("guardrails/{}", percent_encode_segment(id));
        request::execute_json_get(self, &path, &[]).await
    }

    /// Update an existing guardrail. Pass only the fields you want to change
    /// on [`UpdateGuardrailRequest`].
    ///
    /// `PATCH /guardrails/{id}`. **Requires a provisioning key.**
    pub async fn update_guardrail(
        &self,
        id: &str,
        req: &UpdateGuardrailRequest,
    ) -> Result<Guardrail> {
        if id.is_empty() {
            return Err(Error::InvalidInput("id cannot be empty"));
        }
        let path = format!("guardrails/{}", percent_encode_segment(id));
        request::execute_json_method(self, reqwest::Method::PATCH, &path, Some(req)).await
    }

    /// Delete a guardrail by ID. **Irreversible.**
    ///
    /// `DELETE /guardrails/{id}`. **Requires a provisioning key.**
    pub async fn delete_guardrail(&self, id: &str) -> Result<DeleteGuardrailResponse> {
        if id.is_empty() {
            return Err(Error::InvalidInput("id cannot be empty"));
        }
        let path = format!("guardrails/{}", percent_encode_segment(id));
        request::execute_json_method::<(), _>(self, reqwest::Method::DELETE, &path, None).await
    }

    /// List key assignments across all guardrails.
    ///
    /// `GET /guardrails/key-assignments`. **Requires a provisioning key.**
    pub async fn list_all_guardrail_key_assignments(
        &self,
        opts: Option<&ListGuardrailsOptions>,
    ) -> Result<ListGuardrailKeyAssignmentsResponse> {
        let query = opts
            .copied()
            .map(ListGuardrailsOptions::to_query)
            .unwrap_or_default();
        request::execute_json_get(self, "guardrails/key-assignments", &query).await
    }

    /// List key assignments for a specific guardrail.
    ///
    /// `GET /guardrails/{id}/key-assignments`. **Requires a provisioning key.**
    pub async fn list_guardrail_key_assignments(
        &self,
        id: &str,
        opts: Option<&ListGuardrailsOptions>,
    ) -> Result<ListGuardrailKeyAssignmentsResponse> {
        if id.is_empty() {
            return Err(Error::InvalidInput("id cannot be empty"));
        }
        let path = format!("guardrails/{}/key-assignments", percent_encode_segment(id));
        let query = opts
            .copied()
            .map(ListGuardrailsOptions::to_query)
            .unwrap_or_default();
        request::execute_json_get(self, &path, &query).await
    }

    /// Assign API keys (by hash) to a guardrail.
    ///
    /// `POST /guardrails/{id}/key-assignments`. **Requires a provisioning
    /// key.**
    pub async fn assign_keys_to_guardrail(
        &self,
        id: &str,
        req: &AssignKeysRequest,
    ) -> Result<AssignKeysResponse> {
        if id.is_empty() {
            return Err(Error::InvalidInput("id cannot be empty"));
        }
        if req.key_hashes.is_empty() {
            return Err(Error::InvalidInput("key_hashes cannot be empty"));
        }
        let path = format!("guardrails/{}/key-assignments", percent_encode_segment(id));
        request::execute_json(self, &path, req).await
    }

    /// Remove key assignments from a guardrail.
    ///
    /// `DELETE /guardrails/{id}/key-assignments` (with body). **Requires a
    /// provisioning key.**
    pub async fn unassign_keys_from_guardrail(
        &self,
        id: &str,
        req: &AssignKeysRequest,
    ) -> Result<()> {
        if id.is_empty() {
            return Err(Error::InvalidInput("id cannot be empty"));
        }
        if req.key_hashes.is_empty() {
            return Err(Error::InvalidInput("key_hashes cannot be empty"));
        }
        let path = format!("guardrails/{}/key-assignments", percent_encode_segment(id));
        request::execute_no_content_method(self, reqwest::Method::DELETE, &path, Some(req)).await
    }

    /// List member assignments across all guardrails.
    ///
    /// `GET /guardrails/member-assignments`. **Requires a provisioning key.**
    pub async fn list_all_guardrail_member_assignments(
        &self,
        opts: Option<&ListGuardrailsOptions>,
    ) -> Result<ListGuardrailMemberAssignmentsResponse> {
        let query = opts
            .copied()
            .map(ListGuardrailsOptions::to_query)
            .unwrap_or_default();
        request::execute_json_get(self, "guardrails/member-assignments", &query).await
    }

    /// List member assignments for a specific guardrail.
    ///
    /// `GET /guardrails/{id}/member-assignments`. **Requires a provisioning
    /// key.**
    pub async fn list_guardrail_member_assignments(
        &self,
        id: &str,
        opts: Option<&ListGuardrailsOptions>,
    ) -> Result<ListGuardrailMemberAssignmentsResponse> {
        if id.is_empty() {
            return Err(Error::InvalidInput("id cannot be empty"));
        }
        let path = format!(
            "guardrails/{}/member-assignments",
            percent_encode_segment(id)
        );
        let query = opts
            .copied()
            .map(ListGuardrailsOptions::to_query)
            .unwrap_or_default();
        request::execute_json_get(self, &path, &query).await
    }

    /// Assign organization members (by user id) to a guardrail.
    ///
    /// `POST /guardrails/{id}/member-assignments`. **Requires a provisioning
    /// key.**
    pub async fn assign_members_to_guardrail(
        &self,
        id: &str,
        req: &AssignMembersRequest,
    ) -> Result<AssignMembersResponse> {
        if id.is_empty() {
            return Err(Error::InvalidInput("id cannot be empty"));
        }
        if req.member_user_ids.is_empty() {
            return Err(Error::InvalidInput("member_user_ids cannot be empty"));
        }
        let path = format!(
            "guardrails/{}/member-assignments",
            percent_encode_segment(id)
        );
        request::execute_json(self, &path, req).await
    }

    /// Remove member assignments from a guardrail.
    ///
    /// `DELETE /guardrails/{id}/member-assignments` (with body). **Requires a
    /// provisioning key.**
    pub async fn unassign_members_from_guardrail(
        &self,
        id: &str,
        req: &AssignMembersRequest,
    ) -> Result<()> {
        if id.is_empty() {
            return Err(Error::InvalidInput("id cannot be empty"));
        }
        if req.member_user_ids.is_empty() {
            return Err(Error::InvalidInput("member_user_ids cannot be empty"));
        }
        let path = format!(
            "guardrails/{}/member-assignments",
            percent_encode_segment(id)
        );
        request::execute_no_content_method(self, reqwest::Method::DELETE, &path, Some(req)).await
    }

    /// Rerank documents against a query using a reranking model
    /// (e.g. `cohere/rerank-v3.5`).
    ///
    /// `POST /rerank`. Returns results sorted by descending relevance score.
    /// `model`, `query`, and at least one document are required.
    pub async fn rerank(&self, req: &RerankRequest) -> Result<RerankResponse> {
        if req.model.is_empty() {
            return Err(Error::InvalidInput("model is required"));
        }
        if req.query.is_empty() {
            return Err(Error::InvalidInput("query is required"));
        }
        if req.documents.is_empty() {
            return Err(Error::InvalidInput("documents must not be empty"));
        }
        request::execute_json(self, "rerank", req).await
    }

    /// List endpoints compatible with Zero Data Retention.
    ///
    /// `GET /endpoints/zdr`. Returns the endpoints that honor ZDR across all
    /// providers — useful as a preview before enforcing ZDR on a guardrail or
    /// key. No authentication tier requirement beyond a normal API key.
    pub async fn list_zdr_endpoints(&self) -> Result<ZdrEndpointsResponse> {
        request::execute_json_get(self, "endpoints/zdr", &[]).await
    }

    /// List members of the organization associated with the authenticated
    /// management key.
    ///
    /// `GET /organization/members`. **Requires a provisioning key.** Supports
    /// `offset` / `limit` pagination via [`ListOrganizationMembersOptions`].
    pub async fn list_organization_members(
        &self,
        opts: Option<&ListOrganizationMembersOptions>,
    ) -> Result<ListOrganizationMembersResponse> {
        let query = opts
            .copied()
            .map(ListOrganizationMembersOptions::to_query)
            .unwrap_or_default();
        request::execute_json_get(self, "organization/members", &query).await
    }

    /// List workspaces on the organization.
    ///
    /// `GET /workspaces`. **Requires a provisioning (management) API key.**
    /// Supports `offset` / `limit` pagination via [`ListWorkspacesOptions`].
    pub async fn list_workspaces(
        &self,
        opts: Option<&ListWorkspacesOptions>,
    ) -> Result<ListWorkspacesResponse> {
        let query = opts
            .copied()
            .map(ListWorkspacesOptions::to_query)
            .unwrap_or_default();
        request::execute_json_get(self, "workspaces", &query).await
    }

    /// Create a new workspace.
    ///
    /// `POST /workspaces`. **Requires a provisioning key.** `name` and `slug`
    /// must be non-empty.
    pub async fn create_workspace(
        &self,
        req: &CreateWorkspaceRequest,
    ) -> Result<CreateWorkspaceResponse> {
        if req.name.is_empty() {
            return Err(Error::InvalidInput("name is required"));
        }
        if req.slug.is_empty() {
            return Err(Error::InvalidInput("slug is required"));
        }
        request::execute_json(self, "workspaces", req).await
    }

    /// Fetch a single workspace by UUID or slug.
    ///
    /// `GET /workspaces/{id_or_slug}`. **Requires a provisioning key.**
    pub async fn get_workspace(&self, id_or_slug: &str) -> Result<GetWorkspaceResponse> {
        if id_or_slug.is_empty() {
            return Err(Error::InvalidInput("id_or_slug cannot be empty"));
        }
        let path = format!("workspaces/{}", percent_encode_segment(id_or_slug));
        request::execute_json_get(self, &path, &[]).await
    }

    /// Update an existing workspace by UUID or slug. Pass only the fields you
    /// want to change on [`UpdateWorkspaceRequest`].
    ///
    /// `PATCH /workspaces/{id_or_slug}`. **Requires a provisioning key.**
    pub async fn update_workspace(
        &self,
        id_or_slug: &str,
        req: &UpdateWorkspaceRequest,
    ) -> Result<UpdateWorkspaceResponse> {
        if id_or_slug.is_empty() {
            return Err(Error::InvalidInput("id_or_slug cannot be empty"));
        }
        let path = format!("workspaces/{}", percent_encode_segment(id_or_slug));
        request::execute_json_method(self, reqwest::Method::PATCH, &path, Some(req)).await
    }

    /// Delete a workspace by UUID or slug.
    ///
    /// `DELETE /workspaces/{id_or_slug}`. **Requires a provisioning key.** The
    /// default workspace cannot be deleted, and any workspace with active API
    /// keys returns an error.
    pub async fn delete_workspace(&self, id_or_slug: &str) -> Result<DeleteWorkspaceResponse> {
        if id_or_slug.is_empty() {
            return Err(Error::InvalidInput("id_or_slug cannot be empty"));
        }
        let path = format!("workspaces/{}", percent_encode_segment(id_or_slug));
        request::execute_json_method::<(), _>(self, reqwest::Method::DELETE, &path, None).await
    }

    /// Bulk-add organization members to a workspace. Members are assigned the
    /// same role they hold in the organization.
    ///
    /// `POST /workspaces/{id_or_slug}/members/add`. **Requires a provisioning
    /// key.**
    pub async fn add_workspace_members(
        &self,
        id_or_slug: &str,
        user_ids: &[String],
    ) -> Result<BulkAddWorkspaceMembersResponse> {
        if id_or_slug.is_empty() {
            return Err(Error::InvalidInput("id_or_slug cannot be empty"));
        }
        if user_ids.is_empty() {
            return Err(Error::InvalidInput("user_ids cannot be empty"));
        }
        let path = format!(
            "workspaces/{}/members/add",
            percent_encode_segment(id_or_slug)
        );
        let body = BulkWorkspaceMembersRequest { user_ids };
        request::execute_json(self, &path, &body).await
    }

    /// Bulk-remove members from a workspace. Members with active API keys in
    /// the workspace cannot be removed.
    ///
    /// `POST /workspaces/{id_or_slug}/members/remove`. **Requires a
    /// provisioning key.**
    pub async fn remove_workspace_members(
        &self,
        id_or_slug: &str,
        user_ids: &[String],
    ) -> Result<BulkRemoveWorkspaceMembersResponse> {
        if id_or_slug.is_empty() {
            return Err(Error::InvalidInput("id_or_slug cannot be empty"));
        }
        if user_ids.is_empty() {
            return Err(Error::InvalidInput("user_ids cannot be empty"));
        }
        let path = format!(
            "workspaces/{}/members/remove",
            percent_encode_segment(id_or_slug)
        );
        let body = BulkWorkspaceMembersRequest { user_ids };
        request::execute_json(self, &path, &body).await
    }

    /// Internal: serialize the request once, open the first stream, and build
    /// a reconnect closure that re-issues the same body on transient failure.
    async fn open_event_stream<Req, Resp>(
        &self,
        path: &'static str,
        req: &Req,
    ) -> Result<EventStream<Resp>>
    where
        Req: serde::Serialize + ?Sized,
        Resp: serde::de::DeserializeOwned,
    {
        let body_bytes = serde_json::to_vec(req)?;
        let initial = request::open_stream_bytes(self, path, body_bytes.clone()).await?;
        let client = self.clone();
        let reopen: crate::stream::Reopen = Arc::new(move || {
            let client = client.clone();
            let body_bytes = body_bytes.clone();
            async move { request::open_stream_bytes(&client, path, body_bytes).await }.boxed()
        });
        Ok(EventStream::new(initial, reopen))
    }
}

/// Builder for [`Client`].
#[derive(Debug, Default)]
pub struct ClientBuilder {
    api_key: Option<String>,
    base_url: Option<Url>,
    http_client: Option<reqwest::Client>,
    timeout: Option<Duration>,
    retry: Option<RetryConfig>,
    app_name: Option<String>,
    referer: Option<String>,
}

impl ClientBuilder {
    /// Set the API key (required).
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Override the base URL. Must be an absolute URL ending in `/`.
    pub fn base_url(mut self, url: impl AsRef<str>) -> Result<Self> {
        let mut parsed = Url::parse(url.as_ref())
            .map_err(|_| Error::InvalidInput("base_url is not a valid URL"))?;
        if !parsed.path().ends_with('/') {
            let new_path = format!("{}/", parsed.path());
            parsed.set_path(&new_path);
        }
        self.base_url = Some(parsed);
        Ok(self)
    }

    /// Supply a pre-configured `reqwest::Client`. When set, [`Self::timeout`]
    /// is ignored — configure it on the supplied client instead.
    pub fn http_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = Some(client);
        self
    }

    /// Request timeout (used only when no custom `http_client` is supplied).
    pub fn timeout(mut self, d: Duration) -> Self {
        self.timeout = Some(d);
        self
    }

    /// Configure retries with a max attempt count and base delay.
    pub fn retry(mut self, max: u32, base_delay: Duration) -> Self {
        let cfg = RetryConfig {
            max_retries: max,
            initial_delay: base_delay,
            ..RetryConfig::default()
        };
        self.retry = Some(cfg);
        self
    }

    /// Supply a fully-specified [`RetryConfig`].
    pub fn retry_config(mut self, cfg: RetryConfig) -> Self {
        self.retry = Some(cfg);
        self
    }

    /// App attribution: sent as `X-Title` by the request layer.
    pub fn app_name(mut self, name: impl Into<String>) -> Self {
        self.app_name = Some(name.into());
        self
    }

    /// Referer attribution: sent as `HTTP-Referer` by the request layer.
    pub fn referer(mut self, referer: impl Into<String>) -> Self {
        self.referer = Some(referer.into());
        self
    }

    /// Finalize and produce a [`Client`].
    pub fn build(self) -> Result<Client> {
        let api_key = self.api_key.ok_or(Error::MissingField("api_key"))?;
        if api_key.is_empty() {
            return Err(Error::InvalidInput("api_key must not be empty"));
        }
        let base_url = match self.base_url {
            Some(u) => u,
            None => Url::parse(DEFAULT_BASE_URL).expect("DEFAULT_BASE_URL is a valid URL"),
        };
        let http = match self.http_client {
            Some(c) => c,
            None => {
                let mut b = reqwest::Client::builder();
                if let Some(t) = self.timeout {
                    b = b.timeout(t);
                }
                b.build().map_err(Error::Http)?
            }
        };
        let retry = self.retry.unwrap_or_default();
        Ok(Client {
            inner: Arc::new(ClientInner {
                api_key,
                base_url,
                http,
                retry,
                app_name: self.app_name,
                referer: self.referer,
            }),
        })
    }
}

/// Percent-encode a single URL path segment.
///
/// Encodes everything outside the unreserved set (RFC 3986 §2.3) plus `/`,
/// which is enough for OpenRouter identifiers (author slug, model slug, key
/// hash). Avoids pulling in `percent-encoding` for a few-byte helper.
pub(crate) fn percent_encode_segment(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        let unreserved = b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~');
        if unreserved {
            out.push(b as char);
        } else {
            out.push('%');
            out.push_str(&format!("{b:02X}"));
        }
    }
    out
}

/// Strip a `:nitro` or `:floor` suffix from `model` and project it onto
/// `provider.sort` (`throughput` / `price` respectively). A caller-set
/// `provider.sort` always wins — the suffix never overrides it.
pub(crate) fn apply_model_suffix(model: &mut String, provider: &mut Option<Provider>) {
    let sort = if let Some(stripped) = model.strip_suffix(":nitro") {
        let new_model = stripped.to_string();
        *model = new_model;
        "throughput"
    } else if let Some(stripped) = model.strip_suffix(":floor") {
        let new_model = stripped.to_string();
        *model = new_model;
        "price"
    } else {
        return;
    };
    let p = provider.get_or_insert_with(Provider::default);
    if p.sort.is_none() {
        p.sort = Some(sort.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn client_is_send_sync() {
        assert_send_sync::<Client>();
    }

    #[test]
    fn builder_happy_path() {
        let c = Client::builder()
            .api_key("sk-test")
            .app_name("demo")
            .referer("https://demo.example")
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();
        assert_eq!(c.api_key(), "sk-test");
        assert_eq!(c.app_name(), Some("demo"));
        assert_eq!(c.referer(), Some("https://demo.example"));
        assert_eq!(c.base_url().as_str(), DEFAULT_BASE_URL);
    }

    #[test]
    fn missing_api_key_errors() {
        let err = Client::builder().build().unwrap_err();
        assert!(matches!(err, Error::MissingField("api_key")));
    }

    #[test]
    fn empty_api_key_errors() {
        let err = Client::builder().api_key("").build().unwrap_err();
        assert!(matches!(err, Error::InvalidInput(_)));
    }

    #[test]
    fn invalid_base_url_errors() {
        let err = Client::builder().base_url("not a url").unwrap_err();
        assert!(matches!(err, Error::InvalidInput(_)));
    }

    #[test]
    fn base_url_path_gains_trailing_slash() {
        let c = Client::builder()
            .api_key("k")
            .base_url("https://example.com/v2")
            .unwrap()
            .build()
            .unwrap();
        assert!(c.base_url().as_str().ends_with('/'));
    }

    #[test]
    fn clone_shares_inner() {
        let c1 = Client::new("k").unwrap();
        let c2 = c1.clone();
        assert!(Arc::ptr_eq(&c1.inner, &c2.inner));
    }

    #[test]
    fn retry_helper_sets_fields() {
        let c = Client::builder()
            .api_key("k")
            .retry(7, Duration::from_millis(250))
            .build()
            .unwrap();
        assert_eq!(c.retry().max_retries, 7);
        assert_eq!(c.retry().initial_delay, Duration::from_millis(250));
    }

    #[test]
    fn nitro_suffix_maps_to_throughput_sort() {
        let mut m = "openai/gpt-4o:nitro".to_string();
        let mut p = None;
        apply_model_suffix(&mut m, &mut p);
        assert_eq!(m, "openai/gpt-4o");
        assert_eq!(p.unwrap().sort.as_deref(), Some("throughput"));
    }

    #[test]
    fn floor_suffix_maps_to_price_sort() {
        let mut m = "anthropic/claude-3:floor".to_string();
        let mut p = None;
        apply_model_suffix(&mut m, &mut p);
        assert_eq!(m, "anthropic/claude-3");
        assert_eq!(p.unwrap().sort.as_deref(), Some("price"));
    }

    #[test]
    fn caller_set_sort_wins_over_suffix() {
        let mut m = "openai/gpt-4o:nitro".to_string();
        let mut p = Some(Provider {
            sort: Some("latency".to_string()),
            ..Provider::default()
        });
        apply_model_suffix(&mut m, &mut p);
        assert_eq!(m, "openai/gpt-4o");
        assert_eq!(p.unwrap().sort.as_deref(), Some("latency"));
    }

    #[test]
    fn unknown_suffix_passes_through() {
        let mut m = "openai/gpt-4o:exotic".to_string();
        let mut p = None;
        apply_model_suffix(&mut m, &mut p);
        assert_eq!(m, "openai/gpt-4o:exotic");
        assert!(p.is_none());
    }
}
