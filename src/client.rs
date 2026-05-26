//! `Client` and `ClientBuilder`.

use std::sync::Arc;
use std::time::Duration;

use url::Url;

use crate::error::{Error, Result};
use crate::request;
use crate::retry::RetryConfig;
use crate::types::{
    ChatCompletionRequest, ChatCompletionResponse, CompletionRequest, CompletionResponse,
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
        request::execute_json(self, "chat/completions", &req).await
    }

    /// Send a legacy text-completions request and decode the unary response.
    ///
    /// `req.stream` is forced to `Some(false)` for the same reason as
    /// [`Client::chat_complete`].
    pub async fn complete(&self, mut req: CompletionRequest) -> Result<CompletionResponse> {
        req.stream = Some(false);
        request::execute_json(self, "completions", &req).await
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
}
