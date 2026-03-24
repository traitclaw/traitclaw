# Story 7.2: Meta-crate Feature Flags

Status: ready-for-dev

## Story

As a developer,
I want `traitclaw` to feature-gate all optional crate re-exports,
So that I control exactly what gets compiled.

## Acceptance Criteria

1. **Given** `traitclaw/Cargo.toml` with feature flags **When** I add `traitclaw = { features = ["openai", "steering"] }` **Then** feature `openai-compat` (default) → re-exports `traitclaw-openai-compat`
2. **And** feature `openai` → re-exports `traitclaw-openai`
3. **And** feature `anthropic` → re-exports `traitclaw-anthropic`
4. **And** feature `steering` → re-exports `traitclaw-steering`
5. **And** feature `sqlite` → re-exports `traitclaw-memory-sqlite`
6. **And** feature `macros` (default) → re-exports `traitclaw-macros`
7. **And** default features = `["openai-compat", "macros"]`

## Tasks / Subtasks

- [ ] Task 1: Configure feature flags in traitclaw Cargo.toml
- [ ] Task 2: Add conditional re-exports in lib.rs with `#[cfg(feature = "...")]`
- [ ] Task 3: Update prelude with conditional imports
- [ ] Task 4: Write compile tests for each feature combination

## Dev Notes

### Architecture Requirements
- AD-1: Single traitclaw dependency with feature flags
- Default: openai-compat + macros — minimal working agent
- Each feature adds exactly one optional crate

### References
- [Source: _bmad-output/architecture.md#2 Single Dependency Design]
- [Source: _bmad-output/architecture.md#6 Core vs Optional]
- [Source: _bmad-output/epics.md#Story 7.2]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log
