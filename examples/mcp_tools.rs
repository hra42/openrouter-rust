//! Convert MCP tool descriptions into OpenRouter `Tool`s and use them in a
//! chat completion.
//!
//! Run with:
//!
//! ```bash
//! OPENROUTER_API_KEY=sk-... cargo run --example mcp_tools
//! ```

use futures::StreamExt;
use openrouter::{mcp, ChatCompletionRequest, Client, Message, ToolCallAccumulator, ToolChoice};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key =
        std::env::var("OPENROUTER_API_KEY").map_err(|_| "OPENROUTER_API_KEY must be set")?;
    let client = Client::builder()
        .api_key(api_key)
        .app_name("openrouter-rust mcp_tools example")
        .build()?;

    // Pretend these came from an MCP server.
    let mcp_tools = vec![
        json!({
            "name": "list_files",
            "description": "List files in a directory.",
            "inputSchema": {
                "type": "object",
                "properties": {"path": {"type": "string"}},
                "required": ["path"]
            }
        }),
        json!({
            "name": "read_file",
            "description": "Read the contents of a file.",
            "inputSchema": {
                "type": "object",
                "properties": {"path": {"type": "string"}},
                "required": ["path"]
            }
        }),
    ];

    let tools = mcp::convert_tools(&mcp_tools)?;
    println!("converted {} MCP tools", tools.len());

    let req = ChatCompletionRequest::new(
        "google/gemini-3.1-flash-lite",
        vec![
            Message::system("Use the available tools to answer."),
            Message::user("List the files in /tmp."),
        ],
    )
    .with_tools(tools)
    .with_tool_choice(ToolChoice::auto());

    let mut stream = client.chat_complete_stream(req).await?;
    let mut acc = ToolCallAccumulator::new();
    while let Some(item) = stream.next().await {
        let chunk = item?;
        for choice in &chunk.choices {
            if let Some(delta) = &choice.delta {
                acc.push_delta(delta);
            }
        }
    }
    for call in acc.finish() {
        println!(
            "model wants to call: {}({})",
            call.function.name.as_deref().unwrap_or(""),
            call.function.arguments.as_deref().unwrap_or("")
        );
    }
    Ok(())
}
