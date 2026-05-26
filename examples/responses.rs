//! **[BETA]** Demonstrate the Responses API (unary + streaming).
//!
//! Requires the `beta` cargo feature. Skipped quietly when
//! `OPENROUTER_API_KEY` is unset.
//!
//! Run with:
//!
//! ```bash
//! OPENROUTER_API_KEY=sk-... cargo run --example responses --features beta
//! ```

use futures::StreamExt;
use openrouter::responses::{reasoning_effort, ResponsesRequest};
use openrouter::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Ok(api_key) = std::env::var("OPENROUTER_API_KEY") else {
        eprintln!("OPENROUTER_API_KEY not set — skipping.");
        return Ok(());
    };
    let client = Client::builder()
        .api_key(api_key)
        .app_name("openrouter-rust responses example")
        .build()?;

    println!("=== Unary response ===");
    let resp = client
        .create_response(
            ResponsesRequest::new("google/gemini-3.1-flash-lite")
                .input("Say hi in five words")
                .max_output_tokens(64)
                .reasoning_effort(reasoning_effort::LOW),
        )
        .await?;
    println!("text: {}", resp.text_content());
    println!(
        "tokens: in={} out={} total={}",
        resp.usage.input_tokens, resp.usage.output_tokens, resp.usage.total_tokens
    );

    println!("\n=== Streaming response ===");
    let mut stream = client
        .create_response_stream(
            ResponsesRequest::new("google/gemini-3.1-flash-lite")
                .input("Stream me a short haiku about Rust")
                .max_output_tokens(64),
        )
        .await?;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let text = chunk.text_content();
        if !text.is_empty() {
            print!("{text}");
        }
    }
    println!();
    Ok(())
}
