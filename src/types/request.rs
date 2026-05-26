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
