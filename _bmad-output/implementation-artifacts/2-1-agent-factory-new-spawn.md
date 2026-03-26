# Story 2.1: Implement `AgentFactory` Struct with `new()` and `spawn()`

Status: ready-for-dev

## Story

As a **Rust developer creating multiple agents for a pipeline**,
I want to create an `AgentFactory` from a provider and spawn agents with `factory.spawn("system prompt")`,
so that I don't repeat provider configuration for each agent.

## Acceptance Criteria

1. Given a provider implementing `Provider + Clone`, when `AgentFactory::new(provider)` is called, then an `AgentFactory` is returned holding the provider configuration.

2. Given an `AgentFactory` instance, when `factory.spawn("You are a researcher.")` is called, then an `Agent` is returned with the factory's provider (cloned) and the given system prompt.

3. Given an `AgentFactory` instance, when `spawn()` is called multiple times with different system prompts, then each returned agent has its own cloned provider instance and unique system prompt, and agents are fully independent.

4. Given a provider type that is `Send + Sync`, when `AgentFactory` is created with it, then the factory is also `Send + Sync` (NFR9).

5. Given `factory.spawn()` is called, when measured against direct builder usage, then overhead is < 1μs — only one `provider.clone()` + builder call (NFR1).

## Tasks / Subtasks

- [ ] Task 1: Create `AgentFactory` struct (AC: #1, #4)
  - [ ] Define `pub struct AgentFactory<P: Provider + Clone>` in `crates/traitclaw-core/src/factory.rs`
  - [ ] Add `provider: P` field
  - [ ] Implement `new(provider: P) -> Self`
  - [ ] Derive/implement `Debug, Clone` as appropriate
  - [ ] Verify `Send + Sync` bounds propagate from `P`
- [ ] Task 2: Implement `spawn()` method (AC: #2, #3, #5)
  - [ ] `pub fn spawn(&self, system: impl Into<String>) -> Agent`
  - [ ] Clone provider, delegate to `Agent::with_system(self.provider.clone(), system)`
  - [ ] Ensure each agent owns its own provider clone
- [ ] Task 3: Add module and re-export (AC: #1)
  - [ ] Create `crates/traitclaw-core/src/factory.rs`
  - [ ] Add `mod factory;` to `crates/traitclaw-core/src/lib.rs`
  - [ ] Add `pub use factory::AgentFactory;` to lib.rs
- [ ] Task 4: Unit tests (AC: #2, #3)
  - [ ] Test spawn returns agent with correct system prompt
  - [ ] Test multiple spawns produce independent agents
  - [ ] Test with `Arc<dyn Provider>` (already Clone via Arc)

## Dev Notes

- **Crate:** `traitclaw-core` — new file `crates/traitclaw-core/src/factory.rs`
- **ADR-18:** `Provider: Clone` bound. `Arc<dyn Provider>` already impls Clone, so this works with all current providers.
- **ADR-21:** No `AgentFactory` trait — just concrete struct. YAGNI.
- **Key pattern:** Factory is a thin wrapper — `spawn()` = `provider.clone()` + `Agent::with_system()`. Keep it simple.
- **Depends on:** Story 1-1 (`Agent::with_system()` must exist first).
- **No new dependencies** required (NFR12).

### Project Structure Notes

- New file: `crates/traitclaw-core/src/factory.rs`
- Modified: `crates/traitclaw-core/src/lib.rs` (add module)
- Tests: `crates/traitclaw-core/tests/factory_tests.rs`

### References

- [Source: _bmad-output/planning-artifacts/epics-v0.6.0.md#Story 2.1]
- [Source: _bmad-output/planning-artifacts/prd-v0.6.0.md#FR2, FR3]
- [Source: _bmad-output/planning-artifacts/epics-v0.6.0.md#ADR-18]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
