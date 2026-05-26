//! Convert [Model Context Protocol](https://modelcontextprotocol.io/) tool
//! definitions into OpenRouter [`Tool`]s.
//!
//! MCP tools are typically described as JSON with the shape
//! `{ "name": "...", "description": "...", "inputSchema": {...} }`
//! (camelCase). OpenRouter expects the same conceptual fields under
//! `parameters`, so the mapping is mechanical.
//!
//! These helpers take `serde_json::Value` rather than a typed MCP struct so
//! callers don't need to pull in an MCP SDK to interoperate.

use serde_json::Value;

use crate::error::{Error, Result};
use crate::types::{FunctionDef, Tool};

/// Convert a single MCP tool description into an OpenRouter [`Tool`].
///
/// Required fields: `name` (string). Optional: `description` (string),
/// `inputSchema` (object — mapped to `parameters`).
pub fn convert_tool(mcp_tool: &Value) -> Result<Tool> {
    let name = mcp_tool
        .get("name")
        .and_then(Value::as_str)
        .ok_or(Error::InvalidInput("MCP tool is missing required `name`"))?
        .to_string();

    let mut def = FunctionDef {
        name,
        description: None,
        parameters: None,
        strict: None,
    };

    if let Some(desc) = mcp_tool.get("description").and_then(Value::as_str) {
        def.description = Some(desc.to_string());
    }

    if let Some(schema) = mcp_tool.get("inputSchema") {
        if !schema.is_null() {
            def.parameters = Some(schema.clone());
        }
    }

    Ok(Tool::Function { function: def })
}

/// Convert a slice of MCP tool descriptions into OpenRouter [`Tool`]s. Stops
/// at the first conversion error.
pub fn convert_tools(mcp_tools: &[Value]) -> Result<Vec<Tool>> {
    mcp_tools.iter().map(convert_tool).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    fn sample_mcp_tool() -> Value {
        json!({
            "name": "get_weather",
            "description": "Look up the weather for a city.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "location": {"type": "string"}
                },
                "required": ["location"]
            }
        })
    }

    #[test]
    fn convert_tool_maps_input_schema_to_parameters() {
        let tool = convert_tool(&sample_mcp_tool()).unwrap();
        let v = serde_json::to_value(&tool).unwrap();
        assert_eq!(
            v,
            json!({
                "type": "function",
                "function": {
                    "name": "get_weather",
                    "description": "Look up the weather for a city.",
                    "parameters": {
                        "type": "object",
                        "properties": {"location": {"type":"string"}},
                        "required": ["location"]
                    }
                }
            })
        );
    }

    #[test]
    fn convert_tool_without_description_or_schema() {
        let tool = convert_tool(&json!({"name":"bare"})).unwrap();
        let v = serde_json::to_value(&tool).unwrap();
        assert_eq!(v, json!({"type":"function","function":{"name":"bare"}}));
    }

    #[test]
    fn convert_tool_missing_name_errors() {
        let err = convert_tool(&json!({"description":"x"})).unwrap_err();
        assert!(matches!(err, Error::InvalidInput(_)));
    }

    #[test]
    fn convert_tools_handles_multiple() {
        let tools = convert_tools(&[
            sample_mcp_tool(),
            json!({"name":"echo","inputSchema":{"type":"object"}}),
        ])
        .unwrap();
        assert_eq!(tools.len(), 2);
    }
}
