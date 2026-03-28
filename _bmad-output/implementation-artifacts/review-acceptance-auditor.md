@[/bmad-review-acceptance-auditor]
You are an Acceptance Auditor. Review this diff against the spec and context docs. Check for: violations of acceptance criteria, deviations from spec intent, missing implementation of specified behavior, contradictions between spec constraints and actual code. Output findings as a markdown list. Each finding: one-line title, which AC/constraint it violates, and evidence from the diff.

========== SPEC: epics-v0.6.0.md ==========
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


========== DIFF: Chunk 1 (Core API) ==========
```diff
diff --git a/crates/traitclaw-core/src/agent.rs b/crates/traitclaw-core/src/agent.rs
index 439e255..11b739a 100644
--- a/crates/traitclaw-core/src/agent.rs
+++ b/crates/traitclaw-core/src/agent.rs
@@ -163,6 +163,43 @@ impl Agent {
         AgentBuilder::new()
     }
 
+    /// Create an agent with just a provider and system prompt.
+    ///
+    /// This is a convenience shorthand equivalent to:
+    /// ```rust,ignore
+    /// Agent::builder()
+    ///     .provider(provider)
+    ///     .system(system)
+    ///     .build()
+    ///     .unwrap()
+    /// ```
+    ///
+    /// All other settings use their defaults (in-memory memory, no tools,
+    /// no guards, etc.). Use [`Agent::builder()`] for full customization.
+    ///
+    /// # Example
+    ///
+    /// ```rust,no_run
+    /// use traitclaw_core::prelude::*;
+    ///
+    /// # fn example(provider: impl traitclaw_core::traits::provider::Provider) {
+    /// let agent = Agent::with_system(provider, "You are a helpful assistant.");
+    /// # }
+    /// ```
+    /// # Panics
+    ///
+    /// This method cannot panic under normal usage — the internal `build()`
+    /// call only fails when no provider is set, and `with_system` always
+    /// provides one.
+    #[must_use]
+    pub fn with_system(provider: impl Provider, system: impl Into<String>) -> Self {
+        Agent::builder()
+            .provider(provider)
+            .system(system)
+            .build()
+            .expect("Agent::with_system is infallible: provider is always set")
+    }
+
     /// Create an agent directly (prefer using `builder()`).
     #[allow(clippy::too_many_arguments)]
     pub(crate) fn new(
@@ -531,4 +568,81 @@ mod tests {
         assert_eq!(out.usage.iterations, 5);
         assert_eq!(out.usage.duration.as_millis(), 500);
     }
+
+    // --- Agent::with_system() tests (Story 1.1) ---
+
+    use crate::types::completion::{CompletionRequest, CompletionResponse, ResponseContent, Usage};
+    use crate::types::model_info::{ModelInfo, ModelTier};
+    use crate::types::stream::CompletionStream;
+    use async_trait::async_trait;
+
+    struct MockProvider {
+        info: ModelInfo,
+    }
+
+    impl MockProvider {
+        fn new() -> Self {
+            Self {
+                info: ModelInfo::new("mock", ModelTier::Small, 4_096, false, false, false),
+            }
+        }
+    }
+
+    #[async_trait]
+    impl crate::traits::provider::Provider for MockProvider {
+        async fn complete(&self, _req: CompletionRequest) -> crate::Result<CompletionResponse> {
+            Ok(CompletionResponse {
+                content: ResponseContent::Text("ok".into()),
+                usage: Usage {
+                    prompt_tokens: 1,
+                    completion_tokens: 1,
+                    total_tokens: 2,
+                },
+            })
+        }
+        async fn stream(&self, _req: CompletionRequest) -> crate::Result<CompletionStream> {
+            unimplemented!()
+        }
+        fn model_info(&self) -> &ModelInfo {
+            &self.info
+        }
+    }
+
+    #[test]
+    fn test_with_system_str_prompt() {
+        // AC #1, #2: with_system accepts &str and creates a valid agent
+        let agent = Agent::with_system(MockProvider::new(), "You are helpful.");
+        assert_eq!(
+            agent.config.system_prompt.as_deref(),
+            Some("You are helpful.")
+        );
+    }
+
+    #[test]
+    fn test_with_system_string_prompt() {
+        // AC #2: with_system accepts String
+        let prompt = String::from("You are a researcher.");
+        let agent = Agent::with_system(MockProvider::new(), prompt);
+        assert_eq!(
+            agent.config.system_prompt.as_deref(),
+            Some("You are a researcher.")
+        );
+    }
+
+    #[test]
+    fn test_with_system_builder_unchanged() {
+        // AC #3: builder API is unchanged (still works)
+        let result = Agent::builder()
+            .provider(MockProvider::new())
+            .system("test")
+            .build();
+        assert!(result.is_ok());
+    }
+
+    #[test]
+    fn test_with_system_provider_configured() {
+        // AC #4: agent has correct provider
+        let agent = Agent::with_system(MockProvider::new(), "test");
+        assert_eq!(agent.provider.model_info().name, "mock");
+    }
 }
diff --git a/crates/traitclaw-core/src/agent_builder.rs b/crates/traitclaw-core/src/agent_builder.rs
index 1d670eb..4e3f216 100644
--- a/crates/traitclaw-core/src/agent_builder.rs
+++ b/crates/traitclaw-core/src/agent_builder.rs
@@ -97,6 +97,16 @@ impl AgentBuilder {
         self
     }
 
+    /// Set the LLM provider from a pre-wrapped `Arc<dyn Provider>`.
+    ///
+    /// Use this when you already hold a shared provider reference
+    /// (e.g., from [`AgentFactory`](crate::factory::AgentFactory)).
+    #[must_use]
+    pub fn provider_arc(mut self, provider: Arc<dyn Provider>) -> Self {
+        self.provider = Some(provider);
+        self
+    }
+
     /// Set the LLM provider — preferred alias for [`.provider()`][Self::provider].
     ///
     /// Enables the idiomatic `Agent::builder().model(provider).system("...").build()` pattern.
diff --git a/crates/traitclaw-core/src/factory.rs b/crates/traitclaw-core/src/factory.rs
new file mode 100644
index 0000000..8b7534a
--- /dev/null
+++ b/crates/traitclaw-core/src/factory.rs
@@ -0,0 +1,242 @@
+//! Agent factory for shared-provider multi-agent creation.
+//!
+//! `AgentFactory` holds a provider configuration and spawns agents from it,
+//! eliminating repeated builder boilerplate when creating multiple agents
+//! from the same provider.
+
+use std::sync::Arc;
+
+use crate::agent::Agent;
+use crate::agent_builder::AgentBuilder;
+use crate::traits::provider::Provider;
+use crate::Result;
+
+/// A factory for creating multiple agents from a shared provider.
+///
+/// `AgentFactory` solves the "N agents from one provider" problem:
+/// instead of repeating `.provider(p)` for each agent, create
+/// a factory once and call [`spawn()`](Self::spawn) with different prompts.
+///
+/// # Example
+///
+/// ```rust,no_run
+/// use traitclaw_core::factory::AgentFactory;
+/// use traitclaw_core::traits::provider::Provider;
+///
+/// # fn example(provider: impl Provider) {
+/// let factory = AgentFactory::new(provider);
+///
+/// let researcher = factory.spawn("You are a researcher.");
+/// let writer = factory.spawn("You are a technical writer.");
+/// let reviewer = factory.spawn("You are a code reviewer.");
+/// // All three agents share the same provider config (via Arc)
+/// # }
+/// ```
+///
+/// ## How It Works
+///
+/// The factory wraps the provider in `Arc<dyn Provider>`, which is
+/// cheaply cloneable. Each [`spawn()`](Self::spawn) call clones the Arc
+/// (incrementing the reference count) and creates a new agent.
+#[derive(Clone)]
+pub struct AgentFactory {
+    provider: Arc<dyn Provider>,
+}
+
+impl std::fmt::Debug for AgentFactory {
+    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
+        f.debug_struct("AgentFactory")
+            .field("model", &self.provider.model_info().name)
+            .finish()
+    }
+}
+
+impl AgentFactory {
+    /// Create a new factory from a provider.
+    ///
+    /// The provider is wrapped in an `Arc` for cheap cloning. Each
+    /// spawned agent shares the same underlying provider instance.
+    #[must_use]
+    pub fn new(provider: impl Provider) -> Self {
+        Self {
+            provider: Arc::new(provider),
+        }
+    }
+
+    /// Create a factory from an already-wrapped `Arc<dyn Provider>`.
+    ///
+    /// Use this when you already hold a shared provider reference.
+    #[must_use]
+    pub fn from_arc(provider: Arc<dyn Provider>) -> Self {
+        Self { provider }
+    }
+
+    /// Spawn an agent with the factory's provider and a system prompt.
+    ///
+    /// Each spawned agent holds its own `Arc` clone of the provider,
+    /// making agents fully independent (cheap reference-counted sharing).
+    ///
+    /// # Example
+    ///
+    /// ```rust,no_run
+    /// use traitclaw_core::factory::AgentFactory;
+    /// use traitclaw_core::traits::provider::Provider;
+    ///
+    /// # fn example(provider: impl Provider) {
+    /// let factory = AgentFactory::new(provider);
+    /// let agent = factory.spawn("You are a helpful assistant.");
+    /// # }
+    /// ```
+    ///
+    /// # Panics
+    ///
+    /// This method cannot panic under normal usage — the internal builder
+    /// always has a valid provider.
+    #[must_use]
+    pub fn spawn(&self, system: impl Into<String>) -> Agent {
+        AgentBuilder::new()
+            .provider_arc(Arc::clone(&self.provider))
+            .system(system)
+            .build()
+            .expect("AgentFactory::spawn is infallible: provider is always set")
+    }
+
+    /// Spawn an agent with custom builder configuration.
+    ///
+    /// Use this escape hatch when you need more than just a system prompt
+    /// (e.g., adding tools, setting memory, configuring hooks).
+    ///
+    /// The closure receives an [`AgentBuilder`] with the factory's provider
+    /// already set. Call builder methods as needed.
+    ///
+    /// # Example
+    ///
+    /// ```rust,no_run
+    /// use traitclaw_core::factory::AgentFactory;
+    /// use traitclaw_core::traits::provider::Provider;
+    ///
+    /// # fn example(provider: impl Provider) -> traitclaw_core::Result<()> {
+    /// let factory = AgentFactory::new(provider);
+    /// let agent = factory.spawn_with(|b| {
+    ///     b.system("You are a researcher with tools.")
+    ///      .max_iterations(10)
+    /// })?;
+    /// # Ok(())
+    /// # }
+    /// ```
+    ///
+    /// # Errors
+    ///
+    /// Returns an error if the builder customization produces an invalid
+    /// agent configuration.
+    pub fn spawn_with(&self, f: impl FnOnce(AgentBuilder) -> AgentBuilder) -> Result<Agent> {
+        let builder = AgentBuilder::new().provider_arc(Arc::clone(&self.provider));
+        f(builder).build()
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::types::completion::{CompletionRequest, CompletionResponse, ResponseContent, Usage};
+    use crate::types::model_info::{ModelInfo, ModelTier};
+    use crate::types::stream::CompletionStream;
+    use async_trait::async_trait;
+
+    #[derive(Clone)]
+    struct MockCloneProvider {
+        info: ModelInfo,
+    }
+
+    impl MockCloneProvider {
+        fn new() -> Self {
+            Self {
+                info: ModelInfo::new("mock-clone", ModelTier::Small, 4_096, false, false, false),
+            }
+        }
+    }
+
+    #[async_trait]
+    impl Provider for MockCloneProvider {
+        async fn complete(&self, _req: CompletionRequest) -> crate::Result<CompletionResponse> {
+            Ok(CompletionResponse {
+                content: ResponseContent::Text("ok".into()),
+                usage: Usage {
+                    prompt_tokens: 1,
+                    completion_tokens: 1,
+                    total_tokens: 2,
+                },
+            })
+        }
+        async fn stream(&self, _req: CompletionRequest) -> crate::Result<CompletionStream> {
+            unimplemented!()
+        }
+        fn model_info(&self) -> &ModelInfo {
+            &self.info
+        }
+    }
+
+    #[test]
+    fn test_factory_new() {
+        let factory = AgentFactory::new(MockCloneProvider::new());
+        assert_eq!(factory.provider.model_info().name, "mock-clone");
+    }
+
+    #[test]
+    fn test_factory_from_arc() {
+        let provider: Arc<dyn Provider> = Arc::new(MockCloneProvider::new());
+        let factory = AgentFactory::from_arc(provider);
+        assert_eq!(factory.provider.model_info().name, "mock-clone");
+    }
+
+    #[test]
+    fn test_factory_spawn_creates_agent_with_system_prompt() {
+        let factory = AgentFactory::new(MockCloneProvider::new());
+        let agent = factory.spawn("You are a researcher.");
+        assert_eq!(
+            agent.config.system_prompt.as_deref(),
+            Some("You are a researcher.")
+        );
+    }
+
+    #[test]
+    fn test_factory_spawn_produces_independent_agents() {
+        let factory = AgentFactory::new(MockCloneProvider::new());
+        let agent_a = factory.spawn("Agent A");
+        let agent_b = factory.spawn("Agent B");
+
+        assert_eq!(agent_a.config.system_prompt.as_deref(), Some("Agent A"));
+        assert_eq!(agent_b.config.system_prompt.as_deref(), Some("Agent B"));
+        // Both have the same provider model
+        assert_eq!(agent_a.provider.model_info().name, "mock-clone");
+        assert_eq!(agent_b.provider.model_info().name, "mock-clone");
+    }
+
+    #[test]
+    fn test_factory_spawn_with_custom_config() {
+        let factory = AgentFactory::new(MockCloneProvider::new());
+        let agent = factory
+            .spawn_with(|b| b.system("Custom").max_iterations(5))
+            .expect("spawn_with should succeed");
+
+        assert_eq!(agent.config.system_prompt.as_deref(), Some("Custom"));
+        assert_eq!(agent.config.max_iterations, 5);
+    }
+
+    #[test]
+    fn test_factory_spawn_with_no_system() {
+        let factory = AgentFactory::new(MockCloneProvider::new());
+        let agent = factory
+            .spawn_with(|b| b.max_iterations(3))
+            .expect("spawn_with without system should succeed");
+
+        assert!(agent.config.system_prompt.is_none());
+    }
+
+    // Compile-time check: AgentFactory is Send + Sync
+    fn _assert_send_sync<T: Send + Sync>() {}
+    #[test]
+    fn test_factory_is_send_sync() {
+        _assert_send_sync::<AgentFactory>();
+    }
+}
diff --git a/crates/traitclaw-core/src/lib.rs b/crates/traitclaw-core/src/lib.rs
index de757b8..3093350 100644
--- a/crates/traitclaw-core/src/lib.rs
+++ b/crates/traitclaw-core/src/lib.rs
@@ -38,7 +38,9 @@ pub mod config;
 pub mod context_managers;
 pub mod default_strategy;
 pub mod error;
+pub mod factory;
 pub mod memory;
+pub mod pool;
 pub mod registries;
 pub mod retry;
 pub(crate) mod runtime;
@@ -106,6 +108,8 @@ pub use memory::in_memory::InMemoryMemory;
 // Re-export agent
 pub use agent::{Agent, AgentOutput, AgentOutputContent, AgentSession, RunUsage};
 pub use agent_builder::AgentBuilder;
+pub use factory::AgentFactory;
+pub use pool::AgentPool;
 
 /// Prelude module for convenient imports.
 ///
@@ -159,6 +163,10 @@ pub mod prelude {
     pub use crate::agent::{Agent, AgentOutput, AgentOutputContent, AgentSession, RunUsage};
     pub use crate::agent_builder::AgentBuilder;
 
+    // v0.6.0: Composition APIs
+    pub use crate::factory::AgentFactory;
+    pub use crate::pool::AgentPool;
+
     // v0.2.0: Strategy & Hook
     pub use crate::default_strategy::DefaultStrategy;
     pub use crate::traits::hook::{AgentHook, HookAction, LoggingHook};
diff --git a/crates/traitclaw-core/src/pool.rs b/crates/traitclaw-core/src/pool.rs
new file mode 100644
index 0000000..da1f96d
--- /dev/null
+++ b/crates/traitclaw-core/src/pool.rs
@@ -0,0 +1,231 @@
+//! Agent pool for managing and executing groups of agents.
+//!
+//! `AgentPool` holds a collection of agents and provides methods for
+//! sequential pipeline execution (output chaining).
+
+use crate::agent::Agent;
+use crate::agent::AgentOutput;
+use crate::Result;
+
+/// A collection of agents for group execution.
+///
+/// `AgentPool` takes ownership of a `Vec<Agent>` and provides
+/// sequential pipeline execution where each agent's output feeds
+/// into the next agent's input.
+///
+/// # Example
+///
+/// ```rust,no_run
+/// use traitclaw_core::pool::AgentPool;
+/// use traitclaw_core::agent::Agent;
+///
+/// # fn example(agents: Vec<Agent>) {
+/// let pool = AgentPool::new(agents);
+/// assert_eq!(pool.len(), 3);
+/// # }
+/// ```
+pub struct AgentPool {
+    agents: Vec<Agent>,
+}
+
+impl std::fmt::Debug for AgentPool {
+    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
+        f.debug_struct("AgentPool")
+            .field("len", &self.agents.len())
+            .finish()
+    }
+}
+
+impl AgentPool {
+    /// Create a new pool from a vector of agents.
+    #[must_use]
+    pub fn new(agents: Vec<Agent>) -> Self {
+        Self { agents }
+    }
+
+    /// Returns the number of agents in the pool.
+    #[must_use]
+    pub fn len(&self) -> usize {
+        self.agents.len()
+    }
+
+    /// Returns `true` if the pool contains no agents.
+    #[must_use]
+    pub fn is_empty(&self) -> bool {
+        self.agents.is_empty()
+    }
+
+    /// Get a reference to an agent by index.
+    ///
+    /// Returns `None` if the index is out of bounds.
+    #[must_use]
+    pub fn get(&self, index: usize) -> Option<&Agent> {
+        self.agents.get(index)
+    }
+
+    /// Run agents sequentially, chaining outputs.
+    ///
+    /// Each agent receives the previous agent's text output as input.
+    /// The first agent receives the provided `input` string.
+    ///
+    /// # Example
+    ///
+    /// ```rust,no_run
+    /// use traitclaw_core::pool::AgentPool;
+    /// use traitclaw_core::agent::Agent;
+    ///
+    /// # async fn example(pool: &AgentPool) -> traitclaw_core::Result<()> {
+    /// let output = pool.run_sequential("Research Rust async patterns").await?;
+    /// println!("Final output: {}", output.text());
+    /// # Ok(())
+    /// # }
+    /// ```
+    ///
+    /// # Errors
+    ///
+    /// Returns an error immediately if any agent in the pipeline fails.
+    /// Earlier agents' outputs are not available on error.
+    pub async fn run_sequential(&self, input: &str) -> Result<AgentOutput> {
+        if self.agents.is_empty() {
+            return Err(crate::Error::Runtime(
+                "AgentPool::run_sequential called on empty pool".into(),
+            ));
+        }
+
+        let mut current_input = input.to_string();
+        let mut last_output: Option<AgentOutput> = None;
+
+        for agent in &self.agents {
+            let output = agent.run(&current_input).await?;
+            current_input = output.text().to_string();
+            last_output = Some(output);
+        }
+
+        // SAFETY: We checked is_empty above, so last_output is always Some
+        Ok(last_output.expect("pool is non-empty"))
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::traits::provider::Provider;
+    use crate::types::completion::{CompletionRequest, CompletionResponse, ResponseContent, Usage};
+    use crate::types::model_info::{ModelInfo, ModelTier};
+    use crate::types::stream::CompletionStream;
+    use async_trait::async_trait;
+    use std::sync::atomic::{AtomicUsize, Ordering};
+    use std::sync::Arc;
+
+    struct EchoProvider {
+        info: ModelInfo,
+        prefix: String,
+        call_count: Arc<AtomicUsize>,
+    }
+
+    impl EchoProvider {
+        fn new(prefix: &str) -> Self {
+            Self {
+                info: ModelInfo::new("echo", ModelTier::Small, 4_096, false, false, false),
+                prefix: prefix.to_string(),
+                call_count: Arc::new(AtomicUsize::new(0)),
+            }
+        }
+    }
+
+    #[async_trait]
+    impl Provider for EchoProvider {
+        async fn complete(&self, req: CompletionRequest) -> crate::Result<CompletionResponse> {
+            self.call_count.fetch_add(1, Ordering::SeqCst);
+            // Echo back the last user message with our prefix
+            let last_msg = req
+                .messages
+                .iter()
+                .rev()
+                .find(|m| m.role == crate::types::message::MessageRole::User)
+                .map(|m| m.content.clone())
+                .unwrap_or_default();
+            Ok(CompletionResponse {
+                content: ResponseContent::Text(format!("[{}] {}", self.prefix, last_msg)),
+                usage: Usage {
+                    prompt_tokens: 1,
+                    completion_tokens: 1,
+                    total_tokens: 2,
+                },
+            })
+        }
+        async fn stream(&self, _req: CompletionRequest) -> crate::Result<CompletionStream> {
+            unimplemented!()
+        }
+        fn model_info(&self) -> &ModelInfo {
+            &self.info
+        }
+    }
+
+    #[test]
+    fn test_pool_new_and_len() {
+        let agents = vec![
+            Agent::with_system(EchoProvider::new("A"), "Agent A"),
+            Agent::with_system(EchoProvider::new("B"), "Agent B"),
+        ];
+        let pool = AgentPool::new(agents);
+        assert_eq!(pool.len(), 2);
+        assert!(!pool.is_empty());
+    }
+
+    #[test]
+    fn test_pool_empty() {
+        let pool = AgentPool::new(vec![]);
+        assert!(pool.is_empty());
+        assert_eq!(pool.len(), 0);
+    }
+
+    #[test]
+    fn test_pool_get() {
+        let agents = vec![
+            Agent::with_system(EchoProvider::new("A"), "Agent A"),
+            Agent::with_system(EchoProvider::new("B"), "Agent B"),
+            Agent::with_system(EchoProvider::new("C"), "Agent C"),
+        ];
+        let pool = AgentPool::new(agents);
+        assert!(pool.get(0).is_some());
+        assert!(pool.get(1).is_some());
+        assert!(pool.get(2).is_some());
+        assert!(pool.get(5).is_none());
+    }
+
+    #[tokio::test]
+    async fn test_pool_run_sequential_single_agent() {
+        let agents = vec![Agent::with_system(EchoProvider::new("Solo"), "Solo agent")];
+        let pool = AgentPool::new(agents);
+        let output = pool.run_sequential("Hello").await.unwrap();
+        assert_eq!(output.text(), "[Solo] Hello");
+    }
+
+    #[tokio::test]
+    async fn test_pool_run_sequential_pipeline() {
+        let agents = vec![
+            Agent::with_system(EchoProvider::new("R"), "Researcher"),
+            Agent::with_system(EchoProvider::new("W"), "Writer"),
+        ];
+        let pool = AgentPool::new(agents);
+        let output = pool.run_sequential("topic").await.unwrap();
+        // First agent: "[R] topic" → Second agent: "[W] [R] topic"
+        assert_eq!(output.text(), "[W] [R] topic");
+    }
+
+    #[tokio::test]
+    async fn test_pool_run_sequential_empty_pool_errors() {
+        let pool = AgentPool::new(vec![]);
+        let result = pool.run_sequential("anything").await;
+        assert!(result.is_err());
+    }
+
+    #[test]
+    fn test_pool_debug() {
+        let pool = AgentPool::new(vec![Agent::with_system(EchoProvider::new("A"), "A")]);
+        let debug = format!("{pool:?}");
+        assert!(debug.contains("AgentPool"));
+        assert!(debug.contains("len: 1"));
+    }
+}
diff --git a/crates/traitclaw-team/src/group_chat.rs b/crates/traitclaw-team/src/group_chat.rs
new file mode 100644
index 0000000..da2f93a
--- /dev/null
+++ b/crates/traitclaw-team/src/group_chat.rs
@@ -0,0 +1,330 @@
+//! Multi-agent group chat with configurable turn-taking and termination.
+//!
+//! Provides [`RoundRobinGroupChat`] for structured multi-turn conversations
+//! where agents take turns in a fixed order, each seeing the full transcript.
+
+use std::fmt;
+
+use traitclaw_core::agent::Agent;
+use traitclaw_core::types::message::{Message, MessageRole};
+
+// ─────────────────────────────────────────────────────────────────────────────
+// Termination Conditions
+// ─────────────────────────────────────────────────────────────────────────────
+
+/// Trait for determining when a group chat should stop.
+///
+/// Implement this trait for custom termination logic (keyword detection,
+/// quality thresholds, consensus detection, etc.).
+pub trait TerminationCondition: Send + Sync {
+    /// Check whether the chat should terminate.
+    ///
+    /// - `round`: the current round number (0-indexed)
+    /// - `messages`: the full conversation transcript so far
+    fn should_terminate(&self, round: usize, messages: &[Message]) -> bool;
+}
+
+/// Terminate after a fixed number of rounds.
+///
+/// # Example
+///
+/// ```rust
+/// use traitclaw_team::group_chat::MaxRoundsTermination;
+///
+/// let term = MaxRoundsTermination::new(6);
+/// ```
+#[derive(Debug, Clone)]
+pub struct MaxRoundsTermination {
+    max_rounds: usize,
+}
+
+impl MaxRoundsTermination {
+    /// Create a termination condition that stops after `max_rounds` rounds.
+    #[must_use]
+    pub fn new(max_rounds: usize) -> Self {
+        Self { max_rounds }
+    }
+}
+
+impl TerminationCondition for MaxRoundsTermination {
+    fn should_terminate(&self, round: usize, _messages: &[Message]) -> bool {
+        round >= self.max_rounds
+    }
+}
+
+// ─────────────────────────────────────────────────────────────────────────────
+// Group Chat Result
+// ─────────────────────────────────────────────────────────────────────────────
+
+/// The result of a group chat session.
+#[derive(Debug, Clone)]
+pub struct GroupChatResult {
+    /// Full conversation transcript in chronological order.
+    pub transcript: Vec<Message>,
+    /// The final message text produced by the last responding agent.
+    pub final_message: String,
+}
+
+// ─────────────────────────────────────────────────────────────────────────────
+// RoundRobinGroupChat
+// ─────────────────────────────────────────────────────────────────────────────
+
+/// A round-robin group chat where agents take turns responding.
+///
+/// Each agent sees the full conversation history and adds its response.
+/// The chat continues until the termination condition is met.
+///
+/// # Example
+///
+/// ```rust,no_run
+/// use traitclaw_team::group_chat::RoundRobinGroupChat;
+/// use traitclaw_core::agent::Agent;
+///
+/// # async fn example(agents: Vec<Agent>) -> traitclaw_core::Result<()> {
+/// let mut chat = RoundRobinGroupChat::new(agents);
+/// let result = chat.run("Discuss the future of AI").await?;
+/// println!("Transcript has {} messages", result.transcript.len());
+/// println!("Final: {}", result.final_message);
+/// # Ok(())
+/// # }
+/// ```
+pub struct RoundRobinGroupChat {
+    agents: Vec<Agent>,
+    termination: Box<dyn TerminationCondition>,
+}
+
+impl fmt::Debug for RoundRobinGroupChat {
+    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
+        f.debug_struct("RoundRobinGroupChat")
+            .field("agents", &self.agents.len())
+            .finish()
+    }
+}
+
+impl RoundRobinGroupChat {
+    /// Create a new group chat with default termination (`n_agents × 3` rounds).
+    ///
+    /// # Panics
+    ///
+    /// This method cannot panic under normal usage.
+    #[must_use]
+    pub fn new(agents: Vec<Agent>) -> Self {
+        let max_rounds = agents.len().saturating_mul(3).max(1);
+        Self {
+            termination: Box::new(MaxRoundsTermination::new(max_rounds)),
+            agents,
+        }
+    }
+
+    /// Set the maximum number of rounds (convenience method).
+    #[must_use]
+    pub fn with_max_rounds(mut self, n: usize) -> Self {
+        self.termination = Box::new(MaxRoundsTermination::new(n));
+        self
+    }
+
+    /// Set a custom termination condition.
+    #[must_use]
+    pub fn with_termination(mut self, t: impl TerminationCondition + 'static) -> Self {
+        self.termination = Box::new(t);
+        self
+    }
+
+    /// Returns the number of agents in the chat.
+    #[must_use]
+    pub fn len(&self) -> usize {
+        self.agents.len()
+    }
+
+    /// Returns `true` if the chat has no agents.
+    #[must_use]
+    pub fn is_empty(&self) -> bool {
+        self.agents.is_empty()
+    }
+
+    /// Run the group chat starting with the given task prompt.
+    ///
+    /// Agents respond in round-robin order, each seeing the full transcript.
+    /// The chat terminates when the termination condition is met.
+    ///
+    /// # Errors
+    ///
+    /// Returns an error if:
+    /// - The agent pool is empty
+    /// - Any agent fails to produce a response
+    pub async fn run(&self, task: &str) -> traitclaw_core::Result<GroupChatResult> {
+        if self.agents.is_empty() {
+            return Err(traitclaw_core::Error::Runtime(
+                "RoundRobinGroupChat::run() called with no agents".into(),
+            ));
+        }
+
+        let mut transcript = vec![Message {
+            role: MessageRole::User,
+            content: task.to_string(),
+            tool_call_id: None,
+        }];
+
+        let n_agents = self.agents.len();
+        let mut round = 0;
+
+        loop {
+            if self.termination.should_terminate(round, &transcript) {
+                break;
+            }
+
+            let agent_idx = round % n_agents;
+            let agent = &self.agents[agent_idx];
+
+            // Build the context: format transcript as a conversation prompt
+            let context = Self::format_transcript(&transcript);
+            let output = agent.run(&context).await?;
+            let response_text = output.text().to_string();
+
+            transcript.push(Message {
+                role: MessageRole::Assistant,
+                content: response_text,
+                tool_call_id: None,
+            });
+
+            round += 1;
+        }
+
+        let final_message = transcript
+            .last()
+            .map(|m| m.content.clone())
+            .unwrap_or_default();
+
+        Ok(GroupChatResult {
+            transcript,
+            final_message,
+        })
+    }
+
+    /// Format transcript messages into a single context string.
+    fn format_transcript(messages: &[Message]) -> String {
+        messages
+            .iter()
+            .map(|m| {
+                let role = match m.role {
+                    MessageRole::User => "User",
+                    MessageRole::Assistant => "Assistant",
+                    MessageRole::System => "System",
+                    MessageRole::Tool => "Tool",
+                    _ => "Unknown",
+                };
+                format!("[{}]: {}", role, m.content)
+            })
+            .collect::<Vec<_>>()
+            .join("\n\n")
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::tests_common::EchoProvider;
+
+    // ── TerminationCondition ────────────────────────────────────────────────
+
+    #[test]
+    fn test_max_rounds_at_boundary() {
+        let term = MaxRoundsTermination::new(3);
+        assert!(!term.should_terminate(0, &[]));
+        assert!(!term.should_terminate(1, &[]));
+        assert!(!term.should_terminate(2, &[]));
+        assert!(term.should_terminate(3, &[]));
+        assert!(term.should_terminate(4, &[]));
+    }
+
+    #[test]
+    fn test_max_rounds_zero() {
+        let term = MaxRoundsTermination::new(0);
+        assert!(term.should_terminate(0, &[]));
+    }
+
+    // ── RoundRobinGroupChat ─────────────────────────────────────────────────
+
+    #[test]
+    fn test_group_chat_new_default_rounds() {
+        let agents = vec![
+            Agent::with_system(EchoProvider::new("A"), "Agent A"),
+            Agent::with_system(EchoProvider::new("B"), "Agent B"),
+        ];
+        let chat = RoundRobinGroupChat::new(agents);
+        assert_eq!(chat.len(), 2);
+        // Default max_rounds = 2 * 3 = 6
+    }
+
+    #[test]
+    fn test_group_chat_with_max_rounds() {
+        let agents = vec![Agent::with_system(EchoProvider::new("A"), "Agent A")];
+        let chat = RoundRobinGroupChat::new(agents).with_max_rounds(10);
+        assert_eq!(chat.len(), 1);
+    }
+
+    #[tokio::test]
+    async fn test_group_chat_run_basic() {
+        let agents = vec![
+            Agent::with_system(EchoProvider::new("R"), "Researcher"),
+            Agent::with_system(EchoProvider::new("W"), "Writer"),
+        ];
+        let chat = RoundRobinGroupChat::new(agents).with_max_rounds(2);
+        let result = chat.run("Discuss Rust").await.unwrap();
+
+        // Initial user message + 2 agent responses = 3 messages
+        assert_eq!(result.transcript.len(), 3);
+        assert!(!result.final_message.is_empty());
+    }
+
+    #[tokio::test]
+    async fn test_group_chat_round_robin_order() {
+        let agents = vec![
+            Agent::with_system(EchoProvider::new("FIRST"), "First"),
+            Agent::with_system(EchoProvider::new("SECOND"), "Second"),
+        ];
+        let chat = RoundRobinGroupChat::new(agents).with_max_rounds(4);
+        let result = chat.run("Test").await.unwrap();
+
+        // 5 messages: 1 user + 4 agent responses
+        assert_eq!(result.transcript.len(), 5);
+        // Check round-robin order by prefix
+        assert!(result.transcript[1].content.contains("[FIRST]"));
+        assert!(result.transcript[2].content.contains("[SECOND]"));
+        assert!(result.transcript[3].content.contains("[FIRST]"));
+        assert!(result.transcript[4].content.contains("[SECOND]"));
+    }
+
+    #[tokio::test]
+    async fn test_group_chat_empty_agents_returns_error() {
+        let chat = RoundRobinGroupChat::new(vec![]);
+        let result = chat.run("Test").await;
+        assert!(result.is_err());
+    }
+
+    #[tokio::test]
+    async fn test_group_chat_custom_termination() {
+        // Custom termination: stop when any message contains "DONE"
+        struct ContainsKeyword;
+        impl TerminationCondition for ContainsKeyword {
+            fn should_terminate(&self, _round: usize, messages: &[Message]) -> bool {
+                messages.iter().any(|m| m.content.contains("DONE"))
+            }
+        }
+
+        let agents = vec![Agent::with_system(EchoProvider::new("DONE"), "Agent")];
+        let chat = RoundRobinGroupChat::new(agents).with_termination(ContainsKeyword);
+        let result = chat.run("Test").await.unwrap();
+
+        // Should stop after first agent response (contains "DONE")
+        // 1 user + 1 agent = 2 messages
+        assert_eq!(result.transcript.len(), 2);
+    }
+
+    #[test]
+    fn test_group_chat_debug() {
+        let chat = RoundRobinGroupChat::new(vec![Agent::with_system(EchoProvider::new("A"), "A")]);
+        let debug = format!("{chat:?}");
+        assert!(debug.contains("RoundRobinGroupChat"));
+    }
+}
diff --git a/crates/traitclaw-team/src/lib.rs b/crates/traitclaw-team/src/lib.rs
index 1c273b7..0e5cc2b 100644
--- a/crates/traitclaw-team/src/lib.rs
+++ b/crates/traitclaw-team/src/lib.rs
@@ -25,15 +25,109 @@
 
 pub mod conditional_router;
 pub mod execution;
+pub mod group_chat;
 pub mod router;
 pub mod team_context;
 
+#[cfg(test)]
+pub(crate) mod tests_common;
+
 use serde::{Deserialize, Serialize};
+use std::sync::Arc;
+use traitclaw_core::traits::provider::Provider;
 
 pub use conditional_router::ConditionalRouter;
 pub use execution::{run_verification_chain, TeamRunner};
 pub use team_context::TeamContext;
 
+/// Create an [`AgentPool`](traitclaw_core::pool::AgentPool) from a [`Team`] and a provider.
+///
+/// Each role's `system_prompt` is used as the agent's system prompt.
+/// Roles without a `system_prompt` cause an error listing all missing roles.
+///
+/// # Example
+///
+/// ```rust
+/// use traitclaw_team::{AgentRole, Team, pool_from_team};
+/// use traitclaw_core::traits::provider::Provider;
+///
+/// # fn example(provider: impl Provider) -> traitclaw_core::Result<()> {
+/// let team = Team::new("content_team")
+///     .add_role(AgentRole::new("researcher", "Research").with_system_prompt("You research topics."))
+///     .add_role(AgentRole::new("writer", "Write").with_system_prompt("You write articles."));
+///
+/// let pool = pool_from_team(&team, provider)?;
+/// assert_eq!(pool.len(), 2);
+/// # Ok(())
+/// # }
+/// ```
+///
+/// # Errors
+///
+/// Returns an error if any role in the team is missing a `system_prompt`.
+pub fn pool_from_team(
+    team: &Team,
+    provider: impl Provider,
+) -> traitclaw_core::Result<traitclaw_core::pool::AgentPool> {
+    // Check for missing system_prompts first
+    let missing: Vec<&str> = team
+        .roles()
+        .iter()
+        .filter(|r| r.system_prompt.is_none())
+        .map(|r| r.name.as_str())
+        .collect();
+
+    if !missing.is_empty() {
+        return Err(traitclaw_core::Error::Config(format!(
+            "Cannot create AgentPool from team '{}': roles missing system_prompt: [{}]",
+            team.name(),
+            missing.join(", ")
+        )));
+    }
+
+    let factory = traitclaw_core::factory::AgentFactory::new(provider);
+    let agents: Vec<traitclaw_core::Agent> = team
+        .roles()
+        .iter()
+        .map(|role| factory.spawn(role.system_prompt.as_ref().expect("checked above")))
+        .collect();
+
+    Ok(traitclaw_core::pool::AgentPool::new(agents))
+}
+
+/// Create an [`AgentPool`](traitclaw_core::pool::AgentPool) from a [`Team`]
+/// using a pre-wrapped `Arc<dyn Provider>`.
+///
+/// Same as [`pool_from_team`] but accepts a shared provider reference.
+pub fn pool_from_team_arc(
+    team: &Team,
+    provider: Arc<dyn Provider>,
+) -> traitclaw_core::Result<traitclaw_core::pool::AgentPool> {
+    let missing: Vec<&str> = team
+        .roles()
+        .iter()
+        .filter(|r| r.system_prompt.is_none())
+        .map(|r| r.name.as_str())
+        .collect();
+
+    if !missing.is_empty() {
+        return Err(traitclaw_core::Error::Config(format!(
+            "Cannot create AgentPool from team '{}': roles missing system_prompt: [{}]",
+            team.name(),
+            missing.join(", ")
+        )));
+    }
+
+    let factory = traitclaw_core::factory::AgentFactory::from_arc(provider);
+    let agents: Vec<traitclaw_core::Agent> = team
+        .roles()
+        .iter()
+        .map(|role| factory.spawn(role.system_prompt.as_ref().expect("checked above")))
+        .collect();
+
+    Ok(traitclaw_core::pool::AgentPool::new(agents))
+}
+
 /// A team of agents working together.
 pub struct Team {
     name: String,
diff --git a/crates/traitclaw-team/src/tests_common.rs b/crates/traitclaw-team/src/tests_common.rs
new file mode 100644
index 0000000..e8b9f4e
--- /dev/null
+++ b/crates/traitclaw-team/src/tests_common.rs
@@ -0,0 +1,53 @@
+//! Shared test utilities for traitclaw-team tests.
+
+use async_trait::async_trait;
+use traitclaw_core::traits::provider::Provider;
+use traitclaw_core::types::completion::{
+    CompletionRequest, CompletionResponse, ResponseContent, Usage,
+};
+use traitclaw_core::types::message::MessageRole;
+use traitclaw_core::types::model_info::{ModelInfo, ModelTier};
+use traitclaw_core::types::stream::CompletionStream;
+
+/// A provider that echoes back the last user message with a prefix.
+pub struct EchoProvider {
+    info: ModelInfo,
+    prefix: String,
+}
+
+impl EchoProvider {
+    /// Create a new echo provider with the given prefix.
+    pub fn new(prefix: &str) -> Self {
+        Self {
+            info: ModelInfo::new("echo", ModelTier::Small, 4_096, false, false, false),
+            prefix: prefix.to_string(),
+        }
+    }
+}
+
+#[async_trait]
+impl Provider for EchoProvider {
+    async fn complete(&self, req: CompletionRequest) -> traitclaw_core::Result<CompletionResponse> {
+        let last_msg = req
+            .messages
+            .iter()
+            .rev()
+            .find(|m| m.role == MessageRole::User)
+            .map(|m| m.content.clone())
+            .unwrap_or_default();
+        Ok(CompletionResponse {
+            content: ResponseContent::Text(format!("[{}] {}", self.prefix, last_msg)),
+            usage: Usage {
+                prompt_tokens: 1,
+                completion_tokens: 1,
+                total_tokens: 2,
+            },
+        })
+    }
+    async fn stream(&self, _req: CompletionRequest) -> traitclaw_core::Result<CompletionStream> {
+        unimplemented!()
+    }
+    fn model_info(&self) -> &ModelInfo {
+        &self.info
+    }
+}

```
