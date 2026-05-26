//! List all OpenRouter providers.
//!
//! Run with:
//!
//! ```bash
//! OPENROUTER_API_KEY=sk-... cargo run --example list_providers
//! ```

use openrouter::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key =
        std::env::var("OPENROUTER_API_KEY").map_err(|_| "OPENROUTER_API_KEY must be set")?;
    let client = Client::builder()
        .api_key(api_key)
        .app_name("openrouter-rust list_providers example")
        .build()?;

    let resp = client.list_providers().await?;
    println!("{} providers", resp.data.len());
    for p in &resp.data {
        println!("  {:<24}  ({})", p.name, p.slug);
        if let Some(url) = &p.privacy_policy_url {
            println!("    privacy: {url}");
        }
        if let Some(url) = &p.status_page_url {
            println!("    status:  {url}");
        }
    }
    Ok(())
}
