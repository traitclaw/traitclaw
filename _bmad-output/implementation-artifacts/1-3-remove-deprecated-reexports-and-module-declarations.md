# Story 1.3: Remove Deprecated Re-exports and Module Declarations

Status: ready-for-dev

## Story

As a framework maintainer,
I want to remove all deprecated type re-exports from `lib.rs` and the prelude,
so that users importing from `traitclaw_core` see only current types.

## Acceptance Criteria

1. All deprecated re-exports removed from `crates/traitclaw-core/src/lib.rs` (lines ~51-63)
2. `pub mod context_strategy` and `pub mod output_processor` removed from the traits module declaration
3. Deprecated types removed from prelude: `ContextStrategy`, `NoopContextStrategy`, `SlidingWindowStrategy`, `OutputProcessor`, `TruncateProcessor`
4. `traitclaw_core::prelude::*` no longer includes deprecated types
5. `cargo check --workspace` compiles successfully

## Tasks / Subtasks

- [ ] Task 1: Remove deprecated re-exports from `lib.rs` (AC: #1, #2)
  - [ ] Remove `#[allow(deprecated)]` block (line ~51) with `ContextStrategy` re-exports
  - [ ] Remove `#[allow(deprecated)]` block (line ~60) with `OutputProcessor` re-exports
  - [ ] Remove `pub mod context_strategy` from traits module section
  - [ ] Remove `pub mod output_processor` from traits module section
- [ ] Task 2: Clean prelude of deprecated items (AC: #3, #4)
  - [ ] Remove `#[allow(deprecated)]` prelude block (line ~119) with `ContextStrategy` items
  - [ ] Remove `#[allow(deprecated)]` prelude block (line ~129) with `OutputProcessor` items
- [ ] Task 3: Verify compilation (AC: #5)
  - [ ] Run `cargo check --workspace`
  - [ ] Run `cargo test -p traitclaw-core`

## Dev Notes

### Exact Locations in `lib.rs`

```
Line ~51: #[allow(deprecated)] pub use traits::context_strategy::{ ContextStrategy, NoopContextStrategy, SlidingWindowStrategy };
Line ~60: #[allow(deprecated)] pub use traits::output_processor::{ ChainProcessor, NoopProcessor, OutputProcessor, TruncateProcessor };
Line ~119: #[allow(deprecated)] prelude ContextStrategy items
Line ~129: #[allow(deprecated)] prelude OutputProcessor items
```

### CRITICAL: Source files NOT deleted yet
The actual `.rs` files are left on disk — they're deleted in Story 1.4.
In this story we only remove the `pub mod` declarations and re-exports.
The module files will become dead code after this story.

### Dependency: Story 1.2 must be complete first
Story 1.2 removes AgentRuntime fields that import these types. Without Story 1.2, removing the module declarations would cause compile errors in strategy.rs.

### References
- [Source: architecture-v0.9.0.md#Decision 4] — Prelude composition
- [Source: epics-v0.9.0.md#Story 1.3]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log
