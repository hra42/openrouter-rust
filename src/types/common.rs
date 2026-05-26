//! Shared sub-types: tools, response format, provider routing, reasoning.

use serde::{Deserialize, Serialize};

/// A function-call invocation requested by the model.
///
/// Both `id` and `kind` are optional because OpenRouter streams tool calls
/// as fragments: the first chunk for an `index` typically carries `id` +
/// `type` + `function.name`, and continuation chunks for the same `index`
/// carry only additional `function.arguments` bytes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    #[serde(default)]
    pub id: String,
    #[serde(rename = "type", default)]
    pub kind: String,
    pub function: FunctionCall,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub index: Option<u32>,
}

/// Function-call payload (name + serialized JSON arguments).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FunctionCall {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub arguments: Option<String>,
}

/// A tool the model is allowed to call.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Tool {
    Function { function: FunctionDef },
}

/// Definition of a callable function tool.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FunctionDef {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub parameters: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub strict: Option<bool>,
}

/// Tool-selection strategy.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    Mode(String),
    Specific {
        #[serde(rename = "type")]
        kind: String,
        function: FunctionRef,
    },
}

/// Lightweight reference to a tool by function name.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FunctionRef {
    pub name: String,
}

impl ToolChoice {
    /// Let the model decide whether to call a tool.
    pub fn auto() -> Self {
        ToolChoice::Mode("auto".to_string())
    }

    /// Forbid tool calls.
    pub fn none() -> Self {
        ToolChoice::Mode("none".to_string())
    }

    /// Require the model to call some tool.
    pub fn required() -> Self {
        ToolChoice::Mode("required".to_string())
    }

    /// Force the model to call a specific function by name.
    pub fn function(name: impl Into<String>) -> Self {
        ToolChoice::Specific {
            kind: "function".to_string(),
            function: FunctionRef { name: name.into() },
        }
    }
}

impl Tool {
    /// Build a function tool from a `FunctionDef`.
    pub fn function(def: FunctionDef) -> Self {
        Tool::Function { function: def }
    }
}

impl FunctionDef {
    /// Construct a new function definition with a JSON-Schema parameter object.
    pub fn new(name: impl Into<String>, parameters: serde_json::Value) -> Self {
        Self {
            name: name.into(),
            description: None,
            parameters: Some(parameters),
            strict: None,
        }
    }

    /// Attach a human-readable description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Toggle strict schema adherence.
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = Some(strict);
        self
    }
}

/// Response-format hint (JSON mode or JSON schema).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseFormat {
    Text,
    JsonObject,
    JsonSchema { json_schema: JsonSchema },
}

/// Structured-output JSON schema definition.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JsonSchema {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub description: Option<String>,
    pub schema: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub strict: Option<bool>,
}

impl ResponseFormat {
    /// Simple JSON-object mode: the model is asked to emit a valid JSON
    /// object, but the shape is not constrained.
    pub fn json_object() -> Self {
        ResponseFormat::JsonObject
    }

    /// Constrain the response to a named JSON schema.
    pub fn json_schema(name: impl Into<String>, strict: bool, schema: serde_json::Value) -> Self {
        ResponseFormat::JsonSchema {
            json_schema: JsonSchema {
                name: name.into(),
                description: None,
                schema,
                strict: Some(strict),
            },
        }
    }
}

/// Provider routing controls. Fleshed out further in Phase 3.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Provider {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub order: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub allow_fallbacks: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub require_parameters: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub data_collection: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub only: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ignore: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub quantizations: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub sort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub max_price: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub zdr: Option<bool>,
}

impl Provider {
    /// New, empty provider-routing config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Ordered preference list of provider slugs.
    pub fn with_order<S, I>(mut self, order: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        self.order = Some(order.into_iter().map(Into::into).collect());
        self
    }

    /// Sort strategy: `"throughput"`, `"price"`, or `"latency"`.
    pub fn with_sort(mut self, sort: impl Into<String>) -> Self {
        self.sort = Some(sort.into());
        self
    }

    /// Whether OpenRouter may fall back to other providers if the preferred ones fail.
    pub fn with_allow_fallbacks(mut self, allow: bool) -> Self {
        self.allow_fallbacks = Some(allow);
        self
    }

    /// Restrict to this set of providers.
    pub fn with_only<S, I>(mut self, only: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        self.only = Some(only.into_iter().map(Into::into).collect());
        self
    }

    /// Exclude these providers from consideration.
    pub fn with_ignore<S, I>(mut self, ignore: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        self.ignore = Some(ignore.into_iter().map(Into::into).collect());
        self
    }

    /// Permitted quantization tiers (e.g. `"fp8"`, `"int4"`).
    pub fn with_quantizations<S, I>(mut self, q: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        self.quantizations = Some(q.into_iter().map(Into::into).collect());
        self
    }

    /// Per-token max price filter (free-form value, see OpenRouter docs).
    pub fn with_max_price(mut self, price: serde_json::Value) -> Self {
        self.max_price = Some(price);
        self
    }

    /// Data-collection policy: `"allow"` or `"deny"`.
    pub fn with_data_collection(mut self, policy: impl Into<String>) -> Self {
        self.data_collection = Some(policy.into());
        self
    }

    /// Require that providers accept all supplied sampling parameters.
    pub fn with_require_parameters(mut self, required: bool) -> Self {
        self.require_parameters = Some(required);
        self
    }

    /// Per-request Zero-Data-Retention enforcement.
    pub fn with_zdr(mut self, zdr: bool) -> Self {
        self.zdr = Some(zdr);
        self
    }
}

/// Reasoning-tokens configuration.
///
/// `effort` and `max_tokens` are mutually exclusive on OpenRouter's side —
/// setting both yields a 400. Pick the one that matches the constraint you
/// care about (qualitative effort budget vs. hard token cap).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ReasoningConfig {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub exclude: Option<bool>,
}

/// A request-time plugin. Variants serialize with a tagged `id` field
/// (`web`, `file-parser`); new variants can be added without breaking
/// existing callers.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "id", rename_all = "kebab-case")]
pub enum Plugin {
    /// Real-time web search plugin.
    Web(WebPluginConfig),
    /// PDF / file-parser plugin.
    #[serde(rename = "file-parser")]
    File(FilePluginConfig),
}

impl Plugin {
    /// Default web-search plugin (server-side defaults for engine and prompt).
    pub fn web() -> Self {
        Plugin::Web(WebPluginConfig::default())
    }

    /// Web-search plugin with explicit overrides.
    pub fn web_with(config: WebPluginConfig) -> Self {
        Plugin::Web(config)
    }

    /// File-parser plugin with the given PDF parsing engine. Pass `None` to
    /// let OpenRouter pick a default.
    pub fn file_parser(pdf_engine: Option<&str>) -> Self {
        let pdf = pdf_engine.map(|e| FilePdfConfig {
            engine: Some(e.to_string()),
        });
        Plugin::File(FilePluginConfig { pdf })
    }
}

/// Configuration for the `file-parser` plugin.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FilePluginConfig {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub pdf: Option<FilePdfConfig>,
}

/// PDF-specific options for the `file-parser` plugin.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FilePdfConfig {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub engine: Option<String>,
}

/// Configuration for the `web` plugin.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WebPluginConfig {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub max_results: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub search_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub engine: Option<String>,
}

impl WebPluginConfig {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_max_results(mut self, n: u32) -> Self {
        self.max_results = Some(n);
        self
    }
    pub fn with_search_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.search_prompt = Some(prompt.into());
        self
    }
    pub fn with_engine(mut self, engine: impl Into<String>) -> Self {
        self.engine = Some(engine.into());
        self
    }
}

/// A typed annotation attached to an assistant message. OpenRouter emits
/// `url_citation` for the web-search plugin and `file` for the file-parser
/// plugin (so previously-parsed PDFs can be replayed without re-parsing).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Annotation {
    UrlCitation { url_citation: UrlCitation },
    File { file: FileAnnotation },
}

/// Parsed-file annotation: feed it back into a follow-up request to reuse
/// the prior parse result instead of re-running the file-parser.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileAnnotation {
    pub filename: String,
    pub file_data: String,
}

/// A URL citation produced by the web-search plugin.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UrlCitation {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub start_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub end_index: Option<u32>,
}

impl ReasoningConfig {
    /// New, empty reasoning config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the reasoning effort (`"low"`, `"medium"`, `"high"`).
    pub fn with_effort(mut self, effort: impl Into<String>) -> Self {
        self.effort = Some(effort.into());
        self
    }

    /// Cap the number of reasoning tokens.
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Ask the provider to omit reasoning tokens from the response (counts
    /// still appear in usage when supported).
    pub fn with_exclude(mut self, exclude: bool) -> Self {
        self.exclude = Some(exclude);
        self
    }
}
