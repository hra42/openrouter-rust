//! Stream a chat completion with tool/function calling.
//!
//! Defines a `get_weather` tool, sends a request that should trigger a call,
//! and uses [`ToolCallAccumulator`] to assemble the streaming tool-call
//! fragments into complete [`ToolCall`]s.
//!
//! Run with:
//!
//! ```bash
//! OPENROUTER_API_KEY=sk-... cargo run --example tool_calling
//! ```

use futures::StreamExt;
use openrouter::{
    ChatCompletionRequest, Client, FunctionDef, Message, Tool, ToolCallAccumulator, ToolChoice,
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key =
        std::env::var("OPENROUTER_API_KEY").map_err(|_| "OPENROUTER_API_KEY must be set")?;
    let client = Client::builder()
        .api_key(api_key)
        .app_name("openrouter-rust tool_calling example")
        .build()?;

    let weather_tool = Tool::function(
        FunctionDef::new(
            "get_weather",
            json!({
                "type": "object",
                "properties": {
                    "location": {"type": "string", "description": "City name"},
                    "unit": {"type": "string", "enum": ["celsius", "fahrenheit"]}
                },
                "required": ["location"]
            }),
        )
        .with_description("Look up the current weather for a city."),
    );

    let req = ChatCompletionRequest::new(
        "google/gemini-3.1-flash-lite",
        vec![
            Message::system("You are a helpful assistant. Call tools when useful."),
            Message::user("What's the weather in Berlin in celsius?"),
        ],
    )
    .with_tools(vec![weather_tool])
    .with_tool_choice(ToolChoice::auto());

    let mut stream = client.chat_complete_stream(req).await?;
    let mut acc = ToolCallAccumulator::new();
    let mut text = String::new();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        for choice in &chunk.choices {
            if let Some(delta) = &choice.delta {
                if let Some(t) = delta.content.as_deref() {
                    text.push_str(t);
                }
                acc.push_delta(delta);
            }
        }
    }

    if !text.is_empty() {
        println!("text: {text}");
    }
    for call in acc.finish() {
        println!(
            "tool call: id={} name={} args={}",
            call.id,
            call.function.name.as_deref().unwrap_or(""),
            call.function.arguments.as_deref().unwrap_or("")
        );
    }
    Ok(())
}
