# Story 3.4: Re-export and Document `AgentPool`

Status: ready-for-dev

## Story

As a **Rust developer using the `traitclaw` meta-crate**,
I want `AgentPool` available via `traitclaw::prelude::*` with full rustdoc,
so that I can discover the pool API and understand team binding.

## Acceptance Criteria

1. Given `use traitclaw::prelude::*;` is in scope, when `AgentPool::new(agents)` or `AgentPool::from_team(&team, provider)` is called, then it compiles and works correctly.

2. Given `AgentPool`, `new()`, `from_team()`, `run_sequential()`, `get()`, and `len()`, when viewed in `cargo doc`, then each has rustdoc with at least one `# Example` block.

3. Given a unit test suite, when `AgentPool::from_team()` is tested with a mock Team and mock Provider, then it verifies correct role→agent mapping and error on missing system_prompt.

## Tasks / Subtasks

- [ ] Task 1: Add `AgentPool` to prelude (AC: #1)
- [ ] Task 2: Add comprehensive rustdoc to all methods (AC: #2)
- [ ] Task 3: Run `cargo doc --no-deps` verification

## Dev Notes

- **Depends on:** Stories 3-1, 3-2, 3-3.
- **NFR10:** All public types/methods need rustdoc with examples.

### References

- [Source: _bmad-output/planning-artifacts/epics-v0.6.0.md#Story 3.4]
- [Source: _bmad-output/planning-artifacts/prd-v0.6.0.md#FR15, NFR10]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
