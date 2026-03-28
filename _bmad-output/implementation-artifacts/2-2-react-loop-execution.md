# Story 2.2: Think→Act→Observe Loop Execution

Status: ready-for-dev

## Story

As a developer,
I want `ReActStrategy` to autonomously execute reasoning loops,
so that my agent can reason through multi-step problems with tool use.

## Acceptance Criteria

1. Strategy executes Think→Act→Observe cycles when agent processes a query
2. Each cycle emits `ThoughtStep` events (`Think`, `Act`, `Observe`)
3. Loop terminates when strategy produces `Answer` or hits `max_iterations`
4. Tool calls in `Act` step routed through `AgentRuntime` tool registry
5. `ThoughtStep` sequence inspectable after execution (FR19)
6. Correct prompt engineering to instruct LLM to follow ReAct format

## Tasks / Subtasks

- [ ] Task 1: Implement ReAct loop in `execute` (AC: #1, #3)
  - [ ] Construct ReAct system prompt (Think/Act/Observe instructions)
  - [ ] Loop: call LLM → parse response → determine step type
  - [ ] Terminate on `Answer` step or iteration limit
- [ ] Task 2: ThoughtStep emission (AC: #2, #5)
  - [ ] Store `Vec<ThoughtStep>` during execution
  - [ ] Emit `Think` when LLM produces reasoning
  - [ ] Emit `Act` when LLM requests tool call
  - [ ] Emit `Observe` with tool output
  - [ ] Emit `Answer` with final answer
  - [ ] Expose `thought_steps()` accessor method
- [ ] Task 3: Tool integration (AC: #4)
  - [ ] Route `Act` tool calls through runtime's tool registry
  - [ ] Handle tool not found → graceful error as `Observe` step
  - [ ] Handle tool execution error → error message as `Observe` step
- [ ] Task 4: Testing (AC: #1, #2, #3, #5)
  - [ ] Test complete Think→Act→Observe→Answer cycle with mock provider
  - [ ] Test max_iterations termination
  - [ ] Test ThoughtStep sequence correctness
  - [ ] Test tool call routing with mock tools

## Dev Notes

- CRITICAL: Study `AgentStrategy::execute` signature carefully — it likely takes `&self, runtime: &AgentRuntime, messages: &[Message]` or similar
- Use `AgentRuntime.provider.chat()` for LLM calls within the loop
- Parse LLM output to detect: Thought: ... Action: ... Action Input: ... Observation: ... Final Answer: ...
- Consider regex or structured parsing for LLM output
- Runtime overhead target: < 1ms per cycle excluding LLM call (NFR3)

### Project Structure Notes

- Modify: `crates/traitclaw-strategies/src/react/strategy.rs`
- Previous story 2.1 must be complete (provides builder + struct + stub)

### References

- [Source: architecture.md#Decision-2-ThoughtStep-Type-Design]
- [Source: prd.md#FR3-FR18-FR19]
- [Source: architecture.md#Error-Handling-Patterns]

## Dev Agent Record

### Agent Model Used
### Debug Log References
### Completion Notes List
### File List
