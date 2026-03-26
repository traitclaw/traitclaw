# Story 3.2: Implement `AgentPool::run_sequential()`

Status: ready-for-dev

## Story

As a **Rust developer building a content pipeline**,
I want to run agents sequentially where each agent receives the previous agent's output,
so that I can chain agent tasks (e.g., research → write → review).

## Acceptance Criteria

1. Given an `AgentPool` with agents [A, B, C] and an input string, when `pool.run_sequential("initial input").await` is called, then agent A runs with "initial input", agent B runs with A's output, agent C runs with B's output, and the final `AgentOutput` from agent C is returned.

2. Given any agent in the pool returns an error during execution, when `run_sequential()` is running, then execution stops immediately and the error is propagated as `Err`.

3. Given an `AgentPool` with 1 agent, when `run_sequential()` is called, then it runs the single agent and returns its output (edge case).

## Tasks / Subtasks

- [ ] Task 1: Implement `run_sequential()` (AC: #1, #2, #3)
  - [ ] `pub async fn run_sequential(&mut self, input: &str) -> Result<AgentOutput>`
  - [ ] Loop through agents, pass each output as next input
  - [ ] Extract text from `AgentOutput` to use as next agent's input string
  - [ ] Use `?` for error propagation (early return on first error)
- [ ] Task 2: Edge case handling (AC: #3)
  - [ ] Handle single-agent pool correctly
  - [ ] Handle empty pool (return error or empty output — decide)
- [ ] Task 3: Integration tests
  - [ ] Test 3-agent pipeline with mock providers
  - [ ] Test error propagation (middle agent fails)
  - [ ] Test single-agent edge case

## Dev Notes

- **Crate:** `traitclaw-core` → `crates/traitclaw-core/src/pool.rs`
- **Key design:** `&mut self` because `agent.run()` may mutate agent state (memory).
- **Output chaining:** Need to extract text from `AgentOutput` — check `AgentOutput::text()` or equivalent method.
- **Error handling:** Use `thiserror` typed errors, `?` operator per project conventions.
- **Depends on:** Story 3-1 (`AgentPool` struct must exist).
- **No async at construction** — only `run_sequential()` is async (NFR2).

### Project Structure Notes

- Modified: `crates/traitclaw-core/src/pool.rs` (add method)
- Tests: integration test with mock providers

### References

- [Source: _bmad-output/planning-artifacts/epics-v0.6.0.md#Story 3.2]
- [Source: _bmad-output/planning-artifacts/prd-v0.6.0.md#FR7]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
