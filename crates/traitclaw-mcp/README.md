# traitclaw-mcp

[![crates.io](https://img.shields.io/crates/v/traitclaw-mcp.svg)](https://crates.io/crates/traitclaw-mcp)
[![docs.rs](https://docs.rs/traitclaw-mcp/badge.svg)](https://docs.rs/traitclaw-mcp)

**MCP (Model Context Protocol) client for TraitClaw — discover and call tools from MCP servers.**

Connect to any [MCP](https://modelcontextprotocol.io/)-compatible tool server over stdio or SSE transport. Discovered tools implement `ErasedTool` and work seamlessly alongside native TraitClaw tools.

## What is MCP?

The **Model Context Protocol** is an open standard for connecting AI agents to external tools and data sources. MCP servers expose tools (filesystem, databases, APIs, etc.) that AI agents can discover and invoke.

## Usage

```rust
use traitclaw::prelude::*;
use traitclaw_mcp::McpServer;

// Connect to an MCP server via stdio
let server = McpServer::stdio(
    "npx",
    &["-y", "@modelcontextprotocol/server-filesystem", "/tmp"],
).await?;

// Discover available tools
let tools = server.tools(); // &[Arc<McpTool>]

// Register with an agent — MCP tools work like native tools
let mut builder = Agent::builder()
    .provider(my_provider)
    .system("You have filesystem access");

for tool in tools {
    builder = builder.tool_arc(tool.clone());
}

let agent = builder.build()?;
```

## Supported MCP Servers

Any server implementing the MCP specification works. Popular examples:

| Server | Transport | Tools |
|--------|:---------:|-------|
| `@modelcontextprotocol/server-filesystem` | stdio | read/write/list files |
| `@modelcontextprotocol/server-github` | stdio | GitHub API operations |
| `@modelcontextprotocol/server-postgres` | stdio | PostgreSQL queries |
| Custom servers | stdio/SSE | Any tools you define |

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE) or [MIT License](../../LICENSE-MIT) at your option.
