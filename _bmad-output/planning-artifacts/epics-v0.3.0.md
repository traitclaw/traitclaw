---
stepsCompleted: ["step-01-validate-prerequisites", "step-02-design-epics", "step-03-create-stories", "step-04-final-validation"]
inputDocuments:
  - planning-artifacts/prd-v0.3.0.md
  - architecture.md
  - brainstorming/v030-context-window-rescue.md
---

# TraitClaw v0.3.0 — Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for TraitClaw v0.3.0 "Context Window Rescue", decomposing the requirements from the PRD and Architecture into implementable stories.

## Requirements Inventory

### Functional Requirements

FR1: Define async `ContextManager` trait in `traitclaw-core` with `prepare()` and `estimate_tokens()` methods, including blanket impl for existing sync `ContextStrategy` trait.

FR2: Provide built-in context managers: `RuleBasedCompressor` (importance scoring), `LlmCompressor` (provider-powered summarization), `TieredCompressor` (chained multi-tier compression).

FR3: Implement accurate token counting via `TikTokenCounter` (feature-gated `tiktoken-rs`) and `CharApproxCounter` (default, zero deps).

FR4: Define async `OutputTransformer` trait in `traitclaw-core` with context-aware `transform()` method accepting `tool_name`, `output`, and `AgentState`, including blanket impl for existing sync `OutputProcessor` trait.

FR5: Provide built-in output transformers: `BudgetAwareTransformer` (adaptive truncation), `JsonFieldExtractor` (JSON path extraction), `ChainTransformer` (compose multiple), `ProgressiveTransformer` (summary-first).

FR6: Define `ToolRegistry` trait in `traitclaw-core` with read methods (`active_schemas`, `active_tools`, `find`, `all_tools`) and write methods (`register`, `deactivate`, `activate`) using `&self` + interior mutability.

FR7: Provide built-in tool registries: `SimpleRegistry` (immutable default), `DynamicRegistry` (RwLock-backed mutation), `GroupedRegistry` (named groups), `AdaptiveRegistry` (tier-based selection).

FR8: Integrate all three traits into `DefaultStrategy` runtime loop and streaming path, replacing current direct Vec/sync calls with new trait dispatch.

FR9: Provide migration guide (`docs/migration-v0.2-to-v0.3.md`) and 3 new examples (`15-context-manager`, `16-output-transformer`, `17-tool-registry`).

### NonFunctional Requirements

NFR1: Zero overhead on default path — `SimpleRegistry` + `SlidingWindowManager` (blanket impl) + `TruncateTransformer` (blanket impl) must be performance-identical to v0.2.0.

NFR2: Backward compatibility — All v0.2.0 code must compile and run without modification on v0.3.0. No breaking changes (semver minor bump).

NFR3: Deprecation policy — `ContextStrategy` and `OutputProcessor` deprecated in v0.3.0 with `#[deprecated]` attribute. Removed only in v1.0.0 (major version).

NFR4: `RwLock` overhead in `DynamicRegistry` — Read lock < 1ns overhead. Write lock acceptable only for rare registration events (1-2x per session).

NFR5: `LlmCompressor` cost efficiency — One LLM call per compression event (not per iteration). Use cheap model (e.g., gpt-4o-mini) for summarization.

NFR6: MSRV Rust 1.75+ (required for async trait support).

NFR7: No new mandatory dependencies — `tiktoken-rs` behind optional feature flag `"tiktoken"`.

NFR8: Documentation — All public traits and methods must have rustdoc with examples. ADRs for all 3 new traits.

### Additional Requirements

- ADR-5: Async `ContextManager` with blanket impl for sync `ContextStrategy` — LLM-powered compression needs async.
- ADR-6: Async `OutputTransformer` with blanket impl for sync `OutputProcessor` — context-aware processing needs `AgentState`.
- ADR-7: `ToolRegistry` with `&self` + interior mutability for writes — `SimpleRegistry` default is immutable (zero overhead), `DynamicRegistry` uses `RwLock` for runtime mutation.
- ADR-8: Deprecation in v0.3.0, removal in v1.0.0 — breaking changes only in major versions. Blanket impls have zero runtime cost to maintain.
- Crate impact: changes limited to `traitclaw-core` (new traits + impls), `traitclaw` meta-crate (re-exports), and new examples.
- Unified extensibility pattern: all 3 traits follow same async-trait + blanket-impl + builder-injection pattern established in v0.2.0.

### UX Design Requirements

N/A — TraitClaw is a Rust framework/library with no UI. Developer experience is covered by FR9 (examples + migration guide).

### FR Coverage Map

| FR | Epic | Description |
|----|------|-------------|
| FR1 | Epic 1: Smart Context Survival | ContextManager trait + blanket impl |
| FR2 | Epic 1: Smart Context Survival | Built-in context managers |
| FR3 | Epic 1: Smart Context Survival | Token counting |
| FR4 | Epic 2: Intelligent Tool Output | OutputTransformer trait + blanket impl |
| FR5 | Epic 2: Intelligent Tool Output | Built-in output transformers |
| FR6 | Epic 3: Dynamic Tool Management | ToolRegistry trait |
| FR7 | Epic 3: Dynamic Tool Management | Built-in tool registries |
| FR8 | Epic 4: Production-Ready Context Window | DefaultStrategy integration |
| FR9 | Epic 5: Zero-Friction Upgrade | Migration guide + examples |

## Epic List

- **Epic 1: Smart Context Survival** — Async pluggable context management with LLM-powered compression (Phase 1)
- **Epic 2: Intelligent Tool Output Processing** — Context-aware async tool output processing (Phase 2)
- **Epic 3: Dynamic Tool Management** — Dynamic tool registry with activation/deactivation (Phase 2)
- **Epic 4: Production-Ready Context Window** — Wire all traits into DefaultStrategy loop (Phase 3)
- **Epic 5: Zero-Friction Upgrade** — Migration guide, examples, documentation (Phase 4)

---

## Epic 1: Smart Context Survival

**Goal:** Developers can run long-lived agents that retain important context across 50+ iterations without context window overflow, using pluggable async compression strategies.

**FRs:** FR1, FR2, FR3 | **NFRs:** NFR1, NFR2, NFR3, NFR5, NFR6, NFR7

### Story 1.1: ContextManager Trait Definition

As a framework developer,
I want the async `ContextManager` trait defined in `traitclaw-core`,
So that context management is pluggable and supports LLM-powered compression.

**Acceptance Criteria:**

**Given** `traitclaw-core/src/traits/context_manager.rs` is created
**When** I inspect the trait
**Then** `prepare()` is `async` and accepts `&mut Vec<Message>`, `context_window: usize`, `&mut AgentState`
**And** `estimate_tokens()` has a default impl using 4-chars ≈ 1-token approximation
**And** blanket impl `impl<T: ContextStrategy> ContextManager for T` exists
**And** `ContextStrategy` is marked `#[deprecated(since = "0.3.0", note = "Use ContextManager. Will be removed in v1.0.0")]`
**And** trait requires `Send + Sync`
**And** rustdoc includes a usage example
**And** unit test confirms trait is object-safe (`Arc<dyn ContextManager>` compiles)

### Story 1.2: ContextManager Integration into Agent

As a framework developer,
I want `AgentRuntime` to use `ContextManager` instead of `ContextStrategy`,
So that the runtime loop supports async context management.

**Acceptance Criteria:**

**Given** `AgentRuntime` has field `context_manager: Arc<dyn ContextManager>`
**When** `AgentBuilder::context_manager(impl ContextManager)` is called
**Then** the provided manager is stored and used in the runtime loop
**And** `AgentBuilder::context_strategy()` still works (wraps via blanket impl)
**And** `DefaultStrategy` calls `context_manager.prepare().await` instead of sync `prepare()`
**And** all existing v0.2.0 tests pass without modification
**And** backward compatibility test: code using `.context_strategy(SlidingWindowStrategy)` compiles unchanged

### Story 1.3: Token Counting Infrastructure

As a developer,
I want accurate token counting for context budget decisions,
So that compression triggers at the right time.

**Acceptance Criteria:**

**Given** `CharApproxCounter` struct exists with 4-chars ≈ 1-token logic
**When** I enable `"tiktoken"` feature flag in Cargo.toml
**Then** `TikTokenCounter` is available using `tiktoken-rs`
**And** `AgentState` has `estimated_tokens_used` field updated after each `prepare()` call
**And** `tiktoken-rs` is an optional dependency (not required by default)
**And** without feature flag, `CharApproxCounter` is used (zero new deps)
**And** accuracy test compares CharApprox vs TikToken results

### Story 1.4: Built-in Context Managers

As a developer,
I want pre-built context managers for common compression strategies,
So that I get intelligent context management without custom code.

**Acceptance Criteria:**

**Given** `RuleBasedCompressor` is implemented
**When** context exceeds threshold
**Then** it scores messages by importance (system=1.0, recent=0.9, tools=0.7, old=0.3) and removes lowest-scored first
**And** `LlmCompressor::new(provider)` accepts any `Provider` for summarization
**And** `LlmCompressor` has configurable `summary_prompt` template
**And** `LlmCompressor` makes exactly 1 LLM call per compression event (not per iteration)
**And** `TieredCompressor` chains: keep recent N → rule-compress mid → LLM-summarize old
**And** all managers update `AgentState` when messages are compressed
**And** integration test: 50-message conversation stays within token budget

---

## Epic 2: Intelligent Tool Output Processing

**Goal:** Developers can prevent tool outputs from wasting context window tokens through context-aware adaptive processing that understands remaining budget and tool semantics.

**FRs:** FR4, FR5 | **NFRs:** NFR1, NFR2, NFR3

### Story 2.1: OutputTransformer Trait Definition

As a framework developer,
I want the async `OutputTransformer` trait defined in `traitclaw-core`,
So that tool output processing is pluggable and context-aware.

**Acceptance Criteria:**

**Given** `traitclaw-core/src/traits/output_transformer.rs` is created
**When** I inspect the trait
**Then** `transform()` is `async` and receives `tool_name: &str`, `output: String`, `state: &AgentState`
**And** blanket impl `impl<T: OutputProcessor> OutputTransformer for T` exists
**And** `OutputProcessor` is marked `#[deprecated(since = "0.3.0", note = "Use OutputTransformer. Will be removed in v1.0.0")]`
**And** trait requires `Send + Sync`
**And** rustdoc includes a usage example
**And** unit test confirms trait is object-safe

### Story 2.2: OutputTransformer Integration into Agent

As a framework developer,
I want `AgentRuntime` to use `OutputTransformer` instead of `OutputProcessor`,
So that tool outputs are processed with context budget awareness.

**Acceptance Criteria:**

**Given** `AgentRuntime` has field `output_transformer: Arc<dyn OutputTransformer>`
**When** `AgentBuilder::output_transformer(impl OutputTransformer)` is called
**Then** the provided transformer is used after each tool execution
**And** `AgentBuilder::output_processor()` still works (wraps via blanket impl)
**And** `DefaultStrategy::process_tool_calls()` calls `output_transformer.transform().await`
**And** all existing v0.2.0 tests pass without modification
**And** backward compatibility test: code using `.output_processor(TruncateProcessor)` compiles unchanged

### Story 2.3: Built-in Output Transformers

As a developer,
I want pre-built output transformers for common processing strategies,
So that tool outputs don't waste context window tokens.

**Acceptance Criteria:**

**Given** `BudgetAwareTransformer` is implemented
**When** tool output is processed
**Then** it adapts truncation based on remaining token budget:
  - Budget > 80% → keep full output
  - Budget 40-80% → truncate to 5K chars
  - Budget < 40% → aggressive truncate to 1K chars
**And** `JsonFieldExtractor::new(fields)` extracts only specified JSON paths from tool output
**And** `ChainTransformer::new(vec![...])` composes multiple transformers in sequence
**And** `ProgressiveTransformer` returns summary first; provides full output on LLM follow-up request
**And** unit tests verify each transformer's behavior
**And** `ChainTransformer` test: JSON extract → budget-aware truncate works correctly

---

## Epic 3: Dynamic Tool Management

**Goal:** Developers can organize, activate, and deactivate tools at runtime, reducing schema overhead for agents with many tools through intelligent tool selection.

**FRs:** FR6, FR7 | **NFRs:** NFR1, NFR2, NFR4

### Story 3.1: ToolRegistry Trait Definition

As a framework developer,
I want the `ToolRegistry` trait defined in `traitclaw-core`,
So that tool management is pluggable with dynamic activation support.

**Acceptance Criteria:**

**Given** `traitclaw-core/src/traits/tool_registry.rs` is created
**When** I inspect the trait
**Then** it has read methods: `active_schemas()`, `active_tools()`, `find()`, `all_tools()`
**And** it has write methods: `register()`, `deactivate()`, `activate()`
**And** all methods use `&self` (interior mutability for writes in mutable impls)
**And** trait requires `Send + Sync`
**And** rustdoc includes a usage example
**And** unit test confirms trait is object-safe

### Story 3.2: ToolRegistry Integration into Agent

As a framework developer,
I want `AgentRuntime` to use `ToolRegistry` for tool management,
So that only active tool schemas are sent to the LLM each iteration.

**Acceptance Criteria:**

**Given** `AgentRuntime` has field `tool_registry: Arc<dyn ToolRegistry>`
**When** `AgentBuilder::tool_registry(impl ToolRegistry)` is called
**Then** the provided registry is used in the runtime loop
**And** `AgentBuilder::tool(T)` still works (adds to internal `SimpleRegistry`)
**And** `DefaultStrategy` calls `registry.active_schemas()` instead of `tools.iter().map(schema)`
**And** `DefaultStrategy` uses `registry.find(name)` for tool execution lookup
**And** all existing v0.2.0 tests pass without modification

### Story 3.3: Built-in Tool Registries

As a developer,
I want pre-built registries for common tool management patterns,
So that I can reduce tool schema overhead with minimal code.

**Acceptance Criteria:**

**Given** `SimpleRegistry` wraps a `Vec<Arc<dyn ErasedTool>>`
**When** all tools are registered
**Then** `SimpleRegistry` keeps all active (immutable, zero overhead — the default)
**And** `SimpleRegistry::register()` returns `Err("immutable registry")` (clear error)
**And** `DynamicRegistry` supports runtime `register()`/`deactivate()`/`activate()` via `RwLock`
**And** `GroupedRegistry` organizes tools into named groups with group-level activation
**And** `AdaptiveRegistry` auto-limits active tools based on `ModelTier` (Small→5, Medium→15, Large→unlimited)
**And** `DynamicRegistry` read lock overhead is < 1ns per call
**And** unit tests verify each registry's activation/deactivation behavior

---

## Epic 4: Production-Ready Context Window

**Goal:** All three context optimization traits work together seamlessly in the agent runtime, providing a fully integrated context management system tested under production-like conditions.

**FRs:** FR8 | **NFRs:** NFR1, NFR2

### Story 4.1: DefaultStrategy Full Integration

As a framework developer,
I want all three new traits wired into the DefaultStrategy runtime loop,
So that context management, output transformation, and tool selection are coordinated.

**Acceptance Criteria:**

**Given** `DefaultStrategy` runtime loop is refactored
**When** an agent runs with all three traits configured
**Then** the loop sequence is:
  1. Load context (memory + system prompt + user message)
  2. `tool_registry.active_schemas()` → only active tool schemas
  3. inject hints
  4. `context_manager.prepare().await` → async context management
  5. Build `CompletionRequest` with pruned messages + active schemas
  6. LLM call
  7. If tool calls → execute → `output_transformer.transform().await` → inject result
  8. Loop back to step 2
**And** streaming path (`stream_runtime`) also uses all three traits
**And** integration test: agent with `LlmCompressor` + `BudgetAwareTransformer` + `GroupedRegistry` runs successfully
**And** stress test: 50+ iterations on 128K context model without overflow
**And** all existing v0.2.0 examples (01-14) compile and run unchanged

---

## Epic 5: Zero-Friction Upgrade

**Goal:** Developers can upgrade from v0.2.0 with zero code changes, learn new features through guided examples, and adopt traits incrementally.

**FRs:** FR9 | **NFRs:** NFR2, NFR3, NFR8

### Story 5.1: Migration Guide

As a developer upgrading from v0.2.0,
I want a clear migration guide,
So that I understand what changed and how to adopt new features.

**Acceptance Criteria:**

**Given** `docs/migration-v0.2-to-v0.3.md` is created
**When** I read the guide
**Then** it confirms "No breaking changes. v0.2.0 code compiles unchanged."
**And** it lists all deprecated traits with replacement mapping
**And** it shows incremental adoption: adopt ContextManager → OutputTransformer → ToolRegistry one at a time
**And** it provides "before (v0.2.0)" vs "after (v0.3.0)" code snippets for each new feature
**And** deprecation warnings mention v1.0.0 removal timeline

### Story 5.2: Context Manager Example

As a developer,
I want `examples/15-context-manager/` demonstrating intelligent context compression,
So that I can learn how to prevent context overflow.

**Acceptance Criteria:**

**Given** `examples/15-context-manager/` is created
**When** I run the example
**Then** it demonstrates `LlmCompressor` and `TieredCompressor` usage
**And** console output shows context size reduction metrics
**And** README explains when and why to use each compressor
**And** example compiles and runs successfully

### Story 5.3: Output Transformer Example

As a developer,
I want `examples/16-output-transformer/` demonstrating tool output optimization,
So that I can learn how to reduce context waste from tool outputs.

**Acceptance Criteria:**

**Given** `examples/16-output-transformer/` is created
**When** I run the example
**Then** it demonstrates `BudgetAwareTransformer` and `JsonFieldExtractor`
**And** console output shows token savings
**And** example compiles and runs successfully

### Story 5.4: Tool Registry Example

As a developer,
I want `examples/17-tool-registry/` demonstrating dynamic tool management,
So that I can learn how to organize and activate tool groups.

**Acceptance Criteria:**

**Given** `examples/17-tool-registry/` is created
**When** I run the example
**Then** it demonstrates `GroupedRegistry` with group activation/deactivation
**And** it demonstrates `DynamicRegistry` with runtime tool registration
**And** example compiles and runs successfully

---

## Implementation Order (Dependency Graph)

```
Phase 1 (Week 1-2):
  Epic 1: Story 1.1 → 1.2 → 1.3 → 1.4

Phase 2 (Week 3-4):
  Epic 2: Story 2.1 → 2.2 → 2.3  (parallel with Epic 3)
  Epic 3: Story 3.1 → 3.2 → 3.3

Phase 3 (Week 5-6):
  Epic 4: Story 4.1

Phase 4 (Week 7-8):
  Epic 5: Stories 5.1, 5.2, 5.3, 5.4 (parallel)
```

**Critical Path:** Epics 1, 2, 3 must complete before Epic 4 (Integration). Epic 5 (Docs) depends on Epic 4.

