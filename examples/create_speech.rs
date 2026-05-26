//! Synthesize a short MP3 from text and write it to `out.mp3`.
//!
//! Run with:
//!
//! ```bash
//! OPENROUTER_API_KEY=sk-... cargo run --example create_speech
//! ```

use openrouter::{Client, SpeechFormat, SpeechRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Ok(api_key) = std::env::var("OPENROUTER_API_KEY") else {
        eprintln!("OPENROUTER_API_KEY not set — skipping.");
        return Ok(());
    };
    let client = Client::builder()
        .api_key(api_key)
        .app_name("openrouter-rust create_speech example")
        .build()?;

    let resp = client
        .create_speech(&SpeechRequest {
            input: "Hello from the OpenRouter Rust SDK.".into(),
            model: "openai/tts-1".into(),
            voice: "alloy".into(),
            response_format: Some(SpeechFormat::Mp3),
            ..Default::default()
        })
        .await?;

    let out_path = "out.mp3";
    std::fs::write(out_path, &resp.audio)?;
    println!(
        "Wrote {} bytes (Content-Type: {}) to {out_path}",
        resp.audio.len(),
        resp.content_type.as_deref().unwrap_or("?"),
    );
    Ok(())
}
