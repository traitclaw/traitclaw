# Story 4.1: Implement `TerminationCondition` Trait and `MaxRoundsTermination`

Status: ready-for-dev

## Story

As a **Rust developer building multi-agent systems**,
I want a pluggable `TerminationCondition` trait with a default `MaxRoundsTermination`,
so that group chats terminate automatically based on configurable criteria.

## Acceptance Criteria

1. Given the `TerminationCondition` trait, when implemented, then it is defined in `traitclaw-team` crate (ADR-20) and has a method to check whether the chat should terminate based on current state.

2. Given `MaxRoundsTermination::new(max_rounds)`, when the round count reaches `max_rounds`, then the termination condition returns true.

3. Given `MaxRoundsTermination::new(6)`, when only 4 rounds have occurred, then the termination condition returns false.

## Tasks / Subtasks

- [ ] Task 1: Define `TerminationCondition` trait (AC: #1)
  - [ ] Create in `crates/traitclaw-team/src/group_chat.rs` (or new module)
  - [ ] `fn should_terminate(&self, round: usize, messages: &[Message]) -> bool`
  - [ ] Trait must be `Send + Sync`
- [ ] Task 2: Implement `MaxRoundsTermination` (AC: #2, #3)
  - [ ] `pub struct MaxRoundsTermination { max_rounds: usize }`
  - [ ] `pub fn new(max_rounds: usize) -> Self`
  - [ ] Implement `TerminationCondition`: return `round >= self.max_rounds`
- [ ] Task 3: Unit tests
  - [ ] Test MaxRoundsTermination at boundary (exact max, below, above)

## Dev Notes

- **Crate:** `traitclaw-team` (ADR-20: multi-agent coordination is a team concern)
- **Design:** Keep trait minimal — just `should_terminate()`. Users can implement custom conditions (keyword detection, quality thresholds, etc.)
- **No dependencies** on other v0.6.0 features — can be built independently.

### Project Structure Notes

- New file or section: `crates/traitclaw-team/src/group_chat.rs`
- Modified: `crates/traitclaw-team/src/lib.rs` (add module)

### References

- [Source: _bmad-output/planning-artifacts/epics-v0.6.0.md#Story 4.1]
- [Source: _bmad-output/planning-artifacts/prd-v0.6.0.md#FR12]
- [Source: _bmad-output/planning-artifacts/epics-v0.6.0.md#ADR-20]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
