# Story 3.3: Implement `AgentPool::from_team()`

Status: ready-for-dev

## Story

As a **Rust developer who already has `Team` definitions with `AgentRole`s**,
I want to create an `AgentPool` directly from a `Team` and a provider,
so that I don't manually bind each role to an agent instance.

## Acceptance Criteria

1. Given a `Team` with 3 `AgentRole`s each having a `system_prompt` set, when `AgentPool::from_team(&team, provider)` is called, then an `AgentPool` with 3 agents is returned, each configured with its role's system_prompt.

2. Given a `Team` where one `AgentRole` has no `system_prompt`, when `AgentPool::from_team(&team, provider)` is called, then it returns `Err` with a clear error message listing which role(s) are missing system_prompt.

3. Given a valid `Team` and a `Provider + Clone` instance, when `from_team()` creates agents, then each agent receives a cloned provider (O(n) construction, no async work — NFR2).

4. Given the `AgentPool` created from a team, when `pool.len()` is called, then it matches the number of roles in the team.

## Tasks / Subtasks

- [ ] Task 1: Implement `from_team()` (AC: #1, #3, #4)
  - [ ] `pub fn from_team<P: Provider + Clone>(team: &Team, provider: P) -> Result<Self>`
  - [ ] Iterate over team roles, clone provider for each, create agent with `Agent::with_system()`
  - [ ] Collect into `Vec<Agent>`, wrap in `AgentPool::new()`
- [ ] Task 2: Error handling for missing system_prompt (AC: #2)
  - [ ] Check each role for system_prompt before creating agents
  - [ ] Collect all missing role names, return single error listing all
  - [ ] Add error variant to core error type if needed
- [ ] Task 3: Unit tests
  - [ ] Test with valid Team (3 roles with system_prompts)
  - [ ] Test with invalid Team (1 role missing system_prompt)
  - [ ] Test that pool.len() matches team role count

## Dev Notes

- **Crate:** `traitclaw-core` → `crates/traitclaw-core/src/pool.rs`
- **Cross-crate dependency:** Uses `Team` from `traitclaw-team`. Check if `AgentPool` should live in `traitclaw-core` or `traitclaw-team`. Per ADR-20, team-related types are in `traitclaw-team`, but `AgentPool` is in `traitclaw-core`. `from_team()` may need to be a separate impl block gated on team feature, OR `from_team()` lives in `traitclaw-team` as an extension.
- **Depends on:** Story 3-1 (AgentPool struct) + Story 1-1 (Agent::with_system).
- **ADR-18:** Provider: Clone required for factory pattern.

### Project Structure Notes

- Modified: `crates/traitclaw-core/src/pool.rs` OR a new extension in `traitclaw-team`
- Consider: `from_team()` might be better in `traitclaw-team` to avoid `traitclaw-core` depending on `traitclaw-team`

### References

- [Source: _bmad-output/planning-artifacts/epics-v0.6.0.md#Story 3.3]
- [Source: _bmad-output/planning-artifacts/prd-v0.6.0.md#FR6]
- [Source: _bmad-output/planning-artifacts/epics-v0.6.0.md#ADR-18, ADR-19]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
