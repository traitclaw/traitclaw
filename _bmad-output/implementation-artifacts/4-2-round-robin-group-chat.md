# Story 4.2: Implement `RoundRobinGroupChat` Core

Status: ready-for-dev

## Story

As a **Rust developer building collaborative agent systems**,
I want to create a `RoundRobinGroupChat` where agents take turns responding with shared conversation history,
so that multiple agents can collaborate on a task.

## Acceptance Criteria

1. Given a `Vec<Agent>` with agents [A, B, C], when `RoundRobinGroupChat::new(agents)` is called, then a group chat is created with default `max_rounds = n_agents × 3`.

2. Given a `RoundRobinGroupChat` instance, when `chat.with_max_rounds(6)` is called, then the max rounds is updated to 6 (builder-style, returns `self`).

3. Given a `RoundRobinGroupChat` with agents [A, B], when `chat.run("Discuss Rust async").await` is called, then agent A responds first, then B, then A again, cycling in round-robin order, each agent sees the full prior conversation history, and the chat terminates when the termination condition is met.

4. Given `chat.run()` completes, when the result is inspected, then it returns `GroupChatResult` containing the full transcript (`Vec<Message>`) and the final message.

5. Given conversation history during a group chat, when stored, then it uses `Vec<Message>` same as existing agent conversation tracking (NFR3).

## Tasks / Subtasks

- [ ] Task 1: Define `GroupChatResult` struct (AC: #4)
  - [ ] `pub struct GroupChatResult { pub transcript: Vec<Message>, pub final_message: String }`
  - [ ] Implement `Debug`
- [ ] Task 2: Create `RoundRobinGroupChat` struct (AC: #1, #2)
  - [ ] `agents: Vec<Agent>`, `termination: Box<dyn TerminationCondition>`
  - [ ] `new(agents: Vec<Agent>) -> Self` — default termination = MaxRoundsTermination(n_agents * 3)
  - [ ] `with_max_rounds(mut self, n: usize) -> Self` builder method
  - [ ] `with_termination(mut self, t: impl TerminationCondition + 'static) -> Self`
- [ ] Task 3: Implement `run()` (AC: #3, #4, #5)
  - [ ] `pub async fn run(&mut self, task: &str) -> Result<GroupChatResult>`
  - [ ] Initialize transcript with task as first user message
  - [ ] Loop: pick agent by round % n_agents, inject transcript as context, run agent, append response
  - [ ] Check termination condition after each round
  - [ ] Return GroupChatResult with full transcript
- [ ] Task 4: Integration tests
  - [ ] Test 2 agents, 2 rounds
  - [ ] Test termination condition triggers correctly
  - [ ] Test round-robin order is correct

## Dev Notes

- **Crate:** `traitclaw-team` (ADR-20)
- **Key complexity:** Each agent needs to see the full conversation transcript as context. Check how `Agent::run()` handles message history — may need to inject messages into agent's memory or pass as system/user messages.
- **`&mut self`** required because agents are mutated during `run()`.
- **Depends on:** Story 4-1 (TerminationCondition trait).
- **NFR3:** Use `Vec<Message>` for history — same format as existing agent conversation.

### Project Structure Notes

- File: `crates/traitclaw-team/src/group_chat.rs`
- Types: `RoundRobinGroupChat`, `GroupChatResult`

### References

- [Source: _bmad-output/planning-artifacts/epics-v0.6.0.md#Story 4.2]
- [Source: _bmad-output/planning-artifacts/prd-v0.6.0.md#FR9, FR10, FR11]
- [Source: _bmad-output/planning-artifacts/epics-v0.6.0.md#ADR-20]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
