# Story 6.4: Steering Example

Status: ready-for-dev

## Story

As a developer,
I want a steering example showing Guards and Hints,
So that I can learn how to protect and guide my agent.

## Acceptance Criteria

1. **Given** `examples/04-steering/` exists **When** I run the example **Then** it demonstrates `ShellDenyGuard` blocking dangerous commands
2. **And** demonstrates `BudgetHint` warning at threshold
3. **And** shows both manual guard/hint setup and `Steering::auto()`

## Tasks / Subtasks

- [ ] Task 1: Create `examples/04-steering/` with Cargo.toml and main.rs
- [ ] Task 2: Demonstrate manual Guard/Hint setup
- [ ] Task 3: Demonstrate `Steering::auto()` one-liner
- [ ] Task 4: Write README
- [ ] Task 5: Verify compilation

## Dev Notes

### References
- [Source: _bmad-output/architecture.md#7 Developer Experience - Level 4, 5]
- [Source: _bmad-output/architecture.md#8 Examples - 06-steering]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log
