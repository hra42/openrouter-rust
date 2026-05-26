# Zero-data-retention (ZDR)

OpenRouter supports a ZDR mode: only providers that pinky-promise not to
retain prompt or completion data will serve the request.

## Per-request ZDR

```rust,no_run
use openrouter::{ChatCompletionRequest, Client, Message};

let _req = ChatCompletionRequest::new(
    "google/gemini-3.1-flash-lite",
    vec![Message::user("Sensitive prompt")],
)
.with_zdr(true);
```

## Discover ZDR-only endpoints

The discovery endpoint `GET /endpoints/zdr` returns the set of
provider/model endpoints currently certified ZDR. Surface this list to
end-users so they can pre-select compatible models:

```rust,no_run
# use openrouter::Client;
# async fn run(client: &Client) -> openrouter::Result<()> {
let zdr = client.list_zdr_endpoints().await?;
for e in zdr.data {
    println!("{} via {}", e.model_id, e.provider_name);
}
# Ok(()) }
```
