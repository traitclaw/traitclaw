# Story 16.2: Grouped Registry Example

Status: done

## Story

As a developer,
I want `examples/19-grouped-registry/` demonstrating tool group management,
so that I can learn how to organize and switch tool groups.

## Acceptance Criteria

1. `examples/19-grouped-registry/` is created with `Cargo.toml` and `src/main.rs`
2. Demonstrates `GroupedRegistry` with 3 groups, activating/switching between them
3. Console output shows which tool schemas are active per group
4. Example compiles and runs successfully
5. Registered in workspace `Cargo.toml`

## Tasks / Subtasks

- [x] Task 1: Created `examples/19-grouped-registry/Cargo.toml` + `main.rs` + workspace registration (AC: #1, #5)
- [x] Task 2: 3 groups (search×2, code×3, data×3), `activate_group`/`deactivate_group` runtime switching (AC: #2, #3)
- [x] Task 3: `cargo build -p grouped-registry-example` — clean compile, 0 warnings (AC: #4)

## Dev Notes

- Follow patterns from `examples/17-dynamic-tools/`
- Use dummy tools (no real API calls needed)

### References

- [Source: examples/17-dynamic-tools/] — pattern reference

## Dev Agent Record

### Agent Model Used
Gemini 2.5 Flash

### Completion Notes List
- 8 dummy tools across 3 groups, no real API calls
- Builder API for initial setup; runtime API for switching
- `ErasedTool` implemented directly for zero extra dependencies
- Demonstrates `activate_group`, `deactivate_group`, `is_group_active`, `find_tool`

### File List
- `examples/19-grouped-registry/Cargo.toml`
- `examples/19-grouped-registry/src/main.rs`
- `Cargo.toml` (workspace member added)

### Change Log
- 2026-03-26: Initial creation
