//! Shared HTTP plumbing for endpoint methods.
//!
//! Wraps `Client::http()` with header assembly, retry-aware unary execution,
//! and a single-attempt stream opener. All endpoint methods (`chat_complete`,
//! `complete`, and their streaming variants) funnel through this module so the
//! retry / error-classification logic lives in exactly one place.

use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use reqwest::{Method, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::client::Client;
use crate::error::{Error, Result};
use crate::retry::run_with_retry;

const HTTP_REFERER: HeaderName = HeaderName::from_static("http-referer");
const X_TITLE: HeaderName = HeaderName::from_static("x-title");

/// Accept header for JSON unary requests.
const ACCEPT_JSON: &str = "application/json";
/// Accept header for SSE streaming requests.
#[allow(dead_code)] // Consumed by the streaming layer (HRA-122 / HRA-123).
pub(crate) const ACCEPT_SSE: &str = "text/event-stream";

/// Build the base header map common to every request.
fn base_headers(client: &Client, accept: &'static str) -> Result<HeaderMap> {
    let mut h = HeaderMap::with_capacity(5);
    let auth = format!("Bearer {}", client.api_key());
    let mut auth_val = HeaderValue::from_str(&auth)
        .map_err(|_| Error::InvalidInput("api_key is not header-safe"))?;
    auth_val.set_sensitive(true);
    h.insert(AUTHORIZATION, auth_val);
    h.insert(CONTENT_TYPE, HeaderValue::from_static(ACCEPT_JSON));
    h.insert(ACCEPT, HeaderValue::from_static(accept));
    if let Some(referer) = client.referer() {
        if let Ok(v) = HeaderValue::from_str(referer) {
            h.insert(HTTP_REFERER, v);
        }
    }
    if let Some(name) = client.app_name() {
        if let Ok(v) = HeaderValue::from_str(name) {
            h.insert(X_TITLE, v);
        }
    }
    Ok(h)
}

/// Resolve a relative path against the client's base URL.
fn endpoint_url(client: &Client, path: &str) -> Result<reqwest::Url> {
    let trimmed = path.trim_start_matches('/');
    client
        .base_url()
        .join(trimmed)
        .map_err(|_| Error::InvalidInput("endpoint path is not valid"))
}

/// Parse a `Retry-After` header value. Supports integer seconds; HTTP-date
/// values are ignored (treated as absent — the computed backoff will be used).
fn parse_retry_after(resp: &Response) -> Option<Duration> {
    let v = resp.headers().get(reqwest::header::RETRY_AFTER)?;
    let s = v.to_str().ok()?.trim();
    s.parse::<u64>().ok().map(Duration::from_secs)
}

/// Execute a JSON POST and decode the response as `Resp`.
///
/// Retries on transient failures using the client's [`RetryConfig`]. API errors
/// are parsed via [`Error::from_response_body`]; the response's `Retry-After`
/// header is forwarded into the error so the retry layer can honor it.
pub(crate) async fn execute_json<Req, Resp>(client: &Client, path: &str, body: &Req) -> Result<Resp>
where
    Req: Serialize + ?Sized,
    Resp: DeserializeOwned,
{
    let url = endpoint_url(client, path)?;
    let headers = base_headers(client, ACCEPT_JSON)?;
    let body_bytes = serde_json::to_vec(body)?;
    let cfg = client.retry().clone();

    run_with_retry(&cfg, || {
        let url = url.clone();
        let headers = headers.clone();
        let body_bytes = body_bytes.clone();
        async move {
            let resp = client
                .http()
                .request(Method::POST, url)
                .headers(headers)
                .body(body_bytes)
                .send()
                .await?;
            let status = resp.status();
            if status.is_success() {
                let bytes = resp.bytes().await?;
                let decoded: Resp = serde_json::from_slice(&bytes)?;
                Ok(decoded)
            } else {
                Err(api_error_from_response(resp, status).await)
            }
        }
    })
    .await
}

/// Open a streaming POST. Returns the raw `Response` on a 2xx; the caller takes
/// over via [`Response::bytes_stream`]. **Single attempt only** — stream-level
/// reconnect lives in `crate::stream`.
#[allow(dead_code)] // Consumed by the streaming layer (HRA-122 / HRA-123).
pub(crate) async fn open_stream<Req>(client: &Client, path: &str, body: &Req) -> Result<Response>
where
    Req: Serialize + ?Sized,
{
    let url = endpoint_url(client, path)?;
    let headers = base_headers(client, ACCEPT_SSE)?;
    let body_bytes = serde_json::to_vec(body)?;

    let resp = client
        .http()
        .request(Method::POST, url)
        .headers(headers)
        .body(body_bytes)
        .send()
        .await?;
    let status = resp.status();
    if status.is_success() {
        Ok(resp)
    } else {
        Err(api_error_from_response(resp, status).await)
    }
}

async fn api_error_from_response(resp: Response, status: StatusCode) -> Error {
    let retry_after = parse_retry_after(&resp);
    let body = resp.bytes().await.unwrap_or_default();
    Error::from_response_body(status.as_u16(), &body, retry_after)
}
