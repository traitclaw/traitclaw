# Story 5.3: Multi-Agent Orchestration

Status: ready-for-dev

## Story

As a developer,
I want to compose multiple agents into teams,
So that complex tasks can be split across specialized agents.

## Acceptance Criteria

1. **Given** `traitclaw-team` crate with feature `"team"` **When** I create a team with router + specialist agents **Then** messages are routed to the appropriate agent
2. **And** agents can delegate to each other
3. **And** `VerificationChain` enables generate → verify patterns

## Tasks / Subtasks

- [ ] Task 1: Create `traitclaw-team` crate
- [ ] Task 2: Define `Team` struct with router agent
- [ ] Task 3: Implement routing logic
- [ ] Task 4: Implement agent delegation
- [ ] Task 5: Implement `VerificationChain`
- [ ] Task 6: Write tests

## Dev Notes

### Architecture Requirements
- Team has a router agent that decides which specialist to invoke
- Delegation: agent A can call agent B as if calling a tool
- VerificationChain: generate with agent A → verify with agent B → retry if rejected

### References
- [Source: _bmad-output/architecture.md#6 Optional - team]
- [Source: _bmad-output/epics.md#Story 5.3]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log
