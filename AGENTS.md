# AGENTS.md

Operating manual for AI coding agents (Claude Code, Cursor, etc.) working
on this repository.

## What this crate is

`openrouter-rust` is an idiomatic, async Rust SDK for the
[OpenRouter](https://openrouter.ai) HTTP API. It is a port of
[openrouter-go](https://github.com/hra42/openrouter-go); behavior and
defaults are kept in sync with the Go SDK on purpose. When you change
retry constants, error classifications, or other defaults, check the Go
SDK first and call out any intentional divergence in the commit message.

The package is published as `openrouter-rust` on crates.io because the
`openrouter` name was already taken. The library identifier stays
`openrouter`, so user code writes `use openrouter::...`. When you touch
README, docs.rs, install snippets, or other consumer-facing references,
use `openrouter-rust`. When you touch Rust source, doctests, or recipes,
use `openrouter`. The split lives in `Cargo.toml` (`[package] name` vs
`[lib] name`).

## Commands you can run

```bash
cargo build
cargo test --all-features
cargo fmt --all -- --check          # CI-equivalent
cargo clippy --all-targets --all-features -- -D warnings   # CI-equivalent
cargo deny check                    # license/advisory audit
cargo doc --all-features --no-deps
RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps  # strict
cargo run --example build_client
OPENROUTER_API_KEY=… cargo run --example e2e -- chat  # live API
```

CI enforces fmt, clippy (`-D warnings`), tests (ubuntu + macOS), and
`cargo-deny`. MSRV is **1.75** (`rust-toolchain.toml`).

## Live-API testing

Every example, smoke test, or doctest that hits the real OpenRouter API
**must use model `google/gemini-3.1-flash-lite`**. Do not switch to
another model without an explicit ask — this keeps cost predictable and
behavior consistent across smoke tests.

## Layout

- `src/client.rs` — `Client`, `ClientBuilder`, and the per-endpoint
  methods. `Client` is `Clone`-cheap (`Arc<ClientInner>`).
- `src/error.rs` — `Error` enum + `Result` alias. `Error::from_response_body`,
  `is_transient`, and `retry_after` are the authoritative classifiers; the
  retry layer reads them directly. Keep them `pub(crate)` and avoid
  duplicating the logic elsewhere.
- `src/retry.rs` — exponential backoff with ±25% jitter, capped at 30s,
  default 3 retries. Constants intentionally mirror the Go SDK.
- `src/request.rs` — the single consumer of the error parser, retry
  runner, and HTTP helpers. New endpoints go through `execute_json` /
  `execute_json_get` / `execute_stream`.
- `src/stream.rs` — `EventStream` + SSE parser.
- `src/types/` — pure serde models, no I/O. All optional request fields
  use `#[serde(skip_serializing_if = "Option::is_none", default)]`.
- `src/responses.rs` — `[beta]` Responses API, gated behind the `beta`
  cargo feature.
- `src/oauth.rs`, `src/mcp.rs`, `src/webhooks.rs` — OAuth PKCE helpers,
  MCP tool conversion, broadcast-webhook parsing.
- `tests/` — wiremock-based integration tests, one file per endpoint or
  topic. `tests/endpoint_errors.rs` and `tests/error_mapping.rs` carry
  cross-endpoint error coverage; `tests/concurrency.rs` covers cloned
  clients.
- `examples/` — runnable demos per major surface. `examples/run_all.rs`
  orchestrates them in one process; `examples/e2e.rs` is the Go-parity
  CLI smoke binary.
- `docs/recipes/` — markdown recipes, embedded into rustdoc via
  `#[doc = include_str!]`.

## Conventions

- Lints in `Cargo.toml` deny `rust_2018_idioms` and
  `missing_debug_implementations` and warn on `unreachable_pub`. New
  public types need `Debug`; new modules respect the `unreachable_pub`
  boundary (use `pub(crate)` for internals).
- `rustfmt.toml` is intentionally minimal — defaults apply.
- Tests are colocated in `#[cfg(test)] mod tests` blocks for unit tests;
  HTTP-level tests live in `tests/*.rs` using `wiremock`. Use
  `#[tokio::test(start_paused = true)]` for retry/backoff timing so
  `tokio::time::sleep` is virtual.
- `Error::InvalidInput` and `Error::MissingField` carry `&'static str` —
  keep those as compile-time literals, not `String`.
- Don't write WHAT comments. Comments earn their place by explaining a
  non-obvious WHY: hidden constraints, subtle invariants, workarounds.

## Branching & commits

- One branch per Linear parent issue (`hra-<n>`). For multi-child issues,
  one commit per child issue, in the order specified in Linear.
- Commit subject: `type(hra-<n>): short summary`. Body: 1–3 short
  paragraphs explaining the why. No trailing AI co-author lines.
- Never use `--no-verify` or skip hooks. If a pre-commit hook fails,
  diagnose and fix the underlying issue.
- Don't `git push --force` or `git reset --hard` without an explicit ask.

## Documentation expectations

- Every public item has a doc comment. Major surfaces (`Client`,
  builders, streaming, errors, retry, multimodal helpers, OAuth,
  webhooks, MCP, beta Responses) include at least one runnable doc
  example (`no_run` is acceptable when the example would hit the
  network).
- Recipes in `docs/recipes/*.md` are linked from the matching modules
  via `#[doc = include_str!]`.
- `cargo doc --no-deps` with `RUSTDOCFLAGS=-D warnings` must be clean.

## When in doubt

- Match the Go SDK's behavior.
- Read `CLAUDE.md` for project-specific reminders surfaced to Claude
  Code.
- Ask the user before publishing, tagging, or force-pushing.
