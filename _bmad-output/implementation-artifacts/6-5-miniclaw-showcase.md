# Story 6.5: Miniclaw Showcase

Status: ready-for-dev

## Story

As a developer,
I want a mini OpenClaw-like assistant built with TraitClaw,
So that I can see the framework used in a real application.

## Acceptance Criteria

1. **Given** the `showcase/miniclaw` project **When** I build and run it **Then** it provides a working CLI AI assistant
2. **And** codebase is under 1000 lines
3. **And** it demonstrates: Agent, Tools, Memory, Steering
4. **And** README documents progressive enhancement steps

## Tasks / Subtasks

- [ ] Task 1: Create `showcase/miniclaw/` project structure
- [ ] Task 2: Implement basic CLI agent (~50 lines)
- [ ] Task 3: Add memory (+20 lines)
- [ ] Task 4: Add tools (+30 lines)
- [ ] Task 5: Add steering (+10 lines)
- [ ] Task 6: Write progressive enhancement README
- [ ] Task 7: Verify total is under 1000 lines

## Dev Notes

### Architecture Requirements
- Progressive enhancement demo — each step adds 10-30 lines
- If each option costs >30 lines → DX is proven BAD
- Validates AD-8: If miniclaw needs hacks → design is wrong

### Enhancement Steps
| Step | Adds | Lines |
|------|------|:-----:|
| 1. Basic | 1 agent, CLI channel | ~50 |
| 2. + Memory | SQLite persistence | +20 |
| 3. + Tools | Web search, file tools | +30 |
| 4. + Steering | Guard-Hint-Track | +10 |
| 5. + Multi-agent | Agent routing | +40 |

### References
- [Source: _bmad-output/architecture.md#8 Showcase: miniclaw]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log
