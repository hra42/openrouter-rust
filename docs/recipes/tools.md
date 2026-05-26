# Tool / function calling

Declare tools via [`Tool::function`](crate::Tool::function), attach them
to a [`ChatCompletionRequest`](crate::ChatCompletionRequest) with
[`with_tools`](crate::ChatCompletionRequest::with_tools), and steer
selection with [`ToolChoice`](crate::ToolChoice).

```rust,no_run
use openrouter::{ChatCompletionRequest, Client, FunctionDef, Message, Tool, ToolChoice};

#[tokio::main]
async fn main() -> openrouter::Result<()> {
    let client = Client::builder()
        .api_key(std::env::var("OPENROUTER_API_KEY").unwrap())
        .build()?;

    let tool = Tool::function(FunctionDef {
        name: "get_weather".into(),
        description: Some("Return the current weather for a city.".into()),
        parameters: Some(serde_json::json!({
            "type": "object",
            "properties": {"city": {"type": "string"}},
            "required": ["city"],
        })),
        strict: None,
    });

    let req = ChatCompletionRequest::new(
        "google/gemini-3.1-flash-lite",
        vec![Message::user("Weather in Berlin?")],
    )
    .with_tools(vec![tool])
    .with_tool_choice(ToolChoice::auto());

    let resp = client.chat_complete(req).await?;
    if let Some(calls) = resp.choices.first()
        .and_then(|c| c.message.as_ref())
        .and_then(|m| m.tool_calls.clone())
    {
        for call in calls {
            println!(
                "{} {}",
                call.function.name.as_deref().unwrap_or(""),
                call.function.arguments.as_deref().unwrap_or(""),
            );
        }
    }
    Ok(())
}
```

## Streaming tool calls

When streaming, tool-call arguments are emitted as deltas. Feed the
deltas into [`ToolCallAccumulator`](crate::ToolCallAccumulator) — it
will buffer fragments and emit complete tool calls keyed by index.
