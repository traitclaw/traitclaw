# Story 13.2: AdaptiveRegistry Implementation

Status: done

## Story

As a developer deploying across different model tiers,
I want tools automatically limited based on model capabilities,
so that small models aren't overwhelmed by too many tool schemas.

## Acceptance Criteria

1. `AdaptiveRegistry` is implemented in `traitclaw-core` and implements the `ToolRegistry` trait
2. Default limits: Small=5, Medium=15, Large=unlimited
3. `.with_limits(small, medium, large)` allows custom limit configuration
4. `get_tools()` returns at most `limit` tools based on current `ModelTier`
5. Tools registered first have higher priority (first registered = first selected when limited)
6. `ModelTier` is resolved from `ModelInfo` passed at construction
7. Unit test: verify Small tier returns exactly 5 tools from a 30-tool set
8. Unit test: verify Large tier returns all tools
9. Re-exported via `traitclaw::prelude::*`

## Tasks / Subtasks

- [x] Task 1: Add `AdaptiveRegistry` struct + `TierLimits` (AC: #1)
- [x] Task 2: Builder API `.with_limits()` (AC: #2, #3, #6)
- [x] Task 3: Implement `ToolRegistry` trait (AC: #4, #5)
- [x] Task 4: Unit tests — 7 tests (AC: #7, #8)
- [x] Task 5: Re-exports (AC: #9)

## Dev Notes

- File: `crates/traitclaw-core/src/registries.rs` (same file as GroupedRegistry)
- `ModelTier` enum exists in `crates/traitclaw-core/src/types/` — check exact location
- No `RwLock` needed — limits are immutable after construction, tools are fixed
- Priority = insertion order in the Vec
- `find_tool` must search all tools (even beyond limit) to support tool execution

### References

- [Source: epics-v0.4.0.md#Epic 13, Story 13.2]
- [Source: registries.rs] — DynamicRegistry and SimpleRegistry patterns
- [Source: types/] — ModelTier/ModelInfo definitions

## Dev Agent Record

### Agent Model Used
Gemini 2.5 Pro

### Completion Notes List
- All 7 unit tests pass
- `AdaptiveRegistry` + `TierLimits` re-exported via `traitclaw_core::prelude::*` and `traitclaw::*`

### File List
- `crates/traitclaw-core/src/registries.rs` — AdaptiveRegistry impl + tests
- `crates/traitclaw-core/src/lib.rs` — re-exports

### Change Log
- 2026-03-25: Initial implementation complete
