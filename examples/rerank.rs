//! Rerank a small list of candidate documents against a query.
//!
//! Run with:
//!
//! ```bash
//! OPENROUTER_API_KEY=sk-... cargo run --example rerank
//! ```

use openrouter::{Client, RerankRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Ok(api_key) = std::env::var("OPENROUTER_API_KEY") else {
        eprintln!("OPENROUTER_API_KEY not set — skipping.");
        return Ok(());
    };
    let client = Client::builder()
        .api_key(api_key)
        .app_name("openrouter-rust rerank example")
        .build()?;

    let query = "What does the OpenRouter SDK do?";
    let documents = vec![
        "OpenRouter is a unified API for multiple LLM providers.".to_string(),
        "Rust is a systems programming language focused on safety.".to_string(),
        "The OpenRouter Rust SDK provides typed access to chat, embeddings, rerank, TTS and more."
            .to_string(),
        "A baker's recipe for sourdough bread.".to_string(),
    ];

    let resp = client
        .rerank(&RerankRequest {
            model: "cohere/rerank-v3.5".into(),
            query: query.into(),
            documents: documents.clone(),
            top_n: Some(3),
            ..Default::default()
        })
        .await?;

    println!("Top {} results for: {query}", resp.results.len());
    for r in &resp.results {
        println!(
            "  [{:>2}] {:.4}  {}",
            r.index, r.relevance_score, documents[r.index as usize],
        );
    }
    Ok(())
}
