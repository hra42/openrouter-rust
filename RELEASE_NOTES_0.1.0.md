# openrouter-rust 0.1.0

First release of the Rust SDK for the [OpenRouter](https://openrouter.ai)
API. This is a port of [openrouter-go](https://github.com/hra42/openrouter-go);
behavior and defaults are kept in sync with the Go SDK on purpose.

## Highlights

- **Complete API coverage** — chat / completions (blocking + streaming),
  tool calling, structured outputs (JSON Schema + JSON mode), MCP tools,
  transforms, web search, provider routing, reasoning tokens, multimodal
  inputs (images / PDFs / audio / text files), discovery, account,
  provisioning key CRUD, workspaces, organization members, guardrails,
  ZDR endpoint listing, rerank, text-to-speech, async video generation,
  broadcast webhook parsing, OAuth PKCE, and **\[beta\]** Responses API.
- **Idiomatic Rust** — builder pattern for `Client` and requests,
  `Result`-based errors via `thiserror`, async via `tokio` + `reqwest`,
  `serde` for (de)serialization. MSRV 1.75.
- **Streaming-first** — SSE exposed as `futures::Stream<Item = Result<T>>`.
  Dropping the stream cancels the underlying connection.
- **Thread-safe** — `Client` is cheap to clone (`Arc` inside) and safe
  across tasks.
- **Production-quality** — 200+ wiremock-based tests covering happy
  paths and error mapping per endpoint, a concurrency harness for the
  cloned-`Client` pattern, and a Go-parity smoke binary at
  `examples/e2e.rs`.
- **Strict docs** — `#![deny(missing_docs)]` enforced crate-wide; recipes
  embedded in rustdoc; `AGENTS.md` and `CLAUDE.md` for AI coding agents.

## Install

The crate is published as `openrouter-rust` on crates.io (the
`openrouter` name was already taken). The library identifier stays
`openrouter`, so code writes `use openrouter::...`.

```bash
cargo add openrouter-rust
```

Enable the beta Responses API:

```toml
[dependencies]
openrouter-rust = { version = "0.1", features = ["beta"] }
```

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps
cargo deny check
```

See [CHANGELOG.md](./CHANGELOG.md) for the full list of additions.
