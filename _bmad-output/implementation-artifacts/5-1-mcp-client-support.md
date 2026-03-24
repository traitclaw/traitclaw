# Story 5.1: MCP Client Support

Status: ready-for-dev

## Story

As a developer,
I want agents to connect to MCP servers as tool sources,
So that I can use external tool providers.

## Acceptance Criteria

1. **Given** `traitclaw-mcp` crate with feature `"mcp"` **When** I use `.mcp_server("npx @modelcontextprotocol/server-filesystem")` **Then** the agent discovers tools from the MCP server
2. **And** MCP tools are available alongside native tools
3. **And** tool calls to MCP tools are routed to the MCP server

## Tasks / Subtasks

- [ ] Task 1: Create `traitclaw-mcp` crate
- [ ] Task 2: Implement MCP client protocol (JSON-RPC over stdio/SSE)
- [ ] Task 3: Implement tool discovery from MCP server
- [ ] Task 4: Route tool calls to MCP server
- [ ] Task 5: Bridge MCP tools with Agent's tool registry
- [ ] Task 6: Write tests

## Dev Notes

### Architecture Requirements
- MCP protocol: JSON-RPC 2.0 over stdio or SSE transport
- Tools discovered via `tools/list` method
- Tool calls via `tools/call` method
- Must coexist with native tools in Agent

### References
- [Source: _bmad-output/architecture.md#6 Optional - mcp]
- [Source: _bmad-output/epics.md#Story 5.1]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log
