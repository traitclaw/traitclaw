# Story 5.5: Rustdoc, Migration Guide & NFR Validation

Status: ready-for-dev

## Story

As a developer,
I want complete documentation and verified performance,
so that I can confidently adopt v0.7.0 in production.

## Acceptance Criteria

1. All public types have `///` doc comments with `# Examples` section (FR27)
2. `cargo test --doc` passes (doc examples compile)
3. Migration guide at `docs/migration-v0.6-to-v0.7.md` covers all new types (FR26)
4. Compile time delta < 2% vs v0.6.0 (NFR1)
5. Binary size delta < 5% vs v0.6.0 (NFR2)
6. ReAct loop latency < 1ms/cycle excluding LLM (NFR3)
7. MCTS spawn overhead < 100μs/branch (NFR4)

## Tasks / Subtasks

- [ ] Task 1: Rustdoc for all public types (AC: #1, #2)
  - [ ] `ThoughtStep` — doc comment with example
  - [ ] `ReActStrategy` — doc comment with builder example
  - [ ] `ChainOfThoughtStrategy` — doc comment with example
  - [ ] `MctsStrategy` — doc comment with example
  - [ ] `ScoringFn` — doc comment with usage
  - [ ] `StreamingOutputTransformer` — doc comment with impl example
  - [ ] Run `cargo test --doc -p traitclaw-strategies`
  - [ ] Run `cargo test --doc -p traitclaw-core`
- [ ] Task 2: Migration guide (AC: #3)
  - [ ] Create `docs/migration-v0.6-to-v0.7.md`
  - [ ] Document new crate: `traitclaw-strategies`
  - [ ] Document new types and their usage
  - [ ] Document feature flags
  - [ ] Document backward compatibility guarantees
- [ ] Task 3: Performance validation (AC: #4, #5, #6, #7)
  - [ ] Measure compile time: `cargo build --timings` baseline vs v0.7.0
  - [ ] Measure binary size: compare example binary sizes
  - [ ] Benchmark ReAct loop with mock provider
  - [ ] Benchmark MCTS spawn overhead
  - [ ] Document results in release notes

## Dev Notes

- Doc examples must use `no_run` or `ignore` if they require API keys
- Compile time: `cargo build --timings` generates HTML report
- Binary size: `cargo bloat` or `ls -la target/release/examples/`
- Performance: use `std::time::Instant` for micro-benchmarks in tests

### References

- [Source: prd.md#FR26-FR27]
- [Source: prd.md#NFR1-NFR6]

## Dev Agent Record

### Agent Model Used
### Debug Log References
### Completion Notes List
### File List
