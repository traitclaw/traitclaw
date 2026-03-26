# Story 2.2: Implement `AgentFactory::spawn_with()` Escape Hatch

Status: ready-for-dev

## Story

As a **Rust developer who needs custom agent configuration beyond a system prompt**,
I want to use `factory.spawn_with(|builder| builder.system("prompt").tool(my_tool))`,
so that I can customize individual agents while still sharing the factory's provider.

## Acceptance Criteria

1. Given an `AgentFactory` instance, when `factory.spawn_with(|b| b.system("prompt").tool(MyTool))` is called, then an `Agent` is returned with the factory's provider, the system prompt, AND the custom tool.

2. Given the closure receives an `AgentBuilder`, when the closure applies any valid builder method, then the resulting agent includes all customizations from the closure.

3. Given `spawn_with()` is called, when the closure does NOT call `.system()`, then the agent is created without a system prompt (no implicit default).

## Tasks / Subtasks

- [ ] Task 1: Implement `spawn_with()` on `AgentFactory` (AC: #1, #2, #3)
  - [ ] Add method: `pub fn spawn_with(&self, f: impl FnOnce(AgentBuilder) -> AgentBuilder) -> Result<Agent>`
  - [ ] Clone provider, create builder with provider pre-set, pass to closure, call `build()`
  - [ ] Returns `Result<Agent>` since builder customization may produce invalid configs
- [ ] Task 2: Unit tests (AC: #1, #2, #3)
  - [ ] Test spawn_with with system prompt + tool
  - [ ] Test spawn_with without system prompt
  - [ ] Test spawn_with with multiple builder methods

## Dev Notes

- **Crate:** `traitclaw-core` → `crates/traitclaw-core/src/factory.rs` (same file as 2-1)
- **Key difference from `spawn()`:** Returns `Result<Agent>` (not infallible) because the closure may configure the builder in ways that can fail.
- **Depends on:** Story 2-1 (`AgentFactory` struct must exist).
- **Pattern:** `spawn_with` = `provider.clone()` → `builder.provider(cloned)` → `f(builder)` → `.build()`

### Project Structure Notes

- Modified: `crates/traitclaw-core/src/factory.rs` (add method to existing struct)
- Tests: Same test file as 2-1

### References

- [Source: _bmad-output/planning-artifacts/epics-v0.6.0.md#Story 2.2]
- [Source: _bmad-output/planning-artifacts/prd-v0.6.0.md#FR4]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
