# Story 5.2: Create Migration Guide `docs/migration-v0.5-to-v0.6.md`

Status: ready-for-dev

## Story

As a **Rust developer upgrading from TraitClaw v0.5.0**,
I want a clear migration guide explaining what's new and how to adopt the new APIs,
so that I can upgrade with confidence and optionally adopt new patterns.

## Acceptance Criteria

1. Given the migration guide, when read by a v0.5.0 user, then it clearly states: no breaking changes, existing code compiles without modification.

2. Given the migration guide, when reviewed, then it documents all new types (`AgentFactory`, `AgentPool`, `RoundRobinGroupChat`) with before/after code comparisons.

3. Given the guide's "Optional Adoption" section, when read, then it shows how to incrementally adopt `Agent::with_system()` in new code while legacy builder code stays untouched.

## Tasks / Subtasks

- [ ] Task 1: Create migration guide file (AC: #1)
  - [ ] Create `docs/migration-v0.5-to-v0.6.md`
  - [ ] Header with version info, date, "no breaking changes" callout
- [ ] Task 2: Document new APIs with before/after (AC: #2)
  - [ ] `Agent::with_system()` vs builder pattern
  - [ ] `AgentFactory::spawn()` vs repeated builders
  - [ ] `AgentPool::from_team()` vs manual binding
  - [ ] `RoundRobinGroupChat` — new capability (no "before")
- [ ] Task 3: Optional adoption section (AC: #3)
  - [ ] Show incremental adoption path
  - [ ] Emphasize backward compatibility

## Dev Notes

- **Pattern:** Follow existing migration guides: `docs/migration-v0.4-to-v0.5.md`
- **Key message:** Zero breaking changes. All new features are purely additive.
- **Depends on:** All Epic 1-4 features implemented.

### References

- [Source: _bmad-output/planning-artifacts/epics-v0.6.0.md#Story 5.2]
- [Source: _bmad-output/planning-artifacts/prd-v0.6.0.md#FR14]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
