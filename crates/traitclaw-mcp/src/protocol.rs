//! MCP JSON-RPC 2.0 protocol types.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A JSON-RPC 2.0 request.
#[derive(Debug, Serialize)]
pub(crate) struct JsonRpcRequest {
    /// JSON-RPC version (always "2.0").
    pub jsonrpc: &'static str,
    /// Request ID for matching responses.
    pub id: u64,
    /// Method name (e.g., "tools/list", "tools/call").
    pub method: String,
    /// Optional parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request.
    pub fn new(id: u64, method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            method: method.into(),
            params,
        }
    }
}

/// A JSON-RPC 2.0 response.
#[derive(Debug, Deserialize)]
pub(crate) struct JsonRpcResponse {
    /// The response ID (matches request).
    #[allow(dead_code)]
    pub id: Option<u64>,
    /// Result on success.
    pub result: Option<Value>,
    /// Error on failure.
    pub error: Option<JsonRpcError>,
}

/// A JSON-RPC 2.0 error.
#[derive(Debug, Deserialize)]
pub(crate) struct JsonRpcError {
    /// Error code.
    #[allow(dead_code)]
    pub code: i64,
    /// Error message.
    pub message: String,
}

/// An MCP tool definition from `tools/list`.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct McpToolDef {
    /// Tool name.
    pub name: String,
    /// Tool description.
    #[serde(default)]
    pub description: String,
    /// JSON Schema for the input parameters.
    #[serde(default, rename = "inputSchema")]
    pub input_schema: Value,
}

/// Response from `tools/list`.
#[derive(Debug, Deserialize)]
pub(crate) struct ToolsListResponse {
    /// Available tools.
    pub tools: Vec<McpToolDef>,
}

/// Response from `tools/call`.
#[derive(Debug, Deserialize)]
pub(crate) struct ToolCallResponse {
    /// Content blocks returned by the tool.
    pub content: Vec<ToolCallContent>,
}

/// A content block from a tool call response.
#[derive(Debug, Deserialize)]
pub(crate) struct ToolCallContent {
    /// Content type (usually "text").
    #[serde(rename = "type")]
    #[allow(dead_code)]
    pub kind: String,
    /// The text content.
    #[serde(default)]
    pub text: String,
}
