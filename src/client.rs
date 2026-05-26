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
    ChatCompletionRequest, ChatCompletionResponse, CompletionRequest, CompletionResponse,
    ListModelsOptions, ModelEndpointsResponse, ModelsResponse, Provider, ProvidersResponse,
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
