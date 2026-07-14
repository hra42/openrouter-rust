//! Error model for the crate.

use std::time::Duration;

/// Convenient alias for `Result<T, openrouter::Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// Top-level error type for all SDK operations.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// OpenRouter API returned a structured error response.
    #[error("openrouter api error: {status} {code:?} — {message}")]
    Api {
        /// HTTP status code.
        status: u16,
        /// OpenRouter error code (e.g. `"invalid_request_error"`).
        code: Option<String>,
        /// Human-readable error message.
        message: String,
        /// Provider-supplied metadata (raw JSON).
        metadata: Option<serde_json::Value>,
        /// Provider name (when known).
        provider: Option<String>,
        /// `Retry-After` hint parsed from response headers.
        retry_after: Option<Duration>,
    },

    /// HTTP transport-level failure.
    #[error("http transport: {0}")]
    Http(#[from] reqwest::Error),

    /// Browser Fetch or ReadableStream transport failure.
    #[error("browser transport: {0}")]
    BrowserTransport(String),

    /// JSON (de)serialization failed.
    #[error("decode: {0}")]
    Decode(#[from] serde_json::Error),

    /// SSE / streaming-protocol failure.
    #[error("stream: {0}")]
    Stream(String),

    /// Retry budget was exhausted; carries the last attempt's error.
    #[error("retry exhausted after {attempts} attempt(s)")]
    RetryExhausted {
        /// Number of attempts made.
        attempts: u32,
        /// The error from the final attempt.
        #[source]
        source: Box<Error>,
    },

    /// Caller-supplied input failed validation.
    #[error("invalid input: {0}")]
    InvalidInput(&'static str),

    /// Builder missing a required field.
    #[error("builder: missing required field `{0}`")]
    MissingField(&'static str),
}

#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct ApiErrorEnvelope {
    error: ApiErrorBody,
}

#[derive(serde::Deserialize)]
struct ApiErrorBody {
    #[serde(default)]
    code: Option<serde_json::Value>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
    #[serde(default)]
    provider_name: Option<String>,
}

impl Error {
    /// Build an `Error::Api` from a status code, raw response body, and optional
    /// `Retry-After` hint. Tolerates non-JSON bodies and partial payloads.
    pub(crate) fn from_response_body(
        status: u16,
        body: &[u8],
        retry_after: Option<Duration>,
    ) -> Error {
        let parsed: Option<ApiErrorEnvelope> = serde_json::from_slice(body).ok();
        let (code, message, metadata, provider) = match parsed {
            Some(env) => {
                let code = env.error.code.and_then(|v| match v {
                    serde_json::Value::String(s) => Some(s),
                    serde_json::Value::Number(n) => Some(n.to_string()),
                    _ => None,
                });
                let message = env
                    .error
                    .message
                    .unwrap_or_else(|| String::from_utf8_lossy(body).into_owned());
                (code, message, env.error.metadata, env.error.provider_name)
            }
            None => (None, String::from_utf8_lossy(body).into_owned(), None, None),
        };
        Error::Api {
            status,
            code,
            message,
            metadata,
            provider,
            retry_after,
        }
    }

    /// Whether this error is transient and worth retrying.
    pub(crate) fn is_transient(&self) -> bool {
        match self {
            Error::Api { status, .. } => *status == 429 || (500..=599).contains(status),
            Error::Http(e) => e.is_timeout() || is_connect_error(e) || e.is_request(),
            Error::BrowserTransport(_) => true,
            _ => false,
        }
    }

    /// `Retry-After` hint, if any.
    pub(crate) fn retry_after(&self) -> Option<Duration> {
        match self {
            Error::Api { retry_after, .. } => *retry_after,
            _ => None,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn is_connect_error(error: &reqwest::Error) -> bool {
    error.is_connect()
}

#[cfg(target_arch = "wasm32")]
fn is_connect_error(_error: &reqwest::Error) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_structured_api_error() {
        let body = br#"{"error":{"code":"invalid_request_error","message":"bad model","metadata":{"raw":"foo"},"provider_name":"openai"}}"#;
        let err = Error::from_response_body(400, body, None);
        match err {
            Error::Api {
                status,
                code,
                message,
                provider,
                metadata,
                retry_after,
            } => {
                assert_eq!(status, 400);
                assert_eq!(code.as_deref(), Some("invalid_request_error"));
                assert_eq!(message, "bad model");
                assert_eq!(provider.as_deref(), Some("openai"));
                assert!(metadata.is_some());
                assert!(retry_after.is_none());
            }
            _ => panic!("expected Api"),
        }
    }

    #[test]
    fn falls_back_to_raw_body_on_non_json() {
        let err = Error::from_response_body(502, b"upstream gone", None);
        match err {
            Error::Api {
                status,
                message,
                code,
                ..
            } => {
                assert_eq!(status, 502);
                assert_eq!(message, "upstream gone");
                assert!(code.is_none());
            }
            _ => panic!("expected Api"),
        }
    }

    #[test]
    fn numeric_code_is_stringified() {
        let body = br#"{"error":{"code":429,"message":"too many"}}"#;
        let err = Error::from_response_body(429, body, Some(Duration::from_secs(3)));
        if let Error::Api {
            code, retry_after, ..
        } = err
        {
            assert_eq!(code.as_deref(), Some("429"));
            assert_eq!(retry_after, Some(Duration::from_secs(3)));
        } else {
            panic!("expected Api");
        }
    }

    #[test]
    fn is_transient_logic() {
        let server = Error::Api {
            status: 503,
            code: None,
            message: "x".into(),
            metadata: None,
            provider: None,
            retry_after: None,
        };
        let rate = Error::Api {
            status: 429,
            code: None,
            message: "x".into(),
            metadata: None,
            provider: None,
            retry_after: None,
        };
        let bad = Error::Api {
            status: 400,
            code: None,
            message: "x".into(),
            metadata: None,
            provider: None,
            retry_after: None,
        };
        assert!(server.is_transient());
        assert!(rate.is_transient());
        assert!(!bad.is_transient());
        assert!(!Error::InvalidInput("x").is_transient());
    }

    #[test]
    fn display_does_not_panic() {
        let e = Error::MissingField("api_key");
        let _ = format!("{e}");
    }
}
