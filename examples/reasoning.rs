//! Request reasoning tokens and inspect them on both the streaming deltas
//! and the final `usage.completion_tokens_details.reasoning_tokens`.
//!
//! Per CLAUDE.md the default smoke-test model is `google/gemini-3.1-flash-lite`.
//! It may not emit reasoning content — swap to a reasoning-capable model
//! (e.g. an OpenAI o-series or Anthropic thinking model) to see populated
//! `delta.reasoning` chunks. Token counts in usage still work either way.
//!
//! Run with:
//!
//! ```bash
//! OPENROUTER_API_KEY=sk-... cargo run --example reasoning
//! ```

use futures::StreamExt;
use openrouter::{ChatCompletionRequest, Client, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key =
        std::env::var("OPENROUTER_API_KEY").map_err(|_| "OPENROUTER_API_KEY must be set")?;
    let client = Client::builder()
        .api_key(api_key)
        .app_name("openrouter-rust reasoning example")
        .build()?;

    let req = ChatCompletionRequest::new(
        "google/gemini-3.1-flash-lite",
        vec![
            Message::system("Solve carefully."),
            Message::user("What is 17 * 24? Show the steps."),
        ],
    )
    // OpenRouter rejects requests that set both `effort` and `max_tokens`
    // on the same `reasoning` config — pick one. Use `with_reasoning_max_tokens`
    // instead if you want a hard token budget.
    .with_reasoning_effort("medium");

    let mut stream = client.chat_complete_stream(req).await?;
    let mut answer = String::new();
    let mut reasoning = String::new();
    let mut reasoning_tokens: Option<u32> = None;
    while let Some(item) = stream.next().await {
        let chunk = item?;
        if let Some(choice) = chunk.choices.first() {
            if let Some(delta) = &choice.delta {
                if let Some(t) = delta.content.as_deref() {
                    answer.push_str(t);
                }
                if let Some(r) = delta.reasoning.as_deref() {
                    reasoning.push_str(r);
                }
            }
        }
        if let Some(usage) = &chunk.usage {
            if let Some(d) = &usage.completion_tokens_details {
                reasoning_tokens = d.reasoning_tokens;
            }
        }
    }

    println!("answer: {answer}");
    if !reasoning.is_empty() {
        println!("reasoning: {reasoning}");
    }
    println!("reasoning tokens used: {reasoning_tokens:?}");
    Ok(())
}
