//! MCP (Model Context Protocol) client for the `TraitClaw` AI agent framework.
//!
//! Connects to MCP servers over stdio or SSE transport, discovers tools via
//! `tools/list`, and routes tool calls via `tools/call`. MCP tools implement
//! [`ErasedTool`] so they work seamlessly alongside native tools.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use traitclaw_mcp::McpServer;
//!
//! # async fn example() -> traitclaw_core::Result<()> {
//! let server = McpServer::stdio("npx", &["@modelcontextprotocol/server-filesystem"]).await?;
//! let tools = server.tools(); // Vec<Arc<dyn ErasedTool>>
//! # Ok(())
//! # }
//! ```

#![deny(warnings)]
#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod multi_server;
mod protocol;
mod registry;
mod server;
mod tool;

pub use multi_server::{MultiServerMcpRegistry, MultiServerMcpRegistryBuilder};
pub use registry::McpToolRegistry;
pub use server::McpServer;
pub use tool::McpTool;

/// Prelude re-exports for convenient use.
pub mod prelude {
    pub use super::{
        McpServer, McpTool, McpToolRegistry, MultiServerMcpRegistry, MultiServerMcpRegistryBuilder,
    };
}
