//! End-to-end smoke binary against the live OpenRouter API.
//!
//! Mirrors `cmd/openrouter-test/` in the Go SDK: a single binary with one
//! subcommand per major surface. All subcommands default to model
//! `google/gemini-3.1-flash-lite` (see `CLAUDE.md`).
//!
//! ```bash
//! OPENROUTER_API_KEY=sk-... cargo run --example e2e -- chat
//! OPENROUTER_API_KEY=sk-... cargo run --example e2e -- stream
//! OPENROUTER_API_KEY=sk-... cargo run --example e2e -- --help
//! ```
//!
//! The provisioning-key subcommands (`createkey`, `updatekey`, `deletekey`,
//! `listkeys`) require a **provisioning** key, not a runtime API key.

#![allow(clippy::result_large_err)]

use std::io::Write;
use std::time::Duration;

use clap::{Parser, Subcommand};
use futures::StreamExt;
use openrouter::{
    ActivityOptions, ChatCompletionRequest, Client, CompletionRequest, CreateKeyRequest,
    FunctionDef, ListKeysOptions, ListModelsOptions, Message, Tool, ToolChoice, UpdateKeyRequest,
};

const DEFAULT_MODEL: &str = "google/gemini-3.1-flash-lite";

/// OpenRouter Rust SDK end-to-end smoke binary.
#[derive(Parser, Debug)]
#[command(name = "e2e", about = "OpenRouter SDK live-API smoke tests")]
struct Cli {
    /// Model id (defaults to google/gemini-3.1-flash-lite).
    #[arg(long, global = true, default_value = DEFAULT_MODEL)]
    model: String,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Single chat completion.
    Chat,
    /// Streaming chat completion.
    Stream,
    /// Legacy `/completions` request.
    Completion,
    /// Function/tool calling round-trip.
    Tools,
    /// Apply the `middle-out` transform.
    Transforms,
    /// Enable the web-search plugin.
    Websearch,
    /// List available models (truncated).
    Models,
    /// List endpoints for the configured model.
    Endpoints,
    /// List providers.
    Providers,
    /// Show account credits.
    Credits,
    /// Show recent activity (last 24h).
    Activity,
    /// Show the current API key info.
    Key,
    /// List API keys (provisioning).
    Listkeys,
    /// Create a new API key (provisioning). Prints the secret once.
    Createkey {
        /// Display name for the new key.
        #[arg(long, default_value = "e2e-smoke")]
        name: String,
    },
    /// Update an API key by hash (provisioning).
    Updatekey {
        /// Key hash to update.
        hash: String,
        /// New display name.
        #[arg(long)]
        name: Option<String>,
        /// Toggle disabled.
        #[arg(long)]
        disabled: Option<bool>,
    },
    /// Delete an API key by hash (provisioning).
    Deletekey {
        /// Key hash to delete.
        hash: String,
    },
}

fn mk_client() -> Result<Client, Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .map_err(|_| "OPENROUTER_API_KEY must be set for the e2e binary")?;
    Ok(Client::builder()
        .api_key(api_key)
        .app_name("openrouter-rust e2e")
        .referer("https://github.com/hra42/openrouter-rust")
        .timeout(Duration::from_secs(120))
        .build()?)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let client = mk_client()?;
    match cli.cmd {
        Cmd::Chat => chat(&client, &cli.model).await,
        Cmd::Stream => stream(&client, &cli.model).await,
        Cmd::Completion => completion(&client, &cli.model).await,
        Cmd::Tools => tools(&client, &cli.model).await,
        Cmd::Transforms => transforms(&client, &cli.model).await,
        Cmd::Websearch => websearch(&client, &cli.model).await,
        Cmd::Models => models(&client).await,
        Cmd::Endpoints => endpoints(&client, &cli.model).await,
        Cmd::Providers => providers(&client).await,
        Cmd::Credits => credits(&client).await,
        Cmd::Activity => activity(&client).await,
        Cmd::Key => key(&client).await,
        Cmd::Listkeys => listkeys(&client).await,
        Cmd::Createkey { name } => createkey(&client, &name).await,
        Cmd::Updatekey {
            hash,
            name,
            disabled,
        } => updatekey(&client, &hash, name, disabled).await,
        Cmd::Deletekey { hash } => deletekey(&client, &hash).await,
    }
}

async fn chat(client: &Client, model: &str) -> Result<(), Box<dyn std::error::Error>> {
    let req = ChatCompletionRequest::new(
        model,
        vec![Message::user("In one sentence: what is OpenRouter?")],
    );
    let resp = client.chat_complete(req).await?;
    let text = resp
        .choices
        .first()
        .and_then(|c| c.message.as_ref())
        .and_then(|m| m.content_text())
        .unwrap_or("(no content)");
    println!("{text}");
    Ok(())
}

async fn stream(client: &Client, model: &str) -> Result<(), Box<dyn std::error::Error>> {
    let req = ChatCompletionRequest::new(
        model,
        vec![Message::user("Stream a one-sentence haiku about Rust.")],
    );
    let mut stream = client.chat_complete_stream(req).await?;
    let mut out = std::io::stdout().lock();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if let Some(delta) = chunk
            .choices
            .first()
            .and_then(|c| c.delta.as_ref())
            .and_then(|d| d.content.as_deref())
        {
            out.write_all(delta.as_bytes())?;
            out.flush()?;
        }
    }
    writeln!(out)?;
    Ok(())
}

async fn completion(client: &Client, model: &str) -> Result<(), Box<dyn std::error::Error>> {
    let req = CompletionRequest {
        model: model.into(),
        prompt: "One short sentence about Rust:".into(),
        max_tokens: Some(40),
        ..Default::default()
    };
    let resp = client.complete(req).await?;
    if let Some(choice) = resp.choices.first() {
        println!("{}", choice.text);
    }
    Ok(())
}

async fn tools(client: &Client, model: &str) -> Result<(), Box<dyn std::error::Error>> {
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
    let req =
        ChatCompletionRequest::new(model, vec![Message::user("What's the weather in Berlin?")])
            .with_tools(vec![tool])
            .with_tool_choice(ToolChoice::auto());
    let resp = client.chat_complete(req).await?;
    let choice = resp.choices.first().ok_or("no choices")?;
    let msg = choice.message.as_ref().ok_or("no message")?;
    if let Some(calls) = msg.tool_calls.as_ref() {
        for call in calls {
            println!(
                "tool_call: {} args={}",
                call.function.name.as_deref().unwrap_or("(no name)"),
                call.function.arguments.as_deref().unwrap_or("")
            );
        }
    } else if let Some(text) = msg.content_text() {
        println!("{text}");
    }
    Ok(())
}

async fn transforms(client: &Client, model: &str) -> Result<(), Box<dyn std::error::Error>> {
    let req = ChatCompletionRequest::new(
        model,
        vec![Message::user("Summarize: Rust is a systems language.")],
    )
    .with_transforms(["middle-out"]);
    let resp = client.chat_complete(req).await?;
    println!(
        "{}",
        resp.choices
            .first()
            .and_then(|c| c.message.as_ref())
            .and_then(|m| m.content_text())
            .unwrap_or("(no content)")
    );
    Ok(())
}

async fn websearch(client: &Client, model: &str) -> Result<(), Box<dyn std::error::Error>> {
    let req = ChatCompletionRequest::new(
        model,
        vec![Message::user(
            "What is the current stable Rust version? Cite a source.",
        )],
    )
    .with_web_search();
    let resp = client.chat_complete(req).await?;
    let choice = resp.choices.first().ok_or("no choices")?;
    let msg = choice.message.as_ref().ok_or("no message")?;
    if let Some(text) = msg.content_text() {
        println!("{text}");
    }
    if let Some(ann) = msg.annotations.as_ref() {
        for a in ann {
            println!("annotation: {a:?}");
        }
    }
    Ok(())
}

async fn models(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let resp = client
        .list_models(Some(&ListModelsOptions::default()))
        .await?;
    for m in resp.data.iter().take(5) {
        println!("{}", m.id);
    }
    println!("... ({} total)", resp.data.len());
    Ok(())
}

async fn endpoints(client: &Client, model: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (author, slug) = model
        .split_once('/')
        .ok_or("model must be in 'author/slug' form")?;
    let resp = client.list_model_endpoints(author, slug).await?;
    for ep in &resp.data.endpoints {
        println!("{}/{} via {}", resp.data.id, ep.name, ep.provider_name);
    }
    Ok(())
}

async fn providers(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let resp = client.list_providers().await?;
    for p in resp.data.iter().take(10) {
        println!("{}", p.name);
    }
    println!("... ({} total)", resp.data.len());
    Ok(())
}

async fn credits(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let resp = client.get_credits().await?;
    println!(
        "credits: total={} used={} remaining={}",
        resp.data.total_credits,
        resp.data.total_usage,
        resp.data.remaining()
    );
    Ok(())
}

async fn activity(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let resp = client
        .get_activity(Some(&ActivityOptions::default()))
        .await?;
    println!("activity rows: {}", resp.data.len());
    for row in resp.data.iter().take(5) {
        println!("  {row:?}");
    }
    Ok(())
}

async fn key(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let resp = client.get_key().await?;
    println!("{:#?}", resp.data);
    Ok(())
}

async fn listkeys(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let resp = client.list_keys(Some(&ListKeysOptions::default())).await?;
    for k in resp.data.iter().take(10) {
        println!("{} {}", k.hash, k.name);
    }
    println!("... ({} total)", resp.data.len());
    Ok(())
}

async fn createkey(client: &Client, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let resp = client
        .create_key(&CreateKeyRequest {
            name: name.into(),
            ..Default::default()
        })
        .await?;
    println!("hash: {}", resp.data.hash);
    if let Some(secret) = resp.key.as_deref() {
        println!("secret (capture now, only shown once): {secret}");
    }
    Ok(())
}

async fn updatekey(
    client: &Client,
    hash: &str,
    name: Option<String>,
    disabled: Option<bool>,
) -> Result<(), Box<dyn std::error::Error>> {
    let resp = client
        .update_key(
            hash,
            &UpdateKeyRequest {
                name,
                disabled,
                ..Default::default()
            },
        )
        .await?;
    println!("{:#?}", resp.data);
    Ok(())
}

async fn deletekey(client: &Client, hash: &str) -> Result<(), Box<dyn std::error::Error>> {
    let resp = client.delete_key(hash).await?;
    println!("deleted: {}", resp.data.success);
    Ok(())
}
