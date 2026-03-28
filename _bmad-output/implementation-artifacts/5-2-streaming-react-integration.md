# Story 5.2: Streaming Integration with ReAct

Status: ready-for-dev

## Story

As a developer,
I want to use `StreamingOutputTransformer` with `ReActStrategy`,
so that I can stream individual thought steps as they happen.

## Acceptance Criteria

1. `transform_chunk` called for each output chunk during ReAct execution
2. `on_thought_step` called for each `ThoughtStep` event during ReAct loop
3. Streaming latency < 10ms from first token to first emission (NFR5)

## Tasks / Subtasks

- [ ] Task 1: Wire streaming into ReAct loop (AC: #1, #2)
  - [ ] After each ThoughtStep is generated, call `on_thought_step`
  - [ ] During LLM streaming response, call `transform_chunk` per chunk
  - [ ] Handle transformer errors gracefully (log, don't abort)
- [ ] Task 2: Latency optimization (AC: #3)
  - [ ] Ensure streaming happens inline (not batched)
  - [ ] No unnecessary buffering between LLM output and transformer
- [ ] Task 3: Integration tests (AC: #1, #2, #3)
  - [ ] Test with mock StreamingOutputTransformer recording calls
  - [ ] Verify correct ThoughtStep events are streamed
  - [ ] Verify chunk ordering

## Dev Notes

- ReAct strategy from Story 2.2 must expose hook points for streaming
- If AgentRuntime manages the transformer, ReAct calls runtime's streaming methods
- If strategy manages it directly, inject via builder or execute params
- Consider: `ReActStrategy::builder().streaming(transformer).build()?`

### References

- [Source: prd.md#FR13]
- [Source: architecture.md#NFR5]

## Dev Agent Record

### Agent Model Used
### Debug Log References
### Completion Notes List
### File List
