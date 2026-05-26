# Changelog

All notable changes to this crate are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-05-26

Initial release. A Rust port of [openrouter-go](https://github.com/hra42/openrouter-go);
behavior and defaults are kept in sync with the Go SDK on purpose.

### Added

- **Foundation** — `Client` + `ClientBuilder` (`Arc`-shareable, cheap to
  clone), `Error` enum with structured API-error envelope parsing,
  exponential-backoff retry with ±25% jitter (`RetryConfig`), serde types
  for every wire shape, MSRV 1.75.
- **Core endpoints & streaming** — `chat_complete`, `complete`, and
  their SSE counterparts (`chat_complete_stream`, `complete_stream`).
  `EventStream` is a `futures::Stream<Item = Result<…>>`; dropping it
  cancels the underlying HTTP connection.
- **Advanced inference** — tool / function calling (including streaming
  tool-call deltas via `ToolCallAccumulator`), structured outputs (JSON
  Schema + JSON mode), MCP tool conversion, `middle-out` transform,
  web-search plugin, full provider routing (`order`, `sort`, `only`,
  `ignore`, `quantizations`, `max_price`, `data_collection`,
  `require_parameters`, `allow_fallbacks`, `zdr`, `:nitro` / `:floor`),
  reasoning tokens (effort + max-tokens).
- **Multimodal inputs** — images (URL + base64), PDFs (parsing engines,
  annotation reuse), audio, text-file attachments, and a fluent
  `ContentBuilder` for mixed content.
- **Discovery & account** — `list_models`, `list_model_endpoints`,
  `list_providers`, `get_credits`, `get_activity`, `get_key`, full
  provisioning-key CRUD (`list_keys`, `get_key_by_hash`, `create_key`,
  `update_key`, `delete_key`).
- **Org & beta surfaces** — workspaces CRUD + bulk member add/remove,
  organization members listing, guardrails (spend caps, allowlists,
  key/member assignments), `list_zdr_endpoints`, rerank, text-to-speech
  (`create_speech`), async video generation (`create_video`, `get_video`,
  `wait_for_video`, `get_video_content`, `list_video_models`), broadcast
  webhook parser (`parse_broadcast_traces` for OTLP JSON), OAuth PKCE
  helpers (`generate_code_verifier`, `create_s256_code_challenge`,
  `build_auth_url`, `Client::exchange_auth_code`).
- **\[beta\]** Responses API (`Client::create_response`,
  `create_response_stream`) — gated behind the `beta` cargo feature.
- **Testing** — wiremock-based HTTP integration tests for every endpoint,
  cross-endpoint error mapping pinned in `tests/error_mapping.rs`,
  cloned-`Client` concurrency tests in `tests/concurrency.rs`, and a
  Go-parity smoke binary at `examples/e2e.rs`.
- **Docs** — `#![deny(missing_docs)]` enforced crate-wide; recipes for
  quickstart, streaming, tools, structured outputs, multimodal, provider
  routing, ZDR, and key management embedded in rustdoc; AGENTS.md for AI
  coding agents.

[0.1.0]: https://github.com/hra42/openrouter-rust/releases/tag/v0.1.0
