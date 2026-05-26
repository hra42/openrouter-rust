//! Demonstrate the OAuth PKCE helpers: verifier → challenge → auth URL →
//! (callback) → code exchange.
//!
//! This example only prints the values needed to drive the flow manually
//! — it does not start an HTTP server. To actually finish the exchange,
//! set `OPENROUTER_AUTH_CODE` to the `code` your callback received.
//!
//! Run with:
//!
//! ```bash
//! cargo run --example oauth_pkce
//! # then, after redirect:
//! OPENROUTER_AUTH_CODE=… cargo run --example oauth_pkce
//! ```

use openrouter::oauth::{
    build_auth_url, create_s256_code_challenge, generate_code_verifier, AuthUrlParams,
    CodeChallengeMethod, ExchangeAuthCodeRequest,
};
use openrouter::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let verifier = generate_code_verifier();
    let challenge = create_s256_code_challenge(&verifier);

    let callback = std::env::var("OPENROUTER_OAUTH_CALLBACK")
        .unwrap_or_else(|_| "https://localhost:3000/oauth/callback".into());

    let auth_url = build_auth_url(
        "https://openrouter.ai/auth",
        AuthUrlParams {
            callback_url: &callback,
            code_challenge: Some(&challenge),
            code_challenge_method: Some(CodeChallengeMethod::S256),
        },
    )?;

    println!("Verifier (KEEP THIS): {verifier}");
    println!("Challenge:            {challenge}");
    println!("\nRedirect the user to:\n  {auth_url}");

    let Ok(code) = std::env::var("OPENROUTER_AUTH_CODE") else {
        println!(
            "\nSet OPENROUTER_AUTH_CODE=<code-from-callback> (and re-run with the SAME verifier) to exchange."
        );
        return Ok(());
    };
    // For the exchange, the API key on the client is ignored by /auth/keys,
    // but Client requires a non-empty value.
    let client = Client::builder()
        .api_key("placeholder")
        .app_name("openrouter-rust oauth_pkce example")
        .build()?;
    let resp = client
        .exchange_auth_code(&ExchangeAuthCodeRequest {
            code,
            code_verifier: Some(verifier),
            code_challenge_method: Some(CodeChallengeMethod::S256),
        })
        .await?;
    println!("\nNew API key: {}", resp.key);
    if let Some(uid) = resp.user_id {
        println!("Issued to user: {uid}");
    }
    Ok(())
}
