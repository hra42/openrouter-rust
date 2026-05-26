//! Idiomatic async Rust SDK for the [OpenRouter](https://openrouter.ai) API.
//!
//! This crate is a Rust port of [openrouter-go](https://github.com/hra42/openrouter-go).
//! Phase 1 establishes the substrate: client builder, error model, retry/backoff,
//! and the shared serde types. Endpoints are wired in later phases.

#![allow(clippy::result_large_err)]

pub mod client;
pub mod error;
pub mod mcp;
mod request;
pub mod retry;
pub mod stream;
pub mod tool_call_accumulator;
pub mod types;

pub use client::{Client, ClientBuilder};
pub use error::{Error, Result};
pub use retry::RetryConfig;
pub use stream::EventStream;
pub use tool_call_accumulator::ToolCallAccumulator;
pub use types::{
    Annotation, ChatCompletionRequest, ChatCompletionResponse, Choice, CompletionRequest,
    CompletionResponse, Content, ContentPart, Delta, FunctionCall, FunctionDef, Message, Plugin,
    Provider, ReasoningConfig, ResponseFormat, Role, Tool, ToolCall, ToolChoice, UrlCitation,
    Usage, WebPluginConfig,
};
