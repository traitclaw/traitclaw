# Story 13.1: GroupedRegistry Implementation

Status: done

## Story

As a developer with 30+ tools,
I want to organize them into named groups and activate/deactivate groups at runtime,
so that only relevant tool schemas are sent to the LLM, saving tokens.

## Acceptance Criteria

1. `GroupedRegistry` is implemented in `traitclaw-core` and implements the `ToolRegistry` trait
2. `.group("name", tools)` creates a named group during construction
3. `.activate("name")` and `.deactivate("name")` switch groups on/off at runtime
4. Multiple groups can be active simultaneously
5. `get_tools()` returns only tools from active groups
6. `find_tool(name)` searches across ALL groups (active or not)
7. Interior mutability via `RwLock` enables runtime group switching with `&self`
8. Unit test: activate group A → deactivate group A → activate group B verifies correct schemas
9. Unit test: concurrent read access during group switch is safe
10. Re-exported via `traitclaw::prelude::*`

## Tasks / Subtasks

- [x] Task 1: Add `GroupedRegistry` struct to `crates/traitclaw-core/src/registries.rs` (AC: #1, #7)
- [x] Task 2: Implement builder API (AC: #2, #3, #4)
- [x] Task 3: Implement `ToolRegistry` trait (AC: #5, #6)
- [x] Task 4: Unit tests — 12 tests (AC: #8, #9)
- [x] Task 5: Re-exports and integration (AC: #10)

## Dev Notes

- File: `crates/traitclaw-core/src/registries.rs` (existing file with `DynamicRegistry`)
- Follow `DynamicRegistry` patterns for `RwLock` usage
- Use `HashMap<String, Vec<Arc<dyn ErasedTool>>>` for group storage
- `find_tool` MUST search all groups to support tool execution even when group is deactivated
- Group names are case-sensitive strings

### References

- [Source: epics-v0.4.0.md#Epic 13, Story 13.1]
- [Source: registries.rs] — DynamicRegistry pattern reference
- [Source: tool_registry.rs] — ToolRegistry trait definition

## Dev Agent Record

### Agent Model Used
Gemini 2.5 Pro

### Completion Notes List
- All 12 unit tests pass (including concurrent read test)
- All 23 doc-tests pass
- `GroupedRegistry` re-exported via `traitclaw_core::prelude::*` and `traitclaw::*`

### File List
- `crates/traitclaw-core/src/registries.rs` — GroupedRegistry impl + tests
- `crates/traitclaw-core/src/lib.rs` — re-exports

### Change Log
- 2026-03-25: Initial implementation complete
