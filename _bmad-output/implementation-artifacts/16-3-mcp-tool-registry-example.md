# Story 16.3: MCP Tool Registry Example

Status: done

## Story

As a developer,
I want `examples/20-mcp-tool-registry/` demonstrating MCP tool discovery,
so that I can learn how to connect to MCP servers.

## Acceptance Criteria

1. `examples/20-mcp-tool-registry/` is created with `Cargo.toml` and `src/main.rs`
2. Demonstrates `McpToolRegistry` connecting to an MCP server
3. Discovers tools and shows their schemas
4. Example compiles with `cargo build` (runtime requires MCP server)
5. Registered in workspace `Cargo.toml`

## Tasks / Subtasks

- [x] Task 1: Created `examples/20-mcp-tool-registry/Cargo.toml` + `main.rs` + workspace reg (AC: #1, #5)
- [x] Task 2: Single-server `McpToolRegistry` + multi-server `MultiServerMcpRegistry` demos (AC: #2, #3)
  - `MCP_SERVER_CMD` env var for configurable server; graceful fallback when unavailable
- [x] Task 3: `cargo build -p mcp-tool-registry-example` — clean compile, 0 warnings (AC: #4)

## Dev Notes

- Depends on traitclaw-mcp crate with McpToolRegistry
- Server URL from env: `MCP_SERVER_URL` or default `http://localhost:3000`
- Example should compile but gracefully fail at runtime without MCP server

### References

- [Source: traitclaw-mcp/] — MCP crate
- [Depends on: Stories 15.1, 15.2]

## Dev Agent Record

### Agent Model Used
Gemini 2.5 Flash

### Completion Notes List
- Configurable via `MCP_SERVER_CMD` env var
- Gracefully handles unreachable servers with explanatory output
- Demonstrates both `McpToolRegistry` and `MultiServerMcpRegistry`
- Shows `resolve_prefix()` for parsing `"server::tool"` names

### File List
- `examples/20-mcp-tool-registry/Cargo.toml`
- `examples/20-mcp-tool-registry/src/main.rs`
- `Cargo.toml` (workspace member added)

### Change Log
- 2026-03-26: Initial creation
