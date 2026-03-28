# Story 2.1: `ReActStrategy` Builder & Core Structure

Status: ready-for-dev

## Story

As a developer,
I want to instantiate a `ReActStrategy` with configurable options,
so that I can assign it to an agent via the builder pattern.

## Acceptance Criteria

1. `ReActStrategy::builder().max_iterations(10).build()?` returns valid instance
2. Default `max_iterations` is 10 if not specified
3. Builder validates inputs (e.g., `max_iterations > 0`)
4. `ReActStrategy` implements `AgentStrategy` trait
5. `ReActStrategy` is object-safe (`Box<dyn AgentStrategy>`)
6. Feature-gated behind `react` flag

## Tasks / Subtasks

- [ ] Task 1: Create ReAct module structure (AC: #6)
  - [ ] Create `src/react/mod.rs` with public re-exports
  - [ ] Create `src/react/strategy.rs` with `ReActStrategy` struct
- [ ] Task 2: Implement builder (AC: #1, #2, #3)
  - [ ] `ReActStrategyBuilder` with `max_iterations`, `system_prompt` fields
  - [ ] `.builder()` → `ReActStrategyBuilder`
  - [ ] `.max_iterations(n)` → `&mut Self`
  - [ ] `.system_prompt(s)` → `&mut Self`
  - [ ] `.build()` → `Result<ReActStrategy, TraitClawError>`
  - [ ] Validate `max_iterations > 0` in `.build()`
- [ ] Task 3: Implement AgentStrategy trait (AC: #4, #5)
  - [ ] `impl AgentStrategy for ReActStrategy`
  - [ ] Stub `execute` method (full loop in Story 2.2)
  - [ ] Verify object safety: `let _: Box<dyn AgentStrategy> = Box::new(strategy);`
- [ ] Task 4: Unit tests (AC: #1, #2, #3, #5)
  - [ ] Test builder with defaults
  - [ ] Test builder with custom max_iterations
  - [ ] Test builder validation rejects `max_iterations = 0`
  - [ ] Test object safety

## Dev Notes

- Builder pattern per AR6: `.builder().config().build()? → Result<Strategy, TraitClawError>`
- Use `TraitClawError::Strategy(String)` for validation errors (AR11, no new error types)
- `AgentStrategy` trait from `traitclaw-core` — check exact method signatures before implementing
- Look at existing `DefaultStrategy` in `traitclaw-core` for reference implementation

### Project Structure Notes

- Files: `crates/traitclaw-strategies/src/react/mod.rs`, `src/react/strategy.rs`
- Test: co-located `#[cfg(test)] mod tests` in `strategy.rs`

### References

- [Source: architecture.md#Decision-1-Strategy-Module-Structure]
- [Source: architecture.md#Builder-Pattern]
- [Source: architecture.md#Error-Handling-Patterns]

## Dev Agent Record

### Agent Model Used
### Debug Log References
### Completion Notes List
### File List
