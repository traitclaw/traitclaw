# Story 4.1: `MctsStrategy` Builder & Configuration

Status: ready-for-dev

## Story

As a developer,
I want to instantiate a `MctsStrategy` with configurable branch count and search depth,
so that I can tune the exploration-exploitation trade-off for my use case.

## Acceptance Criteria

1. `MctsStrategy::builder().branches(5).max_depth(3).build()?` returns valid instance implementing `AgentStrategy`
2. Default `branches` is 5, `max_depth` is 3
3. Custom `ScoringFn` via `.scoring(Arc::new(|s| score))`
4. Default scoring uses LLM self-evaluation
5. Object-safe (`Box<dyn AgentStrategy>`)
6. Feature-gated behind `mcts` flag

## Tasks / Subtasks

- [ ] Task 1: Create MCTS module structure (AC: #6)
  - [ ] Create `src/mcts/mod.rs`
  - [ ] Create `src/mcts/strategy.rs`
  - [ ] Create `src/mcts/scoring.rs` for ScoringFn type
- [ ] Task 2: Define ScoringFn type (AC: #3)
  - [ ] `pub type ScoringFn = Arc<dyn Fn(&str) -> f64 + Send + Sync>;`
  - [ ] Default implementation using LLM self-evaluation prompt
- [ ] Task 3: Implement builder (AC: #1, #2, #3, #4)
  - [ ] `MctsStrategyBuilder` with `branches`, `max_depth`, `scoring_fn`
  - [ ] Builder methods with defaults
  - [ ] Validate `branches > 0`, `max_depth > 0`
- [ ] Task 4: Stub AgentStrategy impl (AC: #5)
  - [ ] `impl AgentStrategy for MctsStrategy`
  - [ ] Stub `execute` (full parallel logic in Story 4.2)
  - [ ] Verify object safety
- [ ] Task 5: Tests (AC: #1, #2, #3, #5)
  - [ ] Test builder with defaults
  - [ ] Test builder with custom scoring function
  - [ ] Test validation rejects invalid config
  - [ ] Test object safety

## Dev Notes

- `ScoringFn` type defined per AR7: `Arc<dyn Fn(&str) -> f64 + Send + Sync>`
- Arc required for `Clone` and thread-safety (MCTS branches run in parallel)
- Default scoring: prompt LLM "Rate the quality of this answer from 0 to 1: {answer}"
- Builder validates both `branches > 0` and `max_depth > 0`

### Project Structure Notes

- Files: `crates/traitclaw-strategies/src/mcts/mod.rs`, `strategy.rs`, `scoring.rs`

### References

- [Source: architecture.md#Decision-3-MCTS-Structured-Concurrency]
- [Source: architecture.md#AR7]
- [Source: prd.md#FR4-FR5]

## Dev Agent Record

### Agent Model Used
### Debug Log References
### Completion Notes List
### File List
