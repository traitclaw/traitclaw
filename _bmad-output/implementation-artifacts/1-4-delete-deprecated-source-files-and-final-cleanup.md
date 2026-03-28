# Story 1.4: Delete Deprecated Source Files and Final Cleanup

Status: ready-for-dev

## Story

As a framework maintainer,
I want to delete the deprecated trait source files and remove all remaining `#[allow(deprecated)]` annotations,
so that zero deprecated code remains in the codebase.

## Acceptance Criteria

1. `crates/traitclaw-core/src/traits/context_strategy.rs` is deleted (188 lines)
2. `crates/traitclaw-core/src/traits/output_processor.rs` is deleted (159 lines)
3. `grep -rn "allow(deprecated)" crates/` returns zero results
4. `grep -rn "#[deprecated" crates/` returns zero results
5. All remaining tests pass (`cargo test --workspace`)
6. `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes

## Tasks / Subtasks

- [ ] Task 1: Delete deprecated source files (AC: #1, #2)
  - [ ] Delete `crates/traitclaw-core/src/traits/context_strategy.rs`
  - [ ] Delete `crates/traitclaw-core/src/traits/output_processor.rs`
- [ ] Task 2: Verify zero deprecated annotations (AC: #3, #4)
  - [ ] Run grep search for `allow(deprecated)` — expect 0 results
  - [ ] Run grep search for `#[deprecated` — expect 0 results
  - [ ] If any remain, remove them (they should have been handled in Stories 1.1-1.3)
- [ ] Task 3: Full verification (AC: #5, #6)
  - [ ] Run `cargo test --workspace`
  - [ ] Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - [ ] Run `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`

## Dev Notes

### Prerequisites
Stories 1.1, 1.2, and 1.3 MUST be complete before this story. At this point:
- Blanket impls are removed (Story 1.1)
- AgentRuntime fields are removed (Story 1.2)
- Re-exports and module declarations are removed (Story 1.3)
- The source files are now dead code — safe to delete

### Files to Delete
- `crates/traitclaw-core/src/traits/context_strategy.rs` — 188 lines
  - Contains: `ContextStrategy` trait, `NoopContextStrategy`, `SlidingWindowStrategy`, tests
- `crates/traitclaw-core/src/traits/output_processor.rs` — 159 lines
  - Contains: `OutputProcessor` trait, `NoopProcessor`, `TruncateProcessor`, `ChainProcessor`, tests

### Expected Test Count After Deletion
The 658+ tests minus blanket impl tests (removed in Story 1.1) minus deprecated trait tests (deleted here). Net: approximately 650+ tests should pass.

### References
- [Source: architecture-v0.9.0.md#Decision 1] — Source files go second-to-last in deletion order
- [Source: epics-v0.9.0.md#Story 1.4]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log
