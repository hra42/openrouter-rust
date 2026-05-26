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
    create_file_parser_plugin, create_user_message_with_audio,
    create_user_message_with_audio_bytes, create_user_message_with_base64_image,
    create_user_message_with_base64_image_bytes, create_user_message_with_base64_pdf,
    create_user_message_with_files, create_user_message_with_image,
    create_user_message_with_image_detail, create_user_message_with_images,
    create_user_message_with_pdf, create_user_message_with_text_content,
    create_user_message_with_text_file, create_user_message_with_text_files,
    encode_image_bytes_to_base64, encode_image_to_base64, Annotation, AudioFormat,
    ChatCompletionRequest, ChatCompletionResponse, Choice, CompletionRequest, CompletionResponse,
    Content, ContentBuilder, ContentPart, CreditsData, CreditsResponse, Delta, File,
    FileAnnotation, FileParserEngine, FilePdfConfig, FilePluginConfig, FileRef, FunctionCall,
    FunctionDef, ImageDetail, ImageUrl, InputAudio, KeyData, KeyRateLimit, KeyResponse,
    ListModelsOptions, Message, Model, ModelArchitecture, ModelDefaultParameters, ModelEndpoint,
    ModelEndpointPricing, ModelEndpointsArchitecture, ModelEndpointsData, ModelEndpointsResponse,
    ModelPerRequestLimits, ModelPricing, ModelTopProvider, ModelsResponse, Plugin, Provider,
    ProviderInfo, ProvidersResponse, ReasoningConfig, ResponseFormat, Role, Tool, ToolCall,
    ToolChoice, UrlCitation, Usage, WebPluginConfig,
};
