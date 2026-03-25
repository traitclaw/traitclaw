# Story 2.1: OutputTransformer Trait Definition

Status: in-progress

## Story

As a framework developer,
I want the async `OutputTransformer` trait defined in `traitclaw-core`,
so that tool output can be processed asynchronously with context awareness.

## Acceptance Criteria

1. ✅ `traitclaw-core/src/traits/output_transformer.rs` exists with async `OutputTransformer` trait
2. `transform()` is async, accepts `output: String`, `tool_name: &str`, `state: &AgentState`
3. Blanket impl `impl<T: OutputProcessor + 'static> OutputTransformer for T` exists
4. `OutputProcessor` marked `#[deprecated(since = "0.3.0", note = "Use OutputTransformer")]`
5. Trait requires `Send + Sync`
6. `estimate_output_tokens()` with default impl (4-chars ≈ 1-token)
7. Rustdoc with usage example
8. Unit test confirms trait is object-safe (`Arc<dyn OutputTransformer>` compiles)

## Tasks / Subtasks

- [ ] Task 1: Create `output_transformer.rs` with trait definition
- [ ] Task 2: Implement blanket impl for `OutputProcessor`
- [ ] Task 3: Mark `OutputProcessor` as deprecated
- [ ] Task 4: Register module and add re-exports
- [ ] Task 5: Add unit tests
- [ ] Task 6: Run full test suite

## Dev Agent Record

### Agent Model Used
Gemini 2.5 Pro
### Completion Notes List
### File List
### Change Log
