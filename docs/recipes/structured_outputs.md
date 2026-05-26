# Structured outputs

Force a JSON-shaped response either via JSON Schema or unrestricted JSON
mode.

## JSON Schema

```rust,no_run
use openrouter::{ChatCompletionRequest, Client, Message};

#[tokio::main]
async fn main() -> openrouter::Result<()> {
    let client = Client::builder()
        .api_key(std::env::var("OPENROUTER_API_KEY").unwrap())
        .build()?;

    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "city": {"type": "string"},
            "temperature_c": {"type": "number"},
        },
        "required": ["city", "temperature_c"],
        "additionalProperties": false,
    });

    let req = ChatCompletionRequest::new(
        "google/gemini-3.1-flash-lite",
        vec![Message::user("Berlin weather right now as JSON.")],
    )
    .with_json_schema("weather", /* strict = */ true, schema);

    let resp = client.chat_complete(req).await?;
    let text = resp.choices.first()
        .and_then(|c| c.message.as_ref())
        .and_then(|m| m.content_text())
        .unwrap_or("{}");
    let value: serde_json::Value = serde_json::from_str(text)?;
    println!("{value:#}");
    Ok(())
}
```

## JSON mode

Use [`with_json_mode`](crate::ChatCompletionRequest::with_json_mode) when
you don't need a schema and just want valid JSON back.
