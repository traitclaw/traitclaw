//! MCP server connection — stdio-based child process management.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;
use traitclaw_core::{Error, Result};

use crate::protocol::{JsonRpcRequest, JsonRpcResponse, ToolsListResponse};
use crate::tool::McpTool;

/// A connection to an MCP server process.
///
/// Manages the child process lifecycle, sends JSON-RPC requests, and
/// discovers tools via `tools/list`.
pub struct McpServer {
    stdin: Arc<Mutex<ChildStdin>>,
    stdout: Arc<Mutex<BufReader<ChildStdout>>>,
    next_id: AtomicU64,
    tools: Vec<Arc<McpTool>>,
    #[allow(dead_code)]
    child: Child,
}

impl McpServer {
    /// Launch an MCP server as a child process communicating over stdio.
    ///
    /// After launching, automatically calls `initialize` and `tools/list`
    /// to discover available tools.
    ///
    /// # Errors
    ///
    /// Returns an error if the process cannot be spawned or initialization fails.
    pub async fn stdio(program: &str, args: &[&str]) -> Result<Self> {
        let mut child = Command::new(program)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| Error::Runtime(format!("Failed to spawn MCP server: {e}")))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| Error::Runtime("Failed to capture stdin".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| Error::Runtime("Failed to capture stdout".into()))?;

        let stdin = Arc::new(Mutex::new(stdin));
        let stdout = Arc::new(Mutex::new(BufReader::new(stdout)));

        let mut server = Self {
            stdin,
            stdout,
            next_id: AtomicU64::new(1),
            tools: Vec::new(),
            child,
        };

        // Initialize the MCP connection
        server.initialize().await?;

        // Discover tools
        server.discover_tools().await?;

        Ok(server)
    }

    /// Get the discovered tools as `Arc<McpTool>` instances.
    ///
    /// These implement `ErasedTool` and can be added to an Agent's tool registry.
    #[must_use]
    pub fn tools(&self) -> &[Arc<McpTool>] {
        &self.tools
    }

    /// Get tools as `Arc<dyn ErasedTool>` for Agent integration.
    #[must_use]
    pub fn erased_tools(&self) -> Vec<Arc<dyn traitclaw_core::traits::tool::ErasedTool>> {
        self.tools
            .iter()
            .map(|t| Arc::clone(t) as Arc<dyn traitclaw_core::traits::tool::ErasedTool>)
            .collect()
    }

    async fn initialize(&mut self) -> Result<()> {
        let req = JsonRpcRequest::new(
            self.next_id(),
            "initialize",
            Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "traitclaw",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })),
        );
        let _resp = self.send_request(req).await?;

        // Send initialized notification (no response expected)
        let notif = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        let mut stdin = self.stdin.lock().await;
        let line = serde_json::to_string(&notif)
            .map_err(|e| Error::Runtime(format!("JSON error: {e}")))?;
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

        Ok(())
    }

    async fn discover_tools(&mut self) -> Result<()> {
        let req = JsonRpcRequest::new(self.next_id(), "tools/list", None);
        let resp = self.send_request(req).await?;

        let result = resp
            .result
            .ok_or_else(|| Error::Runtime("No result in tools/list response".into()))?;

        let tools_resp: ToolsListResponse = serde_json::from_value(result)
            .map_err(|e| Error::Runtime(format!("Parse tools/list error: {e}")))?;

        self.tools = tools_resp
            .tools
            .into_iter()
            .map(|def| {
                Arc::new(McpTool::new(
                    def.name,
                    def.description,
                    def.input_schema,
                    Arc::clone(&self.stdin),
                    Arc::clone(&self.stdout),
                    self.next_id.load(Ordering::Relaxed),
                ))
            })
            .collect();

        Ok(())
    }

    async fn send_request(&self, req: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let line =
            serde_json::to_string(&req).map_err(|e| Error::Runtime(format!("JSON error: {e}")))?;

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

        // Read response line
        let mut stdout = self.stdout.lock().await;
        let mut line = String::new();
        stdout
            .read_line(&mut line)
            .await
            .map_err(|e| Error::Runtime(format!("Read error: {e}")))?;

        let resp: JsonRpcResponse = serde_json::from_str(&line)
            .map_err(|e| Error::Runtime(format!("Parse response error: {e}")))?;

        if let Some(err) = resp.error {
            return Err(Error::Runtime(format!("MCP error: {}", err.message)));
        }

        Ok(resp)
    }

    fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_rpc_request_serialization() {
        let req = JsonRpcRequest::new(1, "tools/list", None);
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["id"], 1);
        assert_eq!(json["method"], "tools/list");
        assert!(json.get("params").is_none());
    }

    #[test]
    fn test_json_rpc_request_with_params() {
        let req = JsonRpcRequest::new(2, "tools/call", Some(serde_json::json!({"name": "test"})));
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["params"]["name"], "test");
    }
}
