//! Shared serde types for messages, requests, and responses.

mod common;
mod message;
mod request;
mod response;

pub use common::{
    Annotation, FunctionCall, FunctionDef, JsonSchema, Plugin, Provider, ReasoningConfig,
    ResponseFormat, Tool, ToolCall, ToolChoice, UrlCitation, WebPluginConfig,
};
pub use message::{Content, ContentPart, FileRef, ImageUrl, InputAudio, Message, Role};
pub use request::{ChatCompletionRequest, CompletionRequest};
pub use response::{
    ChatCompletionResponse, Choice, CompletionChoice, CompletionResponse, Delta, LogProbs,
    TokenDetails, Usage,
};
