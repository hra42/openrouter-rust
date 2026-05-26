# Key management

OpenRouter exposes two kinds of credentials:

- **API keys** — used to sign individual chat/completion requests.
- **Provisioning keys** — used to create, list, update, and delete API
  keys via `/keys/*`. Required for the methods in this section.

```rust,no_run
use openrouter::{Client, CreateKeyRequest, ListKeysOptions, UpdateKeyRequest};

#[tokio::main]
async fn main() -> openrouter::Result<()> {
    // Use a provisioning key, not a runtime key.
    let client = Client::builder()
        .api_key(std::env::var("OPENROUTER_PROVISIONING_KEY").unwrap())
        .build()?;

    // List existing keys (paginated).
    let listing = client
        .list_keys(Some(&ListKeysOptions::default()))
        .await?;
    for k in listing.data {
        println!("{} {}", k.hash, k.name);
    }

    // Create a new key. The secret value is only returned once.
    let created = client
        .create_key(&CreateKeyRequest {
            name: "service-foo".into(),
            limit: Some(50.0),
            include_byok_in_limit: Some(true),
        })
        .await?;
    if let Some(secret) = created.key {
        // Persist this somewhere safe — you cannot retrieve it again.
        std::fs::write("/run/secrets/openrouter.key", secret).ok();
    }

    // Rotate / disable a key.
    let _ = client
        .update_key(
            &created.data.hash,
            &UpdateKeyRequest {
                disabled: Some(true),
                ..Default::default()
            },
        )
        .await?;

    // Delete it.
    let _ = client.delete_key(&created.data.hash).await?;
    Ok(())
}
```

## Key info for the current runtime key

[`Client::get_key`](crate::Client::get_key) returns the metadata for the
key the client was built with. Use it to inspect remaining limit before
making expensive requests.

## OAuth PKCE

For users authorizing your app on `openrouter.ai`, see the
[`oauth`](crate::oauth) module. Build the auth URL with
[`build_auth_url`](crate::oauth::build_auth_url), then exchange the
returned `code` via
[`Client::exchange_auth_code`](crate::Client::exchange_auth_code) to
receive a key on behalf of the user.
