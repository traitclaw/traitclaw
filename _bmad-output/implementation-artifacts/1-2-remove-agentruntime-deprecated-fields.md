# Story 1.2: Remove AgentRuntime Deprecated Fields

Status: ready-for-dev

## Story

As a framework maintainer,
I want to remove the deprecated `context_strategy` and `output_processor` fields from `AgentRuntime`,
so that custom strategy authors see only current, relevant fields.

## Acceptance Criteria

1. `AgentRuntime` struct in `strategy.rs` has 12 pub fields (down from 14)
2. `context_strategy: Arc<dyn ContextStrategy>` field is removed
3. `output_processor: Arc<dyn OutputProcessor>` field is removed
4. All `#[allow(deprecated)]` annotations in `strategy.rs` are removed
5. All `AgentRuntime` construction sites are updated (no deprecated field population)
6. `cargo check --workspace` compiles successfully

## Tasks / Subtasks

- [ ] Task 1: Remove deprecated fields from `AgentRuntime` (AC: #1, #2, #3, #4)
  - [ ] Remove `#[allow(deprecated)]` imports for `ContextStrategy` and `OutputProcessor` in `strategy.rs` (lines ~41, ~47)
  - [ ] Remove `#[allow(deprecated)]` on struct definition (line ~61)
  - [ ] Remove `context_strategy: Arc<dyn ContextStrategy>` field
  - [ ] Remove `output_processor: Arc<dyn OutputProcessor>` field
- [ ] Task 2: Update construction site in `default_strategy.rs` (AC: #5)
  - [ ] Remove `#[allow(deprecated)]` import (line ~30)
  - [ ] Remove deprecated field population in `AgentRuntime` construction (~line 242 area)
  - [ ] Remove unused deprecated trait imports
- [ ] Task 3: Update construction site in `agent.rs` (AC: #5)
  - [ ] Remove `#[allow(deprecated)]` imports (lines ~9, ~16)
  - [ ] Remove deprecated field population in `AgentRuntime` construction (lines ~126, ~158)
  - [ ] Remove unused deprecated trait imports
- [ ] Task 4: Update construction site in `agent_builder.rs` (AC: #5)
  - [ ] Remove `#[allow(deprecated)]` annotations (lines ~12, ~19, ~46, ~65)
  - [ ] Remove deprecated builder fields and methods (`.context_strategy()`, `.output_processor()`)
  - [ ] Remove unused deprecated trait imports
- [ ] Task 5: Update construction site in `test-utils/runtime.rs` (AC: #5)
  - [ ] Remove `#[allow(deprecated)]` annotations (lines ~91, ~117)
  - [ ] Remove deprecated field population in `make_runtime()` helper
- [ ] Task 6: Verify compilation (AC: #6)
  - [ ] Run `cargo check --workspace`
  - [ ] Run `cargo test --workspace`

## Dev Notes

### Architecture Decision Reference
[Source: architecture-v0.9.0.md#Decision 2: AgentRuntime Field Removal]
- Remove 2 deprecated fields, keep all other fields as `pub`
- AgentRuntime goes from 14 to 12 fields

### Exact Construction Sites (ALL must be updated)

| File | Lines | What to Remove |
|------|-------|----------------|
| `crates/traitclaw-core/src/traits/strategy.rs` | ~41,47,61 | Field definitions, deprecated imports |
| `crates/traitclaw-core/src/default_strategy.rs` | ~30,242 | Field values in constructor |
| `crates/traitclaw-core/src/agent.rs` | ~9,16,126,158 | Field values when building runtime |
| `crates/traitclaw-core/src/agent_builder.rs` | ~12,19,46,65 | Builder fields and setter methods |
| `crates/traitclaw-test-utils/src/runtime.rs` | ~91,117 | Field values in make_runtime() |

### CRITICAL: traitclaw-strategies is CLEAN
Verified via grep: `traitclaw-strategies` does NOT reference `context_strategy` or `output_processor`. No changes needed there.

### Previous Story Intelligence
Story 1.1 removed blanket impls — the deprecated traits themselves still exist as source files but are no longer used as bridge targets.

### References
- [Source: architecture-v0.9.0.md#Decision 2]
- [Source: epics-v0.9.0.md#Story 1.2]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log
