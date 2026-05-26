//! List the members of the organization associated with a provisioning key.
//!
//! Requires a **provisioning key**. Without `OPENROUTER_PROVISIONING_KEY`
//! set this prints a notice and exits cleanly.
//!
//! Run with:
//!
//! ```bash
//! OPENROUTER_PROVISIONING_KEY=sk-... cargo run --example list_organization_members
//! ```

use openrouter::{Client, ListOrganizationMembersOptions};

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
        .app_name("openrouter-rust list_organization_members example")
        .build()?;

    let opts = ListOrganizationMembersOptions::new().limit(50);
    let resp = client.list_organization_members(Some(&opts)).await?;
    println!("Organization members ({}):", resp.total_count);
    for m in &resp.data {
        let name = match (m.first_name.as_deref(), m.last_name.as_deref()) {
            (Some(f), Some(l)) => format!("{f} {l}"),
            (Some(f), None) => f.to_string(),
            (None, Some(l)) => l.to_string(),
            (None, None) => "(no name)".to_string(),
        };
        println!("  {:<40}  {:<24}  {:?}", m.email, name, m.role);
    }
    Ok(())
}
