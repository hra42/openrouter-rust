//! Guardrails demo: create a guardrail, list, fetch, update, delete.
//!
//! Also shows the ZDR endpoints listing (no provisioning key needed for
//! `list_zdr_endpoints`).
//!
//! Requires a **provisioning key** for the CRUD/assignment portion. Without
//! `OPENROUTER_PROVISIONING_KEY` set this example only lists ZDR endpoints
//! (using `OPENROUTER_API_KEY`) and exits cleanly.
//!
//! Run with:
//!
//! ```bash
//! OPENROUTER_PROVISIONING_KEY=sk-... cargo run --example guardrails
//! ```

use openrouter::{Client, CreateGuardrailRequest, ResetInterval, UpdateGuardrailRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(api_key) = std::env::var("OPENROUTER_API_KEY") {
        let read_only = Client::builder()
            .api_key(api_key)
            .app_name("openrouter-rust guardrails example (zdr)")
            .build()?;
        let zdr = read_only.list_zdr_endpoints().await?;
        println!("ZDR-compatible endpoints: {}", zdr.data.len());
        for ep in zdr.data.iter().take(3) {
            println!(
                "  {:<32}  {:<20}  ctx={}",
                ep.model_id, ep.provider_name, ep.context_length,
            );
        }
        if zdr.data.len() > 3 {
            println!("  … and {} more", zdr.data.len() - 3);
        }
    }

    let Ok(api_key) = std::env::var("OPENROUTER_PROVISIONING_KEY") else {
        eprintln!(
            "\nOPENROUTER_PROVISIONING_KEY not set — skipping the CRUD demo (a provisioning key is required)."
        );
        return Ok(());
    };
    let client = Client::builder()
        .api_key(api_key)
        .app_name("openrouter-rust guardrails example")
        .build()?;

    let name = format!("rust-sdk-demo-{}", std::process::id());
    println!("\nCreating guardrail '{name}' ($5/day cap, ZDR enforced)…");
    let created = client
        .create_guardrail(&CreateGuardrailRequest {
            name: name.clone(),
            description: Some("Created by the Rust SDK guardrails example".into()),
            limit_usd: Some(5.0),
            reset_interval: Some(ResetInterval::Daily),
            enforce_zdr: Some(true),
            ..Default::default()
        })
        .await?;
    let id = created.id.clone();
    println!("  id: {id}");

    println!("\nFetching back…");
    let g = client.get_guardrail(&id).await?;
    println!(
        "  name={} limit_usd={:?} reset={:?}",
        g.name, g.limit_usd, g.reset_interval,
    );

    println!("\nUpdating description…");
    client
        .update_guardrail(
            &id,
            &UpdateGuardrailRequest {
                description: Some("updated by example".into()),
                ..Default::default()
            },
        )
        .await?;

    println!("\nDeleting guardrail (irreversible)…");
    let del = client.delete_guardrail(&id).await?;
    println!("  deleted: {}", del.deleted);
    Ok(())
}
