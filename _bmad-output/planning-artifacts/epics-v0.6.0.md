---
stepsCompleted: ["step-01-validate-prerequisites.md", "step-02-design-epics.md", "step-03-create-stories.md", "step-04-final-validation.md"]
inputDocuments: ["prd-v0.6.0.md", "architecture.md"]
---

# TraitClaw v0.6.0 "Composition" - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for TraitClaw v0.6.0 "Composition", decomposing the requirements from the PRD and Architecture requirements into implementable stories.

## Requirements Inventory

### Functional Requirements

FR1: `Agent::with_system(provider, system_prompt)` — Single-line agent creation shorthand equivalent to `Agent::builder().provider(p).system(s).build()?`, infallible (no configuration that can fail at this scope), purely additive to existing API.

FR2: `AgentFactory<P: Provider + Clone>` struct with `new(provider)` constructor — shared-provider factory that holds provider config for reuse across multiple agents.

FR3: `AgentFactory::spawn(system_prompt)` — Creates a fully-configured `Agent` with the factory's provider (cloned) and the given system prompt.

FR4: `AgentFactory::spawn_with(closure)` — Escape hatch for full `AgentBuilder` customization while still using the factory's shared provider.

FR5: `AgentPool::new(agents: Vec<Agent>)` — Creates an agent pool from a pre-built vector of agents with owned semantics.

FR6: `AgentPool::from_team(team: &Team, provider: P)` — Maps each `AgentRole` in a `Team` to a live `Agent` using the role's system_prompt, returns `Result` with clear error if roles are missing system_prompt.

FR7: `AgentPool::run_sequential(input)` — Runs agents in order, each receives previous agent's output as input. Returns final `AgentOutput`.

FR8: `AgentPool::get(index)` and `AgentPool::len()` — Accessor methods for pool introspection.

FR9: `RoundRobinGroupChat::new(agents)` — Creates a round-robin multi-agent group chat with shared conversation history, default `max_rounds = n_agents × 3`.

FR10: `RoundRobinGroupChat::with_max_rounds(n)` — Builder-style method to customize maximum rounds.

FR11: `RoundRobinGroupChat::run(task)` — Executes round-robin conversation where each agent sees full history, returns `GroupChatResult` with transcript + final message.

FR12: `TerminationCondition` trait — Pluggable termination condition for group chats (default: `MaxRoundsTermination`).

FR13: Example `24-agent-factory/` — Progressive runnable example demonstrating: (1) `Agent::with_system()`, (2) `AgentFactory::spawn()`, (3) `AgentPool::from_team()`, (4) `RoundRobinGroupChat`.

FR14: Migration guide `docs/migration-v0.5-to-v0.6.md` documenting all new APIs and upgrade path.

FR15: All new types re-exported via `traitclaw::prelude::*`.

FR16: README updated with "Multi-Agent Quickstart" section showing the 3-tier API.

### NonFunctional Requirements

NFR1: **Performance** — `AgentFactory::spawn()` overhead < 1μs vs direct builder (thin wrapper, zero heap allocation beyond `provider.clone()`).

NFR2: **Performance** — `AgentPool::from_team()` is O(n) where n = number of roles, no async work at construction time.

NFR3: **Performance** — `RoundRobinGroupChat` conversation history stored as `Vec<Message>`, same as existing agent conversation tracking.

NFR4: **Backward Compatibility** — All v0.5.0 code compiles and runs on v0.6.0 without modification. Verified by running all 20 existing examples in CI.

NFR5: **MSRV** — Rust 1.75+ maintained (existing requirement, no change).

NFR6: **Semver** — Minor version bump (0.5.0 → 0.6.0). Zero breaking changes.

NFR7: **API Surface** — No new traits. New types only: `AgentFactory`, `AgentPool`, `RoundRobinGroupChat`, `GroupChatResult`, `TerminationCondition`. ≤ 5 new types/methods total.

NFR8: **Compile Time** — Build time increase < 2% measured with `cargo build --timings`.

NFR9: **Thread Safety** — `AgentFactory: Send + Sync` when `P: Send + Sync`.

NFR10: **Documentation** — All new public types and methods have rustdoc with at least one `# Example` block.

NFR11: **Testing** — Unit tests for `AgentFactory::spawn()` with mock provider, unit tests for `AgentPool::from_team()` verifying role→agent mapping, integration test for `RoundRobinGroupChat` with 2 mock agents and 2 rounds.

NFR12: **Dependencies** — No new dependencies for any of the four features. Implemented purely in existing crates.

### Additional Requirements

- ADR-18: `Provider: Clone` bound on `AgentFactory` — factory must give each agent its own provider instance. `Arc<dyn Provider>` already impls Clone.
- ADR-19: `AgentPool` takes `Vec<Agent>` (owned) — consistent with existing `Team::bind()` ownership model, prevents shared mutable state.
- ADR-20: `RoundRobinGroupChat` in `traitclaw-team` (not `traitclaw-core`) — multi-agent coordination is a team concern, core stays minimal.
- ADR-21: No `AgentFactory` trait, only concrete struct — factory behavior doesn't need polymorphism. YAGNI.
- Crate impact: `traitclaw-core` (Agent::with_system, AgentFactory, AgentPool), `traitclaw-team` (RoundRobinGroupChat, GroupChatResult, TerminationCondition), `traitclaw` meta-crate (prelude re-exports).
- No new feature flags required for core composition APIs.
- Provider uses `Arc<dyn Provider>` internally — Clone is already available via Arc.

### UX Design Requirements

N/A — TraitClaw is a Rust library/framework. No UI/UX design document applicable.

### FR Coverage Map

| FR | Epic | Description |
|----|:----:|-------------|
| FR1 | 1 | `Agent::with_system()` shorthand |
| FR2 | 2 | `AgentFactory::new()` constructor |
| FR3 | 2 | `AgentFactory::spawn()` |
| FR4 | 2 | `AgentFactory::spawn_with()` |
| FR5 | 3 | `AgentPool::new()` |
| FR6 | 3 | `AgentPool::from_team()` |
| FR7 | 3 | `AgentPool::run_sequential()` |
| FR8 | 3 | `AgentPool::get/len` accessors |
| FR9 | 4 | `RoundRobinGroupChat::new()` |
| FR10 | 4 | `with_max_rounds()` |
| FR11 | 4 | `RoundRobinGroupChat::run()` |
| FR12 | 4 | `TerminationCondition` trait |
| FR13 | 5 | Example `24-agent-factory/` |
| FR14 | 5 | Migration guide |
| FR15 | 1-4 | Prelude re-exports (each epic) |
| FR16 | 5 | README update |

## Epic List

### Epic 1: Single-Agent Ergonomics
Developer creates an agent with just 1 line of code instead of 5-7 lines of builder boilerplate — the simplest possible entry point for TraitClaw.
**FRs covered:** FR1, FR15

### Epic 2: Factory-Powered Multi-Agent Creation
Developer creates multiple agents sharing the same provider with `factory.spawn()` — reducing 80% of boilerplate when 3-5 agents are needed from the same LLM provider.
**FRs covered:** FR2, FR3, FR4, FR15

### Epic 3: Agent Pool & Team Execution
Developer creates agent groups from `Team` definitions and runs sequential pipelines — bridging the gap between TraitClaw's declarative team model and runtime execution.
**FRs covered:** FR5, FR6, FR7, FR8, FR15

### Epic 4: Round-Robin Group Chat
Developer runs multi-turn collaboration between agents where each agent sees full conversation history and the chat terminates automatically based on configurable conditions.
**FRs covered:** FR9, FR10, FR11, FR12, FR15

### Epic 5: Documentation, Examples & Release Readiness
Developer discovers and learns the right API through README, progressive example, and migration guide — enabling adoption and smooth upgrades from v0.5.0.
**FRs covered:** FR13, FR14, FR16

---

## Epic 1: Single-Agent Ergonomics

Developer creates an agent with just 1 line of code instead of 5-7 lines of builder boilerplate — the simplest possible entry point for TraitClaw.

### Story 1.1: Implement `Agent::with_system()` Shorthand

As a **Rust developer building AI agents**,
I want to create an agent with `Agent::with_system(provider, "system prompt")` in a single line,
So that I can skip the builder pattern for simple single-agent use cases.

**Acceptance Criteria:**

**Given** a valid `Provider` instance and a system prompt string
**When** `Agent::with_system(provider, "You are a helpful assistant.")` is called
**Then** an `Agent` is returned with the provider and system prompt configured
**And** the agent behaves identically to one created via `Agent::builder().provider(p).system(s).build()?`

**Given** any type implementing `Into<String>` (e.g., `&str`, `String`)
**When** passed as the `system` parameter
**Then** it is accepted without explicit conversion

**Given** `Agent::with_system()` is called
**When** compared to the existing `Agent::builder()` API
**Then** the builder API is unchanged — no existing methods are modified or removed (purely additive)

**Given** a unit test with a mock provider
**When** `Agent::with_system(mock_provider, "test prompt")` is called
**Then** the resulting agent has the correct system prompt and provider configured

### Story 1.2: Re-export and Document `Agent::with_system()`

As a **Rust developer using the `traitclaw` meta-crate**,
I want `Agent::with_system()` to be available via `traitclaw::prelude::*` with full rustdoc,
So that I can discover and use the shorthand without additional imports.

**Acceptance Criteria:**

**Given** `use traitclaw::prelude::*;` is in scope
**When** `Agent::with_system(provider, "prompt")` is called
**Then** it compiles and works correctly without additional imports

**Given** the `Agent::with_system()` method
**When** viewed in `cargo doc`
**Then** it has rustdoc with at least one `# Example` block showing usage

**Given** the method documentation
**When** a developer reads it
**Then** it clearly explains the equivalence to `Agent::builder().provider(p).system(s).build()?`

---

## Epic 2: Factory-Powered Multi-Agent Creation

Developer creates multiple agents sharing the same provider with `factory.spawn()` — reducing 80% of boilerplate when 3-5 agents are needed from the same LLM provider.

### Story 2.1: Implement `AgentFactory` Struct with `new()` and `spawn()`

As a **Rust developer creating multiple agents for a pipeline**,
I want to create an `AgentFactory` from a provider and spawn agents with `factory.spawn("system prompt")`,
So that I don't repeat provider configuration for each agent.

**Acceptance Criteria:**

**Given** a provider implementing `Provider + Clone`
**When** `AgentFactory::new(provider)` is called
**Then** an `AgentFactory` is returned holding the provider configuration

**Given** an `AgentFactory` instance
**When** `factory.spawn("You are a researcher.")` is called
**Then** an `Agent` is returned with the factory's provider (cloned) and the given system prompt

**Given** an `AgentFactory` instance
**When** `spawn()` is called multiple times with different system prompts
**Then** each returned agent has its own cloned provider instance and unique system prompt
**And** agents are fully independent (modifying one does not affect others)

**Given** a provider type that is `Send + Sync`
**When** `AgentFactory` is created with it
**Then** the factory is also `Send + Sync` (NFR9)

**Given** `factory.spawn()` is called
**When** measured against direct builder usage
**Then** overhead is < 1μs — only one `provider.clone()` + builder call, zero additional heap allocation (NFR1)

### Story 2.2: Implement `AgentFactory::spawn_with()` Escape Hatch

As a **Rust developer who needs custom agent configuration beyond a system prompt**,
I want to use `factory.spawn_with(|builder| builder.system("prompt").tool(my_tool))`,
So that I can customize individual agents while still sharing the factory's provider.

**Acceptance Criteria:**

**Given** an `AgentFactory` instance
**When** `factory.spawn_with(|b| b.system("prompt").tool(MyTool))` is called
**Then** an `Agent` is returned with the factory's provider, the system prompt, AND the custom tool

**Given** the closure receives an `AgentBuilder`
**When** the closure applies any valid builder method
**Then** the resulting agent includes all customizations from the closure

**Given** `spawn_with()` is called
**When** the closure does NOT call `.system()`
**Then** the agent is created without a system prompt (no implicit default)

### Story 2.3: Re-export and Document `AgentFactory`

As a **Rust developer using the `traitclaw` meta-crate**,
I want `AgentFactory` available via `traitclaw::prelude::*` with full rustdoc,
So that I can discover the factory pattern and understand the `Provider: Clone` requirement.

**Acceptance Criteria:**

**Given** `use traitclaw::prelude::*;` is in scope
**When** `AgentFactory::new(provider)` is called
**Then** it compiles and works correctly

**Given** `AgentFactory`, `spawn()`, and `spawn_with()` methods
**When** viewed in `cargo doc`
**Then** each has rustdoc with at least one `# Example` block

**Given** the `AgentFactory` documentation
**When** a developer reads it
**Then** it clearly explains the `Provider: Clone` bound and notes that `Arc<dyn Provider>` satisfies it

**Given** a unit test with a mock `Clone + Provider`
**When** `AgentFactory::new()` and `spawn()` are tested
**Then** the test verifies correct provider cloning and system prompt assignment

---

## Epic 3: Agent Pool & Team Execution

Developer creates agent groups from `Team` definitions and runs sequential pipelines — bridging the gap between TraitClaw's declarative team model and runtime execution.

### Story 3.1: Implement `AgentPool::new()` and Accessor Methods

As a **Rust developer orchestrating multiple agents**,
I want to create an `AgentPool` from a vector of agents and inspect it with `get()` and `len()`,
So that I can group agents for sequential execution.

**Acceptance Criteria:**

**Given** a `Vec<Agent>` of pre-built agents
**When** `AgentPool::new(agents)` is called
**Then** an `AgentPool` is returned that takes ownership of the agents

**Given** an `AgentPool` with 3 agents
**When** `pool.len()` is called
**Then** it returns `3`

**Given** an `AgentPool` with 3 agents
**When** `pool.get(1)` is called
**Then** it returns `Some(&Agent)` referencing the second agent

**Given** an `AgentPool` with 3 agents
**When** `pool.get(5)` is called
**Then** it returns `None`

### Story 3.2: Implement `AgentPool::run_sequential()`

As a **Rust developer building a content pipeline**,
I want to run agents sequentially where each agent receives the previous agent's output,
So that I can chain agent tasks (e.g., research → write → review).

**Acceptance Criteria:**

**Given** an `AgentPool` with agents [A, B, C] and an input string
**When** `pool.run_sequential("initial input").await` is called
**Then** agent A runs with "initial input", agent B runs with A's output, agent C runs with B's output
**And** the final `AgentOutput` from agent C is returned

**Given** any agent in the pool returns an error during execution
**When** `run_sequential()` is running
**Then** execution stops immediately and the error is propagated as `Err`

**Given** an `AgentPool` with 1 agent
**When** `run_sequential()` is called
**Then** it runs the single agent and returns its output (edge case)

### Story 3.3: Implement `AgentPool::from_team()`

As a **Rust developer who already has `Team` definitions with `AgentRole`s**,
I want to create an `AgentPool` directly from a `Team` and a provider,
So that I don't manually bind each role to an agent instance.

**Acceptance Criteria:**

**Given** a `Team` with 3 `AgentRole`s, each having a `system_prompt` set
**When** `AgentPool::from_team(&team, provider)` is called
**Then** an `AgentPool` with 3 agents is returned, each configured with its role's system_prompt

**Given** a `Team` where one `AgentRole` has no `system_prompt`
**When** `AgentPool::from_team(&team, provider)` is called
**Then** it returns `Err` with a clear error message listing which role(s) are missing system_prompt

**Given** a valid `Team` and a `Provider + Clone` instance
**When** `from_team()` creates agents
**Then** each agent receives a cloned provider (O(n) construction, no async work — NFR2)

**Given** the `AgentPool` created from a team
**When** `pool.len()` is called
**Then** it matches the number of roles in the team

### Story 3.4: Re-export and Document `AgentPool`

As a **Rust developer using the `traitclaw` meta-crate**,
I want `AgentPool` available via `traitclaw::prelude::*` with full rustdoc,
So that I can discover the pool API and understand team binding.

**Acceptance Criteria:**

**Given** `use traitclaw::prelude::*;` is in scope
**When** `AgentPool::new(agents)` or `AgentPool::from_team(&team, provider)` is called
**Then** it compiles and works correctly

**Given** `AgentPool`, `new()`, `from_team()`, `run_sequential()`, `get()`, and `len()`
**When** viewed in `cargo doc`
**Then** each has rustdoc with at least one `# Example` block

**Given** a unit test suite
**When** `AgentPool::from_team()` is tested with a mock Team and mock Provider
**Then** it verifies correct role→agent mapping and error on missing system_prompt

---

## Epic 4: Round-Robin Group Chat

Developer runs multi-turn collaboration between agents where each agent sees full conversation history and the chat terminates automatically based on configurable conditions.

### Story 4.1: Implement `TerminationCondition` Trait and `MaxRoundsTermination`

As a **Rust developer building multi-agent systems**,
I want a pluggable `TerminationCondition` trait with a default `MaxRoundsTermination`,
So that group chats terminate automatically based on configurable criteria.

**Acceptance Criteria:**

**Given** the `TerminationCondition` trait
**When** implemented
**Then** it is defined in `traitclaw-team` crate (ADR-20)
**And** it has a method to check whether the chat should terminate based on current state

**Given** `MaxRoundsTermination::new(max_rounds)`
**When** the round count reaches `max_rounds`
**Then** the termination condition returns true

**Given** `MaxRoundsTermination::new(6)` (e.g., 2 agents × 3)
**When** only 4 rounds have occurred
**Then** the termination condition returns false

### Story 4.2: Implement `RoundRobinGroupChat` Core

As a **Rust developer building collaborative agent systems**,
I want to create a `RoundRobinGroupChat` where agents take turns responding with shared conversation history,
So that multiple agents can collaborate on a task.

**Acceptance Criteria:**

**Given** a `Vec<Agent>` with agents [A, B, C]
**When** `RoundRobinGroupChat::new(agents)` is called
**Then** a group chat is created with default `max_rounds = n_agents × 3` (= 9)

**Given** a `RoundRobinGroupChat` instance
**When** `chat.with_max_rounds(6)` is called
**Then** the max rounds is updated to 6 (builder-style, returns `self`)

**Given** a `RoundRobinGroupChat` with agents [A, B]
**When** `chat.run("Discuss Rust async").await` is called
**Then** agent A responds first, then B, then A again, cycling in round-robin order
**And** each agent sees the full prior conversation history (all previous responses)
**And** the chat terminates when the termination condition is met

**Given** `chat.run()` completes
**When** the result is inspected
**Then** it returns `GroupChatResult` containing the full transcript (`Vec<Message>`) and the final message

**Given** conversation history during a group chat
**When** stored
**Then** it uses `Vec<Message>` same as existing agent conversation tracking (NFR3)

### Story 4.3: Re-export and Document Group Chat Types

As a **Rust developer using the `traitclaw` meta-crate**,
I want `RoundRobinGroupChat`, `GroupChatResult`, and `TerminationCondition` available via `traitclaw::prelude::*`,
So that I can use group chat without importing from sub-crates.

**Acceptance Criteria:**

**Given** `use traitclaw::prelude::*;` is in scope
**When** `RoundRobinGroupChat::new(agents)` is called
**Then** it compiles and works correctly

**Given** `RoundRobinGroupChat`, `GroupChatResult`, `TerminationCondition`, and `MaxRoundsTermination`
**When** viewed in `cargo doc`
**Then** each has rustdoc with at least one `# Example` block

**Given** the integration test suite
**When** a test for `RoundRobinGroupChat` is run with 2 mock agents and 2 rounds
**Then** the correct round-robin order is verified and `GroupChatResult` contains the full transcript (NFR11)

---

## Epic 5: Documentation, Examples & Release Readiness

Developer discovers and learns the right API through README, progressive example, and migration guide — enabling adoption and smooth upgrades from v0.5.0.

### Story 5.1: Create Example `24-agent-factory/`

As a **Rust developer evaluating TraitClaw for multi-agent use cases**,
I want a runnable example that demonstrates all composition APIs progressively,
So that I can see them in action and adapt the patterns to my use case.

**Acceptance Criteria:**

**Given** the example directory `examples/24-agent-factory/`
**When** `cargo run -p agent-factory` is executed
**Then** the example compiles and runs, demonstrating all four APIs in sequence

**Given** the example code
**When** read by a developer
**Then** it progressively demonstrates: (1) `Agent::with_system()`, (2) `AgentFactory::spawn()`, (3) `AgentPool::from_team()`, (4) `RoundRobinGroupChat`

**Given** the example runs
**When** output is printed
**Then** it produces meaningful output showing each API stage (not just "ok")

### Story 5.2: Create Migration Guide `docs/migration-v0.5-to-v0.6.md`

As a **Rust developer upgrading from TraitClaw v0.5.0**,
I want a clear migration guide explaining what's new and how to adopt the new APIs,
So that I can upgrade with confidence and optionally adopt new patterns.

**Acceptance Criteria:**

**Given** the migration guide
**When** read by a v0.5.0 user
**Then** it clearly states: no breaking changes, existing code compiles without modification

**Given** the migration guide
**When** reviewed
**Then** it documents all new types (`AgentFactory`, `AgentPool`, `RoundRobinGroupChat`) with before/after code comparisons

**Given** the guide's "Optional Adoption" section
**When** read
**Then** it shows how to incrementally adopt `Agent::with_system()` in new code while legacy builder code stays untouched

### Story 5.3: Update README with Multi-Agent Quickstart

As a **potential TraitClaw user visiting the repository**,
I want a "Multi-Agent Quickstart" section in the README showing the 3-tier API,
So that I can immediately understand TraitClaw's multi-agent ergonomics.

**Acceptance Criteria:**

**Given** the README.md
**When** updated
**Then** it contains a "Multi-Agent Quickstart" section with working code examples

**Given** the quickstart code
**When** read
**Then** it shows all 3 tiers: (1) `Agent::with_system()`, (2) `AgentFactory::spawn()`, (3) `AgentPool`/`RoundRobinGroupChat`

**Given** the README
**When** reviewed
**Then** the quickstart fits in a single code block (≤ 15 lines) for maximum impact

### Story 5.4: Backward Compatibility Verification

As a **TraitClaw maintainer**,
I want all 20 existing v0.5.0 examples to compile and run on v0.6.0,
So that we guarantee zero breaking changes (NFR4).

**Acceptance Criteria:**

**Given** all 20 existing examples from v0.5.0
**When** `cargo build --examples` is run against v0.6.0
**Then** all examples compile successfully with zero errors and zero warnings

**Given** `cargo build --timings` is run
**When** compared to v0.5.0 baseline
**Then** compile time increase is < 2% (NFR8)

**Given** the API surface of v0.6.0
**When** compared to v0.5.0
**Then** no existing public types, methods, or traits have been removed or changed in signature (NFR6)
