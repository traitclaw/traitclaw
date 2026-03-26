# Story 5.3: Update README with Multi-Agent Quickstart

Status: ready-for-dev

## Story

As a **potential TraitClaw user visiting the repository**,
I want a "Multi-Agent Quickstart" section in the README showing the 3-tier API,
so that I can immediately understand TraitClaw's multi-agent ergonomics.

## Acceptance Criteria

1. Given the README.md, when updated, then it contains a "Multi-Agent Quickstart" section with working code examples.

2. Given the quickstart code, when read, then it shows all 3 tiers: (1) `Agent::with_system()`, (2) `AgentFactory::spawn()`, (3) `AgentPool`/`RoundRobinGroupChat`.

3. Given the README, when reviewed, then the quickstart fits in a single code block (≤ 15 lines) for maximum impact.

## Tasks / Subtasks

- [ ] Task 1: Add Multi-Agent Quickstart section to README (AC: #1, #2, #3)
  - [ ] Insert after existing quickstart section
  - [ ] Show 3-tier API in ≤ 15 lines of code
  - [ ] Ensure code compiles (test as doc-test or manual verification)
- [ ] Task 2: Review README flow (AC: #1)
  - [ ] Ensure natural reading order from single-agent to multi-agent

## Dev Notes

- **File:** `README.md` (project root)
- **Tone:** Marketing/showcase — this is the first thing visitors see.
- **Keep it short:** ≤ 15 lines of code. Show the "wow factor" of composition API.
- **Depends on:** All Epic 1-4 features.

### References

- [Source: _bmad-output/planning-artifacts/epics-v0.6.0.md#Story 5.3]
- [Source: _bmad-output/planning-artifacts/prd-v0.6.0.md#FR16]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
