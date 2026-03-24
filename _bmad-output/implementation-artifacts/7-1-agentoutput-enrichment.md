# Story 7.1: AgentOutput Enrichment

Status: ready-for-dev

## Story

As a developer,
I want richer agent output with usage stats and multiple variants,
So that I can build production applications with proper monitoring.

## Acceptance Criteria

1. **Given** the `AgentOutput` enum **When** I call `agent.run(input)` **Then** `AgentOutput` has variants: `Text(String)`, `Structured(Value)`, `Error(String)`
2. **And** `output.text()` returns `Option<&str>` (not panic on non-text variants)
3. **And** `output.usage()` returns `RunUsage { tokens, iterations, duration }`

## Tasks / Subtasks

- [ ] Task 1: Enhance `AgentOutput` enum (AC: 1, 2)
  - [ ] Add Structured(Value) and Error(String) variants
  - [ ] Safe accessor methods: `text()`, `structured()`, `is_error()`
- [ ] Task 2: Add `RunUsage` tracking (AC: 3)
  - [ ] Track total tokens, iterations, duration in runtime
  - [ ] Include in AgentOutput
- [ ] Task 3: Update runtime to populate usage
- [ ] Task 4: Write tests

## Dev Notes

### References
- [Source: _bmad-output/epics.md#Story 7.1]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log
