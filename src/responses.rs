//! **[BETA]** OpenRouter Responses API.
//!
//! Gated behind the `beta` cargo feature; enable with
//! `cargo build --features beta`.
//!
//! > **WARNING — beta API:** this surface may have breaking changes at any
//! > time. For stable production use prefer [`crate::Client::chat_complete`].
//!
//! Mirrors the Go SDK (`responses.go`, `responses_models.go`,
//! `responses_options.go`). The Rust port substitutes Go's functional
//! options with a builder struct ([`ResponsesRequest`]).

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::client::Client;
use crate::error::{Error, Result};
use crate::request;
use crate::stream::EventStream;
use crate::types::Plugin;

/// Reasoning effort levels accepted by the API.
pub mod reasoning_effort {
    pub const MINIMAL: &str = "minimal";
    pub const LOW: &str = "low";
    pub const MEDIUM: &str = "medium";
    pub const HIGH: &str = "high";
}

/// Reasoning configuration for [`ResponsesRequest`].
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ResponsesReasoning {
    /// One of `"minimal"`, `"low"`, `"medium"`, `"high"`. Validated at
    /// request time.
    pub effort: String,
}

/// A content part inside a [`ResponsesInputItem::message`] input. Currently
/// only `input_text` is supported.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ResponsesInputContent {
    #[serde(rename = "type")]
    pub kind: String,
    pub text: String,
}

impl ResponsesInputContent {
    pub fn input_text(text: impl Into<String>) -> Self {
        Self {
            kind: "input_text".into(),
            text: text.into(),
        }
    }
}

/// One item in the structured input array. Use the builder constructors
/// rather than constructing the struct directly.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ResponsesInputItem {
    /// `"message"` or `"function_call_output"`.
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub status: Option<String>,
    /// `"user"`, `"assistant"`, or `"system"` (only for `message` items).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub content: Vec<ResponsesInputContent>,
    /// Only for `function_call_output` items.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub call_id: Option<String>,
    /// Only for `function_call_output` items — the function's return value.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub output: Option<String>,
}

impl ResponsesInputItem {
    /// Build a message with an arbitrary role.
    pub fn message(role: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            kind: "message".into(),
            role: Some(role.into()),
            content: vec![ResponsesInputContent::input_text(text)],
            ..Default::default()
        }
    }

    pub fn user(text: impl Into<String>) -> Self {
        Self::message("user", text)
    }

    pub fn assistant(text: impl Into<String>) -> Self {
        Self::message("assistant", text)
    }

    pub fn system(text: impl Into<String>) -> Self {
        Self::message("system", text)
    }

    /// Build the return value of an earlier function call.
    pub fn function_call_output(call_id: impl Into<String>, output: impl Into<String>) -> Self {
        Self {
            kind: "function_call_output".into(),
            call_id: Some(call_id.into()),
            output: Some(output.into()),
            ..Default::default()
        }
    }
}

/// Tool definition for the Responses API. Unlike `chat/completions` this
/// is a flat structure (no nested `function: {…}`).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ResponsesTool {
    /// Always `"function"`.
    #[serde(rename = "type")]
    pub kind: String,
    pub name: String,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub description: String,
    /// `Some(true)` enables strict schema validation; `None` keeps the
    /// server default (serialized as `null` per the upstream contract).
    pub strict: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub parameters: Option<Value>,
}

impl ResponsesTool {
    pub fn function(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: Value,
    ) -> Self {
        Self {
            kind: "function".into(),
            name: name.into(),
            description: description.into(),
            strict: None,
            parameters: Some(parameters),
        }
    }
}

/// Input to [`Client::create_response`]: either a single string or a
/// structured sequence of [`ResponsesInputItem`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponsesInput {
    Text(String),
    Items(Vec<ResponsesInputItem>),
}

impl From<&str> for ResponsesInput {
    fn from(s: &str) -> Self {
        ResponsesInput::Text(s.to_string())
    }
}

impl From<String> for ResponsesInput {
    fn from(s: String) -> Self {
        ResponsesInput::Text(s)
    }
}

impl From<Vec<ResponsesInputItem>> for ResponsesInput {
    fn from(v: Vec<ResponsesInputItem>) -> Self {
        ResponsesInput::Items(v)
    }
}

/// Request body for the Responses API.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ResponsesRequest {
    pub model: String,
    pub input: Option<ResponsesInput>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub reasoning: Option<ResponsesReasoning>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tools: Vec<ResponsesTool>,
    /// `"auto"`, `"none"`, or `{"type":"function","function":{"name":...}}`.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tool_choice: Option<Value>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub plugins: Vec<Plugin>,
}

impl ResponsesRequest {
    /// Start a builder with the given model.
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            ..Default::default()
        }
    }

    pub fn input(mut self, input: impl Into<ResponsesInput>) -> Self {
        self.input = Some(input.into());
        self
    }

    pub fn max_output_tokens(mut self, n: u32) -> Self {
        self.max_output_tokens = Some(n);
        self
    }

    pub fn temperature(mut self, t: f64) -> Self {
        self.temperature = Some(t);
        self
    }

    pub fn top_p(mut self, p: f64) -> Self {
        self.top_p = Some(p);
        self
    }

    pub fn reasoning_effort(mut self, effort: impl Into<String>) -> Self {
        self.reasoning = Some(ResponsesReasoning {
            effort: effort.into(),
        });
        self
    }

    pub fn tools(mut self, tools: impl IntoIterator<Item = ResponsesTool>) -> Self {
        self.tools = tools.into_iter().collect();
        self
    }

    pub fn tool_choice(mut self, choice: Value) -> Self {
        self.tool_choice = Some(choice);
        self
    }

    /// Append plugins (does not overwrite existing ones).
    pub fn plugins(mut self, plugins: impl IntoIterator<Item = Plugin>) -> Self {
        self.plugins.extend(plugins);
        self
    }

    /// Convenience: enable the `web` plugin with `max_results`.
    pub fn web_search(mut self, max_results: u32) -> Self {
        use crate::types::WebPluginConfig;
        self.plugins.push(Plugin::web_with(
            WebPluginConfig::new().with_max_results(max_results),
        ));
        self
    }

    /// Validate inputs that the Go SDK validates client-side before sending.
    fn validate(&self) -> Result<()> {
        if self.model.is_empty() {
            return Err(Error::InvalidInput("model is required"));
        }
        let input = self
            .input
            .as_ref()
            .ok_or(Error::InvalidInput("input is required"))?;
        match input {
            ResponsesInput::Text(s) if s.is_empty() => {
                return Err(Error::InvalidInput("input string cannot be empty"));
            }
            ResponsesInput::Items(v) if v.is_empty() => {
                return Err(Error::InvalidInput("input array cannot be empty"));
            }
            ResponsesInput::Items(items) => {
                for item in items {
                    if item.kind.is_empty() {
                        return Err(Error::InvalidInput("input item type is required"));
                    }
                    match item.kind.as_str() {
                        "message" => {
                            let role = item
                                .role
                                .as_deref()
                                .ok_or(Error::InvalidInput("message role is required"))?;
                            if !matches!(role, "user" | "assistant" | "system") {
                                return Err(Error::InvalidInput(
                                    "message role must be user/assistant/system",
                                ));
                            }
                        }
                        "function_call_output"
                            if item.call_id.as_deref().unwrap_or("").is_empty() =>
                        {
                            return Err(Error::InvalidInput(
                                "function_call_output requires call_id",
                            ));
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        if let Some(r) = &self.reasoning {
            if !matches!(
                r.effort.as_str(),
                reasoning_effort::MINIMAL
                    | reasoning_effort::LOW
                    | reasoning_effort::MEDIUM
                    | reasoning_effort::HIGH
            ) {
                return Err(Error::InvalidInput(
                    "reasoning.effort must be minimal/low/medium/high",
                ));
            }
        }
        for tool in &self.tools {
            if tool.kind.is_empty() {
                return Err(Error::InvalidInput("tool type is required"));
            }
            if tool.name.is_empty() {
                return Err(Error::InvalidInput("tool name is required"));
            }
        }
        Ok(())
    }
}

/// Annotation attached to an [`ResponsesOutputContent`] (e.g. URL citation).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ResponsesAnnotation {
    #[serde(rename = "type", default)]
    pub kind: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub url: String,
    #[serde(default)]
    pub start_index: i64,
    #[serde(default)]
    pub end_index: i64,
}

/// A content item inside [`ResponsesOutput::content`].
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ResponsesOutputContent {
    #[serde(rename = "type", default)]
    pub kind: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub text: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub annotations: Vec<ResponsesAnnotation>,
    /// Encrypted reasoning chain (for `reasoning` content).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub encrypted_content: String,
    /// Key reasoning steps as text (for `reasoning` content).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub summary: Vec<String>,
}

/// A single output item: `"message"` or `"function_call"`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ResponsesOutput {
    #[serde(rename = "type", default)]
    pub kind: String,
    #[serde(default)]
    pub id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub status: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub role: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<ResponsesOutputContent>,
    /// `function_call` only.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub call_id: String,
    /// `function_call` only.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// `function_call` only — JSON-encoded arguments.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub arguments: String,
}

/// Token usage for a Responses request.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponsesUsage {
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub total_tokens: u64,
    #[serde(default)]
    pub reasoning_tokens: u64,
}

/// Full unary or streaming-chunk response.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ResponsesResponse {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub object: String,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub output: Vec<ResponsesOutput>,
    #[serde(default)]
    pub usage: ResponsesUsage,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub metadata: Option<Value>,
}

impl ResponsesResponse {
    /// Return the first `output_text` text content, or empty string.
    pub fn text_content(&self) -> &str {
        for o in &self.output {
            if o.kind == "message" {
                for c in &o.content {
                    if c.kind == "output_text" && !c.text.is_empty() {
                        return &c.text;
                    }
                }
            }
        }
        ""
    }

    /// Return all `function_call` outputs.
    pub fn function_calls(&self) -> Vec<&ResponsesOutput> {
        self.output
            .iter()
            .filter(|o| o.kind == "function_call")
            .collect()
    }

    /// Return all annotations across every output content item.
    pub fn annotations(&self) -> Vec<&ResponsesAnnotation> {
        self.output
            .iter()
            .flat_map(|o| o.content.iter().flat_map(|c| c.annotations.iter()))
            .collect()
    }

    /// Return the reasoning summary if present.
    pub fn reasoning_summary(&self) -> Option<&[String]> {
        for o in &self.output {
            for c in &o.content {
                if c.kind == "reasoning" && !c.summary.is_empty() {
                    return Some(&c.summary);
                }
            }
        }
        None
    }
}

impl Client {
    /// **[BETA]** Submit a unary Responses API request.
    ///
    /// `POST /responses`. `req.stream` is forced to `Some(false)` to keep
    /// the unary path honest. Returns the decoded [`ResponsesResponse`].
    pub async fn create_response(&self, mut req: ResponsesRequest) -> Result<ResponsesResponse> {
        req.stream = Some(false);
        req.validate()?;
        request::execute_json(self, "responses", &req).await
    }

    /// **[BETA]** Open a streaming Responses API request.
    ///
    /// `POST /responses` with SSE. Returns an [`EventStream`] whose items
    /// deserialize into [`ResponsesResponse`] chunks. Reconnect / cancel
    /// semantics match [`Client::chat_complete_stream`].
    pub async fn create_response_stream(
        &self,
        mut req: ResponsesRequest,
    ) -> Result<EventStream<ResponsesResponse>> {
        req.stream = Some(true);
        req.validate()?;
        self.open_event_stream("responses", &req).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn untagged_input_serializes_string() {
        let req = ResponsesRequest::new("m").input("hello");
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["input"], serde_json::json!("hello"));
    }

    #[test]
    fn untagged_input_serializes_items() {
        let req = ResponsesRequest::new("m").input(vec![ResponsesInputItem::user("hi")]);
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["input"][0]["type"], "message");
        assert_eq!(v["input"][0]["role"], "user");
        assert_eq!(v["input"][0]["content"][0]["type"], "input_text");
        assert_eq!(v["input"][0]["content"][0]["text"], "hi");
    }

    #[test]
    fn validate_rejects_empty_text_input() {
        let err = ResponsesRequest::new("m").input("").validate().unwrap_err();
        assert!(matches!(err, Error::InvalidInput(_)));
    }

    #[test]
    fn validate_rejects_bad_reasoning_effort() {
        let mut req = ResponsesRequest::new("m").input("hi");
        req.reasoning = Some(ResponsesReasoning {
            effort: "absurd".into(),
        });
        let err = req.validate().unwrap_err();
        assert!(matches!(err, Error::InvalidInput(_)));
    }

    #[test]
    fn text_content_extraction() {
        let r = ResponsesResponse {
            output: vec![ResponsesOutput {
                kind: "message".into(),
                content: vec![ResponsesOutputContent {
                    kind: "output_text".into(),
                    text: "hello world".into(),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        };
        assert_eq!(r.text_content(), "hello world");
    }
}
