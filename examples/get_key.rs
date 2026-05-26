//! Show metadata about the currently authenticated API key.
//!
//! Run with:
//!
//! ```bash
//! OPENROUTER_API_KEY=sk-... cargo run --example get_key
//! ```

use openrouter::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key =
        std::env::var("OPENROUTER_API_KEY").map_err(|_| "OPENROUTER_API_KEY must be set")?;
    let client = Client::builder()
        .api_key(api_key)
        .app_name("openrouter-rust get_key example")
        .build()?;

    let resp = client.get_key().await?;
    let d = &resp.data;
    println!("label:             {}", d.label);
    println!("usage:             ${:.4}", d.usage);
    match d.limit {
        Some(l) => println!("limit:             ${l:.2}"),
        None => println!("limit:             (none)"),
    }
    if let Some(r) = d.limit_remaining {
        println!("limit_remaining:   ${r:.4}");
    }
    println!("is_free_tier:      {}", d.is_free_tier);
    println!("is_provisioning:   {}", d.is_provisioning_key);
    if let Some(rl) = &d.rate_limit {
        println!(
            "rate_limit:        {} requests / {}",
            rl.requests, rl.interval
        );
    }
    Ok(())
}
