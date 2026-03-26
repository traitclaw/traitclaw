# Story 1.2: Re-export and Document `Agent::with_system()`

Status: review

## Story

As a **Rust developer using the `traitclaw` meta-crate**,
I want `Agent::with_system()` to be available via `traitclaw::prelude::*` with full rustdoc,
so that I can discover and use the shorthand without additional imports.

## Acceptance Criteria

1. Given `use traitclaw::prelude::*;` is in scope, when `Agent::with_system(provider, "prompt")` is called, then it compiles and works correctly without additional imports.

2. Given the `Agent::with_system()` method, when viewed in `cargo doc`, then it has rustdoc with at least one `# Example` block showing usage.

3. Given the method documentation, when a developer reads it, then it clearly explains the equivalence to `Agent::builder().provider(p).system(s).build()?`.

## Tasks / Subtasks

- [x] Task 1: Verify prelude re-export (AC: #1)
  - [x] Confirm `Agent` is already in `traitclaw::prelude` (confirmed: line 107 of lib.rs)
  - [x] Doc-test for `with_system()` compiles via `use traitclaw_core::prelude::*`
- [x] Task 2: Add rustdoc to `with_system()` (AC: #2, #3)
  - [x] Doc comment with description, equivalence note, and `# Example` block (added in Story 1-1)
  - [x] Doc-test compiles successfully

## Dev Notes

- **No code changes needed**: `Agent` is already re-exported via prelude. Since `with_system()` is an inherent method on `Agent`, it's automatically available.
- **Rustdoc already added in Story 1-1** — includes `# Example`, `# Panics`, and equivalence note.

### References

- [Source: _bmad-output/planning-artifacts/epics-v0.6.0.md#Story 1.2]
- [Source: _bmad-output/planning-artifacts/prd-v0.6.0.md#NFR10]

## Dev Agent Record

### Agent Model Used

Gemini 2.5 Pro (Antigravity)

### Completion Notes List

- ✅ Verified `Agent` is already in `traitclaw_core::prelude` (lib.rs line 107)
- ✅ Rustdoc with `# Example` block was added in Story 1-1 implementation
- ✅ Doc-test for `with_system()` compiles successfully
- ✅ No additional code changes needed — purely verification story

### File List

- No files modified (all work completed in Story 1-1)
