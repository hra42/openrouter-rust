//! Full provisioning-key CRUD demo: list, create, fetch, update, delete.
//!
//! Requires a **provisioning key**. Gated so that running the full example
//! suite is safe — without `OPENROUTER_PROVISIONING_KEY` set, this prints a
//! notice and exits cleanly.
//!
//! Run with:
//!
//! ```bash
//! OPENROUTER_PROVISIONING_KEY=sk-... cargo run --example key_management
//! ```
//!
//! Caution: this example creates a real API key and then deletes it. The
//! secret is printed once — it cannot be recovered after deletion.

use openrouter::{Client, CreateKeyRequest, UpdateKeyRequest};

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
        .app_name("openrouter-rust key_management example")
        .build()?;

    println!("Existing keys:");
    let listing = client.list_keys(None).await?;
    for key in &listing.data {
        println!(
            "  {:<32}  disabled={}  hash={}",
            key.label, key.disabled, key.hash
        );
    }

    let label = format!("rust-sdk-demo-{}", std::process::id());
    println!("\nCreating key '{label}' with a $1 limit…");
    let created = client
        .create_key(&CreateKeyRequest {
            name: label.clone(),
            limit: Some(1.0),
            include_byok_in_limit: None,
        })
        .await?;
    let hash = created.data.hash.clone();
    println!("  hash:   {hash}");
    if let Some(secret) = &created.key {
        println!("  secret: {secret}  (only shown once)");
    }

    println!("\nFetching the key back by hash…");
    let fetched = client.get_key_by_hash(&hash).await?;
    println!("  limit:  {}", fetched.data.limit);

    println!("\nDisabling the key via update…");
    let updated = client
        .update_key(
            &hash,
            &UpdateKeyRequest {
                disabled: Some(true),
                ..Default::default()
            },
        )
        .await?;
    println!("  disabled: {}", updated.data.disabled);

    println!("\nDeleting the key (irreversible)…");
    let deleted = client.delete_key(&hash).await?;
    println!("  success: {}", deleted.data.success);
    Ok(())
}
