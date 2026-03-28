# Story 1.2: Implement `ThoughtStep` Enum

Status: ready-for-dev

## Story

As a developer,
I want a shared `ThoughtStep` type for all strategies,
so that I can observe and log reasoning steps in a structured format.

## Acceptance Criteria

1. `ThoughtStep` enum in `src/common/thought_step.rs` with variants: `Think { content: String }`, `Act { tool_name: String, tool_input: serde_json::Value }`, `Observe { tool_output: String }`, `Answer { content: String }`
2. Derives `Debug, Clone, Serialize`
3. Re-exported from `traitclaw_strategies::common::ThoughtStep`
4. Unit tests verify serialization for each variant
5. Re-exported from `traitclaw_strategies::ThoughtStep` (top-level convenience)

## Tasks / Subtasks

- [ ] Task 1: Create ThoughtStep enum (AC: #1, #2)
  - [ ] Create `src/common/thought_step.rs`
  - [ ] Define enum with 4 variants
  - [ ] Add derives: `Debug, Clone, Serialize`
- [ ] Task 2: Re-exports (AC: #3, #5)
  - [ ] Re-export from `common/mod.rs`
  - [ ] Re-export from `lib.rs` top-level
- [ ] Task 3: Unit tests (AC: #4)
  - [ ] Test `ThoughtStep::Think` serialization
  - [ ] Test `ThoughtStep::Act` serialization with JSON value
  - [ ] Test `ThoughtStep::Observe` serialization
  - [ ] Test `ThoughtStep::Answer` serialization
  - [ ] Test `Clone` behavior
  - [ ] Test `Debug` output

## Dev Notes

- Architecture Decision 2: Enum (not trait) for exhaustive matching and zero allocation overhead
- `serde_json::Value` for `tool_input` — flexible JSON structure
- Consider adding `#[serde(tag = "type")]` for clean JSON serialization
- No `Deserialize` needed initially — ThoughtStep is output-only

### Project Structure Notes

- File: `crates/traitclaw-strategies/src/common/thought_step.rs`
- Tests: inline `#[cfg(test)] mod tests` in same file (co-located per conventions)

### References

- [Source: architecture.md#Decision-2-ThoughtStep-Type-Design]
- [Source: architecture.md#Testing-Patterns]

## Dev Agent Record

### Agent Model Used
### Debug Log References
### Completion Notes List
### File List
