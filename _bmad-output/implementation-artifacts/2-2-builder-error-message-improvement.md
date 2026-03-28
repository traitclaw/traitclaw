# Story 2.2: Builder Error Message Improvement

Status: ready-for-dev

## Story

As an agent developer,
I want clear, actionable error messages when I misconfigure the AgentBuilder,
so that I can quickly identify and fix configuration issues.

## Acceptance Criteria

1. `AgentBuilder::build()` without a provider returns: `"AgentBuilder: no provider configured. Use .provider(my_provider) before .build()"`
2. All builder validation errors follow format: `"[Component]: [what happened]. Use .[method]() to fix."`
3. Tests asserting on error message content are updated to match new format
4. All tests pass (`cargo test --workspace`)

## Tasks / Subtasks

- [ ] Task 1: Audit current builder error messages (AC: #2)
  - [ ] Read `crates/traitclaw-core/src/agent_builder.rs` fully
  - [ ] List all current error messages in `build()` validation
  - [ ] Identify which messages need improvement
- [ ] Task 2: Update error messages to standardized format (AC: #1, #2)
  - [ ] Update missing provider error message
  - [ ] Update any other validation errors to follow `"[Component]: [what happened]. Use .[method]() to fix."` format
- [ ] Task 3: Update tests (AC: #3, #4)
  - [ ] Find tests asserting on error message content
  - [ ] Update expected strings to match new format
  - [ ] Run `cargo test --workspace` to verify

## Dev Notes

### Architecture Decision Reference
[Source: architecture-v0.9.0.md#Decision 5: Error Message Format]
- Standardized format: `"[Component]: [what happened]. Use .[method]() to fix."`
- Actionable errors reduce debugging time
- Component prefix identifies where the error originates

### Error Message Pattern

```rust
// Before (v0.8.0)
Err(Error::Config("Provider is required".into()))

// After (v0.9.0)
Err(Error::Config(
    "AgentBuilder: no provider configured. Use .provider(my_provider) before .build()".into()
))
```

### CRITICAL: Don't change Error enum variants
Only change the string content inside existing error variants. The `Error::Config` enum variant stays the same.

### References
- [Source: architecture-v0.9.0.md#Decision 5]
- [Source: epics-v0.9.0.md#Story 2.2]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log
