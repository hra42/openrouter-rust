//! Ask the model for a structured response and deserialize it via serde.
//!
//! Run with:
//!
//! ```bash
//! OPENROUTER_API_KEY=sk-... cargo run --example structured_output
//! ```

use openrouter::{ChatCompletionRequest, Client, Message};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Printed via Debug.
struct CityFact {
    city: String,
    country: String,
    population_millions: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key =
        std::env::var("OPENROUTER_API_KEY").map_err(|_| "OPENROUTER_API_KEY must be set")?;
    let client = Client::builder()
        .api_key(api_key)
        .app_name("openrouter-rust structured_output example")
        .build()?;

    let schema = json!({
        "type": "object",
        "properties": {
            "city": {"type": "string"},
            "country": {"type": "string"},
            "population_millions": {"type": "number"}
        },
        "required": ["city", "country", "population_millions"],
        "additionalProperties": false
    });

    let req = ChatCompletionRequest::new(
        "google/gemini-3.1-flash-lite",
        vec![
            Message::system("Respond with a single JSON object matching the requested schema."),
            Message::user("Give me one interesting city fact."),
        ],
    )
    .with_json_schema("city_fact", true, schema);

    let resp = client.chat_complete(req).await?;
    let text = resp
        .choices
        .first()
        .and_then(|c| c.message.as_ref())
        .and_then(|m| m.content_text())
        .ok_or("no content in response")?;

    let fact: CityFact = serde_json::from_str(text)?;
    println!("Parsed: {fact:?}");
    Ok(())
}
