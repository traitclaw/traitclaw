# Story 5.3: Per-Strategy Examples

Status: ready-for-dev

## Story

As a developer,
I want runnable examples for each strategy,
so that I can quickly understand how to use them in my own projects.

## Acceptance Criteria

1. `examples/25-react-strategy/` demonstrates ReAct usage
2. `examples/26-cot-strategy/` demonstrates CoT usage
3. `examples/27-mcts-strategy/` demonstrates MCTS usage
4. `examples/28-streaming-thought/` demonstrates StreamingOutputTransformer
5. Each example has `README.md` explaining the strategy
6. `cargo build --examples` compiles all examples
7. Examples follow existing pattern (Cargo.toml, src/main.rs, README.md)

## Tasks / Subtasks

- [ ] Task 1: ReAct example (AC: #1, #7)
  - [ ] `examples/25-react-strategy/Cargo.toml`
  - [ ] `examples/25-react-strategy/src/main.rs`
  - [ ] `examples/25-react-strategy/README.md`
  - [ ] Demo: agent solving a multi-step problem with tools
- [ ] Task 2: CoT example (AC: #2, #7)
  - [ ] `examples/26-cot-strategy/Cargo.toml`
  - [ ] `examples/26-cot-strategy/src/main.rs`
  - [ ] `examples/26-cot-strategy/README.md`
  - [ ] Demo: agent reasoning through a complex question
- [ ] Task 3: MCTS example (AC: #3, #7)
  - [ ] `examples/27-mcts-strategy/Cargo.toml`
  - [ ] `examples/27-mcts-strategy/src/main.rs`
  - [ ] `examples/27-mcts-strategy/README.md`
  - [ ] Demo: agent exploring multiple paths with scoring
- [ ] Task 4: Streaming example (AC: #4, #7)
  - [ ] `examples/28-streaming-thought/Cargo.toml`
  - [ ] `examples/28-streaming-thought/src/main.rs`
  - [ ] `examples/28-streaming-thought/README.md`
  - [ ] Demo: streaming ThoughtStep events to terminal
- [ ] Task 5: Compilation check (AC: #6)
  - [ ] `cargo build --examples` succeeds

## Dev Notes

- Follow existing example pattern: look at `examples/01-basic-agent/` for structure
- Each example depends on `traitclaw` with appropriate features
- Use `dotenv` for API key loading (consistent with other examples)
- Examples should be self-contained and educational

### References

- [Source: prd.md#FR24-FR25]

## Dev Agent Record

### Agent Model Used
### Debug Log References
### Completion Notes List
### File List
