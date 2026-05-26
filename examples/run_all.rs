//! Exercise every Phase 3 feature against the live API in a single binary.
//!
//! Mirrors the Go SDK's aggregate example. Each section is independent — if
//! one fails the next still runs, and a final summary lists pass/fail per
//! feature so you can spot regressions at a glance.
//!
//! Run with:
//!
//! ```bash
//! OPENROUTER_API_KEY=sk-... cargo run --example run_all
//! ```

use std::fmt::Write as _;

use futures::StreamExt;
use openrouter::{
    create_file_parser_plugin, create_user_message_with_image, mcp, Annotation,
    ChatCompletionRequest, Client, CompletionRequest, ContentBuilder, FileParserEngine,
    FunctionDef, ImageDetail, Message, Role, Tool, ToolCallAccumulator, ToolChoice,
};
use serde::Deserialize;
use serde_json::json;

const MODEL: &str = "google/gemini-3.1-flash-lite";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key =
        std::env::var("OPENROUTER_API_KEY").map_err(|_| "OPENROUTER_API_KEY must be set")?;
    let client = Client::builder()
        .api_key(api_key)
        .app_name("openrouter-rust run_all example")
        .build()?;

    let sections: &[Section] = &[
        ("tool_calling", run_tool_calling),
        ("structured_output", run_structured_output),
        ("transforms", run_transforms),
        ("provider_routing", run_provider_routing),
        ("reasoning", run_reasoning),
        ("web_search", run_web_search),
        ("mcp_tools", run_mcp_tools),
        ("multimodal", run_multimodal),
    ];

    let mut summary = Vec::new();
    for (name, f) in sections {
        println!("\n=== {name} ===");
        let result = f(&client).await;
        match &result {
            Ok(()) => println!("[{name}] ok"),
            Err(e) => println!("[{name}] FAIL: {e}"),
        }
        summary.push((*name, result));
    }

    println!("\n=== summary ===");
    let mut report = String::new();
    let mut failed = 0;
    for (name, result) in &summary {
        match result {
            Ok(()) => writeln!(report, "  ok    {name}")?,
            Err(e) => {
                failed += 1;
                writeln!(report, "  FAIL  {name}: {e}")?;
            }
        }
    }
    print!("{report}");
    if failed > 0 {
        return Err(format!("{failed} feature(s) failed").into());
    }
    Ok(())
}

type Fut =
    std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error>>>>>;
type Section = (&'static str, fn(&Client) -> Fut);

fn run_tool_calling(client: &Client) -> Fut {
    let client = client.clone();
    Box::pin(async move {
        let tool = Tool::function(
            FunctionDef::new(
                "get_weather",
                json!({
                    "type":"object",
                    "properties":{"location":{"type":"string"}},
                    "required":["location"]
                }),
            )
            .with_description("Look up the weather for a city."),
        );
        let req = ChatCompletionRequest::new(
            MODEL,
            vec![
                Message::system("Call tools when useful."),
                Message::user("What's the weather in Berlin?"),
            ],
        )
        .with_tools(vec![tool])
        .with_tool_choice(ToolChoice::auto());

        let mut stream = client.chat_complete_stream(req).await?;
        let mut acc = ToolCallAccumulator::new();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            for choice in &chunk.choices {
                if let Some(d) = &choice.delta {
                    acc.push_delta(d);
                }
            }
        }
        for call in acc.finish() {
            println!(
                "  tool call: {}({})",
                call.function.name.as_deref().unwrap_or(""),
                call.function.arguments.as_deref().unwrap_or("")
            );
        }
        Ok(())
    })
}

fn run_structured_output(client: &Client) -> Fut {
    let client = client.clone();
    Box::pin(async move {
        #[derive(Debug, Deserialize)]
        #[allow(dead_code)]
        struct CityFact {
            city: String,
            country: String,
            population_millions: f64,
        }
        let req = ChatCompletionRequest::new(
            MODEL,
            vec![
                Message::system("Respond with JSON matching the schema."),
                Message::user("Give me one city fact."),
            ],
        )
        .with_json_schema(
            "city_fact",
            true,
            json!({
                "type":"object",
                "properties":{
                    "city":{"type":"string"},
                    "country":{"type":"string"},
                    "population_millions":{"type":"number"}
                },
                "required":["city","country","population_millions"],
                "additionalProperties": false
            }),
        );
        let resp = client.chat_complete(req).await?;
        let text = resp
            .choices
            .first()
            .and_then(|c| c.message.as_ref())
            .and_then(|m| m.content_text())
            .ok_or("no content")?;
        let fact: CityFact = serde_json::from_str(text)?;
        println!("  parsed: {fact:?}");
        Ok(())
    })
}

fn run_transforms(client: &Client) -> Fut {
    let client = client.clone();
    Box::pin(async move {
        let req = ChatCompletionRequest::new(
            MODEL,
            vec![Message::user("Reply in one sentence: what is OpenRouter?")],
        )
        .with_transforms(["middle-out"]);
        let resp = client.chat_complete(req).await?;
        println!(
            "  chat (middle-out): {}",
            resp.choices
                .first()
                .and_then(|c| c.message.as_ref())
                .and_then(|m| m.content_text())
                .unwrap_or("")
        );

        let comp_req =
            CompletionRequest::new(MODEL, "OpenRouter is").with_transforms(Vec::<String>::new());
        let comp_resp = client.complete(comp_req).await?;
        println!(
            "  completion (transforms disabled): {}",
            comp_resp
                .choices
                .first()
                .map(|c| c.text.as_str())
                .unwrap_or("")
        );
        Ok(())
    })
}

fn run_provider_routing(client: &Client) -> Fut {
    let client = client.clone();
    Box::pin(async move {
        let req =
            ChatCompletionRequest::new(MODEL, vec![Message::user("Reply with exactly: hello")])
                .with_zdr(true)
                .with_nitro();
        let resp = client.chat_complete(req).await?;
        println!(
            "  reply: {} (provider: {})",
            resp.choices
                .first()
                .and_then(|c| c.message.as_ref())
                .and_then(|m| m.content_text())
                .unwrap_or(""),
            resp.provider.as_deref().unwrap_or("?")
        );

        let suffix_req = ChatCompletionRequest::new(
            format!("{MODEL}:floor"),
            vec![Message::user("Same reply.")],
        );
        let suffix_resp = client.chat_complete(suffix_req).await?;
        println!(
            "  floor reply: {}",
            suffix_resp
                .choices
                .first()
                .and_then(|c| c.message.as_ref())
                .and_then(|m| m.content_text())
                .unwrap_or("")
        );
        Ok(())
    })
}

fn run_reasoning(client: &Client) -> Fut {
    let client = client.clone();
    Box::pin(async move {
        let req = ChatCompletionRequest::new(
            MODEL,
            vec![Message::user("What is 17 * 24? Show your steps.")],
        )
        .with_reasoning_effort("medium");
        let mut stream = client.chat_complete_stream(req).await?;
        let mut answer = String::new();
        let mut reasoning = String::new();
        let mut reasoning_tokens: Option<u32> = None;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            if let Some(choice) = chunk.choices.first() {
                if let Some(d) = &choice.delta {
                    if let Some(t) = d.content.as_deref() {
                        answer.push_str(t);
                    }
                    if let Some(r) = d.reasoning.as_deref() {
                        reasoning.push_str(r);
                    }
                }
            }
            if let Some(u) = &chunk.usage {
                if let Some(d) = &u.completion_tokens_details {
                    reasoning_tokens = d.reasoning_tokens;
                }
            }
        }
        println!("  answer: {answer}");
        if !reasoning.is_empty() {
            println!("  reasoning: {reasoning}");
        }
        println!("  reasoning tokens: {reasoning_tokens:?}");
        Ok(())
    })
}

fn run_web_search(client: &Client) -> Fut {
    let client = client.clone();
    Box::pin(async move {
        let req = ChatCompletionRequest::new(
            MODEL,
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
            .ok_or("no message")?;
        println!("  answer: {}", msg.content_text().unwrap_or(""));
        if let Some(ann) = &msg.annotations {
            for a in ann {
                match a {
                    Annotation::UrlCitation { url_citation } => {
                        println!(
                            "  cite: {} ({})",
                            url_citation.title.as_deref().unwrap_or(""),
                            url_citation.url
                        );
                    }
                    Annotation::File { file } => {
                        println!("  file annotation: {}", file.filename);
                    }
                }
            }
        }
        Ok(())
    })
}

fn run_mcp_tools(client: &Client) -> Fut {
    let client = client.clone();
    Box::pin(async move {
        let mcp_tools = vec![
            json!({
                "name":"list_files",
                "description":"List files in a directory.",
                "inputSchema":{
                    "type":"object",
                    "properties":{"path":{"type":"string"}},
                    "required":["path"]
                }
            }),
            json!({
                "name":"read_file",
                "description":"Read the contents of a file.",
                "inputSchema":{
                    "type":"object",
                    "properties":{"path":{"type":"string"}},
                    "required":["path"]
                }
            }),
        ];
        let tools = mcp::convert_tools(&mcp_tools)?;
        println!("  converted {} MCP tools", tools.len());
        let req = ChatCompletionRequest::new(
            MODEL,
            vec![
                Message::system("Use the tools to answer."),
                Message::user("List the files in /tmp."),
            ],
        )
        .with_tools(tools)
        .with_tool_choice(ToolChoice::auto());
        let mut stream = client.chat_complete_stream(req).await?;
        let mut acc = ToolCallAccumulator::new();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            for choice in &chunk.choices {
                if let Some(d) = &choice.delta {
                    acc.push_delta(d);
                }
            }
        }
        for call in acc.finish() {
            println!(
                "  model wants to call: {}({})",
                call.function.name.as_deref().unwrap_or(""),
                call.function.arguments.as_deref().unwrap_or("")
            );
        }
        Ok(())
    })
}

fn run_multimodal(client: &Client) -> Fut {
    let client = client.clone();
    Box::pin(async move {
        // 1. Image-URL roundtrip via the convenience constructor.
        let req = ChatCompletionRequest::new(
            MODEL,
            vec![create_user_message_with_image(
                "Describe this image in one sentence.",
                "https://hra42.com/test-image.png",
            )],
        );
        let resp = client.chat_complete(req).await?;
        let answer = resp
            .choices
            .first()
            .and_then(|c| c.message.as_ref())
            .and_then(|m| m.content_text())
            .unwrap_or("");
        println!("  image answer: {answer}");

        // 2. ContentBuilder composing text + image with explicit detail.
        let msg = ContentBuilder::new()
            .add_text("What colors dominate this image?")
            .add_image_with_detail(
                "https://hra42.com/test-image.png",
                ImageDetail::Low,
            )
            .build_message(Role::User);
        let resp = client
            .chat_complete(ChatCompletionRequest::new(MODEL, vec![msg]))
            .await?;
        println!(
            "  builder answer: {}",
            resp.choices
                .first()
                .and_then(|c| c.message.as_ref())
                .and_then(|m| m.content_text())
                .unwrap_or("")
        );

        // 3. PDF parsing roundtrip with the file-parser plugin.
        let pdf_req = ChatCompletionRequest::new(
            MODEL,
            vec![openrouter::create_user_message_with_pdf(
                "Summarize this PDF in one sentence.",
                "https://www.w3.org/WAI/ER/tests/xhtml/testfiles/resources/pdf/dummy.pdf",
                "dummy.pdf",
            )],
        )
        .with_plugins(vec![create_file_parser_plugin(FileParserEngine::PdfText)]);
        let pdf_resp = client.chat_complete(pdf_req).await?;
        let pdf_msg = pdf_resp
            .choices
            .first()
            .and_then(|c| c.message.as_ref())
            .ok_or("no pdf message")?;
        println!("  pdf answer: {}", pdf_msg.content_text().unwrap_or(""));
        if let Some(anns) = &pdf_msg.annotations {
            for a in anns {
                if let Annotation::File { file } = a {
                    println!("  pdf annotation reusable for: {}", file.filename);
                }
            }
        }
        Ok(())
    })
}
