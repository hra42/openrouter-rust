//! Exercise provider-routing options: ordered preferences, sort strategy,
//! per-request ZDR, and the `:nitro` model suffix.
//!
//! Run with:
//!
//! ```bash
//! OPENROUTER_API_KEY=sk-... cargo run --example provider_routing
//! ```

use openrouter::{ChatCompletionRequest, Client, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key =
        std::env::var("OPENROUTER_API_KEY").map_err(|_| "OPENROUTER_API_KEY must be set")?;
    let client = Client::builder()
        .api_key(api_key)
        .app_name("openrouter-rust provider_routing example")
        .build()?;

    // Builder-style routing: prefer a single provider, require ZDR, and ask
    // OpenRouter to optimize for throughput.
    let req = ChatCompletionRequest::new(
        "google/gemini-3.1-flash-lite",
        vec![Message::user("Reply with exactly: hello")],
    )
    .with_provider_order(["google-ai-studio"])
    .with_zdr(true)
    .with_nitro();
    let resp = client.chat_complete(req).await?;
    println!(
        "routed reply: {} (provider: {})",
        resp.choices
            .first()
            .and_then(|c| c.message.as_ref())
            .and_then(|m| m.content_text())
            .unwrap_or(""),
        resp.provider.as_deref().unwrap_or("?")
    );

    // Suffix form: `:floor` gets auto-translated to provider.sort="price".
    let suffix_req = ChatCompletionRequest::new(
        "google/gemini-3.1-flash-lite:floor",
        vec![Message::user("Same reply.")],
    );
    let suffix_resp = client.chat_complete(suffix_req).await?;
    println!(
        "floor reply: {}",
        suffix_resp
            .choices
            .first()
            .and_then(|c| c.message.as_ref())
            .and_then(|m| m.content_text())
            .unwrap_or("")
    );
    Ok(())
}
