# Story 4.2: Parallel Branch Evaluation & Path Selection

Status: ready-for-dev

## Story

As a developer,
I want `MctsStrategy` to execute parallel reasoning branches and select the best path,
so that my agent explores multiple approaches and picks the optimal answer.

## Acceptance Criteria

1. Strategy spawns `branches` parallel tasks via `tokio::spawn` + `JoinSet`
2. Each branch explores reasoning paths up to `max_depth`
3. Each branch scored using configured `ScoringFn`
4. Highest-scoring path selected as final result
5. Branch scores and selected path exposed for post-execution analysis (FR20)
6. All spawned tasks properly cleaned up via `JoinSet`
7. MCTS spawn overhead < 100μs per branch (NFR4)

## Tasks / Subtasks

- [ ] Task 1: Implement parallel execution (AC: #1, #6)
  - [ ] Create `JoinSet<(f64, String, Vec<ThoughtStep>)>` for branch results
  - [ ] Spawn `branches` tasks via `tokio::spawn`
  - [ ] Each task: call LLM with slightly varied prompts
  - [ ] Await all tasks via `JoinSet::join_next()`
  - [ ] Handle task panics gracefully
- [ ] Task 2: Branch scoring (AC: #2, #3)
  - [ ] Each branch produces reasoning at specified depth
  - [ ] Apply `ScoringFn` to each branch's final output
  - [ ] Collect (score, output, thought_steps) tuples
- [ ] Task 3: Path selection (AC: #4, #5)
  - [ ] Select branch with highest score
  - [ ] Store all branch scores for analysis
  - [ ] Expose `branch_results()` accessor with scores and paths
  - [ ] Return selected path's output as strategy result
- [ ] Task 4: Tests (AC: #1, #3, #4, #5, #6)
  - [ ] Test parallel execution with mock provider
  - [ ] Test scoring function application
  - [ ] Test highest-score selection
  - [ ] Test branch results accessibility
  - [ ] Test JoinSet cleanup on error

## Dev Notes

- CRITICAL: Use `tokio::task::JoinSet` (not manual Vec<JoinHandle>) per AR3
- JoinSet automatically cancels remaining tasks when dropped — automatic cleanup
- Provider must be `Clone + Send + Sync` to share across tasks — verify this
- Consider adding temperature variation across branches for diversity
- Spawn overhead target: < 100μs/branch (NFR4) — benchmark in tests

### Project Structure Notes

- Modify: `crates/traitclaw-strategies/src/mcts/strategy.rs`
- Depends on: Story 4.1 (builder + struct)

### References

- [Source: architecture.md#Decision-3-MCTS-Structured-Concurrency]
- [Source: prd.md#FR5-FR20]
- [Source: architecture.md#NFR4]

## Dev Agent Record

### Agent Model Used
### Debug Log References
### Completion Notes List
### File List
