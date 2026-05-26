# openrouter-rust

[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](./LICENSE)
[![Status](https://img.shields.io/badge/status-planning-orange.svg)](#roadmap)

An idiomatic, async Rust SDK for the [OpenRouter](https://openrouter.ai) API — a Rust port of [openrouter-go](https://github.com/hra42/openrouter-go).

> 🚧 **Status:** Planning. Implementation has not started.

## Goals

- **Complete API coverage** — every endpoint exposed by the Go SDK, including beta surfaces (Responses API, video generation, OAuth PKCE, broadcast webhooks).
- **Idiomatic Rust** — builder pattern for `Client` and requests, `Result`-based errors via `thiserror`, async via `tokio` + `reqwest`, `serde` for (de)serialization.
- **Streaming-first** — full SSE support with reconnection, exposed as `futures::Stream<Item = Result<T>>`.
- **Thread-safe** — `Client` is cheap to clone (`Arc` inside) and safe across tasks.
- **Lean dependencies** — `reqwest`, `serde`, `tokio`, `thiserror`, `futures`. No unnecessary deps.

## Planned feature surface

The SDK will mirror the Go implementation:

- Chat completions and legacy completions (streaming + non-streaming)
- Tool / function calling, including streaming tool-call deltas
- Structured outputs (JSON Schema + JSON mode)
- MCP tool conversion utilities
- Message transforms (`middle-out`)
- Web search plugin
- Full provider routing: order, sort, only / ignore, quantizations, max price, data-collection policy, require parameters, ZDR
- Model suffixes (`:nitro`, `:floor`) and reasoning tokens
- Multimodal inputs: images, PDFs (with parsing engines + annotation reuse), audio, text files, plus a `ContentBuilder` for mixed content
- Discovery: list models, list model endpoints, list providers
- Account: credits, activity analytics, current key info
- Provisioning-key management: list / get / create / update / delete API keys
- Workspaces, organization members, guardrails (spend caps, allowlists)
- Rerank endpoint
- Text-to-speech (`/audio/speech`)
- Async video generation (submit / poll / download)
- Broadcast webhook parsing (OTLP JSON)
- OAuth PKCE authorization-code exchange helper
- **\[beta]** Responses API (reasoning, tools, web search, streaming) — gated behind a `beta` cargo feature

## Quickstart (planned API sketch)

```rust
use openrouter::{Client, Message, Role};

#[tokio::main]
async fn main() -> openrouter::Result<()> {
    let client = Client::builder()
        .api_key(std::env::var("OPENROUTER_API_KEY")?)
        .app_name("my-app")
        .referer("https://my-app.example.com")
        .build()?;

    let response = client
        .chat_complete()
        .model("anthropic/claude-3-opus")
        .messages([Message::user("Hello, how are you?")])
        .send()
        .await?;

    println!("{}", response.choices[0].message.content_text());
    Ok(())
}
```

Streaming:

```rust
use futures::StreamExt;

let mut stream = client
    .chat_complete_stream()
    .model("anthropic/claude-3-opus")
    .messages([Message::user("Stream me a poem.")])
    .send()
    .await?;

while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    if let Some(delta) = chunk.choices.first().and_then(|c| c.delta.content.as_deref()) {
        print!("{delta}");
    }
}
```

> The exact API may shift during Phase 1 (foundation). Locked-in shape lands with the v0.1.0 release.

## Roadmap

Work is broken into 7 phases:

1. **Foundation** ✅ — crate scaffolding, client builder, error model, retry/backoff, core types
2. **Core Endpoints & Streaming** ✅ — chat, legacy completions, SSE infrastructure
3. **Advanced Inference** ✅ — tool calling, structured outputs, MCP, transforms, web search, provider routing, reasoning
4. **Multimodal Inputs** ✅ — images, PDFs (with parsing engines + annotation reuse), audio, text files, `ContentBuilder` for mixed content
5. **Discovery & Account** — models, endpoints, providers, credits, activity, API key CRUD
6. **Org & Beta Surfaces** — workspaces, members, guardrails, rerank, TTS, video, webhooks, OAuth PKCE, Responses API
7. **Testing, Docs & Release** — unit + E2E test coverage, docs site, crates.io publish

## Reference

- Go SDK this port is based on: <https://github.com/hra42/openrouter-go>
- OpenRouter API docs: <https://openrouter.ai/docs>

## License

Released into the public domain under [The Unlicense](./LICENSE). Same license as the Go SDK.
