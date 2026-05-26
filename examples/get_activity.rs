//! Print recent daily activity grouped by model endpoint.
//!
//! Requires a **provisioning key** (regular inference keys return 401).
//!
//! Run with:
//!
//! ```bash
//! OPENROUTER_PROVISIONING_KEY=sk-... cargo run --example get_activity
//! ```

use openrouter::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Ok(api_key) = std::env::var("OPENROUTER_PROVISIONING_KEY") else {
        eprintln!(
            "OPENROUTER_PROVISIONING_KEY not set — skipping (a provisioning key is required)."
        );
        return Ok(());
    };
    let client = Client::builder()
        .api_key(api_key)
        .app_name("openrouter-rust get_activity example")
        .build()?;

    let resp = client.get_activity(None).await?;
    println!("{} activity row(s)", resp.data.len());
    for row in resp.data.iter().take(20) {
        println!(
            "  {}  {:<40}  {:<14}  reqs={:>6}  usage=${:.4}",
            row.date, row.model, row.provider_name, row.requests, row.usage,
        );
    }
    Ok(())
}
