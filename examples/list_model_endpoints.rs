//! Compare provider endpoints (prices, context length, status) for one model.
//!
//! Run with:
//!
//! ```bash
//! OPENROUTER_API_KEY=sk-... cargo run --example list_model_endpoints
//! ```

use openrouter::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key =
        std::env::var("OPENROUTER_API_KEY").map_err(|_| "OPENROUTER_API_KEY must be set")?;
    let client = Client::builder()
        .api_key(api_key)
        .app_name("openrouter-rust list_model_endpoints example")
        .build()?;

    let resp = client
        .list_model_endpoints("google", "gemini-3.1-flash-lite")
        .await?;
    println!(
        "{} — {} endpoint(s)",
        resp.data.id,
        resp.data.endpoints.len()
    );
    println!(
        "{:<32}  {:>14}  {:>14}  {:>10}  {:>8}",
        "provider", "prompt $/tok", "completion $/tok", "ctx", "status"
    );
    for ep in &resp.data.endpoints {
        println!(
            "{:<32}  {:>14}  {:>14}  {:>10.0}  {:>8.2}",
            ep.provider_name,
            ep.pricing.prompt,
            ep.pricing.completion,
            ep.context_length,
            ep.status
        );
    }
    Ok(())
}
