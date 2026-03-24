# Story 7.4: Error Recovery & Graceful Degradation

Status: ready-for-dev

## Story

As a developer,
I want the agent to handle failures gracefully without crashing,
So that my application stays resilient.

## Acceptance Criteria

1. **Given** the agent runtime loop **When** a tool execution fails **Then** error message is returned to LLM as tool result (not agent crash)
2. **And** when a Guard panics → `catch_unwind` treats it as Allow + logs warning
3. **And** when Memory operations fail → log error + continue (don't crash agent)
4. **And** when provider times out → return friendly error message

## Tasks / Subtasks

- [ ] Task 1: Add `catch_unwind` for Guard panics (AC: 2)
  - [ ] Wrap guard.check() in catch_unwind
  - [ ] On panic: treat as Allow, log warning
- [ ] Task 2: Add Memory error resilience (AC: 3)
  - [ ] Wrap memory operations in try blocks
  - [ ] On error: log and continue
- [ ] Task 3: Add provider timeout handling (AC: 4)
  - [ ] Detect timeout errors
  - [ ] Return user-friendly error message
- [ ] Task 4: Verify tool execution errors (AC: 1)
  - [ ] Ensure tool errors → error message to LLM, not crash
- [ ] Task 5: Write tests for each resilience scenario

## Dev Notes

### Architecture Requirements
- NO panics in library code — everything is recoverable
- `catch_unwind` for Guard panics (external code safety)
- Memory failures are non-fatal — logging is essential
- Provider timeouts → friendly error, not stack trace

### References
- [Source: _bmad-output/project-context.md#Anti-Patterns to AVOID]
- [Source: _bmad-output/epics.md#Story 7.4]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log
