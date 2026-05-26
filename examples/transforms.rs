//! Demonstrate the `transforms` request field (e.g. `middle-out` compression
//! for long conversations that overflow the context window).
//!
//! Run with:
//!
//! ```bash
//! OPENROUTER_API_KEY=sk-... cargo run --example transforms
//! ```

use openrouter::{ChatCompletionRequest, Client, CompletionRequest, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key =
        std::env::var("OPENROUTER_API_KEY").map_err(|_| "OPENROUTER_API_KEY must be set")?;
    let client = Client::builder()
        .api_key(api_key)
        .app_name("openrouter-rust transforms example")
        .build()?;

    // Chat: opt into middle-out compression.
    let chat_req = ChatCompletionRequest::new(
        "google/gemini-3.1-flash-lite",
        vec![
            Message::system("You are a concise assistant."),
            Message::user("Summarize: OpenRouter unifies LLM APIs."),
        ],
    )
    .with_transforms(["middle-out"]);
    let chat_resp = client.chat_complete(chat_req).await?;
    println!(
        "chat (middle-out): {}",
        chat_resp
            .choices
            .first()
            .and_then(|c| c.message.as_ref())
            .and_then(|m| m.content_text())
            .unwrap_or("")
    );

    // Legacy completions: empty array explicitly disables default transforms.
    let comp_req = CompletionRequest::new("google/gemini-3.1-flash-lite", "OpenRouter is")
        .with_transforms(Vec::<String>::new());
    let comp_resp = client.complete(comp_req).await?;
    println!(
        "completion (transforms disabled): {}",
        comp_resp
            .choices
            .first()
            .map(|c| c.text.as_str())
            .unwrap_or("")
    );
    Ok(())
}
