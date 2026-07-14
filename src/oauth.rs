//! OAuth PKCE helpers for the OpenRouter authorization flow.
//!
//! The flow:
//!
//! 1. Generate a verifier with [`generate_code_verifier`] (random, 43-char,
//!    base64url-no-padding per RFC 7636).
//! 2. Derive a challenge with [`create_s256_code_challenge`].
//! 3. Build the user-facing authorization URL with [`build_auth_url`] and
//!    redirect the user to it.
//! 4. When OpenRouter redirects back with `?code=…`, call
//!    [`exchange_auth_code`] to swap the code for an API key.
//!
//! Shapes mirror the Go SDK (`oauth.go`, `oauth_endpoint.go`).

use rand::RngCore;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::client::Client;
use crate::error::{Error, Result};
use crate::request;

const TOKEN_ENDPOINT: &str = "https://openrouter.ai/api/v1/auth/keys";

/// PKCE code-challenge method.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodeChallengeMethod {
    /// SHA-256 hashing per RFC 7636.
    #[serde(rename = "S256")]
    S256,
    /// Plain text — verifier is the challenge.
    #[serde(rename = "plain")]
    Plain,
}

/// Generate a cryptographically random PKCE code verifier: 32 random
/// bytes encoded as base64url without padding, yielding a 43-character
/// string per RFC 7636.
pub fn generate_code_verifier() -> String {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;
    let mut buf = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut buf);
    URL_SAFE_NO_PAD.encode(buf)
}

/// Create a PKCE code challenge from a verifier using the S256 method:
/// `BASE64URL(SHA256(verifier))`.
pub fn create_s256_code_challenge(verifier: &str) -> String {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;
    use sha2::Digest;
    let digest = sha2::Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
}

/// Parameters for [`build_auth_url`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AuthUrlParams<'a> {
    /// HTTPS URL OpenRouter will redirect to after authorization
    /// (required).
    pub callback_url: &'a str,
    /// PKCE code challenge (optional but recommended).
    pub code_challenge: Option<&'a str>,
    /// Method used to compute `code_challenge` (optional).
    pub code_challenge_method: Option<CodeChallengeMethod>,
}

/// Build the user-facing authorization URL. `base_url` is typically
/// `https://openrouter.ai/auth`.
pub fn build_auth_url(base_url: &str, params: AuthUrlParams<'_>) -> Result<String> {
    if params.callback_url.is_empty() {
        return Err(Error::InvalidInput("callback_url is required"));
    }
    let mut u =
        Url::parse(base_url).map_err(|_| Error::InvalidInput("base_url is not a valid URL"))?;
    {
        let mut q = u.query_pairs_mut();
        q.append_pair("callback_url", params.callback_url);
        if let Some(c) = params.code_challenge {
            q.append_pair("code_challenge", c);
        }
        if let Some(m) = params.code_challenge_method {
            let v = match m {
                CodeChallengeMethod::S256 => "S256",
                CodeChallengeMethod::Plain => "plain",
            };
            q.append_pair("code_challenge_method", v);
        }
    }
    Ok(u.into())
}

/// Request body for [`Client::exchange_auth_code`].
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ExchangeAuthCodeRequest {
    /// Authorization code received on the callback URL.
    pub code: String,
    /// PKCE verifier matching the challenge used at auth-URL build time.
    /// Required when PKCE was used.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub code_verifier: Option<String>,
    /// Method used to derive the challenge.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub code_challenge_method: Option<CodeChallengeMethod>,
}

/// Response from `POST /auth/keys`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ExchangeAuthCodeResponse {
    /// The newly-issued API key. **Stored only once** — capture it
    /// immediately.
    #[serde(default)]
    pub key: String,
    /// Owning user id, when returned.
    #[serde(default)]
    pub user_id: Option<String>,
}

/// Exchange an OAuth authorization code without requiring an existing API key.
///
/// This is the browser-friendly entry point for the second half of the PKCE
/// flow. The authorization code is single-use, so the exchange is not retried.
pub async fn exchange_auth_code(req: &ExchangeAuthCodeRequest) -> Result<ExchangeAuthCodeResponse> {
    exchange_auth_code_at(TOKEN_ENDPOINT, req).await
}

async fn exchange_auth_code_at(
    endpoint: &str,
    req: &ExchangeAuthCodeRequest,
) -> Result<ExchangeAuthCodeResponse> {
    if req.code.is_empty() {
        return Err(Error::InvalidInput("code is required"));
    }
    let response = reqwest::Client::new()
        .post(endpoint)
        .json(req)
        .send()
        .await?;
    let status = response.status();
    let body = response.bytes().await?;
    if status.is_success() {
        Ok(serde_json::from_slice(&body)?)
    } else {
        Err(Error::from_response_body(status.as_u16(), &body, None))
    }
}

impl Client {
    /// Exchange an authorization code for an API key (`POST /auth/keys`).
    ///
    /// This is the second step of the OAuth PKCE flow, called after the
    /// user has authorized the application at OpenRouter and been
    /// redirected back with a `?code=…` query parameter. When PKCE was
    /// used to build the auth URL, [`ExchangeAuthCodeRequest::code_verifier`]
    /// must be the verifier that produced the challenge.
    pub async fn exchange_auth_code(
        &self,
        req: &ExchangeAuthCodeRequest,
    ) -> Result<ExchangeAuthCodeResponse> {
        if req.code.is_empty() {
            return Err(Error::InvalidInput("code is required"));
        }
        request::execute_json(self, "auth/keys", req).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn verifier_is_43_chars_and_url_safe() {
        let v = generate_code_verifier();
        assert_eq!(v.len(), 43);
        for c in v.chars() {
            assert!(
                c.is_ascii_alphanumeric() || c == '-' || c == '_',
                "non-urlsafe char {c}"
            );
        }
    }

    #[test]
    fn s256_challenge_known_vector() {
        // RFC 7636 §B test vector
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let expected = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
        assert_eq!(create_s256_code_challenge(verifier), expected);
    }

    #[test]
    fn build_auth_url_requires_callback() {
        let err = build_auth_url(
            "https://openrouter.ai/auth",
            AuthUrlParams {
                callback_url: "",
                ..Default::default()
            },
        )
        .unwrap_err();
        assert!(matches!(err, Error::InvalidInput(_)));
    }

    #[test]
    fn build_auth_url_appends_params() {
        let url = build_auth_url(
            "https://openrouter.ai/auth",
            AuthUrlParams {
                callback_url: "https://app.example/cb",
                code_challenge: Some("CHAL"),
                code_challenge_method: Some(CodeChallengeMethod::S256),
            },
        )
        .unwrap();
        assert!(url.contains("callback_url=https%3A%2F%2Fapp.example%2Fcb"));
        assert!(url.contains("code_challenge=CHAL"));
        assert!(url.contains("code_challenge_method=S256"));
    }

    #[tokio::test]
    async fn public_exchange_does_not_need_an_api_key() {
        let server = MockServer::start().await;
        let request = ExchangeAuthCodeRequest {
            code: "oauth-code".into(),
            code_verifier: Some("verifier".into()),
            code_challenge_method: Some(CodeChallengeMethod::S256),
        };
        Mock::given(method("POST"))
            .and(path("/auth/keys"))
            .and(body_json(&request))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "key": "sk-user",
                "user_id": "user-1"
            })))
            .expect(1)
            .mount(&server)
            .await;

        let endpoint = format!("{}/auth/keys", server.uri());
        let response = exchange_auth_code_at(&endpoint, &request).await.unwrap();
        assert_eq!(response.key, "sk-user");
        assert_eq!(response.user_id.as_deref(), Some("user-1"));
    }
}
