# Streaming

OpenRouter's chat and completion endpoints expose Server-Sent Events. The
SDK exposes them as
[`futures::Stream`](https://docs.rs/futures/latest/futures/stream/trait.Stream.html)
items via [`Client::chat_complete_stream`](crate::Client::chat_complete_stream)
and [`Client::complete_stream`](crate::Client::complete_stream).

```rust,no_run
use futures::StreamExt;
use openrouter::{ChatCompletionRequest, Client, Message};

#[tokio::main]
async fn main() -> openrouter::Result<()> {
    let client = Client::builder()
        .api_key(std::env::var("OPENROUTER_API_KEY").unwrap())
        .build()?;

    let req = ChatCompletionRequest::new(
        "google/gemini-3.1-flash-lite",
        vec![Message::user("Stream a haiku about Rust.")],
    );
    let mut stream = client.chat_complete_stream(req).await?;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if let Some(delta) = chunk.choices.first()
            .and_then(|c| c.delta.as_ref())
            .and_then(|d| d.content.as_deref())
        {
            print!("{delta}");
        }
    }
    Ok(())
}
```

## Cancellation

Dropping the [`EventStream`](crate::EventStream) cancels the underlying
HTTP connection immediately. Combine with `tokio::select!` to enforce
timeouts or abort on user input.

## Tool-call deltas

Streaming tool-call arguments arrive in fragments. Use
[`ToolCallAccumulator`](crate::ToolCallAccumulator) to reassemble the
deltas into completed calls before dispatching.
