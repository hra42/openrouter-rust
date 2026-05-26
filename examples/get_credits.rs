//! Show the authenticated key's credit balance.
//!
//! Run with:
//!
//! ```bash
//! OPENROUTER_API_KEY=sk-... cargo run --example get_credits
//! ```

use openrouter::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key =
        std::env::var("OPENROUTER_API_KEY").map_err(|_| "OPENROUTER_API_KEY must be set")?;
    let client = Client::builder()
        .api_key(api_key)
        .app_name("openrouter-rust get_credits example")
        .build()?;

    let resp = client.get_credits().await?;
    println!("purchased: ${:.4}", resp.data.total_credits);
    println!("used:      ${:.4}", resp.data.total_usage);
    println!("remaining: ${:.4}", resp.data.remaining());
    Ok(())
}
