//! Shared sub-types: tools, response format, provider routing, reasoning.

use serde::{Deserialize, Serialize};

/// A function-call invocation requested by the model.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
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

/// Reasoning-tokens configuration.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ReasoningConfig {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub exclude: Option<bool>,
}
