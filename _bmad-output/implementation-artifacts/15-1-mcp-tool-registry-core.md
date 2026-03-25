# Story 15.1: McpToolRegistry Core Implementation

Status: done

## Story

As a developer using MCP-compatible tool servers,
I want to auto-discover and register tools from MCP servers,
so that I can use any MCP tool without manual schema definition.

## Acceptance Criteria

1. `McpToolRegistry` is implemented in `traitclaw-mcp` and implements the `ToolRegistry` trait
2. `McpToolRegistry::connect(url).await` connects to an MCP server
3. Tools are discovered via MCP `tools/list` method
4. MCP tool schemas are mapped to TraitClaw `ToolSchema` format
5. `find_tool(name)` returns an MCP-backed tool that routes execution through MCP protocol
6. `get_tools()` returns all discovered tool schemas
7. Integration test with mock MCP server: discover 5 tools, execute one, verify result

## Tasks / Subtasks

- [x] Task 1: `McpToolRegistry` struct wrapping `McpServer` (AC: #1, #2)
- [x] Task 2: `stdio()` constructor — connect + discover via McpServer (AC: #3, #4)
- [x] Task 3: `ToolRegistry` impl — `get_tools()`, `find_tool()`, `len()`, `is_empty()` (AC: #5, #6)
- [x] Task 4: 7 unit tests with FakeRegistry/FakeTool pattern (AC: #7)
- [x] Task 5: All 10 tests pass; doc-tests pass

## Dev Notes

- Crate: `crates/traitclaw-mcp/` (existing crate)
- Check existing MCP client code for connection patterns
- MCP protocol: JSON-RPC 2.0 over stdio or SSE transport
- `McpTool` is an adapter struct that wraps MCP tool info and implements `ErasedTool`
- This story focuses on single-server support; multi-server is Story 15.2

### References

- [Source: epics-v0.4.0.md#Epic 15, Story 15.1]
- [MCP Specification] — https://modelcontextprotocol.io/docs
- [Source: traitclaw-mcp/] — existing MCP crate code

## Dev Agent Record

### Agent Model Used
Gemini 2.5 Flash

### Completion Notes List
- 10/10 unit tests pass, 3 doc-tests pass
- `McpToolRegistry::stdio()` wraps existing `McpServer::stdio()`
- Tests use FakeRegistry/FakeTool to avoid needing a live MCP process
- `McpToolRegistry` re-exported in lib.rs and prelude

### File List
- `crates/traitclaw-mcp/src/registry.rs` — new module
- `crates/traitclaw-mcp/src/lib.rs` — module declaration + re-export

### Change Log
- 2026-03-26: Initial implementation complete
