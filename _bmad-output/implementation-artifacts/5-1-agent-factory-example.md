# Story 5.1: Create Example `24-agent-factory/`

Status: ready-for-dev

## Story

As a **Rust developer evaluating TraitClaw for multi-agent use cases**,
I want a runnable example that demonstrates all composition APIs progressively,
so that I can see them in action and adapt the patterns to my use case.

## Acceptance Criteria

1. Given the example directory `examples/24-agent-factory/`, when `cargo run -p agent-factory` is executed, then the example compiles and runs, demonstrating all four APIs in sequence.

2. Given the example code, when read by a developer, then it progressively demonstrates: (1) `Agent::with_system()`, (2) `AgentFactory::spawn()`, (3) `AgentPool::from_team()`, (4) `RoundRobinGroupChat`.

3. Given the example runs, when output is printed, then it produces meaningful output showing each API stage (not just "ok").

## Tasks / Subtasks

- [ ] Task 1: Create example directory and Cargo.toml (AC: #1)
  - [ ] Create `examples/24-agent-factory/`
  - [ ] Add `Cargo.toml` with dependencies on `traitclaw`
- [ ] Task 2: Implement progressive example (AC: #2, #3)
  - [ ] Section 1: `Agent::with_system()` — single agent usage
  - [ ] Section 2: `AgentFactory::spawn()` — multi-agent creation
  - [ ] Section 3: `AgentPool` — sequential pipeline
  - [ ] Section 4: `RoundRobinGroupChat` — collaborative discussion
  - [ ] Add descriptive print statements between sections
- [ ] Task 3: Verify compilation and output (AC: #1)

## Dev Notes

- **Depends on:** ALL Epic 1-4 stories must be complete.
- **Pattern:** Follow existing example structure (e.g., `examples/23-multi-agent-team/`).
- **Provider:** Use OpenAI-compatible provider with `.env` for API key.
- **Output:** Should clearly label each API tier with headers and meaningful responses.

### Project Structure Notes

- New directory: `examples/24-agent-factory/`
- New files: `Cargo.toml`, `src/main.rs`
- Reference: `examples/23-multi-agent-team/` for structure

### References

- [Source: _bmad-output/planning-artifacts/epics-v0.6.0.md#Story 5.1]
- [Source: _bmad-output/planning-artifacts/prd-v0.6.0.md#FR13]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
