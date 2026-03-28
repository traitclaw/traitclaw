---
stepsCompleted: ["step-01-validate", "step-02-design-epics", "step-03-create-stories", "step-04-final-validation"]
status: 'complete'
completedAt: '2026-03-28'
inputDocuments: ["prd.md", "architecture.md"]
---

# TraitClaw v0.7.0 - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for TraitClaw v0.7.0 "Reasoning", decomposing the requirements from the PRD and Architecture into implementable stories.

## Requirements Inventory

### Functional Requirements

- FR1: Developer can instantiate a `ReActStrategy` with default configuration and assign it to an agent via builder
- FR2: Developer can configure `ReActStrategy` max iterations (loop limit) and tool-calling behavior
- FR3: `ReActStrategy` can autonomously execute Think→Act→Observe loops until answering or hitting max iterations
- FR4: Developer can instantiate a `MctsStrategy` with configurable branch count and search depth
- FR5: `MctsStrategy` can spawn parallel reasoning branches, score each path, and select the highest-scoring result
- FR6: Developer can instantiate a `ChainOfThoughtStrategy` with configurable max steps
- FR7: `ChainOfThoughtStrategy` can inject step-by-step reasoning into the agent's prompt and return structured thought steps
- FR8: Developer can swap any built-in strategy for another by changing only the strategy constructor — no other code changes required
- FR9: Developer can use built-in strategies alongside custom `AgentStrategy` implementations in the same application
- FR10: All built-in strategies implement the existing `AgentStrategy` trait without modifications to the trait interface
- FR11: Developer can implement `StreamingOutputTransformer` trait to transform agent output as it streams
- FR12: `StreamingOutputTransformer` can be composed into the agent builder pipeline alongside existing `OutputTransformer`
- FR13: Developer can use `StreamingOutputTransformer` with `ReActStrategy` to stream individual thought steps in real-time
- FR14: Developer can add `traitclaw-strategies` as a standalone dependency via `cargo add`
- FR15: Developer can selectively enable/disable individual strategies via Cargo feature flags
- FR16: All strategies are enabled by default (batteries-included); developer can opt-out with `default-features = false`
- FR17: `traitclaw-strategies` re-exports through the `traitclaw` meta-crate via `features = ["strategies"]`
- FR18: `ReActStrategy` emits typed `ThoughtStep` events (Think, Act, Observe, Answer) during execution
- FR19: Developer can inspect `ThoughtStep` sequence after strategy execution for debugging/logging
- FR20: `MctsStrategy` exposes branch scores and selected path for post-execution analysis
- FR21: All v0.6.0 public APIs remain unchanged and functional in v0.7.0
- FR22: All existing examples (1–24) compile and run without modification on v0.7.0
- FR23: Custom `AgentStrategy` implementations created against v0.2.0+ trait work without changes
- FR24: Each built-in strategy has a dedicated runnable example demonstrating core usage
- FR25: `StreamingOutputTransformer` has a dedicated example demonstrating streaming thought steps
- FR26: Migration guide documents all new types and adoption path from v0.6.0
- FR27: All new public types have complete rustdoc documentation with code examples

### NonFunctional Requirements

- NFR1: Compile time — `cargo build --timings` delta vs v0.6.0 < 2% increase with default features
- NFR2: Binary size — `cargo bloat` delta vs v0.6.0 < 5% increase with default features
- NFR3: Runtime overhead — ReAct loop latency per iteration (excluding LLM call) < 1ms per Think→Act→Observe cycle
- NFR4: MCTS parallelism — Tokio task spawn overhead per branch < 100μs per branch spawn
- NFR5: Streaming latency — Time from first token to first `StreamingOutputTransformer` emission < 10ms
- NFR6: Feature-gated build — Build time with `default-features = false` + single strategy < 50% of full build time
- NFR7: MSRV — Rust 1.75+ — no nightly features required
- NFR8: Async runtime — `tokio` only (consistent with existing crate ecosystem)
- NFR9: Dependency budget — Zero new required dependencies beyond `traitclaw-core` and `tokio`
- NFR10: API ergonomics — All strategy constructors follow builder pattern consistent with `AgentBuilder`
- NFR11: Error types — Strategy errors use existing `TraitClawError` enum — no new top-level error types
- NFR12: Trait object safety — Built-in strategies must be object-safe (`dyn AgentStrategy`) for dynamic dispatch

### Additional Requirements

From Architecture document:
- AR1: New `traitclaw-strategies` crate must be registered in workspace `Cargo.toml`
- AR2: `ThoughtStep` enum design: `Think`, `Act`, `Observe`, `Answer` variants with `#[derive(Debug, Clone, Serialize)]`
- AR3: MCTS uses `tokio::spawn` + `JoinSet` for structured concurrency with automatic cleanup
- AR4: `StreamingOutputTransformer` trait lives in `traitclaw-core`, separate from existing `OutputTransformer`
- AR5: Feature flags: `react`, `mcts`, `cot` — additive, no inter-flag dependencies, default all-on
- AR6: Builder pattern: `.builder().config().build()? → Result<Strategy, TraitClawError>`
- AR7: `ScoringFn` for MCTS: `Arc<dyn Fn(&str) -> f64 + Send + Sync>`
- AR8: Module structure: `common/`, `react/`, `mcts/`, `cot/` modules with `mod.rs` + `strategy.rs` each
- AR9: Internal types use `pub(crate)`, only public API re-exported from `lib.rs`
- AR10: Meta-crate re-exports via `features = ["strategies"]` → `dep:traitclaw-strategies`

### UX Design Requirements

N/A — Library project, no UI.

### FR Coverage Map

| FR | Epic | Description |
|----|------|-------------|
| FR1 | Epic 2 | ReAct instantiation with default config |
| FR2 | Epic 2 | ReAct configurable max iterations |
| FR3 | Epic 2 | ReAct Think→Act→Observe loop execution |
| FR4 | Epic 4 | MCTS instantiation with config |
| FR5 | Epic 4 | MCTS parallel branch evaluation |
| FR6 | Epic 3 | CoT instantiation with config |
| FR7 | Epic 3 | CoT step-by-step reasoning |
| FR8 | Epic 2 | Strategy swap via constructor only |
| FR9 | Epic 2 | Built-in + custom strategies coexist |
| FR10 | Epic 2 | All strategies impl `AgentStrategy` |
| FR11 | Epic 5 | `StreamingOutputTransformer` trait |
| FR12 | Epic 5 | Streaming composes with `OutputTransformer` |
| FR13 | Epic 5 | Streaming + ReAct thought steps |
| FR14 | Epic 1 | `cargo add traitclaw-strategies` |
| FR15 | Epic 1 | Feature flag selection |
| FR16 | Epic 1 | Default all-on, opt-out |
| FR17 | Epic 1 | Meta-crate re-export |
| FR18 | Epic 2 | ReAct emits `ThoughtStep` events |
| FR19 | Epic 2 | Post-execution `ThoughtStep` inspection |
| FR20 | Epic 4 | MCTS branch scores exposure |
| FR21 | Epic 5 | v0.6.0 API unchanged |
| FR22 | Epic 5 | Existing examples compile on v0.7.0 |
| FR23 | Epic 5 | Custom strategies backward compat |
| FR24 | Epic 5 | Per-strategy example |
| FR25 | Epic 5 | Streaming example |
| FR26 | Epic 5 | Migration guide |
| FR27 | Epic 5 | Complete rustdoc |

**Coverage: 27/27 FRs ✅ | 12/12 NFRs ✅ | 10/10 ARs ✅**

## Epic List

### Epic 1: Strategies Crate Foundation
Developer can add `traitclaw-strategies` as a dependency and access shared strategy types.
**FRs covered:** FR14, FR15, FR16, FR17
**ARs covered:** AR1, AR2, AR5, AR8, AR9, AR10
**Delivers:** Crate scaffold, `Cargo.toml`, feature flags, `ThoughtStep` enum, workspace registration, meta-crate re-export.

### Epic 2: ReAct Strategy
Developer can build agents with autonomous Think→Act→Observe reasoning loops.
**FRs covered:** FR1, FR2, FR3, FR8, FR9, FR10, FR18, FR19
**ARs covered:** AR6
**Delivers:** `ReActStrategy` with builder, tool-calling, configurable iterations, ThoughtStep emissions, interchangeability.

### Epic 3: Chain-of-Thought Strategy
Developer can build agents with structured step-by-step reasoning.
**FRs covered:** FR6, FR7
**ARs covered:** AR6
**Delivers:** `ChainOfThoughtStrategy` with builder, configurable max steps, structured thought output.

### Epic 4: MCTS Strategy
Developer can build agents that explore parallel reasoning branches and select optimal paths.
**FRs covered:** FR4, FR5, FR20
**ARs covered:** AR3, AR6, AR7
**Delivers:** `MctsStrategy` with parallel branch evaluation, scoring, and path selection.

### Epic 5: Streaming, Documentation & Release
Developer can stream thought steps in real-time and has complete docs for adoption.
**FRs covered:** FR11, FR12, FR13, FR21, FR22, FR23, FR24, FR25, FR26, FR27
**NFRs verified:** NFR1-12
**Delivers:** `StreamingOutputTransformer` trait, per-strategy examples, migration guide, backward compatibility, rustdoc.

---

## Epic 1: Strategies Crate Foundation

### Story 1.1: Scaffold `traitclaw-strategies` Crate

As a developer,
I want to add `traitclaw-strategies` as a workspace crate,
So that I have a properly configured crate to implement reasoning strategies.

**Acceptance Criteria:**

**Given** the TraitClaw workspace
**When** `traitclaw-strategies` is added to workspace `Cargo.toml`
**Then** a new crate exists at `crates/traitclaw-strategies/` with:
- `Cargo.toml` with `traitclaw-core` dependency
- Feature flags: `react`, `mcts`, `cot` (default = all on)
- `src/lib.rs` with feature-gated module declarations
- `src/common/mod.rs` with public re-exports
**And** `cargo check -p traitclaw-strategies` succeeds
**And** `cargo check -p traitclaw-strategies --no-default-features` succeeds
**And** internal types use `pub(crate)`, public API re-exported from `lib.rs`

### Story 1.2: Implement `ThoughtStep` Enum

As a developer,
I want a shared `ThoughtStep` type for all strategies,
So that I can observe and log reasoning steps in a structured format.

**Acceptance Criteria:**

**Given** `traitclaw-strategies` crate exists
**When** `ThoughtStep` enum is implemented in `src/common/thought_step.rs`
**Then** it has variants: `Think { content }`, `Act { tool_name, tool_input }`, `Observe { tool_output }`, `Answer { content }`
**And** it derives `Debug, Clone, Serialize`
**And** it is re-exported from `traitclaw_strategies::common::ThoughtStep`
**And** unit tests verify serialization for each variant

### Story 1.3: Meta-Crate Re-Export

As a developer,
I want to access `traitclaw-strategies` through the `traitclaw` meta-crate,
So that I can use a single dependency for the full framework.

**Acceptance Criteria:**

**Given** `traitclaw-strategies` crate exists and compiles
**When** `traitclaw` meta-crate `Cargo.toml` has `strategies = ["dep:traitclaw-strategies"]`
**Then** `traitclaw::strategies::ThoughtStep` is accessible when `strategies` feature is enabled
**And** existing `traitclaw` features and re-exports remain unchanged
**And** `cargo test -p traitclaw` passes without regressions

## Epic 2: ReAct Strategy

### Story 2.1: `ReActStrategy` Builder & Core Structure

As a developer,
I want to instantiate a `ReActStrategy` with configurable options,
So that I can assign it to an agent via the builder pattern.

**Acceptance Criteria:**

**Given** `traitclaw-strategies` crate with `react` feature enabled
**When** developer calls `ReActStrategy::builder().max_iterations(10).build()?`
**Then** a valid `ReActStrategy` instance is returned
**And** default `max_iterations` is 10 if not specified
**And** builder validates inputs (e.g., `max_iterations > 0`)
**And** `ReActStrategy` implements `AgentStrategy` trait
**And** `ReActStrategy` is object-safe (`Box<dyn AgentStrategy>`)

### Story 2.2: Think→Act→Observe Loop Execution

As a developer,
I want `ReActStrategy` to autonomously execute reasoning loops,
So that my agent can reason through multi-step problems with tool use.

**Acceptance Criteria:**

**Given** an `Agent` configured with `ReActStrategy` and available tools
**When** the agent processes a user query
**Then** the strategy executes Think→Act→Observe cycles
**And** each cycle emits corresponding `ThoughtStep` events (`Think`, `Act`, `Observe`)
**And** the loop terminates when strategy produces `Answer` or hits `max_iterations`
**And** tool calls in `Act` step are routed through `AgentRuntime` tool registry
**And** `ThoughtStep` sequence is inspectable after execution (FR19)

### Story 2.3: Strategy Interchangeability

As a developer,
I want to swap `ReActStrategy` for any other `AgentStrategy` implementation,
So that I can experiment with different reasoning approaches without refactoring.

**Acceptance Criteria:**

**Given** an `Agent` configured with `ReActStrategy`
**When** developer replaces `.strategy(ReActStrategy::builder().build()?)` with `.strategy(custom_strategy)`
**Then** the agent compiles and runs correctly with the new strategy
**And** no other code changes are required (FR8)
**And** built-in and custom strategies coexist in the same application (FR9)
**And** unit test demonstrates both strategies running in same test

## Epic 3: Chain-of-Thought Strategy

### Story 3.1: `ChainOfThoughtStrategy` Builder & Execution

As a developer,
I want to instantiate a `ChainOfThoughtStrategy` that injects step-by-step reasoning,
So that my agent produces structured, explainable thought chains.

**Acceptance Criteria:**

**Given** `traitclaw-strategies` crate with `cot` feature enabled
**When** developer calls `ChainOfThoughtStrategy::builder().max_steps(5).build()?`
**Then** a valid `ChainOfThoughtStrategy` instance is returned that implements `AgentStrategy`
**And** the strategy injects step-by-step reasoning instructions into the agent's prompt
**And** each reasoning step is captured as a `ThoughtStep::Think` event
**And** the final answer is captured as `ThoughtStep::Answer`
**And** execution terminates when answer is produced or `max_steps` reached
**And** `ThoughtStep` sequence is inspectable after execution
**And** object-safe (`Box<dyn AgentStrategy>`)

## Epic 4: MCTS Strategy

### Story 4.1: `MctsStrategy` Builder & Configuration

As a developer,
I want to instantiate a `MctsStrategy` with configurable branch count and search depth,
So that I can tune the exploration-exploitation trade-off for my use case.

**Acceptance Criteria:**

**Given** `traitclaw-strategies` crate with `mcts` feature enabled
**When** developer calls `MctsStrategy::builder().branches(5).max_depth(3).build()?`
**Then** a valid `MctsStrategy` instance is returned that implements `AgentStrategy`
**And** default `branches` is 5 and `max_depth` is 3 if not specified
**And** custom `ScoringFn` can be provided via `.scoring(Arc::new(|s| score))`
**And** default scoring uses LLM self-evaluation
**And** object-safe (`Box<dyn AgentStrategy>`)

### Story 4.2: Parallel Branch Evaluation & Path Selection

As a developer,
I want `MctsStrategy` to execute parallel reasoning branches and select the best path,
So that my agent explores multiple approaches and picks the optimal answer.

**Acceptance Criteria:**

**Given** an `Agent` configured with `MctsStrategy`
**When** the agent processes a user query
**Then** the strategy spawns `branches` parallel tasks via `tokio::spawn` + `JoinSet`
**And** each branch explores reasoning paths up to `max_depth`
**And** each branch is scored using the configured `ScoringFn`
**And** the highest-scoring path is selected as the final result
**And** branch scores and selected path are exposed for post-execution analysis (FR20)
**And** all spawned tasks are properly cleaned up via `JoinSet`

## Epic 5: Streaming, Documentation & Release

### Story 5.1: `StreamingOutputTransformer` Trait

As a developer,
I want a trait to transform agent output as it streams,
So that I can process thought steps in real-time without waiting for completion.

**Acceptance Criteria:**

**Given** `traitclaw-core` crate
**When** `StreamingOutputTransformer` trait is added in `src/streaming.rs`
**Then** it has `async fn transform_chunk(&self, chunk: &str) -> Result<String>` (required)
**And** it has `async fn on_thought_step(&self, step: &ThoughtStep) -> Result<()>` (default no-op)
**And** trait is `Send + Sync` and re-exported from `traitclaw_core`
**And** trait composes alongside existing `OutputTransformer` (additive, not replacing)
**And** existing `OutputTransformer` behavior is unchanged

### Story 5.2: Streaming Integration with ReAct

As a developer,
I want to use `StreamingOutputTransformer` with `ReActStrategy`,
So that I can stream individual thought steps as they happen.

**Acceptance Criteria:**

**Given** an `Agent` with `ReActStrategy` and a `StreamingOutputTransformer`
**When** the agent processes a query
**Then** `transform_chunk` is called for each output chunk
**And** `on_thought_step` is called for each `ThoughtStep` event
**And** streaming latency < 10ms from first token to first emission (NFR5)

### Story 5.3: Per-Strategy Examples

As a developer,
I want runnable examples for each strategy,
So that I can quickly understand how to use them in my own projects.

**Acceptance Criteria:**

**Given** all strategies implemented
**When** examples are created at `examples/XX-react-strategy/`, `XX-cot-strategy/`, `XX-mcts-strategy/`
**Then** each example compiles and demonstrates core usage
**And** `StreamingOutputTransformer` example at `XX-streaming-thought/` demonstrates streaming
**And** each example has a `README.md` explaining the strategy
**And** `cargo build --examples` compiles all examples

### Story 5.4: Backward Compatibility & Regression Suite

As a developer,
I want v0.7.0 to be fully backward compatible with v0.6.0,
So that my existing code continues to work without modifications.

**Acceptance Criteria:**

**Given** the complete v0.7.0 codebase
**When** all existing examples (1–24) are compiled and tested
**Then** all compile without modification (FR22)
**And** all v0.6.0 public APIs remain unchanged (FR21)
**And** custom `AgentStrategy` implementations from v0.2.0+ work unchanged (FR23)
**And** `cargo test --workspace` passes with zero regressions

### Story 5.5: Rustdoc, Migration Guide & NFR Validation

As a developer,
I want complete documentation and verified performance,
So that I can confidently adopt v0.7.0 in production.

**Acceptance Criteria:**

**Given** all features implemented
**When** documentation is finalized
**Then** all public types have `///` doc comments with `# Examples` (FR27)
**And** `cargo test --doc` passes (doc examples compile)
**And** migration guide at `docs/migration-v0.6-to-v0.7.md` covers all new types
**And** compile time delta < 2% (NFR1), binary size delta < 5% (NFR2)
**And** ReAct loop latency < 1ms/cycle (NFR3), MCTS spawn < 100μs/branch (NFR4)
