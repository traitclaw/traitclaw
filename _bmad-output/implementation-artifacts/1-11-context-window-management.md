# Story 1.11: Context Window Management

Status: review

## Story

As a developer,
I want pluggable context window management to prevent overflow,
So that agents handle long conversations without crashing.

## Acceptance Criteria

1. **Given** a `ContextStrategy` trait is defined in `traitclaw-core/src/traits/` **When** no strategy is configured **Then** `SlidingWindowStrategy` is used by default (threshold = 0.85)
2. **And** it estimates tokens (4 chars ≈ 1 token) and removes oldest non-system messages when over threshold
3. **And** `AgentState.last_output_truncated` is set to `true` when messages are removed
4. **And** `AgentBuilder::context_strategy(impl ContextStrategy)` allows custom strategies
5. **And** `NoopContextStrategy` is available for users who want no automatic management
6. **And** runtime calls `context_strategy.prepare()` before every LLM call

## Tasks / Subtasks

- [x] Task 1: `ContextStrategy` trait + `NoopContextStrategy` (AC: 1, 5)
- [x] Task 2: `SlidingWindowStrategy` (AC: 1, 2, 3)
  - [x] Default threshold = 0.85
  - [x] Token estimation: chars / 4
  - [x] Removes oldest non-system messages
  - [x] Sets `AgentState.last_output_truncated`
- [x] Task 3: Integration (AC: 4, 6)
  - [x] `AgentBuilder::context_strategy()` setter
  - [x] Default to `SlidingWindowStrategy`
  - [x] Runtime calls `prepare()` before each LLM request
- [x] Task 4: Tests (AC: all)
  - [x] Sliding window removes old messages
  - [x] System messages preserved
  - [x] Noop does nothing
  - [x] AgentState flag set on truncation

## Dev Notes

### Architecture Requirements
- `ContextStrategy` trait is sync (not async) — must be fast
- Token estimation is approximate (4 chars ≈ 1 token) for MVP
- Context window size comes from `provider.model_info().context_window`
- System messages are NEVER removed

### References
- [Source: _bmad-output/epics.md#Story 1.11]

## Dev Agent Record

### Agent Model Used
gemini-2.5-pro (antigravity)

### Debug Log References
- `cargo test` (workspace) → all passed
- `cargo clippy --all-targets` → clean

### Completion Notes List
- Created from scratch: `traits/context_strategy.rs` with trait + 2 implementations.
- Integrated into Agent struct, AgentBuilder, and runtime loop.
- Fixed 5 clippy lints (doc_markdown, cast safety, too_many_arguments).

### File List
- `crates/traitclaw-core/src/traits/context_strategy.rs` (NEW)
- `crates/traitclaw-core/src/traits.rs` (module registration)
- `crates/traitclaw-core/src/lib.rs` (exports + prelude)
- `crates/traitclaw-core/src/agent.rs` (context_strategy field)
- `crates/traitclaw-core/src/agent_builder.rs` (setter + default)
- `crates/traitclaw-core/src/runtime.rs` (prepare() call)

### Change Log
- 2026-03-24: All tasks complete.
