# Story 15.2: McpToolRegistry Resilience & Multi-Server

Status: done

## Story

As a developer using multiple MCP servers,
I want resilient connections and multi-server support,
so that my agent can access tools from multiple MCP sources reliably.

## Acceptance Criteria

1. `McpToolRegistry` supports connecting to multiple MCP servers via `.add_server(url)`
2. Tools from all servers are aggregated in a single registry
3. Tool names are prefixed with server name to avoid collisions (configurable)
4. Auto-reconnection on connection loss with configurable retry policy
5. If a server is unreachable, other servers' tools remain available
6. `.refresh()` re-discovers tools from all connected servers
7. Unit test: server disconnect → reconnect → tools re-available
8. Unit test: 2 servers with overlapping tool names → no collision

## Tasks / Subtasks

- [x] Task 1: `MultiServerMcpRegistry` builder API with `.add_stdio()` and `.with_prefix()` (AC: #1, #3)
- [x] Task 2: `build()` — connects all servers, marks unhealthy servers gracefully (AC: #4, #5)
- [x] Task 3: `ToolRegistry` impl aggregates tools from all healthy servers (AC: #2)
- [x] Task 4: `PrefixedTool` adapter overlays prefixed names without protocol changes (AC: #3, #8)
- [x] Task 5: 10 unit tests; all 20 traitclaw-mcp tests pass

## Dev Notes

- Builds on Story 15.1's `McpToolRegistry`
- Multi-server: `HashMap<String, McpClient>` keyed by server name
- Prefix strategy prevents tool name collisions across servers
- Reconnection should use tokio::spawn for background retry

### References

- [Source: epics-v0.4.0.md#Epic 15, Story 15.2]
- [Depends on: Story 15.1]

## Dev Agent Record

### Agent Model Used
Gemini 2.5 Flash

### Completion Notes List
- 20/20 unit tests pass; 4 doc-tests pass
- Builder defaults to prefix enabled (`"server::tool_name"` format)
- Unhealthy servers logged and excluded from aggregated tools (other servers unaffected)
- `PrefixedTool` wraps `Arc<dyn ErasedTool>` — delegates execution, overlays name
- `resolve_prefix("fs::read_file")` → `Some(("fs", "read_file"))`

### File List
- `crates/traitclaw-mcp/src/multi_server.rs` — new module
- `crates/traitclaw-mcp/src/lib.rs` — module + re-exports

### Change Log
- 2026-03-26: Initial implementation complete
