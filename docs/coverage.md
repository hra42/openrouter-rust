# Coverage

Phase 7 (HRA-152) targets ≥85% line coverage on non-trivial code. Coverage is
not gated in CI — run it locally before cutting a release.

## Setup

```bash
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview
```

## Run

```bash
# Summary, all features:
cargo llvm-cov --all-features --summary-only

# HTML report at target/llvm-cov/html/index.html:
cargo llvm-cov --all-features --html

# LCOV for CI / Codecov:
cargo llvm-cov --all-features --lcov --output-path lcov.info
```

## What's covered

- Happy-path tests per endpoint live in `tests/*.rs` (e.g.
  `tests/chat_complete.rs`, `tests/streaming.rs`).
- Per-endpoint error-path coverage is consolidated in
  `tests/endpoint_errors.rs`; representative non-2xx responses are mapped to
  [`openrouter::Error::Api`].
- Cross-endpoint error-mapping invariants (numeric `code`, non-JSON bodies,
  decode failures, builder validation) live in `tests/error_mapping.rs`.
- Cloned-`Client` concurrency / retry races live in `tests/concurrency.rs`.

## What's intentionally not covered

- `Error::Http` paths driven by `reqwest::Error` internals (transport
  timeouts, connection drops) — covered indirectly through the retry layer.
- Beta features behind `--features beta` need that flag to be exercised by
  `cargo llvm-cov`.
