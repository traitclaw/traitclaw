# Story 6.2: Tool Calling Example

Status: ready-for-dev

## Story

As a developer,
I want a tool calling example with `#[derive(Tool)]`,
So that I can learn how to add tools to an agent.

## Acceptance Criteria

1. **Given** `examples/02-tool-calling/` exists **When** I run the example **Then** it defines 2 tools (Calculator + mock WeatherLookup) using `#[derive(Tool)]`
2. **And** validates: tool schema generation, tool execution, result feedback to LLM
3. **And** README explains tool definition and registration

## Tasks / Subtasks

- [ ] Task 1: Create `examples/02-tool-calling/` with Cargo.toml and main.rs
- [ ] Task 2: Implement Calculator and WeatherLookup tools
- [ ] Task 3: Register tools with agent and demonstrate tool calling
- [ ] Task 4: Write README
- [ ] Task 5: Verify compilation

## Dev Notes

### References
- [Source: _bmad-output/architecture.md#7 Developer Experience - Level 2]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log
