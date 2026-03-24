# baseclaw-mcp

[![crates.io](https://img.shields.io/crates/v/baseclaw-mcp.svg)](https://crates.io/crates/baseclaw-mcp)
[![docs.rs](https://docs.rs/baseclaw-mcp/badge.svg)](https://docs.rs/baseclaw-mcp)

**MCP (Model Context Protocol) client for BaseClaw — discover and call tools from MCP servers.**

Connect to any [MCP](https://modelcontextprotocol.io/)-compatible tool server over stdio or SSE transport. Discovered tools implement `ErasedTool` and work seamlessly alongside native BaseClaw tools.

## What is MCP?

The **Model Context Protocol** is an open standard for connecting AI agents to external tools and data sources. MCP servers expose tools (filesystem, databases, APIs, etc.) that AI agents can discover and invoke.

## Usage

```rust
use baseclaw::prelude::*;
use baseclaw_mcp::McpServer;

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
