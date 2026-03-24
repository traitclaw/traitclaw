# Story 1.13: Tool Output Processing

Status: review

## Story

As a developer,
I want pluggable tool output processing (truncation, transformation, filtering),
So that I can prevent context overflow and customize tool output handling.

## Acceptance Criteria

1. **Given** an `OutputProcessor` trait is defined (sync) **When** no processor is configured **Then** `TruncateProcessor` is used by default (max 10,000 chars)
2. **And** output exceeding the limit is truncated with `"[output truncated]"` suffix
3. **And** `NoopProcessor` is available for users who want raw output
4. **And** `ChainProcessor` allows composing multiple processors in a pipeline
5. **And** `AgentBuilder::output_processor(impl OutputProcessor)` allows custom processors
6. **And** runtime applies processor after each tool execution, before adding result to messages

## Tasks / Subtasks

- [x] Task 1: `OutputProcessor` trait + `NoopProcessor` (AC: 1, 3)
- [x] Task 2: `TruncateProcessor` (AC: 1, 2) — default 10k, truncates with suffix
- [x] Task 3: `ChainProcessor` (AC: 4) — composes processors in order
- [x] Task 4: Integration (AC: 5, 6)
  - [x] `AgentBuilder::output_processor()` setter
  - [x] Default `TruncateProcessor`
  - [x] Runtime applies after tool execution
- [x] Task 5: Tests (AC: all)
  - [x] Truncation at boundary
  - [x] Chain applies in order
  - [x] Noop returns unchanged

## Dev Notes

### Architecture Requirements
- OutputProcessor is SYNC — must be fast, no I/O
- Applied after tool execution, before adding result to messages
- ChainProcessor is the composition mechanism

### References
- [Source: _bmad-output/epics.md#Story 1.13]

## Dev Agent Record

### Agent Model Used
gemini-2.5-pro (antigravity)

### Debug Log References
- `cargo test` (workspace) → all passed
- `cargo clippy --all-targets` → clean

### Completion Notes List
- Created `OutputProcessor` trait + 3 implementations from scratch.
- Integrated into Agent struct, Builder, and Runtime (applied after tool execution).

### File List
- `crates/traitclaw-core/src/traits/output_processor.rs` (NEW)
- `crates/traitclaw-core/src/traits.rs` (module registration)
- `crates/traitclaw-core/src/lib.rs` (exports + prelude)
- `crates/traitclaw-core/src/agent.rs` (output_processor field)
- `crates/traitclaw-core/src/agent_builder.rs` (setter + default)
- `crates/traitclaw-core/src/runtime.rs` (applies processor)

### Change Log
- 2026-03-24: All tasks complete.
