//! Tool call types.
//!
//! These types represent tool calls requested by an LLM, following the
//! `OpenAI` Chat Completions `API` format.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A function call within a tool call request.
///
/// Mirrors the `OpenAI` `function` object inside a `tool_call`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    /// The name of the function to call.
    pub name: String,
    /// Arguments to the function as a JSON-encoded string (`OpenAI` format)
    /// or a parsed [`Value`] depending on the context.
    ///
    /// Providers send this as a raw JSON string; we parse it into a [`Value`]
    /// after reception. See [`ToolCall::arguments`].
    pub arguments: String,
}

/// A tool call requested by the LLM.
///
/// Follows the `OpenAI` `tool_call` format:
/// ```json
/// {
///   "id": "call_abc123",
///   "type": "function",
///   "function": { "name": "web_search", "arguments": "{\"query\":\"rust\"}" }
/// }
/// ```
///
/// Internally we flatten `function.name` and parse `function.arguments` into
/// a [`Value`] for ergonomic use in the runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique identifier for this tool call (assigned by the LLM).
    pub id: String,
    /// Name of the tool to call.
    pub name: String,
    /// Arguments for the tool as a parsed JSON object.
    pub arguments: Value,
}

/// Wire-format tool call matching the `OpenAI` `tool_call` object exactly.
///
/// Used for (de)serialization with providers; converted to [`ToolCall`] by
/// provider adapters before passing to the runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireToolCall {
    /// Unique identifier.
    pub id: String,
    /// Always `"function"` in current providers.
    #[serde(rename = "type")]
    pub type_: String,
    /// The function call details.
    pub function: FunctionCall,
}

impl TryFrom<WireToolCall> for ToolCall {
    type Error = serde_json::Error;

    /// Convert a wire-format tool call into the internal runtime format.
    ///
    /// Parses `function.arguments` (a JSON-encoded string) into a
    /// [`serde_json::Value`] for ergonomic use in the runtime.
    ///
    /// # Errors
    ///
    /// Returns `serde_json::Error` if `arguments` is not valid JSON.
    fn try_from(wire: WireToolCall) -> Result<Self, Self::Error> {
        let arguments: Value = serde_json::from_str(&wire.function.arguments)?;
        Ok(Self {
            id: wire.id,
            name: wire.function.name,
            arguments,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_call_creation() {
        let call = ToolCall {
            id: "call_123".into(),
            name: "web_search".into(),
            arguments: serde_json::json!({"query": "rust"}),
        };
        assert_eq!(call.name, "web_search");
        assert_eq!(call.id, "call_123");
    }

    #[test]
    fn test_tool_call_roundtrip() {
        let call = ToolCall {
            id: "call_abc".into(),
            name: "get_weather".into(),
            arguments: serde_json::json!({"city": "Hanoi"}),
        };
        let json = serde_json::to_string(&call).unwrap();
        let decoded: ToolCall = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.id, call.id);
        assert_eq!(decoded.name, call.name);
        assert_eq!(decoded.arguments, call.arguments);
    }

    #[test]
    fn test_wire_tool_call_type_field_serializes_as_type() {
        let wire = WireToolCall {
            id: "call_xyz".into(),
            type_: "function".into(),
            function: FunctionCall {
                name: "calc".into(),
                arguments: "{\"x\": 1}".into(),
            },
        };
        let json = serde_json::to_string(&wire).unwrap();
        // The `type_` field must serialize as `"type"` per OpenAI spec
        assert!(json.contains("\"type\":\"function\""), "got: {json}");
        assert!(json.contains("\"function\":{"), "got: {json}");
    }

    #[test]
    fn test_wire_tool_call_deserialize() {
        let raw = r#"{
            "id": "call_1",
            "type": "function",
            "function": { "name": "search", "arguments": "{\"q\":\"test\"}" }
        }"#;
        let wire: WireToolCall = serde_json::from_str(raw).unwrap();
        assert_eq!(wire.id, "call_1");
        assert_eq!(wire.type_, "function");
        assert_eq!(wire.function.name, "search");
    }

    #[test]
    fn test_function_call_roundtrip() {
        let fc = FunctionCall {
            name: "my_tool".into(),
            arguments: r#"{"key":"value"}"#.into(),
        };
        let json = serde_json::to_string(&fc).unwrap();
        let decoded: FunctionCall = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.name, fc.name);
        assert_eq!(decoded.arguments, fc.arguments);
    }
}
