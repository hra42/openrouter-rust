//! List available OpenRouter models, optionally filtered by category.
//!
//! Run with:
//!
//! ```bash
//! OPENROUTER_API_KEY=sk-... cargo run --example list_models
//! ```

use openrouter::{Client, ListModelsOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key =
        std::env::var("OPENROUTER_API_KEY").map_err(|_| "OPENROUTER_API_KEY must be set")?;
    let client = Client::builder()
        .api_key(api_key)
        .app_name("openrouter-rust list_models example")
        .build()?;

    let opts = ListModelsOptions::new().category("programming");
    let resp = client.list_models(Some(&opts)).await?;

    println!("{} models in category 'programming'", resp.data.len());
    for model in resp.data.iter().take(10) {
        let ctx = model
            .context_length
            .map(|c| format!("{c:.0} tok"))
            .unwrap_or_else(|| "n/a".into());
        println!(
            "  {:<48}  ctx={:<10}  prompt={}/Mtok",
            model.id, ctx, model.pricing.prompt
        );
    }
    Ok(())
}
