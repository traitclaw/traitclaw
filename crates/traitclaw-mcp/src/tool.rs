//! MCP tool — implements `ErasedTool` for seamless Agent integration.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use traitclaw_core::traits::tool::{ErasedTool, ToolSchema};
use traitclaw_core::{Error, Result};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::process::ChildStdin;
use tokio::sync::Mutex;

use crate::protocol::{JsonRpcRequest, JsonRpcResponse, ToolCallResponse};

/// A tool discovered from an MCP server.
///
/// Implements [`ErasedTool`] so it can be used alongside native tools in an Agent.
pub struct McpTool {
    name: String,
    description: String,
    input_schema: Value,
    stdin: Arc<Mutex<ChildStdin>>,
    stdout: Arc<Mutex<tokio::io::BufReader<tokio::process::ChildStdout>>>,
    next_id: AtomicU64,
}

impl McpTool {
    /// Create a new `McpTool` from an MCP tool definition.
    pub(crate) fn new(
        name: String,
        description: String,
        input_schema: Value,
        stdin: Arc<Mutex<ChildStdin>>,
        stdout: Arc<Mutex<tokio::io::BufReader<tokio::process::ChildStdout>>>,
        base_id: u64,
    ) -> Self {
        Self {
            name,
            description,
            input_schema,
            stdin,
            stdout,
            // Start IDs high to avoid collisions with server-level requests
            next_id: AtomicU64::new(base_id + 1000),
        }
    }

    fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }
}

#[async_trait]
impl ErasedTool for McpTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name.clone(),
            description: self.description.clone(),
            parameters: self.input_schema.clone(),
        }
    }

    async fn execute_json(&self, input: Value) -> Result<Value> {
        let req = JsonRpcRequest::new(
            self.next_id(),
            "tools/call",
            Some(serde_json::json!({
                "name": self.name,
                "arguments": input
            })),
        );

        let line =
            serde_json::to_string(&req).map_err(|e| Error::Runtime(format!("JSON error: {e}")))?;

        // Send request
        let mut stdin = self.stdin.lock().await;
        stdin
            .write_all(line.as_bytes())
            .await
            .map_err(|e| Error::Runtime(format!("Write error: {e}")))?;
        stdin
            .write_all(b"\n")
            .await
            .map_err(|e| Error::Runtime(format!("Write error: {e}")))?;
        stdin
            .flush()
            .await
            .map_err(|e| Error::Runtime(format!("Flush error: {e}")))?;
        drop(stdin);

        // Read response
        let mut stdout = self.stdout.lock().await;
        let mut resp_line = String::new();
        stdout
            .read_line(&mut resp_line)
            .await
            .map_err(|e| Error::Runtime(format!("Read error: {e}")))?;
        drop(stdout);

        let resp: JsonRpcResponse = serde_json::from_str(&resp_line)
            .map_err(|e| Error::Runtime(format!("Parse error: {e}")))?;

        if let Some(err) = resp.error {
            return Err(Error::Runtime(format!("MCP tool error: {}", err.message)));
        }

        let result = resp
            .result
            .ok_or_else(|| Error::Runtime("No result in tool call response".into()))?;

        // Parse content blocks and join text
        let call_resp: ToolCallResponse = serde_json::from_value(result)
            .map_err(|e| Error::Runtime(format!("Parse tool result: {e}")))?;

        let text: String = call_resp
            .content
            .iter()
            .map(|c| c.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        Ok(Value::String(text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_schema_generation() {
        let schema = ToolSchema {
            name: "read_file".into(),
            description: "Read a file from disk".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" }
                }
            }),
        };
        assert_eq!(schema.name, "read_file");
        assert_eq!(schema.description, "Read a file from disk");
        assert_eq!(schema.parameters["type"], "object");
    }
}
