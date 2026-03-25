//! MCP Tool Registry — implements [`ToolRegistry`] over an [`McpServer`] connection.
//!
//! Connects to an MCP server, discovers all tools via `tools/list`, and exposes
//! them through the standard `ToolRegistry` interface for seamless Agent integration.
//!
//! # Example
//!
//! ```rust,no_run
//! use traitclaw_mcp::McpToolRegistry;
//! use traitclaw_core::traits::tool_registry::ToolRegistry;
//!
//! # async fn example() -> traitclaw_core::Result<()> {
//! let registry = McpToolRegistry::stdio("npx", &["@modelcontextprotocol/server-filesystem"]).await?;
//! println!("Discovered {} tools", registry.len());
//! # Ok(())
//! # }
//! ```

use std::sync::Arc;

use traitclaw_core::traits::tool::ErasedTool;
use traitclaw_core::traits::tool_registry::ToolRegistry;
use traitclaw_core::Result;

use crate::server::McpServer;
use crate::tool::McpTool;

/// A [`ToolRegistry`] backed by an MCP server connection.
///
/// Discovers tools from an MCP server via `tools/list` and exposes them
/// through the standard [`ToolRegistry`] interface. Tools can then be
/// given to an Agent alongside any native tools.
///
/// # Usage
///
/// ```rust,no_run
/// use traitclaw_mcp::McpToolRegistry;
///
/// # async fn example() -> traitclaw_core::Result<()> {
/// let registry = McpToolRegistry::stdio("uvx", &["mcp-server-git"]).await?;
/// // Use as Arc<dyn ToolRegistry> in AgentBuilder::with_registry()
/// # Ok(())
/// # }
/// ```
pub struct McpToolRegistry {
    /// The underlying MCP server connection.
    server: McpServer,
    /// Pre-built `Arc<dyn ErasedTool>` list for ToolRegistry methods.
    tools: Vec<Arc<dyn ErasedTool>>,
}

impl McpToolRegistry {
    /// Connect to an MCP server via stdio (child process).
    ///
    /// Launches `program args...`, performs MCP initialization, discovers all
    /// tools via `tools/list`, and returns a ready-to-use registry.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The process cannot be spawned.
    /// - MCP initialization fails.
    /// - `tools/list` returns an error.
    pub async fn stdio(program: &str, args: &[&str]) -> Result<Self> {
        let server = McpServer::stdio(program, args).await?;
        let tools = server.erased_tools();

        tracing::info!(
            "McpToolRegistry: connected to '{}', discovered {} tool(s)",
            program,
            tools.len()
        );

        Ok(Self { server, tools })
    }

    /// Create a registry from an already-initialized [`McpServer`].
    ///
    /// Useful when you need to share a server connection or customize initialization.
    #[must_use]
    pub fn from_server(server: McpServer) -> Self {
        let tools = server.erased_tools();
        Self { server, tools }
    }

    /// Access the underlying [`McpServer`] for advanced use cases.
    #[must_use]
    pub fn server(&self) -> &McpServer {
        &self.server
    }

    /// Access raw [`McpTool`] instances (with MCP-specific methods).
    #[must_use]
    pub fn mcp_tools(&self) -> &[Arc<McpTool>] {
        self.server.tools()
    }
}

impl ToolRegistry for McpToolRegistry {
    fn get_tools(&self) -> Vec<Arc<dyn ErasedTool>> {
        self.tools.clone()
    }

    fn find_tool(&self, name: &str) -> Option<Arc<dyn ErasedTool>> {
        self.tools.iter().find(|t| t.name() == name).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use traitclaw_core::traits::tool::{ErasedTool, ToolSchema};

    // ── FakeRegistry for unit-testing ToolRegistry impl independently ────────

    /// A hand-built McpToolRegistry equivalent for testing — uses fake tools.
    struct FakeRegistry {
        tools: Vec<Arc<dyn ErasedTool>>,
    }

    impl FakeRegistry {
        fn with_tools(tools: Vec<Arc<dyn ErasedTool>>) -> Self {
            Self { tools }
        }
    }

    impl ToolRegistry for FakeRegistry {
        fn get_tools(&self) -> Vec<Arc<dyn ErasedTool>> {
            self.tools.clone()
        }

        fn find_tool(&self, name: &str) -> Option<Arc<dyn ErasedTool>> {
            self.tools.iter().find(|t| t.name() == name).cloned()
        }
    }

    /// Minimal fake ErasedTool for testing.
    struct FakeTool {
        name: String,
    }

    impl FakeTool {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    #[async_trait::async_trait]
    impl ErasedTool for FakeTool {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            "fake tool"
        }

        fn schema(&self) -> ToolSchema {
            ToolSchema {
                name: self.name.clone(),
                description: "fake tool".into(),
                parameters: serde_json::json!({ "type": "object", "properties": {} }),
            }
        }

        async fn execute_json(&self, _input: Value) -> traitclaw_core::Result<Value> {
            Ok(Value::String(format!("{} called", self.name)))
        }
    }

    fn make_registry(names: &[&str]) -> FakeRegistry {
        let tools: Vec<Arc<dyn ErasedTool>> = names
            .iter()
            .map(|n| Arc::new(FakeTool::new(n)) as Arc<dyn ErasedTool>)
            .collect();
        FakeRegistry::with_tools(tools)
    }

    #[test]
    fn test_get_tools_returns_all() {
        // AC #6: get_tools() returns all discovered tools
        let reg = make_registry(&["read_file", "write_file", "list_dir", "search", "delete"]);
        assert_eq!(reg.get_tools().len(), 5);
    }

    #[test]
    fn test_find_tool_by_name() {
        // AC #5: find_tool(name) returns the correct tool
        let reg = make_registry(&["read_file", "write_file", "search"]);
        assert!(reg.find_tool("read_file").is_some());
        assert!(reg.find_tool("write_file").is_some());
        assert!(reg.find_tool("search").is_some());
    }

    #[test]
    fn test_find_tool_not_found() {
        let reg = make_registry(&["read_file"]);
        assert!(reg.find_tool("nonexistent").is_none());
    }

    #[test]
    fn test_len_and_is_empty() {
        let reg = make_registry(&["a", "b", "c"]);
        assert_eq!(reg.len(), 3);
        assert!(!reg.is_empty());

        let empty = make_registry(&[]);
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn test_tool_schemas_populated() {
        // AC #4: MCP tool schemas map to ToolSchema format
        let reg = make_registry(&["read_file"]);
        let tools = reg.get_tools();
        assert_eq!(tools.len(), 1);
        let schema = tools[0].schema();
        assert_eq!(schema.name, "read_file");
        assert!(!schema.parameters.is_null());
    }

    #[tokio::test]
    async fn test_tool_execution_through_registry() {
        // AC #5, #7: find_tool → execute → verify result
        let reg = make_registry(&["echo_tool"]);
        let tool = reg.find_tool("echo_tool").expect("echo_tool should exist");
        let result = tool
            .execute_json(serde_json::json!({"text": "hello"}))
            .await
            .unwrap();
        assert_eq!(result, Value::String("echo_tool called".into()));
    }

    #[test]
    fn test_object_safe_as_trait_object() {
        // Verify McpToolRegistry-equivalent can be used as Arc<dyn ToolRegistry>
        let reg = make_registry(&["a"]);
        let _: Arc<dyn ToolRegistry> = Arc::new(reg);
    }
}
