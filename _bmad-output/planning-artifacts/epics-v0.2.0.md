---
version: v0.2.0
basedOn: prd.md
inputDocuments:
  - planning-artifacts/prd.md
  - architecture.md
  - epics.md
---

# TraitClaw v0.2.0 — Epic & Story Breakdown

> "The Openness Update" — AgentStrategy, AgentHook, Router, CompressedMemory

## Requirements Traceability

| PRD Feature | Epic | Stories |
|-------------|------|---------|
| F1: AgentStrategy | Epic 1 | 1.1, 1.2, 1.3, 1.4 |
| F2: AgentHook | Epic 2 | 2.1, 2.2, 2.3, 2.4 |
| F3: Router | Epic 3 | 3.1, 3.2, 3.3 |
| F4: CompressedMemory | Epic 4 | 4.1, 4.2 |
| F5: Examples | Epic 5 | 5.1, 5.2, 5.3, 5.4 |
| Docs & Release | Epic 6 | 6.1, 6.2, 6.3 |

## Epic List

- **Epic 1: AgentStrategy** — Extract the Agent loop into a pluggable Strategy trait (Phase 1)
- **Epic 2: AgentHook** — Async lifecycle hooks for observability & interception (Phase 1)
- **Epic 3: Router** — Pluggable multi-agent routing for traitclaw-team (Phase 2)
- **Epic 4: CompressedMemory** — Decorator-pattern context compression (Phase 2)
- **Epic 5: Examples** — Dedicated examples for each new extensibility point (Phase 3)
- **Epic 6: Documentation & Release** — Migration guide, ADRs, version bump, publish (Phase 3)

---

## Epic 1: AgentStrategy

**Goal:** Extract the hardcoded agent loop from `Agent::run()` into a `Box<dyn AgentStrategy>`, enabling developers to plug in custom reasoning architectures (MCTS, ReAct, Chain-of-Thought) while preserving backward compatibility through `DefaultStrategy`.

**ADR Reference:** ADR-1 — Dynamic dispatch (`Box<dyn AgentStrategy>`). LLM latency >> vtable overhead.

### Story 1.1: Define AgentStrategy Trait

As a framework developer,
I want the `AgentStrategy` trait defined in `traitclaw-core`,
So that the core crate provides the contract for custom reasoning loops.

**Acceptance Criteria:**

- **Given** `traitclaw-core/src/traits/strategy.rs` is created
- **When** a developer inspects the trait
- **Then** it has a single method: `async fn execute(&self, runtime: &AgentRuntime, input: &str) -> Result<AgentOutput>`
- **And** `AgentRuntime` is a new struct exposing provider, tools, memory, guards, hints, tracker, config (but NOT the strategy itself — avoids recursion)
- **And** the trait requires `Send + Sync + 'static`
- **And** rustdoc includes a usage example

**Tasks:**
1. Create `strategy.rs` in traits module
2. Define `AgentRuntime` struct (extracted from `Agent` internals)
3. Define `AgentStrategy` trait
4. Add `pub mod strategy` to traits `mod.rs`
5. Unit test: trait is object-safe (`Box<dyn AgentStrategy>` compiles)

### Story 1.2: Implement DefaultStrategy

As a framework developer,
I want `DefaultStrategy` to encapsulate the current v0.1.0 loop logic,
So that existing behavior is preserved exactly.

**Acceptance Criteria:**

- **Given** `DefaultStrategy` is implemented
- **When** an Agent is built without specifying a strategy
- **Then** `DefaultStrategy` is used automatically
- **And** the loop logic is identical to v0.1.0 `Agent::run()` (copy + refactor)
- **And** all existing integration tests pass without modification
- **And** streaming is supported via `DefaultStrategy::stream()`

**Tasks:**
1. Create `DefaultStrategy` struct
2. Move loop logic from `runtime.rs` into `DefaultStrategy::execute()`
3. Ensure Guard/Hint/Tracker hooks are called at the same points
4. Run full existing test suite — zero regressions

### Story 1.3: Integrate Strategy into Agent

As a framework developer,
I want `Agent` to delegate to its strategy for execution,
So that `agent.run()` and `agent.stream()` use the pluggable strategy.

**Acceptance Criteria:**

- **Given** `Agent` struct has a `strategy: Box<dyn AgentStrategy>` field
- **When** `agent.run(input)` is called
- **Then** it delegates to `self.strategy.execute(&self.runtime, input)`
- **And** `AgentBuilder` has `.strategy(impl AgentStrategy)` method
- **And** if no strategy is set, `DefaultStrategy` is used
- **And** `.set_strategy(s)` allows swapping strategy at runtime

**Tasks:**
1. Add `strategy` field to `Agent` struct
2. Extract `AgentRuntime` from `Agent` fields
3. Modify `Agent::run()` to delegate to strategy
4. Add `.strategy()` to `AgentBuilder`
5. Backward compatibility test: existing v0.1.0 code compiles unchanged

### Story 1.4: Backward Compatibility Verification

As a framework developer,
I want to prove that all v0.1.0 examples compile and run correctly on v0.2.0,
So that the upgrade path is zero-friction.

**Acceptance Criteria:**

- **Given** all 9 existing examples (01-09)
- **When** compiled against the v0.2.0 crates
- **Then** every example compiles without any code changes
- **And** every example produces the same functional behavior
- **And** a CI test verifies this

**Tasks:**
1. Run all examples against v0.2.0 branch
2. Fix any compilation issues (without changing example code)
3. Add CI job: `cargo build --examples`

---

## Epic 2: AgentHook

**Goal:** Introduce async lifecycle hooks for observability (traces, metrics) and interception (blocking/modifying tool calls), separate from the existing Tracker system which serves the Steering subsystem.

**ADR Reference:** ADR-2 — Async hooks using Rust 1.75+ native `async fn` in traits.

### Story 2.1: Define AgentHook Trait

As a framework developer,
I want the `AgentHook` trait defined in `traitclaw-core`,
So that developers can observe and intercept agent lifecycle events.

**Acceptance Criteria:**

- **Given** `traitclaw-core/src/traits/hook.rs` is created
- **When** a developer implements `AgentHook`
- **Then** it has these async methods (all with empty default impls):
  - `on_agent_start(&self, input: &str)`
  - `on_agent_end(&self, output: &AgentOutput, duration: Duration)`
  - `on_provider_start(&self, request: &CompletionRequest)`
  - `on_provider_end(&self, response: &CompletionResponse, duration: Duration)`
  - `before_tool_execute(&self, name: &str, args: &Value) -> HookAction`
  - `after_tool_execute(&self, name: &str, result: &Value, duration: Duration)`
  - `on_stream_chunk(&self, chunk: &str)`
  - `on_error(&self, error: &AgentError)`
- **And** `HookAction` enum has `Continue` and `Block(String)` variants
- **And** the trait requires `Send + Sync + 'static`

**Tasks:**
1. Create `hook.rs` in traits module
2. Define `HookAction` enum
3. Define `AgentHook` trait with default impls
4. Add `pub mod hook` to traits `mod.rs`
5. Unit test: verify default impls compile and do nothing

### Story 2.2: Integrate Hooks into Agent Runtime

As a framework developer,
I want hooks called at the correct lifecycle points in the agent runtime,
So that hook implementations receive accurate events.

**Acceptance Criteria:**

- **Given** `Agent` has `hooks: Vec<Box<dyn AgentHook>>`
- **When** `agent.run()` executes
- **Then** `on_agent_start` is called before execution begins
- **And** `on_provider_start/end` wraps every LLM call
- **And** `before_tool_execute` is called before each tool (after Guard check)
- **And** if `before_tool_execute` returns `Block(reason)`, tool is skipped and reason is returned to LLM
- **And** `after_tool_execute` is called after each tool completes
- **And** `on_agent_end` is called when execution finishes
- **And** `on_error` is called on any error
- **And** all hooks in the Vec are called sequentially

**Tasks:**
1. Add `hooks` field to `Agent` struct
2. Add `.hook(impl AgentHook)` to `AgentBuilder`
3. Instrument `DefaultStrategy` with hook calls
4. Instrument streaming path with `on_stream_chunk`
5. Integration test: verify hook call order with a recording hook

### Story 2.3: NoopHook and LoggingHook Implementations

As a framework developer,
I want built-in hook implementations for common cases,
So that developers have ready-to-use hooks.

**Acceptance Criteria:**

- **Given** `NoopHook` (implicit — the default empty impls)
- **And** `LoggingHook` is provided
- **When** `LoggingHook` is registered
- **Then** it logs all lifecycle events using `tracing` at configurable levels
- **And** logs include: event name, duration, token counts, tool names
- **And** `LoggingHook::new(Level::INFO)` configures the log level

**Tasks:**
1. Implement `LoggingHook` using `tracing`
2. Unit tests for log output
3. Document in rustdoc with usage example

### Story 2.4: Hook Interception Test

As a framework developer,
I want a test proving hooks can block tool execution,
So that the interception mechanism is validated.

**Acceptance Criteria:**

- **Given** a `SecurityHook` that blocks tools with "dangerous" in the name
- **When** the LLM requests a tool named "dangerous_operation"
- **Then** `before_tool_execute` returns `HookAction::Block("Blocked by security policy")`
- **And** the tool is NOT executed
- **And** the block reason is sent back to LLM as the tool result
- **And** the agent continues normally with the next LLM response

**Tasks:**
1. Create `SecurityHook` test implementation
2. Integration test: agent with SecurityHook, mock tool, mock provider
3. Verify block message reaches LLM as tool result

---

## Epic 3: Router

**Goal:** Replace the hardcoded orchestration logic in `traitclaw-team` with a pluggable `Router` trait, enabling developers to build custom multi-agent workflows (sequential, leader-follower, graph-based, state machine).

**ADR Reference:** ADR-3 — Simple `trait Router` with no graph dependency.

### Story 3.1: Define Router Trait

As a framework developer,
I want the `Router` trait defined in `traitclaw-team`,
So that team orchestration routing is pluggable.

**Acceptance Criteria:**

- **Given** `traitclaw-team/src/router.rs` is created
- **When** a developer inspects the trait
- **Then** it has: `fn route(&self, message: &TeamMessage, state: &TeamState) -> RoutingDecision`
- **And** `RoutingDecision` enum has: `SendTo(AgentId)`, `Broadcast`, `Complete(String)`
- **And** `TeamMessage` contains: sender, content, metadata
- **And** `TeamState` contains: agent list, message history, current iteration
- **And** the trait requires `Send + Sync + 'static`

**Tasks:**
1. Create `router.rs` module
2. Define `TeamMessage`, `TeamState`, `RoutingDecision` types
3. Define `Router` trait
4. Unit test: trait is object-safe

### Story 3.2: Implement Default Routers

As a framework developer,
I want `SequentialRouter` and `LeaderRouter` as default implementations,
So that existing team patterns work out of the box.

**Acceptance Criteria:**

- **Given** `SequentialRouter` is implemented
- **When** used in a team
- **Then** it routes messages round-robin through all agents in order
- **And** completes when all agents have responded
- **Given** `LeaderRouter` is implemented
- **When** used in a team
- **Then** a designated leader agent receives all messages first
- **And** the leader decides which specialist agent handles each subtask
- **And** specialist responses are routed back to the leader for synthesis

**Tasks:**
1. Implement `SequentialRouter`
2. Implement `LeaderRouter`
3. Unit tests for both routers
4. Integration test with mock agents

### Story 3.3: Integrate Router into Team

As a framework developer,
I want `Team` to use `Router` for orchestration,
So that existing team logic is refactored to be pluggable.

**Acceptance Criteria:**

- **Given** `Team` struct has a `router: Box<dyn Router>` field
- **When** `Team::builder().router(SequentialRouter::new()).build()`
- **Then** the team uses the provided router for message dispatching
- **And** if no router is set, `SequentialRouter` is used by default
- **And** existing team functionality (from v0.1.0) is preserved
- **And** `Team::builder().router(CustomRouter::new())` allows full customization

**Tasks:**
1. Add `router` field to `Team` struct
2. Refactor `Team::run()` to delegate to router
3. Add `.router()` to `TeamBuilder`
4. Backward compatibility test

---

## Epic 4: CompressedMemory

**Goal:** Implement a decorator-pattern Memory wrapper that automatically summarizes old messages to prevent context window overflow, enabling agents to run indefinitely.

**ADR Reference:** ADR-4 — Decorator pattern wrapping `Memory` trait. Stackable.

### Story 4.1: Implement CompressedMemory Decorator

As a framework developer,
I want `CompressedMemory` that wraps any `Memory` implementation,
So that context is automatically compressed when approaching capacity limits.

**Acceptance Criteria:**

- **Given** `CompressedMemory::wrap(inner_memory, compressor_provider, threshold)`
- **When** `messages()` is called and message count exceeds `threshold` % of context window
- **Then** older messages (except system) are summarized into a single summary message
- **And** the summary + recent N messages are returned
- **And** the summary is generated by `compressor_provider` (a cheap LLM like gpt-4o-mini)
- **And** `append()` delegates to inner memory normally
- **And** `CompressedMemory<M>` implements `Memory` for any `M: Memory`

**Tasks:**
1. Create `CompressedMemory<M: Memory>` struct
2. Implement `Memory` trait with compression logic
3. Summary prompt template for compressor LLM
4. Unit test: verify compression triggers at threshold
5. Integration test: 100-message conversation stays within budget

### Story 4.2: Stackable Decorator Verification

As a framework developer,
I want to verify that memory decorators can be stacked,
So that developers can compose multiple memory enhancements.

**Acceptance Criteria:**

- **Given** `CompressedMemory::wrap(SqliteMemory::new(...))`
- **When** used as the agent's memory
- **Then** messages are persisted to SQLite AND compressed automatically
- **And** the following chain also works: `CompressedMemory::wrap(InMemoryMemory::new())`
- **And** API docs show stacking examples

**Tasks:**
1. Integration test: CompressedMemory wrapping SqliteMemory
2. Integration test: CompressedMemory wrapping InMemoryMemory
3. Document stacking pattern in rustdoc

---

## Epic 5: Examples

**Goal:** Provide runnable, well-documented examples for every new v0.2.0 feature.

### Story 5.1: Custom Strategy Example

As a developer,
I want `examples/10-custom-strategy/` demonstrating a simple ReAct-style strategy,
So that I can learn how to implement custom reasoning loops.

**Acceptance Criteria:**

- **Given** `examples/10-custom-strategy/` exists
- **When** I run the example
- **Then** it implements a `SimpleReActStrategy` that explicitly separates Think/Act/Observe phases
- **And** demonstrates the difference from `DefaultStrategy`
- **And** README explains the strategy pattern and when to use custom strategies
- **And** compiles and runs successfully

### Story 5.2: Lifecycle Hooks Example

As a developer,
I want `examples/11-lifecycle-hooks/` demonstrating logging and interception hooks,
So that I can learn how to add observability and security to my agents.

**Acceptance Criteria:**

- **Given** `examples/11-lifecycle-hooks/` exists
- **When** I run the example
- **Then** it shows a `TimingHook` that measures LLM call duration
- **And** shows a `CostTracker` hook that accumulates token costs
- **And** shows a `SecurityHook` that blocks specific tool calls
- **And** README explains Hook vs Tracker distinction

### Story 5.3: Custom Router Example

As a developer,
I want `examples/12-custom-router/` demonstrating a state-machine router,
So that I can learn how to build complex multi-agent workflows.

**Acceptance Criteria:**

- **Given** `examples/12-custom-router/` exists
- **When** I run the example
- **Then** it implements a `CodeReviewRouter` with states: Code → Review → (Fix/Deploy)
- **And** demonstrates conditional routing based on review results
- **And** README explains the Router pattern and when to use custom routers

### Story 5.4: Compressed Memory Example

As a developer,
I want `examples/13-compressed-memory/` showing automatic context management,
So that I can learn how to handle long-running agents.

**Acceptance Criteria:**

- **Given** `examples/13-compressed-memory/` exists
- **When** I run the example
- **Then** it shows an agent handling 50+ messages without context overflow
- **And** demonstrates the decorator wrapping pattern
- **And** README explains when and why to use CompressedMemory

---

## Epic 6: Documentation & Release

**Goal:** Complete all documentation, migration guides, and publish v0.2.0 to crates.io.

### Story 6.1: Migration Guide

As a developer upgrading from v0.1.0,
I want a migration guide explaining what changed,
So that I can upgrade safely.

**Acceptance Criteria:**

- **Given** `docs/migration-v0.1-to-v0.2.md` exists
- **When** I read it
- **Then** it confirms: "No breaking changes. v0.1.0 code compiles unchanged."
- **And** lists new features with one-paragraph explanations
- **And** shows "before (v0.1.0)" vs "after (v0.2.0)" code snippets for each new feature

### Story 6.2: Architecture Decision Records

As a developer,
I want ADRs documenting the v0.2.0 design decisions,
So that future contributors understand the rationale.

**Acceptance Criteria:**

- **Given** `docs/adr/` directory exists
- **When** I read the ADRs
- **Then** ADR-1 (Strategy dispatch), ADR-2 (Hook concurrency), ADR-3 (Router design), ADR-4 (Memory decorator) are documented
- **And** each ADR follows the format: Context → Decision → Consequences

### Story 6.3: Version Bump & Publish

As a framework maintainer,
I want all crates bumped to v0.2.0 and published to crates.io,
So that the community can use the new features.

**Acceptance Criteria:**

- **Given** all 12 crate `Cargo.toml` files
- **When** version is bumped from 0.1.0 to 0.2.0
- **Then** all inter-crate dependency versions are updated consistently
- **And** `cargo build` succeeds
- **And** `cargo test` passes
- **And** `scripts/publish.sh` successfully publishes all 12 crates
- **And** README is updated with v0.2.0 features

---

## Implementation Order (Dependency Graph)

```
Phase 1 (Week 1-2):
  Epic 1: Story 1.1 → 1.2 → 1.3 → 1.4
  Epic 2: Story 2.1 → 2.2 → 2.3 → 2.4

Phase 2 (Week 3-4):
  Epic 3: Story 3.1 → 3.2 → 3.3
  Epic 4: Story 4.1 → 4.2

Phase 3 (Week 5-7):
  Epic 5: Stories 5.1, 5.2, 5.3, 5.4 (parallel)
  Epic 6: Story 6.1 → 6.2 → 6.3
```

**Critical Path:** Epic 1 (Strategy) must complete before Epic 2 (Hook) can be fully integrated, because hooks are called from within the strategy's execute method.
