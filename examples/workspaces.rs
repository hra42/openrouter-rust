//! Workspace lifecycle demo: list, create, fetch, update, bulk-add/remove
//! members, delete.
//!
//! Requires a **provisioning key**. Without `OPENROUTER_PROVISIONING_KEY` set
//! this prints a notice and exits cleanly, so it's safe to include in
//! `examples/run_all.rs`.
//!
//! Run with:
//!
//! ```bash
//! OPENROUTER_PROVISIONING_KEY=sk-... cargo run --example workspaces
//! ```
//!
//! Optional `OPENROUTER_TEST_USER_ID` adds + removes that user from the
//! created workspace before deletion; otherwise the membership steps are
//! skipped.

use openrouter::{Client, CreateWorkspaceRequest, UpdateWorkspaceRequest};

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
        .app_name("openrouter-rust workspaces example")
        .build()?;

    println!("Existing workspaces:");
    let listing = client.list_workspaces(None).await?;
    for ws in &listing.data {
        println!("  {:<32}  slug={}  id={}", ws.name, ws.slug, ws.id);
    }
    println!("  (total {})", listing.total_count);

    let slug = format!("rust-sdk-demo-{}", std::process::id());
    let name = format!("Rust SDK demo {}", std::process::id());
    println!("\nCreating workspace '{name}' (slug={slug})…");
    let created = client
        .create_workspace(&CreateWorkspaceRequest {
            name: name.clone(),
            slug: slug.clone(),
            description: Some("Created by the Rust SDK workspaces example".into()),
            ..Default::default()
        })
        .await?;
    let id = created.data.id.clone();
    println!("  id: {id}");

    println!("\nFetching the workspace back by slug…");
    let fetched = client.get_workspace(&slug).await?;
    println!(
        "  name={}  description={:?}",
        fetched.data.name, fetched.data.description,
    );

    println!("\nRenaming via update…");
    let renamed = format!("{name} (renamed)");
    client
        .update_workspace(
            &slug,
            &UpdateWorkspaceRequest {
                name: Some(renamed.clone()),
                ..Default::default()
            },
        )
        .await?;
    println!("  new name: {renamed}");

    if let Ok(user_id) = std::env::var("OPENROUTER_TEST_USER_ID") {
        println!("\nAdding member {user_id}…");
        let added = client
            .add_workspace_members(&slug, std::slice::from_ref(&user_id))
            .await?;
        println!("  added_count: {}", added.added_count);

        println!("Removing member {user_id}…");
        let removed = client
            .remove_workspace_members(&slug, std::slice::from_ref(&user_id))
            .await?;
        println!("  removed_count: {}", removed.removed_count);
    } else {
        println!("\n(set OPENROUTER_TEST_USER_ID to exercise bulk member add/remove)");
    }

    println!("\nDeleting workspace…");
    let deleted = client.delete_workspace(&slug).await?;
    println!("  deleted: {}", deleted.deleted);
    Ok(())
}
