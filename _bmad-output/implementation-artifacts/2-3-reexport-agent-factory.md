# Story 2.3: Re-export and Document `AgentFactory`

Status: ready-for-dev

## Story

As a **Rust developer using the `traitclaw` meta-crate**,
I want `AgentFactory` available via `traitclaw::prelude::*` with full rustdoc,
so that I can discover the factory pattern and understand the `Provider: Clone` requirement.

## Acceptance Criteria

1. Given `use traitclaw::prelude::*;` is in scope, when `AgentFactory::new(provider)` is called, then it compiles and works correctly.

2. Given `AgentFactory`, `spawn()`, and `spawn_with()` methods, when viewed in `cargo doc`, then each has rustdoc with at least one `# Example` block.

3. Given the `AgentFactory` documentation, when a developer reads it, then it clearly explains the `Provider: Clone` bound and notes that `Arc<dyn Provider>` satisfies it.

4. Given a unit test with a mock `Clone + Provider`, when `AgentFactory::new()` and `spawn()` are tested, then the test verifies correct provider cloning and system prompt assignment.

## Tasks / Subtasks

- [ ] Task 1: Add `AgentFactory` to prelude (AC: #1)
  - [ ] Add `pub use traitclaw_core::AgentFactory;` to `traitclaw` prelude
- [ ] Task 2: Add comprehensive rustdoc (AC: #2, #3)
  - [ ] Document `AgentFactory` struct with module-level docs
  - [ ] Document `new()`, `spawn()`, `spawn_with()` with examples
  - [ ] Include note about `Arc<dyn Provider>` satisfying Clone bound
- [ ] Task 3: Verify with `cargo doc --no-deps` (AC: #2)

## Dev Notes

- **Crate:** `traitclaw` meta-crate (prelude) + `traitclaw-core` (rustdoc)
- **Depends on:** Stories 2-1 and 2-2 must be complete.
- **NFR10:** All public types/methods need rustdoc with examples.

### Project Structure Notes

- Modified: `crates/traitclaw/src/lib.rs` (prelude)
- Modified: `crates/traitclaw-core/src/factory.rs` (rustdoc)

### References

- [Source: _bmad-output/planning-artifacts/epics-v0.6.0.md#Story 2.3]
- [Source: _bmad-output/planning-artifacts/prd-v0.6.0.md#NFR10, FR15]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
