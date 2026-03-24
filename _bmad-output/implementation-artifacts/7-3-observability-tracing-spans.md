# Story 7.3: Observability (tracing spans)

Status: ready-for-dev

## Story

As a developer,
I want structured tracing spans for agent operations,
So that I can integrate with OpenTelemetry and debug production issues.

## Acceptance Criteria

1. **Given** the agent runtime uses `tracing` crate **When** I configure a tracing subscriber **Then** spans are emitted for: `agent.run`, each iteration, each tool call, each LLM request
2. **And** span attributes include: model, session_id, iteration, tokens_used
3. **And** optional `tracing-opentelemetry` integration is documented

## Tasks / Subtasks

- [ ] Task 1: Add tracing spans to runtime loop
  - [ ] `agent.run` span with model and session_id
  - [ ] Per-iteration span with iteration count
  - [ ] Per-LLM-call span with request details
  - [ ] Per-tool-call span with tool name
- [ ] Task 2: Add span attributes
  - [ ] model, session_id, iteration, tokens_used
- [ ] Task 3: Document OpenTelemetry integration
- [ ] Task 4: Write tests verifying spans are emitted

## Dev Notes

### Architecture Requirements
- Use `tracing::instrument` attribute on key functions
- Spans should be zero-cost when no subscriber configured
- No `println!` in library code — all logging via `tracing`

### References
- [Source: _bmad-output/project-context.md#Technology Stack - tracing]
- [Source: _bmad-output/epics.md#Story 7.3]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log
