# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Async Rust SDK for the [OpenRouter](https://openrouter.ai) API. This is a port of [openrouter-go](https://github.com/hra42/openrouter-go); behavior and defaults are kept in sync with the Go SDK on purpose (e.g. retry constants in `src/retry.rs` mirror the Go ones).

**Crate name on crates.io is `openrouter-client`** (the `openrouter`, `openrouter-rust`, and `openrouter-rs` names were all already taken — `openrouter-rs` and `openrouter_rs` normalize to the same name on crates.io). The library identifier stays `openrouter`, so user code writes `use openrouter::...`. The split is configured in `Cargo.toml` via `[package] name = "openrouter-client"` + `[lib] name = "openrouter"`. When updating README / install snippets / docs.rs / crates.io links, use `openrouter-client`. When updating Rust source (imports, doctests, recipes), use `openrouter`. The **GitHub repo** is still `openrouter-rust` — keep that in repository URLs, CI badges, and the `[0.1.0]` link in CHANGELOG.md.

Status: Phase 1–7 are landed; v0.1.0 is the current target on crates.io. See the roadmap in `README.md`.

## Commands

```bash
cargo build
cargo test                                       # all tests
cargo test --all-features
cargo test <name>                                # single test by name substring
cargo test -p openrouter-client <module>::tests::  # tests in a module (package name)
cargo fmt --all -- --check                       # CI-equivalent fmt check
cargo clippy --all-targets --all-features -- -D warnings   # CI-equivalent
cargo deny check                                 # license/advisory/ban audit (CI runs this)
cargo doc --all-features --no-deps
RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps   # strict
cargo run --example build_client
OPENROUTER_API_KEY=… cargo run --example e2e -- chat    # live API smoke
```

CI (`.github/workflows/ci.yml`) runs `fmt`, `clippy`, `test` (ubuntu + macos), and `cargo-deny`, all with `RUSTFLAGS=-D warnings`. Treat clippy warnings as build failures locally too.

MSRV is **1.75** (`rust-toolchain.toml` pins stable; `clippy.toml` enforces msrv).

## Architecture

Crate root `src/lib.rs` is a thin re-export façade plus `#![deny(missing_docs)]`. The substrate is split into these modules:

- **`client`** — `Client` + `ClientBuilder`. `Client` is `Clone`-cheap (`Arc<ClientInner>`); all per-request state lives behind the `Arc` so it's freely shareable across tasks. The builder validates eagerly (missing/empty `api_key`, malformed `base_url`) and returns `Error::MissingField` / `Error::InvalidInput`. Base URL is normalized to always end with `/`. If the caller supplies their own `reqwest::Client`, `timeout()` on the builder is ignored — that's by design.
- **`error`** — Single `Error` enum + `Result<T>` alias. `Error::from_response_body` is the central parser for OpenRouter's `{"error": {code, message, metadata, provider_name}}` envelope; it tolerates non-JSON bodies and numeric `code` values. `Error::is_transient` (429 + 5xx + transport-level timeouts/connect/request errors) and `Error::retry_after` drive the retry layer. **Keep these two methods authoritative** — the retry middleware reads them directly; don't duplicate the classification logic at call sites.
- **`retry`** — `RetryConfig` + `run_with_retry`. Exponential backoff with ±25% jitter, capped at 30s, default 3 retries. A `Retry-After` from `Error::Api` overrides the computed delay **only when it's larger**. After more than one attempt, the final error is wrapped in `Error::RetryExhausted { attempts, source }`; a first-and-only failure returns the raw error. Constants (`DEFAULT_*`, `MAX_RECONNECT_BACKOFF`) intentionally match the Go SDK — don't drift them without also updating the Go side or noting the divergence.
- **`request`** — internal HTTP helpers (`execute_json`, `execute_json_get`, `execute_stream`). The single consumer of `Error::from_response_body`, `is_transient`, `retry_after`, and `run_with_retry`. New endpoints go through one of these helpers.
- **`stream`** — `EventStream` + SSE parser. Dropping the stream cancels the underlying connection.
- **`types`** — Pure serde models (no I/O). Split into `common`, `message`, `request`, `response`, plus per-feature files (`multimodal`, `discovery`, `account`, `audio_speech`, `video`, `rerank`, `guardrails`, `workspace`, `organization`). All optional request fields use `#[serde(skip_serializing_if = "Option::is_none", default)]` so wire output stays minimal.
- **`responses`** — `[beta]` Responses API, gated behind the `beta` cargo feature.
- **`oauth`**, **`mcp`**, **`webhooks`**, **`tool_call_accumulator`** — feature-scoped modules (OAuth PKCE helpers, MCP tool conversion, broadcast-webhook parsing, streaming tool-call reassembly).

## Live API testing

Any example, smoke test, or manual run that hits the real OpenRouter API must use **`google/gemini-3.1-flash-lite`** as the model. Do not switch to other models without an explicit ask — keeps cost predictable and behavior consistent across smoke tests.

## Where things live

- `tests/*.rs` — wiremock-based HTTP integration tests, one file per endpoint or topic. Cross-endpoint error coverage in `tests/endpoint_errors.rs` and `tests/error_mapping.rs`; cloned-`Client` concurrency in `tests/concurrency.rs`.
- `examples/*.rs` — runnable demos per major surface. `examples/run_all.rs` orchestrates them in one process; `examples/e2e.rs` is the Go-parity CLI smoke binary with subcommands matching `cmd/openrouter-test/`.
- `docs/recipes/*.md` — markdown recipes embedded into rustdoc via `#[doc = include_str!]` and linked from the README.
- `docs/coverage.md` — how to run `cargo llvm-cov` locally (not CI-gated).
- `AGENTS.md` — companion guide for non-Claude AI coding agents; mostly mirrors this file.
- `CHANGELOG.md` — Keep a Changelog format; release notes for each crates.io version.

## Conventions

- Lints in `Cargo.toml` deny `rust_2018_idioms` and `missing_debug_implementations`, and warn on `unreachable_pub`. `src/lib.rs` adds `#![deny(missing_docs)]` — every new public item needs a doc comment. New public types also need `Debug`; new modules should respect the `unreachable_pub` boundary (use `pub(crate)` for internals).
- `rustfmt.toml` is intentionally minimal — defaults apply.
- Tests are colocated in `#[cfg(test)] mod tests` blocks for unit tests; HTTP-level tests live in `tests/*.rs` using `wiremock`. Use `#[tokio::test(start_paused = true)]` for any retry/backoff test so `tokio::time::sleep` is virtual (see `retry.rs` tests for the pattern).
- `wiremock` + `pretty_assertions` + `clap` are available as dev-deps.
- `Error` carries `&'static str` for `InvalidInput` / `MissingField` — keep those as compile-time literals, not `String`.
- Don't write WHAT comments. A comment earns its place only when it explains a non-obvious WHY (hidden constraint, subtle invariant, workaround).
