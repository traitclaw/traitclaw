# Story 2.3: Strategy Interchangeability

Status: ready-for-dev

## Story

As a developer,
I want to swap `ReActStrategy` for any other `AgentStrategy` implementation,
so that I can experiment with different reasoning approaches without refactoring.

## Acceptance Criteria

1. Swapping strategy constructor requires no other code changes (FR8)
2. Built-in and custom strategies coexist in same application (FR9)
3. All built-in strategies impl `AgentStrategy` without trait modifications (FR10)
4. Unit test demonstrates multiple strategies in same test

## Tasks / Subtasks

- [ ] Task 1: Verify interchangeability (AC: #1, #3)
  - [ ] Confirm `ReActStrategy` compiles as `Box<dyn AgentStrategy>`
  - [ ] Confirm swap from `DefaultStrategy` to `ReActStrategy` requires only constructor change
  - [ ] Verify `AgentStrategy` trait is unchanged from v0.2.0
- [ ] Task 2: Coexistence test (AC: #2, #4)
  - [ ] Create integration test with `ReActStrategy` and a custom `AgentStrategy`
  - [ ] Both used in same test function
  - [ ] Both assigned to agents via same builder API
- [ ] Task 3: Documentation (AC: #1)
  - [ ] Add rustdoc example on `ReActStrategy` showing swap pattern

## Dev Notes

- This is primarily a verification/testing story — the interchangeability should be inherent from implementing `AgentStrategy`
- If `AgentStrategy` trait had to be modified, this is a CRITICAL architectural violation — raise immediately
- Test pattern: create two agents with different strategies, run both, verify both produce results

### References

- [Source: prd.md#FR8-FR9-FR10]
- [Source: architecture.md#Core-Architectural-Decisions]

## Dev Agent Record

### Agent Model Used
### Debug Log References
### Completion Notes List
### File List
