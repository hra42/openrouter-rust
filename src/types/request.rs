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
}
