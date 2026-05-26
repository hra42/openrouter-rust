//! Enable the web-search plugin and print any URL citations returned with
//! the answer.
//!
//! Run with:
//!
//! ```bash
//! OPENROUTER_API_KEY=sk-... cargo run --example web_search
//! ```

use openrouter::{Annotation, ChatCompletionRequest, Client, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key =
        std::env::var("OPENROUTER_API_KEY").map_err(|_| "OPENROUTER_API_KEY must be set")?;
    let client = Client::builder()
        .api_key(api_key)
        .app_name("openrouter-rust web_search example")
        .build()?;

    let req = ChatCompletionRequest::new(
        "google/gemini-3.1-flash-lite",
        vec![Message::user(
            "What is the latest stable Rust version? Cite a source.",
        )],
    )
    .with_web_search();

    let resp = client.chat_complete(req).await?;
    let msg = resp
        .choices
        .first()
        .and_then(|c| c.message.as_ref())
        .ok_or("no message in response")?;
    println!("answer: {}", msg.content_text().unwrap_or(""));
    if let Some(annotations) = &msg.annotations {
        for a in annotations {
            match a {
                Annotation::UrlCitation { url_citation } => {
                    println!(
                        "cite: {} ({})",
                        url_citation.title.as_deref().unwrap_or(""),
                        url_citation.url
                    );
                }
            }
        }
    }
    Ok(())
}
