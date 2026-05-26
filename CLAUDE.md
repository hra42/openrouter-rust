# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Async Rust SDK for the [OpenRouter](https://openrouter.ai) API. This is a port of [openrouter-go](https://github.com/hra42/openrouter-go); behavior and defaults are kept in sync with the Go SDK on purpose (e.g. retry constants in `src/retry.rs` mirror the Go ones).

Status: early — Phase 1 (foundation) is in. Endpoints get wired in later phases per the 7-phase roadmap in `README.md`. Code marked `#[allow(dead_code)] // Wired into the request layer in Phase 2.` is intentional, not stale.

## Commands

```bash
cargo build
cargo test                                  # all tests
cargo test --all-features
cargo test <name>                           # single test by name substring
cargo test -p openrouter <module>::tests::  # tests in a module
cargo fmt --all -- --check                  # CI-equivalent fmt check
cargo clippy --all-targets --all-features -- -D warnings   # CI-equivalent
cargo deny check                            # license/advisory/ban audit (CI runs this)
cargo run --example build_client
```

CI (`.github/workflows/ci.yml`) runs `fmt`, `clippy`, `test` (ubuntu + macos), and `cargo-deny`, all with `RUSTFLAGS=-D warnings`. Treat clippy warnings as build failures locally too.

MSRV is **1.75** (`rust-toolchain.toml` pins stable; `clippy.toml` enforces msrv).

## Architecture

Crate root `src/lib.rs` is a thin re-export façade. The substrate is split into four modules:

- **`client`** — `Client` + `ClientBuilder`. `Client` is `Clone`-cheap (`Arc<ClientInner>`); all per-request state lives behind the `Arc` so it's freely shareable across tasks. The builder validates eagerly (missing/empty `api_key`, malformed `base_url`) and returns `Error::MissingField` / `Error::InvalidInput`. Base URL is normalized to always end with `/`. If the caller supplies their own `reqwest::Client`, `timeout()` on the builder is ignored — that's by design.
- **`error`** — Single `Error` enum + `Result<T>` alias. `Error::from_response_body` is the central parser for OpenRouter's `{"error": {code, message, metadata, provider_name}}` envelope; it tolerates non-JSON bodies and numeric `code` values. `Error::is_transient` (429 + 5xx + transport-level timeouts/connect/request errors) and `Error::retry_after` drive the retry layer. **Keep these two methods authoritative** — the retry middleware reads them directly; don't duplicate the classification logic at call sites.
- **`retry`** — `RetryConfig` + `run_with_retry`. Exponential backoff with ±25% jitter, capped at 30s, default 3 retries. A `Retry-After` from `Error::Api` overrides the computed delay **only when it's larger**. After more than one attempt, the final error is wrapped in `Error::RetryExhausted { attempts, source }`; a first-and-only failure returns the raw error. Constants (`DEFAULT_*`, `MAX_RECONNECT_BACKOFF`) intentionally match the Go SDK — don't drift them without also updating the Go side or noting the divergence.
- **`types`** — Pure serde models (no I/O). Split into `common`, `message`, `request`, `response`. All optional request fields use `#[serde(skip_serializing_if = "Option::is_none", default)]` so wire output stays minimal.

Phase 2 will add the request-execution layer; it should be the only consumer of `Error::from_response_body`, `is_transient`, `retry_after`, and `run_with_retry`. Until then those are `pub(crate)` + `#[allow(dead_code)]` — preserve that visibility.

## Conventions

- Lints in `Cargo.toml` deny `rust_2018_idioms` and `missing_debug_implementations`, and warn on `unreachable_pub`. New public types need `Debug`; new modules should respect the `unreachable_pub` boundary (use `pub(crate)` for internals).
- `rustfmt.toml` is intentionally minimal — defaults apply.
- Tests are colocated in `#[cfg(test)] mod tests` blocks in the same file as the code under test. Use `#[tokio::test(start_paused = true)]` for any retry/backoff test so `tokio::time::sleep` is virtual (see `retry.rs` tests for the pattern).
- `wiremock` + `pretty_assertions` are available as dev-deps for HTTP-level integration tests once the request layer lands.
- `Error` carries `&'static str` for `InvalidInput` / `MissingField` — keep those as compile-time literals, not `String`.
