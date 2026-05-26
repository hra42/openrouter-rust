//! Request payloads.

use serde::{Deserialize, Serialize};

use super::{Message, Plugin, Provider, ReasoningConfig, ResponseFormat, Tool, ToolChoice};

/// Chat-completions request payload.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    /// Model id (e.g. `google/gemini-3.1-flash-lite`).
    pub model: String,
    /// Conversation messages.
    pub messages: Vec<Message>,

    /// Sampling temperature (0.0 = deterministic, ~1.0 = creative).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub temperature: Option<f64>,
    /// Nucleus sampling probability.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub top_p: Option<f64>,
    /// Cap on tokens considered each step.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub top_k: Option<u32>,
    /// Max generated tokens.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub max_tokens: Option<u32>,
    /// Stream-mode flag. The SDK forces `Some(true)` for streaming
    /// methods and `Some(false)` for blocking methods.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub stream: Option<bool>,
    /// Stop sequence(s) — string or array of strings.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub stop: Option<serde_json::Value>,
    /// Sampling seed for reproducibility.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub seed: Option<i64>,
    /// Frequency penalty.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub frequency_penalty: Option<f64>,
    /// Presence penalty.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub presence_penalty: Option<f64>,
    /// Repetition penalty.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub repetition_penalty: Option<f64>,
    /// Per-token bias map (provider-specific shape).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub logit_bias: Option<serde_json::Value>,
    /// Request log-probabilities for the chosen tokens.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub logprobs: Option<bool>,
    /// Number of alternate tokens to report logprobs for.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub top_logprobs: Option<u32>,
    /// Min-p sampling threshold.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub min_p: Option<f64>,
    /// Top-a sampling threshold.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub top_a: Option<f64>,

    /// Tools the model may call.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tools: Option<Vec<Tool>>,
    /// Tool-selection strategy.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tool_choice: Option<ToolChoice>,
    /// Output format constraint (JSON schema / JSON mode).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub response_format: Option<ResponseFormat>,
    /// Provider-routing parameters.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub provider: Option<Provider>,
    /// Reasoning-token configuration.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub reasoning: Option<ReasoningConfig>,
    /// Message-transform pipeline (e.g. `["middle-out"]`).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub transforms: Option<Vec<String>>,
    /// Plugin configuration (web search, file parsing, ...).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub plugins: Option<Vec<Plugin>>,
    /// Provider-specific usage reporting hint.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub usage: Option<serde_json::Value>,
    /// Opaque end-user identifier for abuse monitoring.
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

    /// Replace the provider-routing config with `provider`.
    pub fn with_provider(mut self, provider: Provider) -> Self {
        self.provider = Some(provider);
        self
    }

    fn provider_mut(&mut self) -> &mut Provider {
        self.provider.get_or_insert_with(Provider::default)
    }

    /// Preferred provider order.
    pub fn with_provider_order<S, I>(mut self, order: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        self.provider_mut().order = Some(order.into_iter().map(Into::into).collect());
        self
    }

    /// Sort strategy: `"throughput"`, `"price"`, or `"latency"`.
    pub fn with_provider_sort(mut self, sort: impl Into<String>) -> Self {
        self.provider_mut().sort = Some(sort.into());
        self
    }

    /// Restrict to this set of providers.
    pub fn with_only_providers<S, I>(mut self, only: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        self.provider_mut().only = Some(only.into_iter().map(Into::into).collect());
        self
    }

    /// Exclude these providers.
    pub fn with_ignore_providers<S, I>(mut self, ignore: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        self.provider_mut().ignore = Some(ignore.into_iter().map(Into::into).collect());
        self
    }

    /// Permitted quantization tiers.
    pub fn with_quantizations<S, I>(mut self, q: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        self.provider_mut().quantizations = Some(q.into_iter().map(Into::into).collect());
        self
    }

    /// Per-token max-price filter.
    pub fn with_max_price(mut self, price: serde_json::Value) -> Self {
        self.provider_mut().max_price = Some(price);
        self
    }

    /// Data-collection policy: `"allow"` or `"deny"`.
    pub fn with_data_collection(mut self, policy: impl Into<String>) -> Self {
        self.provider_mut().data_collection = Some(policy.into());
        self
    }

    /// Require providers to accept all sampling parameters.
    pub fn with_require_parameters(mut self, required: bool) -> Self {
        self.provider_mut().require_parameters = Some(required);
        self
    }

    /// Allow / disallow OpenRouter to use other providers as fallbacks.
    pub fn with_allow_fallbacks(mut self, allow: bool) -> Self {
        self.provider_mut().allow_fallbacks = Some(allow);
        self
    }

    /// Per-request Zero-Data-Retention enforcement.
    pub fn with_zdr(mut self, zdr: bool) -> Self {
        self.provider_mut().zdr = Some(zdr);
        self
    }

    /// Shorthand for `with_provider_sort("throughput")` — equivalent to a
    /// `:nitro` model suffix.
    pub fn with_nitro(self) -> Self {
        self.with_provider_sort("throughput")
    }

    /// Shorthand for `with_provider_sort("price")` — equivalent to a
    /// `:floor` model suffix.
    pub fn with_floor(self) -> Self {
        self.with_provider_sort("price")
    }

    /// Replace the full reasoning config.
    pub fn with_reasoning(mut self, reasoning: ReasoningConfig) -> Self {
        self.reasoning = Some(reasoning);
        self
    }

    fn reasoning_mut(&mut self) -> &mut ReasoningConfig {
        self.reasoning.get_or_insert_with(ReasoningConfig::default)
    }

    /// Set the reasoning effort (`"low"`, `"medium"`, `"high"`).
    pub fn with_reasoning_effort(mut self, effort: impl Into<String>) -> Self {
        self.reasoning_mut().effort = Some(effort.into());
        self
    }

    /// Cap the reasoning-token budget for this request.
    pub fn with_reasoning_max_tokens(mut self, max_tokens: u32) -> Self {
        self.reasoning_mut().max_tokens = Some(max_tokens);
        self
    }

    /// Ask the provider to omit reasoning content from the response.
    pub fn with_reasoning_exclude(mut self, exclude: bool) -> Self {
        self.reasoning_mut().exclude = Some(exclude);
        self
    }

    /// Replace the plugin list.
    pub fn with_plugins(mut self, plugins: Vec<Plugin>) -> Self {
        self.plugins = Some(plugins);
        self
    }

    /// Enable the default web-search plugin. Equivalent to
    /// `with_plugins(vec![Plugin::web()])`, but preserves any plugins
    /// already configured (web is pushed onto the existing list).
    pub fn with_web_search(mut self) -> Self {
        self.plugins
            .get_or_insert_with(Vec::new)
            .push(Plugin::web());
        self
    }
}

/// Legacy text-completions request payload.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// Model id.
    pub model: String,
    /// Prompt to complete.
    pub prompt: String,

    /// Sampling temperature.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub temperature: Option<f64>,
    /// Nucleus sampling probability.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub top_p: Option<f64>,
    /// Max generated tokens.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub max_tokens: Option<u32>,
    /// Stream-mode flag.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub stream: Option<bool>,
    /// Stop sequence(s).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub stop: Option<serde_json::Value>,
    /// Sampling seed for reproducibility.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub seed: Option<i64>,
    /// Frequency penalty.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub frequency_penalty: Option<f64>,
    /// Presence penalty.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub presence_penalty: Option<f64>,
    /// Provider-routing parameters.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub provider: Option<Provider>,
    /// Message-transform pipeline.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub transforms: Option<Vec<String>>,
    /// Plugin configuration.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub plugins: Option<Vec<Plugin>>,
    /// Opaque end-user identifier for abuse monitoring.
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

    /// Replace the provider-routing config.
    pub fn with_provider(mut self, provider: Provider) -> Self {
        self.provider = Some(provider);
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
    fn with_provider_helpers_compose_into_one_object() {
        let req = ChatCompletionRequest::new("x/y", vec![Message::user("hi")])
            .with_provider_order(["openai", "anthropic"])
            .with_only_providers(["openai"])
            .with_zdr(true)
            .with_nitro();
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(
            v["provider"],
            json!({
                "order": ["openai", "anthropic"],
                "only": ["openai"],
                "zdr": true,
                "sort": "throughput"
            })
        );
    }

    #[test]
    fn with_web_search_serializes_default_plugin() {
        let req = ChatCompletionRequest::new("x/y", vec![Message::user("hi")]).with_web_search();
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["plugins"], json!([{"id":"web"}]));
    }

    #[test]
    fn with_plugins_custom_config_serializes() {
        use crate::types::{Plugin, WebPluginConfig};
        let plugin = Plugin::web_with(
            WebPluginConfig::new()
                .with_max_results(3)
                .with_search_prompt("Cite sources.")
                .with_engine("native"),
        );
        let req =
            ChatCompletionRequest::new("x/y", vec![Message::user("hi")]).with_plugins(vec![plugin]);
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(
            v["plugins"],
            json!([{
                "id":"web",
                "max_results":3,
                "search_prompt":"Cite sources.",
                "engine":"native"
            }])
        );
    }

    #[test]
    fn with_reasoning_helpers_compose() {
        let req = ChatCompletionRequest::new("x/y", vec![Message::user("hi")])
            .with_reasoning_effort("high")
            .with_reasoning_max_tokens(512)
            .with_reasoning_exclude(false);
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(
            v["reasoning"],
            json!({"effort":"high","max_tokens":512,"exclude":false})
        );
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
