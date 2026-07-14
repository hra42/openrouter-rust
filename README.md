# openrouter-rust

[![Crates.io](https://img.shields.io/crates/v/openrouter-client.svg)](https://crates.io/crates/openrouter-client)
[![Docs.rs](https://docs.rs/openrouter-client/badge.svg)](https://docs.rs/openrouter-client)
[![CI](https://github.com/hra42/openrouter-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/hra42/openrouter-rust/actions/workflows/ci.yml)
[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](./LICENSE)

An idiomatic, async Rust SDK for the [OpenRouter](https://openrouter.ai) API.
A Rust port of [openrouter-go](https://github.com/hra42/openrouter-go);
behavior and defaults are kept in sync with the Go SDK on purpose.

## Install

```bash
cargo add openrouter-client
```

The crate is published as `openrouter-client` on crates.io (`openrouter`,
`openrouter-rust`, and `openrouter-rs` were all already taken), but it
imports as `openrouter` — your code writes `use openrouter::...`.

MSRV is **1.75**. Optional `beta` feature gates the Responses API:

```toml
[dependencies]
openrouter-client = { version = "0.2", features = ["beta"] }
```

Browser WebAssembly builds use the opt-in `browser` feature:

```toml
[dependencies]
openrouter-client = { version = "0.2", features = ["browser"] }
```

The feature selects browser randomness, timers, Fetch/ReadableStream transport,
and local futures for `wasm32-unknown-unknown`. Native builds keep their
existing Tokio, reqwest streaming, and rustls transport.

## Quickstart

```rust,no_run
use openrouter::{ChatCompletionRequest, Client, Message};

#[tokio::main]
async fn main() -> openrouter::Result<()> {
    let client = Client::builder()
        .api_key(std::env::var("OPENROUTER_API_KEY").unwrap())
        .app_name("my-app")
        .build()?;

    let req = ChatCompletionRequest::new(
        "google/gemini-3.1-flash-lite",
        vec![Message::user("Hello, who are you?")],
    );
    let resp = client.chat_complete(req).await?;
    if let Some(text) = resp.choices.first()
        .and_then(|c| c.message.as_ref())
        .and_then(|m| m.content_text())
    {
        println!("{text}");
    }
    Ok(())
}
```

Streaming:

```rust,no_run
use futures::StreamExt;
use openrouter::{ChatCompletionRequest, Client, Message};

# async fn run() -> openrouter::Result<()> {
let client = Client::builder().api_key("sk-…").build()?;
let req = ChatCompletionRequest::new(
    "google/gemini-3.1-flash-lite",
    vec![Message::user("Stream a haiku.")],
);
let mut stream = client.chat_complete_stream(req).await?;
while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    if let Some(delta) = chunk.choices.first()
        .and_then(|c| c.delta.as_ref())
        .and_then(|d| d.content.as_deref())
    {
        print!("{delta}");
    }
}
# Ok(()) }
```

## Features

| Surface | Status |
|---|---|
| Chat completions + legacy completions (blocking + streaming) | ✅ |
| Tool / function calling (incl. streaming tool-call deltas) | ✅ |
| Structured outputs (JSON Schema + JSON mode) | ✅ |
| MCP tool conversion | ✅ |
| Message transforms (`middle-out`) | ✅ |
| Web-search plugin | ✅ |
| Provider routing (order, sort, only/ignore, quantization, max price, ZDR) | ✅ |
| Reasoning tokens (effort + max-tokens) | ✅ |
| Multimodal (images, PDFs with parsing engines, audio, text files) | ✅ |
| Discovery (`/models`, `/models/{author}/{slug}/endpoints`, `/providers`) | ✅ |
| Account (credits, activity, current key, ZDR endpoint listing) | ✅ |
| Provisioning key CRUD | ✅ |
| Workspaces + organization members | ✅ |
| Guardrails (spend caps, allowlists, key/member assignments) | ✅ |
| Rerank (`/rerank`) | ✅ |
| Text-to-speech (`/audio/speech`) | ✅ |
| Async video generation (submit / poll / download) | ✅ |
| Broadcast webhook parser (OTLP JSON) | ✅ |
| OAuth PKCE helpers | ✅ |
| Browser WebAssembly (`browser` feature) | ✅ |
| **\[beta\]** Responses API (gated behind the `beta` cargo feature) | ✅ |

## Recipes

In-tree recipes are embedded in rustdoc and viewable on docs.rs:

- [Quickstart](./docs/recipes/quickstart.md)
- [Streaming](./docs/recipes/streaming.md)
- [Tools](./docs/recipes/tools.md)
- [Structured outputs](./docs/recipes/structured_outputs.md)
- [Multimodal](./docs/recipes/multimodal.md)
- [Provider routing](./docs/recipes/provider_routing.md)
- [ZDR](./docs/recipes/zdr.md)
- [Key management](./docs/recipes/key_management.md)

## End-to-end smoke tests

The `e2e` example is a single binary that mirrors the Go SDK's
`cmd/openrouter-test/` layout. Every subcommand hits the live OpenRouter
API using `google/gemini-3.1-flash-lite` by default.

```bash
OPENROUTER_API_KEY=sk-... cargo run --example e2e -- --help
OPENROUTER_API_KEY=sk-... cargo run --example e2e -- chat
OPENROUTER_API_KEY=sk-... cargo run --example e2e -- stream
OPENROUTER_API_KEY=sk-... cargo run --example e2e -- tools
```

Subcommands: `chat`, `stream`, `completion`, `tools`, `transforms`,
`websearch`, `models`, `endpoints`, `providers`, `credits`, `activity`,
`key`, `listkeys`, `createkey`, `updatekey`, `deletekey`. The `*key`
provisioning subcommands require a provisioning key, not a runtime key.

## Roadmap

| # | Phase | Status |
|---|---|---|
| 1 | Foundation — client builder, error model, retry/backoff, core types | ✅ |
| 2 | Core Endpoints & Streaming — chat, legacy completions, SSE | ✅ |
| 3 | Advanced Inference — tools, structured outputs, MCP, transforms, web search, provider routing, reasoning | ✅ |
| 4 | Multimodal Inputs — images, PDFs, audio, text files, `ContentBuilder` | ✅ |
| 5 | Discovery & Account — models, endpoints, providers, credits, activity, key CRUD | ✅ |
| 6 | Org & Beta Surfaces — workspaces, members, guardrails, ZDR, rerank, TTS, video, webhooks, OAuth, **\[beta\]** Responses API | ✅ |
| 7 | Testing, Docs & Release — unit + E2E test coverage, docs site, crates.io publish | ✅ |

## Reference

- Go SDK this port is based on: <https://github.com/hra42/openrouter-go>
- OpenRouter API docs: <https://openrouter.ai/docs>
- Changelog: [CHANGELOG.md](./CHANGELOG.md)
- AI agent guide: [AGENTS.md](./AGENTS.md)

## License

Released into the public domain under [The Unlicense](./LICENSE). Same
license as the Go SDK.
