# 10 — MCP Client

Connect to Model Context Protocol servers for external tools.

## What it does

1. Connects to an **MCP filesystem server** via stdio transport
2. Discovers available tools automatically
3. Registers MCP tools with a TraitClaw agent (they implement `ErasedTool`)

## Prerequisites

```bash
# Install Node.js / npx (required for MCP servers)
# The server is auto-installed on first run via npx -y
```

## Key APIs

```rust
let server = McpServer::stdio("npx", &["-y", "@mcp/server-filesystem", "/tmp"]).await?;
let tools = server.tools();  // Vec<Arc<dyn ErasedTool>>

// Register with agent — MCP tools work just like native tools
let agent = Agent::builder()
    .provider(provider)
    .erased_tool(tool)
    .build()?;
```

## Running

```bash
export OPENAI_API_KEY="sk-..."
cargo run
```
