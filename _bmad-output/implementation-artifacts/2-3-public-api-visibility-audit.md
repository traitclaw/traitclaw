# Story 2.3: Public API Visibility Audit

Status: ready-for-dev

## Story

As a framework maintainer,
I want every `pub` item reviewed for appropriate visibility,
so that the API surface is clean and intentional before v1.0 freeze.

## Acceptance Criteria

1. All `pub mod` declarations in `traitclaw-core` reviewed for appropriate visibility
2. No module exposes internal implementation details unnecessarily
3. `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` builds with zero warnings
4. No broken doc links exist

## Tasks / Subtasks

- [ ] Task 1: Audit public modules in `lib.rs` (AC: #1, #2)
  - [ ] Review each `pub mod` declaration in `crates/traitclaw-core/src/lib.rs`
  - [ ] Verify `pub(crate) mod streaming` remains correctly scoped
  - [ ] Check if any internal modules are unnecessarily public
  - [ ] Downgrade visibility where appropriate (e.g., `pub(crate)` for internal-only modules)
- [ ] Task 2: Verify documentation builds cleanly (AC: #3, #4)
  - [ ] Run `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`
  - [ ] Fix any broken doc links (especially links to removed deprecated types)
  - [ ] Verify all public items have doc comments (`#![deny(missing_docs)]` is enforced)
- [ ] Task 3: Final verification (AC: #3)
  - [ ] Run `cargo test --workspace`
  - [ ] Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`

## Dev Notes

### What "Visibility Audit" Means
This is NOT about adding new public API — it's about ensuring existing `pub` items are appropriate:
- Internal helper types → should be `pub(crate)` not `pub`
- Types only used in tests → should be behind `#[cfg(test)]`
- Re-exports → should match what users actually need

### CRITICAL: Don't change trait signatures
This is a visibility-only audit. Do NOT change any method signatures, type parameters, or trait bounds.

### Doc Link Fix Pattern
After removing deprecated types, some doc comments may have broken `[`link`]` references:
```rust
// Broken (after removal)
/// See [`ContextStrategy`] for the legacy approach.

// Fixed
/// See [`ContextManager`] for pluggable context window management.
```

### References
- [Source: architecture-v0.9.0.md#Project Structure & Boundaries]
- [Source: epics-v0.9.0.md#Story 2.3]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log
