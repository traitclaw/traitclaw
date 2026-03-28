# Story 1.1: Remove Blanket Implementations

Status: review

## Story

As a framework maintainer,
I want to remove the bridge blanket impls that convert deprecated traits into new traits,
so that the codebase has no hidden compatibility layers.

## Acceptance Criteria

1. The blanket impl `impl<T: ContextStrategy> ContextManager for T` is removed from `context_manager.rs`
2. The blanket impl `impl<T: OutputProcessor> OutputTransformer for T` is removed from `output_transformer.rs`
3. The `#[allow(deprecated)]` import of `ContextStrategy` is removed from `context_manager.rs`
4. The `#[allow(deprecated)]` import of `OutputProcessor` is removed from `output_transformer.rs`
5. Tests for blanket impls are removed (`test_blanket_impl_delegates_to_context_strategy`, `test_blanket_impl_delegates_to_output_processor`)
6. Module-level doc comments referencing deprecated traits are updated
7. `cargo check --workspace` compiles successfully

## Tasks / Subtasks

- [x] Task 1: Remove blanket impl from `context_manager.rs` (AC: #1, #3, #6)
  - [x] Remove `#[allow(deprecated)] use crate::traits::context_strategy::ContextStrategy;` import (line ~38)
  - [x] Remove blanket impl block `impl<T: ContextStrategy + 'static> ContextManager for T` (lines ~83-95)
  - [x] Remove test `test_blanket_impl_delegates_to_context_strategy` (in #[cfg(test)] module)
  - [x] Update module-level doc comment (lines 1-8) — remove references to `ContextStrategy` and blanket impl
- [x] Task 2: Remove blanket impl from `output_transformer.rs` (AC: #2, #4, #6)
  - [x] Remove `#[allow(deprecated)] use crate::traits::output_processor::OutputProcessor;` import (line ~46-47)
  - [x] Remove blanket impl block `impl<T: OutputProcessor + 'static> OutputTransformer for T` (lines ~82-89)
  - [x] Remove test `test_blanket_impl_delegates_to_output_processor` (in #[cfg(test)] module)
  - [x] Update module-level doc comment (lines 1-8) — remove references to `OutputProcessor` and blanket impl
- [x] Task 3: Verify compilation (AC: #7)
  - [x] Run `cargo check --workspace` — PASSED
  - [x] Run `cargo test -p traitclaw-core` — 32 passed, 0 failed

## Dev Notes

### Architecture Decision Reference
[Source: architecture-v0.9.0.md#Decision 3: Blanket Impl Removal]
- Delete both blanket impls entirely
- These only existed as migration bridge from v0.3.0 → v0.9.0
- Without deprecated traits, nothing bridges

### Exact File Locations

**context_manager.rs** (`crates/traitclaw-core/src/traits/context_manager.rs`):
- Line 38: `#[allow(deprecated)]` import
- Lines 83-95: Blanket impl block
- Test: `test_blanket_impl_delegates_to_context_strategy` in `#[cfg(test)]` module (~line 125)

**output_transformer.rs** (`crates/traitclaw-core/src/traits/output_transformer.rs`):
- Lines 46-47: `#[allow(deprecated)]` import
- Lines 82-89: Blanket impl block  
- Test: `test_blanket_impl_delegates_to_output_processor` in `#[cfg(test)]` module (~line 118-139)

### CRITICAL: Do NOT delete the source files yet
The deprecated trait source files (`context_strategy.rs`, `output_processor.rs`) are NOT deleted in this story. They are deleted in Story 1.4. Only remove the blanket impls and their imports in the NEW trait files.

### Testing Standards
- Run `cargo test -p traitclaw-core` after each task
- Remaining tests in `context_manager.rs` and `output_transformer.rs` must still pass
- Object safety tests should be untouched

### References
- [Source: architecture-v0.9.0.md#Decision 1] — Blanket impls go first in deletion order
- [Source: architecture-v0.9.0.md#Decision 3] — Full blanket impl removal spec
- [Source: epics-v0.9.0.md#Story 1.1]

## Dev Agent Record

### Agent Model Used
Gemini (Antigravity)

### Debug Log References
- Compilation error after blanket impl removal: `agent_builder.rs` used `SlidingWindowStrategy` and `TruncateProcessor` as defaults for `context_manager` and `output_transformer` fields via blanket impl bridge. Fixed by replacing with `RuleBasedCompressor::default()` and `BudgetAwareTruncator::default()`.

### Completion Notes List
- ✅ Removed blanket impl `impl<T: ContextStrategy> ContextManager for T` from context_manager.rs
- ✅ Removed blanket impl `impl<T: OutputProcessor> OutputTransformer for T` from output_transformer.rs
- ✅ Removed 2 deprecated imports with `#[allow(deprecated)]`
- ✅ Removed 2 blanket impl tests
- ✅ Updated module-level docs in both files (removed references to deprecated traits)
- ✅ Updated trait-level docs (removed "Migration from" sections)
- ✅ Fixed agent_builder.rs defaults: SlidingWindowStrategy → RuleBasedCompressor, TruncateProcessor → BudgetAwareTruncator
- ✅ cargo check --workspace: PASSED (all examples compile)
- ✅ cargo test -p traitclaw-core: 32 passed, 0 failed

### File List
- `crates/traitclaw-core/src/traits/context_manager.rs` — removed blanket impl, deprecated import, test, updated docs
- `crates/traitclaw-core/src/traits/output_transformer.rs` — removed blanket impl, deprecated import, test, updated docs
- `crates/traitclaw-core/src/agent_builder.rs` — replaced bridged defaults with new-style defaults

### Change Log
- 2026-03-28: Story 1.1 completed — removed both blanket implementations and updated builder defaults
