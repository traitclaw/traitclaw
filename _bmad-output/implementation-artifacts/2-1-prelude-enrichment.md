# Story 2.1: Prelude Enrichment

Status: ready-for-dev

## Story

As an agent developer,
I want commonly-used types available in the prelude,
so that `use traitclaw::prelude::*` gives me everything I need for typical usage.

## Acceptance Criteria

1. `CompressedMemory` is accessible via `traitclaw_core::prelude::*`
2. `RetryConfig` is accessible via `traitclaw_core::prelude::*`
3. `RetryProvider` is accessible via `traitclaw_core::prelude::*`
4. `DynamicRegistry` is accessible via `traitclaw_core::prelude::*`
5. Every item in the prelude is non-deprecated and commonly used
6. `cargo doc --workspace --no-deps` shows correct prelude documentation

## Tasks / Subtasks

- [ ] Task 1: Add commonly-used types to prelude (AC: #1, #2, #3, #4)
  - [ ] Add `pub use crate::memory::compressed::CompressedMemory;` to prelude
  - [ ] Add `pub use crate::retry::{RetryConfig, RetryProvider};` to prelude
  - [ ] Add `pub use crate::registries::DynamicRegistry;` to prelude (verify correct path)
- [ ] Task 2: Verify prelude completeness (AC: #5, #6)
  - [ ] Review complete prelude contents — no deprecated items remain
  - [ ] Run `cargo doc --workspace --no-deps` to verify doc generation
  - [ ] Run `cargo test --workspace` to verify no regressions

## Dev Notes

### Architecture Decision Reference
[Source: architecture-v0.9.0.md#Decision 4: Prelude Composition]
- These 4 types are used in >60% of examples
- Prelude should be the recommended import path for common usage

### CRITICAL: Verify import paths before adding
The exact module paths may differ from what's listed. Check the actual `pub use` or `pub mod` declarations in `lib.rs` first:
- `CompressedMemory` — likely in `crate::memory::compressed` or similar
- `RetryConfig` / `RetryProvider` — likely in `crate::retry` module
- `DynamicRegistry` — likely in `crate::registries` module

Run this to find exact paths:
```bash
grep -rn "pub struct CompressedMemory" crates/traitclaw-core/
grep -rn "pub struct RetryConfig" crates/traitclaw-core/
grep -rn "pub struct RetryProvider" crates/traitclaw-core/
grep -rn "pub struct DynamicRegistry" crates/traitclaw-core/
```

### References
- [Source: architecture-v0.9.0.md#Decision 4]
- [Source: epics-v0.9.0.md#Story 2.1]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log
