# Story 3.1: `ChainOfThoughtStrategy` Builder & Execution

Status: ready-for-dev

## Story

As a developer,
I want to instantiate a `ChainOfThoughtStrategy` that injects step-by-step reasoning,
so that my agent produces structured, explainable thought chains.

## Acceptance Criteria

1. `ChainOfThoughtStrategy::builder().max_steps(5).build()?` returns valid instance implementing `AgentStrategy`
2. Strategy injects step-by-step reasoning instructions into agent's prompt
3. Each reasoning step captured as `ThoughtStep::Think` event
4. Final answer captured as `ThoughtStep::Answer`
5. Execution terminates when answer produced or `max_steps` reached
6. `ThoughtStep` sequence inspectable after execution
7. Object-safe (`Box<dyn AgentStrategy>`)
8. Feature-gated behind `cot` flag

## Tasks / Subtasks

- [ ] Task 1: Create CoT module structure (AC: #8)
  - [ ] Create `src/cot/mod.rs`
  - [ ] Create `src/cot/strategy.rs`
- [ ] Task 2: Implement builder (AC: #1)
  - [ ] `ChainOfThoughtStrategyBuilder` with `max_steps` field
  - [ ] `.builder()` → builder, `.max_steps(n)` → self, `.build()` → Result
  - [ ] Default `max_steps = 5`
  - [ ] Validate `max_steps > 0`
- [ ] Task 3: Implement AgentStrategy (AC: #2, #3, #4, #5, #6, #7)
  - [ ] Construct CoT system prompt ("Think step by step...")
  - [ ] Parse LLM output to extract individual reasoning steps
  - [ ] Track `Vec<ThoughtStep>` during execution
  - [ ] Detect final answer and emit `Answer`
  - [ ] Expose `thought_steps()` accessor
- [ ] Task 4: Tests (AC: #1, #3, #5, #7)
  - [ ] Test builder with defaults and custom config
  - [ ] Test CoT execution with mock provider
  - [ ] Test max_steps termination
  - [ ] Test ThoughtStep sequence

## Dev Notes

- Simpler than ReAct: no tool calling, pure reasoning
- CoT prompt: prepend "Let's think step by step." or structured instruction
- Parse step boundaries from LLM output (numbered steps, "Step N:", etc.)
- Follow same builder pattern as `ReActStrategy` (AR6)

### Project Structure Notes

- Files: `crates/traitclaw-strategies/src/cot/mod.rs`, `src/cot/strategy.rs`

### References

- [Source: architecture.md#Decision-1-Strategy-Module-Structure]
- [Source: prd.md#FR6-FR7]
- [Source: architecture.md#Builder-Pattern]

## Dev Agent Record

### Agent Model Used
### Debug Log References
### Completion Notes List
### File List
