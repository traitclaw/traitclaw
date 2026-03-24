# Story 1.12: Tool Execution Strategy

Status: review

## Story

As a developer,
I want configurable tool execution (sequential, parallel, or custom),
So that I can optimize for safety or speed depending on use case.

## Acceptance Criteria

1. **Given** an `ExecutionStrategy` trait with `#[async_trait]` is defined **When** no strategy is configured **Then** `SequentialStrategy` is used by default (backward-compatible)
2. **And** `ParallelStrategy { max_concurrency }` runs tool calls concurrently via `tokio::JoinSet`
3. **And** `AdaptiveStrategy` uses `Tracker::recommended_concurrency()` to decide
4. **And** Guard checks still run before each tool execution regardless of strategy
5. **And** `AgentBuilder::execution_strategy(impl ExecutionStrategy)` allows custom strategies
6. **And** runtime delegates to `strategy.execute_batch()` instead of inline loop

## Tasks / Subtasks

- [x] Task 1: `ExecutionStrategy` trait (AC: 1)
- [x] Task 2: `SequentialStrategy` (AC: 1) — default, executes in order
- [x] Task 3: `ParallelStrategy` (AC: 2) — bounded via Semaphore
- [x] Task 4: `AdaptiveStrategy` (AC: 3) — queries Tracker
- [x] Task 5: Integration (AC: 5, 6)
  - [x] `AgentBuilder::execution_strategy()` setter
  - [x] Runtime delegates to `strategy.execute_batch()`
- [x] Task 6: Tests (AC: all)
  - [x] Sequential executes in order
  - [x] Parallel executes concurrently
  - [x] Guard blocks propagate

## Dev Notes

### Architecture Requirements
- Guard checks must remain synchronous and run before each tool, even in parallel mode
- `PendingToolCall` struct: { id, name, arguments }
- `ToolResult` struct: { id, output }
- `tokio::JoinSet` for bounded concurrency

### References
- [Source: _bmad-output/epics.md#Story 1.12]

## Dev Agent Record

### Agent Model Used
gemini-2.5-pro (antigravity)

### Debug Log References
- `cargo test` (workspace) → all passed
- `cargo clippy --all-targets` → clean

### Completion Notes List
- Created `ExecutionStrategy` trait + 3 implementations from scratch.
- Removed old inline tool execution from runtime, replaced with strategy delegation.
- Fixed unused imports and test mock types.

### File List
- `crates/traitclaw-core/src/traits/execution_strategy.rs` (NEW)
- `crates/traitclaw-core/src/traits.rs` (module registration)
- `crates/traitclaw-core/src/lib.rs` (exports + prelude)
- `crates/traitclaw-core/src/agent.rs` (execution_strategy field)
- `crates/traitclaw-core/src/agent_builder.rs` (setter + default)
- `crates/traitclaw-core/src/runtime.rs` (strategy delegation)

### Change Log
- 2026-03-24: All tasks complete.
