//! Request payloads.

use serde::{Deserialize, Serialize};

use super::{Message, Provider, ReasoningConfig, ResponseFormat, Tool, ToolChoice};

/// Chat-completions request payload.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub stop: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub seed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub frequency_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub presence_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub repetition_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub logit_bias: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub logprobs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub top_logprobs: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub min_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub top_a: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tool_choice: Option<ToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub response_format: Option<ResponseFormat>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub provider: Option<Provider>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub reasoning: Option<ReasoningConfig>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub transforms: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub usage: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub user: Option<String>,
}

impl ChatCompletionRequest {
    /// Construct a new request with just the required fields.
    pub fn new(model: impl Into<String>, messages: Vec<Message>) -> Self {
        Self {
            model: model.into(),
            messages,
            ..Default::default()
        }
    }

    /// Attach the list of tools the model may call.
    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Set the tool-selection strategy.
    pub fn with_tool_choice(mut self, choice: ToolChoice) -> Self {
        self.tool_choice = Some(choice);
        self
    }

    /// Set the response format directly.
    pub fn with_response_format(mut self, format: ResponseFormat) -> Self {
        self.response_format = Some(format);
        self
    }

    /// Constrain responses to a named JSON schema. Sugar over
    /// `with_response_format(ResponseFormat::json_schema(...))`.
    pub fn with_json_schema(
        self,
        name: impl Into<String>,
        strict: bool,
        schema: serde_json::Value,
    ) -> Self {
        self.with_response_format(ResponseFormat::json_schema(name, strict, schema))
    }

    /// Ask the model to emit a JSON object (no schema). Sugar over
    /// `with_response_format(ResponseFormat::json_object())`.
    pub fn with_json_mode(self) -> Self {
        self.with_response_format(ResponseFormat::json_object())
    }

    /// Set the message-transform pipeline. Passing an empty slice
    /// explicitly disables OpenRouter's default transforms.
    pub fn with_transforms<S, I>(mut self, transforms: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        self.transforms = Some(transforms.into_iter().map(Into::into).collect());
        self
    }
}

/// Legacy text-completions request payload.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub model: String,
    pub prompt: String,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub stop: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub seed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub frequency_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub presence_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub provider: Option<Provider>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub transforms: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub user: Option<String>,
}

impl CompletionRequest {
    /// Construct a new legacy completion request.
    pub fn new(model: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            prompt: prompt.into(),
            ..Default::default()
        }
    }

    /// Set the message-transform pipeline. Passing an empty slice
    /// explicitly disables OpenRouter's default transforms.
    pub fn with_transforms<S, I>(mut self, transforms: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        self.transforms = Some(transforms.into_iter().map(Into::into).collect());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FunctionDef, Tool, ToolChoice};
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn with_tools_serializes_only_set_fields() {
        let req = ChatCompletionRequest::new("x/y", vec![Message::user("hi")])
            .with_tools(vec![Tool::function(FunctionDef::new(
                "f",
                json!({"type":"object"}),
            ))])
            .with_tool_choice(ToolChoice::required());
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(
            v,
            json!({
                "model": "x/y",
                "messages": [{"role":"user","content":"hi"}],
                "tools": [{
                    "type": "function",
                    "function": {"name":"f","parameters":{"type":"object"}}
                }],
                "tool_choice": "required"
            })
        );
    }

    #[test]
    fn tool_choice_function_serializes() {
        let v = serde_json::to_value(ToolChoice::function("get_weather")).unwrap();
        assert_eq!(
            v,
            json!({"type":"function","function":{"name":"get_weather"}})
        );
    }

    #[test]
    fn with_json_mode_serializes() {
        let req = ChatCompletionRequest::new("x/y", vec![Message::user("hi")]).with_json_mode();
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["response_format"], json!({"type":"json_object"}));
    }

    #[test]
    fn with_transforms_serializes_array() {
        let req = ChatCompletionRequest::new("x/y", vec![Message::user("hi")])
            .with_transforms(["middle-out"]);
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["transforms"], json!(["middle-out"]));
    }

    #[test]
    fn with_transforms_empty_disables_defaults() {
        let req = ChatCompletionRequest::new("x/y", vec![Message::user("hi")])
            .with_transforms(Vec::<String>::new());
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["transforms"], json!([]));
    }

    #[test]
    fn completion_with_transforms_serializes_array() {
        let req = CompletionRequest::new("x/y", "hello").with_transforms(["middle-out"]);
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["transforms"], json!(["middle-out"]));
    }

    #[test]
    fn with_json_schema_serializes_strict_named_schema() {
        let req = ChatCompletionRequest::new("x/y", vec![Message::user("hi")]).with_json_schema(
            "answer",
            true,
            json!({"type":"object","properties":{"x":{"type":"number"}}}),
        );
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(
            v["response_format"],
            json!({
                "type": "json_schema",
                "json_schema": {
                    "name": "answer",
                    "schema": {"type":"object","properties":{"x":{"type":"number"}}},
                    "strict": true
                }
            })
        );
    }
}
