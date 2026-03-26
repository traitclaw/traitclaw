# Story 3.1: Implement `AgentPool::new()` and Accessor Methods

Status: ready-for-dev

## Story

As a **Rust developer orchestrating multiple agents**,
I want to create an `AgentPool` from a vector of agents and inspect it with `get()` and `len()`,
so that I can group agents for sequential execution.

## Acceptance Criteria

1. Given a `Vec<Agent>` of pre-built agents, when `AgentPool::new(agents)` is called, then an `AgentPool` is returned that takes ownership of the agents.

2. Given an `AgentPool` with 3 agents, when `pool.len()` is called, then it returns `3`.

3. Given an `AgentPool` with 3 agents, when `pool.get(1)` is called, then it returns `Some(&Agent)` referencing the second agent.

4. Given an `AgentPool` with 3 agents, when `pool.get(5)` is called, then it returns `None`.

## Tasks / Subtasks

- [ ] Task 1: Create `AgentPool` struct (AC: #1)
  - [ ] Define `pub struct AgentPool` in `crates/traitclaw-core/src/pool.rs`
  - [ ] `agents: Vec<Agent>` field (ADR-19: owned Vec)
  - [ ] Implement `new(agents: Vec<Agent>) -> Self`
- [ ] Task 2: Implement accessor methods (AC: #2, #3, #4)
  - [ ] `pub fn len(&self) -> usize`
  - [ ] `pub fn is_empty(&self) -> bool` (Clippy requires this with `len()`)
  - [ ] `pub fn get(&self, index: usize) -> Option<&Agent>`
- [ ] Task 3: Module setup
  - [ ] Create `crates/traitclaw-core/src/pool.rs`
  - [ ] Add `mod pool;` and `pub use pool::AgentPool;` to lib.rs
- [ ] Task 4: Unit tests

## Dev Notes

- **Crate:** `traitclaw-core` — new file `crates/traitclaw-core/src/pool.rs`
- **ADR-19:** `Vec<Agent>` (owned) — consistent with `Team::bind()` ownership model.
- **Important:** Include `is_empty()` alongside `len()` or Clippy will fail.
- **No dependencies on other v0.6.0 features** — this can be built independently.

### Project Structure Notes

- New file: `crates/traitclaw-core/src/pool.rs`
- Modified: `crates/traitclaw-core/src/lib.rs`

### References

- [Source: _bmad-output/planning-artifacts/epics-v0.6.0.md#Story 3.1]
- [Source: _bmad-output/planning-artifacts/prd-v0.6.0.md#FR5, FR8]
- [Source: _bmad-output/planning-artifacts/epics-v0.6.0.md#ADR-19]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
