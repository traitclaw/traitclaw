# Story 5.1: `StreamingOutputTransformer` Trait

Status: ready-for-dev

## Story

As a developer,
I want a trait to transform agent output as it streams,
so that I can process thought steps in real-time without waiting for completion.

## Acceptance Criteria

1. `StreamingOutputTransformer` trait added in `traitclaw-core/src/streaming.rs`
2. Has `async fn transform_chunk(&self, chunk: &str) -> Result<String>` (required)
3. Has `async fn on_thought_step(&self, step: &ThoughtStep) -> Result<()>` (default no-op)
4. Trait is `Send + Sync` and re-exported from `traitclaw_core`
5. Composes alongside existing `OutputTransformer` (additive, not replacing)
6. Existing `OutputTransformer` behavior unchanged

## Tasks / Subtasks

- [ ] Task 1: Create trait definition (AC: #1, #2, #3, #4)
  - [ ] Create `crates/traitclaw-core/src/streaming.rs`
  - [ ] Define `StreamingOutputTransformer` trait with `#[async_trait]`
  - [ ] `transform_chunk` required method
  - [ ] `on_thought_step` default method (no-op)
  - [ ] Ensure `Send + Sync` bounds
- [ ] Task 2: Integration with core (AC: #4, #5, #6)
  - [ ] Add `mod streaming;` to `traitclaw-core/src/lib.rs`
  - [ ] Re-export `StreamingOutputTransformer` from lib.rs
  - [ ] Add to prelude if appropriate
  - [ ] Verify existing `OutputTransformer` unchanged
- [ ] Task 3: AgentBuilder integration (AC: #5)
  - [ ] Add `streaming_transformer` field to `AgentBuilder` or `AgentConfig`
  - [ ] `.streaming_transformer(impl StreamingOutputTransformer)` builder method
  - [ ] Optional — no breaking change if not set
- [ ] Task 4: Tests (AC: #2, #3, #5, #6)
  - [ ] Test custom implementation of trait
  - [ ] Test default no-op on_thought_step
  - [ ] Test existing OutputTransformer still works
  - [ ] Verify backward compatibility

## Dev Notes

- CRITICAL: This modifies `traitclaw-core` — must be strictly additive (AR4)
- `ThoughtStep` is defined in `traitclaw-strategies` but referenced here — need to use a generic type or re-export
- Alternative: define a simpler `StreamingChunk` type in core, keep `ThoughtStep` in strategies
- Consider: `on_thought_step` may need `ThoughtStep` imported — this creates circular dependency if ThoughtStep is in strategies
- SOLUTION: `on_thought_step` takes `&serde_json::Value` or define a minimal `StreamEvent` enum in core

### Project Structure Notes

- New file: `crates/traitclaw-core/src/streaming.rs`
- Modify: `crates/traitclaw-core/src/lib.rs`
- Potential modify: `crates/traitclaw-core/src/agent/builder.rs`

### References

- [Source: architecture.md#Decision-4-StreamingOutputTransformer-Placement]
- [Source: prd.md#FR11-FR12]

## Dev Agent Record

### Agent Model Used
### Debug Log References
### Completion Notes List
### File List
