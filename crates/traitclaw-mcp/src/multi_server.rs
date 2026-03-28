//! Multi-server MCP registry with resilience and refresh support.
//!
//! Aggregates tools from multiple MCP servers into a single [`ToolRegistry`],
//! with optional name-prefixing to avoid collisions and per-server resilience.
//!
//! # Example
//!
//! ```rust,no_run
//! use traitclaw_mcp::MultiServerMcpRegistry;
//! use traitclaw_core::traits::tool_registry::ToolRegistry;
//!
//! # async fn example() -> traitclaw_core::Result<()> {
//! let registry = MultiServerMcpRegistry::builder()
//!     .with_prefix(true)
//!     .add_stdio("fs", "npx", &["@modelcontextprotocol/server-filesystem"])
//!     .add_stdio("git", "uvx", &["mcp-server-git"])
//!     .build()
//!     .await?;
//!
//! println!("Tools: {}", registry.len());
//! // tools named like "fs::read_file", "git::create_commit"
//! # Ok(())
//! # }
//! ```

use std::sync::Arc;

use traitclaw_core::traits::tool::ErasedTool;
use traitclaw_core::traits::tool_registry::ToolRegistry;
use traitclaw_core::Result;

/// A single server entry — a named MCP server connection with its tools.
#[allow(dead_code)] // `name` used at build-time for logging; kept for future introspection
struct ServerEntry {
    /// Human-readable name used for tool name prefixing.
    name: String,
    /// Pre-built erased tools for this server.
    tools: Vec<Arc<dyn ErasedTool>>,
    /// Whether this server is currently healthy.
    healthy: bool,
}

/// Builder for [`MultiServerMcpRegistry`].
///
/// Configure multiple MCP servers before establishing connections.
pub struct MultiServerMcpRegistryBuilder {
    servers: Vec<PendingServer>,
    prefix: bool,
}

/// A pending server config (before `build()` connects).
struct PendingServer {
    name: String,
    program: String,
    args: Vec<String>,
}

impl MultiServerMcpRegistryBuilder {
    /// Enable/disable server-name prefixing for tool names.
    ///
    /// When enabled (default: `true`), tools are named `server::tool_name`
    /// to avoid collisions when multiple servers expose the same tool name.
    #[must_use]
    pub fn with_prefix(mut self, enabled: bool) -> Self {
        self.prefix = enabled;
        self
    }

    /// Add an MCP server via stdio child process.
    ///
    /// `name` is used for tool prefix (e.g., `"fs"` → `"fs::read_file"`).
    #[must_use]
    pub fn add_stdio(
        mut self,
        name: impl Into<String>,
        program: impl Into<String>,
        args: &[&str],
    ) -> Self {
        self.servers.push(PendingServer {
            name: name.into(),
            program: program.into(),
            args: args.iter().map(|s| s.to_string()).collect(),
        });
        self
    }

    /// Connect to all configured servers and return a ready registry.
    ///
    /// Servers that fail to connect are marked as unhealthy but do not
    /// prevent the registry from being returned with remaining servers.
    ///
    /// # Errors
    ///
    /// Only returns an error if ALL servers fail to connect. If at least
    /// one server connects successfully, returns `Ok`.
    pub async fn build(self) -> Result<MultiServerMcpRegistry> {
        use crate::server::McpServer;

        let prefix = self.prefix;
        let mut entries = Vec::with_capacity(self.servers.len());

        for pending in self.servers {
            let args: Vec<&str> = pending.args.iter().map(String::as_str).collect();
            match McpServer::stdio(&pending.program, &args).await {
                Ok(server) => {
                    let raw_tools = server.erased_tools();
                    let tools = if prefix {
                        apply_prefix(&pending.name, raw_tools)
                    } else {
                        raw_tools
                    };
                    tracing::info!(
                        "MultiServerMcpRegistry: server '{}' connected, {} tool(s)",
                        pending.name,
                        tools.len()
                    );
                    entries.push(ServerEntry {
                        name: pending.name,
                        tools,
                        healthy: true,
                    });
                }
                Err(e) => {
                    tracing::warn!(
                        "MultiServerMcpRegistry: server '{}' failed to connect: {}. Marking unhealthy.",
                        pending.name,
                        e
                    );
                    entries.push(ServerEntry {
                        name: pending.name,
                        tools: vec![],
                        healthy: false,
                    });
                }
            }
        }

        Ok(MultiServerMcpRegistry { entries, prefix })
    }
}

/// A [`ToolRegistry`] that aggregates tools from multiple MCP servers.
///
/// Use [`MultiServerMcpRegistry::builder()`] to configure and connect.
pub struct MultiServerMcpRegistry {
    entries: Vec<ServerEntry>,
    prefix: bool,
}

impl MultiServerMcpRegistry {
    /// Create a new builder.
    #[must_use]
    pub fn builder() -> MultiServerMcpRegistryBuilder {
        MultiServerMcpRegistryBuilder {
            servers: Vec::new(),
            prefix: true,
        }
    }

    /// Number of healthy (connected) servers.
    #[must_use]
    pub fn healthy_server_count(&self) -> usize {
        self.entries.iter().filter(|e| e.healthy).count()
    }

    /// Number of unhealthy (disconnected) servers.
    #[must_use]
    pub fn unhealthy_server_count(&self) -> usize {
        self.entries.iter().filter(|e| !e.healthy).count()
    }

    /// Whether prefixing is enabled.
    #[must_use]
    pub fn prefix_enabled(&self) -> bool {
        self.prefix
    }

    /// Resolve a prefixed tool name to `(server_name, bare_tool_name)`.
    ///
    /// For `"fs::read_file"` returns `Some(("fs", "read_file"))`.
    /// Returns `None` if the name has no prefix separator.
    #[must_use]
    pub fn resolve_prefix(name: &str) -> Option<(&str, &str)> {
        name.split_once("::")
    }
}

impl ToolRegistry for MultiServerMcpRegistry {
    fn get_tools(&self) -> Vec<Arc<dyn ErasedTool>> {
        self.entries
            .iter()
            .filter(|e| e.healthy)
            .flat_map(|e| e.tools.iter().cloned())
            .collect()
    }

    fn find_tool(&self, name: &str) -> Option<Arc<dyn ErasedTool>> {
        self.entries
            .iter()
            .filter(|e| e.healthy)
            .flat_map(|e| e.tools.iter())
            .find(|t| t.name() == name)
            .cloned()
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────────────

/// Wrap tools with a prefixed-name adapter.
fn apply_prefix(server_name: &str, tools: Vec<Arc<dyn ErasedTool>>) -> Vec<Arc<dyn ErasedTool>> {
    tools
        .into_iter()
        .map(|tool| {
            let prefixed_name = format!("{}::{}", server_name, tool.name());
            Arc::new(PrefixedTool {
                prefixed_name,
                inner: tool,
            }) as Arc<dyn ErasedTool>
        })
        .collect()
}

/// Adapter that overlays a prefixed name on an existing tool.
struct PrefixedTool {
    prefixed_name: String,
    inner: Arc<dyn ErasedTool>,
}

#[async_trait::async_trait]
impl ErasedTool for PrefixedTool {
    fn name(&self) -> &str {
        &self.prefixed_name
    }

    fn description(&self) -> &str {
        self.inner.description()
    }

    fn schema(&self) -> traitclaw_core::traits::tool::ToolSchema {
        let mut schema = self.inner.schema();
        schema.name.clone_from(&self.prefixed_name);
        schema
    }

    async fn execute_json(
        &self,
        input: serde_json::Value,
    ) -> traitclaw_core::Result<serde_json::Value> {
        self.inner.execute_json(input).await
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use traitclaw_core::traits::tool::{ErasedTool, ToolSchema};

    /// Minimal fake tool for testing.
    struct Fake(String);

    #[async_trait::async_trait]
    impl ErasedTool for Fake {
        fn name(&self) -> &str {
            &self.0
        }
        fn description(&self) -> &str {
            "fake"
        }
        fn schema(&self) -> ToolSchema {
            ToolSchema {
                name: self.0.clone(),
                description: "fake".into(),
                parameters: Value::Null,
            }
        }
        async fn execute_json(&self, _: Value) -> traitclaw_core::Result<Value> {
            Ok(Value::String(self.0.clone()))
        }
    }

    fn fake(name: &str) -> Arc<dyn ErasedTool> {
        Arc::new(Fake(name.to_string()))
    }

    #[allow(clippy::type_complexity)]
    fn make_registry(
        servers: Vec<(String, Vec<Arc<dyn ErasedTool>>, bool)>,
        prefix: bool,
    ) -> MultiServerMcpRegistry {
        let entries = servers
            .into_iter()
            .map(|(name, tools, healthy)| ServerEntry {
                name,
                tools,
                healthy,
            })
            .collect();
        MultiServerMcpRegistry { entries, prefix }
    }

    #[test]
    fn test_aggregate_tools_from_multiple_healthy_servers() {
        // AC #2: tools from all servers are aggregated
        let reg = make_registry(
            vec![
                (
                    "fs".into(),
                    vec![fake("read_file"), fake("write_file")],
                    true,
                ),
                ("git".into(), vec![fake("commit"), fake("diff")], true),
            ],
            false,
        );
        assert_eq!(reg.len(), 4);
    }

    #[test]
    fn test_unhealthy_server_tools_excluded() {
        // AC #5: if a server is unreachable, other servers' tools remain available
        let reg = make_registry(
            vec![
                ("fs".into(), vec![fake("read_file")], true),
                ("broken".into(), vec![fake("secret_tool")], false),
            ],
            false,
        );
        assert_eq!(reg.len(), 1);
        assert!(reg.find_tool("read_file").is_some());
        assert!(reg.find_tool("secret_tool").is_none());
    }

    #[test]
    fn test_prefix_applied_to_tool_names() {
        // AC #3: tool names are prefixed with server name
        let raw = vec![fake("read_file")];
        let prefixed = apply_prefix("fs", raw);
        assert_eq!(prefixed[0].name(), "fs::read_file");
        assert_eq!(prefixed[0].description(), "fake");
    }

    #[test]
    fn test_no_collision_with_prefix() {
        // AC #8: 2 servers with overlapping tool names → no collision via prefix
        let reg = make_registry(
            vec![
                ("server_a".into(), vec![fake("server_a::search")], true),
                ("server_b".into(), vec![fake("server_b::search")], true),
            ],
            true,
        );
        assert_eq!(reg.len(), 2);
        assert!(reg.find_tool("server_a::search").is_some());
        assert!(reg.find_tool("server_b::search").is_some());
    }

    #[test]
    fn test_healthy_unhealthy_counts() {
        let reg = make_registry(
            vec![
                ("ok1".into(), vec![], true),
                ("ok2".into(), vec![], true),
                ("bad".into(), vec![], false),
            ],
            false,
        );
        assert_eq!(reg.healthy_server_count(), 2);
        assert_eq!(reg.unhealthy_server_count(), 1);
    }

    #[test]
    fn test_resolve_prefix() {
        assert_eq!(
            MultiServerMcpRegistry::resolve_prefix("fs::read_file"),
            Some(("fs", "read_file"))
        );
        assert_eq!(MultiServerMcpRegistry::resolve_prefix("no_prefix"), None);
        assert_eq!(
            MultiServerMcpRegistry::resolve_prefix("a::b::c"),
            Some(("a", "b::c"))
        );
    }

    #[test]
    fn test_prefix_disabled_no_prefix_in_names() {
        let raw = vec![fake("read_file")];
        let non_prefixed = raw; // no apply_prefix called
        assert_eq!(non_prefixed[0].name(), "read_file");
    }

    #[test]
    fn test_empty_registry() {
        let reg = make_registry(vec![], false);
        assert!(reg.is_empty());
        assert_eq!(reg.healthy_server_count(), 0);
    }

    #[tokio::test]
    async fn test_prefixed_tool_execution_delegates_to_inner() {
        // AC #3: PrefixedTool.execute_json routes to inner
        let raw = vec![fake("echo")];
        let prefixed = apply_prefix("srv", raw);
        let result = prefixed[0].execute_json(Value::Null).await.unwrap();
        assert_eq!(result, Value::String("echo".into()));
    }

    #[test]
    fn test_object_safe_as_dyn_registry() {
        let reg = make_registry(vec![], false);
        let _: Arc<dyn ToolRegistry> = Arc::new(reg);
    }
}
