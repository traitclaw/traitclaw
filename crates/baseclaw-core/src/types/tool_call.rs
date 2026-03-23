//! Tool call types.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A tool call requested by the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique identifier for this tool call.
    pub id: String,
    /// Name of the tool to call.
    pub name: String,
    /// Arguments for the tool as a JSON object.
    pub arguments: Value,
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
    }
}
