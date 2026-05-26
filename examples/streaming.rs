//! Stream a chat completion and print assistant tokens as they arrive.
//!
//! Run with:
//!
//! ```bash
//! OPENROUTER_API_KEY=sk-... cargo run --example streaming
//! ```
//!
//! Dropping the returned `EventStream` cancels the underlying HTTP connection
//! cleanly — combine with `tokio::select!` for timeout/cancellation patterns.

use std::io::Write;

use futures::StreamExt;
use openrouter::{ChatCompletionRequest, Client, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key =
        std::env::var("OPENROUTER_API_KEY").map_err(|_| "OPENROUTER_API_KEY must be set")?;
    let client = Client::builder()
        .api_key(api_key)
        .app_name("openrouter-rust streaming example")
        .build()?;

    let req = ChatCompletionRequest {
        model: "google/gemini-3.1-flash-lite".into(),
        messages: vec![
            Message::system("You are a concise assistant."),
            Message::user("In one short sentence, what is OpenRouter?"),
        ],
        ..Default::default()
    };

    let mut stream = client.chat_complete_stream(req).await?;
    let mut stdout = std::io::stdout().lock();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        if let Some(choice) = chunk.choices.first() {
            if let Some(delta) = &choice.delta {
                if let Some(text) = delta.content.as_deref() {
                    stdout.write_all(text.as_bytes())?;
                    stdout.flush()?;
                }
            }
            if let Some(reason) = &choice.finish_reason {
                writeln!(stdout, "\n[finish_reason: {reason}]")?;
            }
        }
        if let Some(usage) = &chunk.usage {
            writeln!(
                stdout,
                "[usage: prompt={:?} completion={:?} total={:?}]",
                usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
            )?;
        }
    }
    Ok(())
}
