---
stepsCompleted: [1, 2, 3]
inputDocuments:
  - architecture.md
  - project-context.md
  - brainstorming/architecture-deep-dive.md
  - brainstorming/guard-hint-track-steering.md
---

# TraitClaw - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for TraitClaw, decomposing the Architecture requirements into implementable stories. Organized by the 3-phase MVP roadmap.

## Requirements Inventory

### Functional Requirements

- FR1: Agent must complete a chat interaction with any OpenAI-compatible LLM
- FR2: Agent must support tool calling with automatic JSON Schema generation
- FR3: Agent must support conversation memory (in-memory default)
- FR4: Agent must support streaming responses via async streams
- FR5: Agent must support structured output with Rust type deserialization
- FR6: Agent must support Guard trait for blocking dangerous actions
- FR7: Agent must support Hint trait for injecting guidance messages
- FR8: Agent must support Tracker trait for runtime monitoring
- FR9: Agent must support multiple LLM providers (OpenAI, Anthropic, Ollama)
- FR10: Agent must support persistent memory (SQLite)
- FR11: Agent must support multi-agent orchestration
- FR12: Agent must support RAG pipeline
- FR13: Agent must support MCP protocol
- FR14: Agent must provide evaluation framework
- FR15: Agent must support session management (multi-user, multi-conversation)
- FR16: Agent must manage context window (prevent overflow)
- FR17: Agent must support configurable tool execution strategy (sequential/parallel)
- FR18: Agent must support tool output processing (truncation, transformation)
- FR19: Agent must support provider retry with backoff
- FR20: Agent must support one-line steering auto-configuration

### NonFunctional Requirements

- NFR1: Zero-cost abstractions — unused features add no overhead
- NFR2: Compile-time safety — type errors caught at compile time
- NFR3: `Send + Sync + 'static` on all trait objects
- NFR4: Feature-gated optional dependencies
- NFR5: 80%+ test coverage on traitclaw-core
- NFR6: All public items must have doc comments
- NFR7: CI: cargo fmt + clippy + test + doc must pass
- NFR8: Max 300 lines per file

### Additional Requirements (Architecture)

- Workspace structure with multiple crates under `crates/`
- Builder pattern for Agent construction
- `prelude` module for convenient imports
- Provider must include `ModelInfo` with `ModelTier` for steering auto-config
- Guard/Hint/Tracker traits in core, implementations in optional `steering` crate

### FR Coverage Map

| FR | Epic | Stories |
|----|------|---------|
| FR1 | Epic 1 | 1.1, 1.2, 1.3, 1.4, 1.6 |
| FR2 | Epic 2 | 2.1, 2.2, 2.3 |
| FR3 | Epic 1 | 1.5 |
| FR4 | Epic 1 | 1.8 |
| FR5 | Epic 3 | 3.3 |
| FR6, FR7, FR8 | Epic 3 | 3.1, 3.2, 3.4 |
| FR9 | Epic 4 | 4.1, 4.2, 4.3 |
| FR10 | Epic 4 | 4.4 |
| FR11 | Epic 5 | 5.3 |
| FR12 | Epic 5 | 5.2 |
| FR13 | Epic 5 | 5.1 |
| FR14 | Epic 5 | 5.4 |
| FR15 | Epic 1 | 1.10 |
| FR16 | Epic 1 | 1.11 |
| FR17 | Epic 1 | 1.12 |
| FR18 | Epic 1 | 1.13 |
| FR19 | Epic 1 | 1.14 |
| FR20 | Epic 3 | 3.4 |

## Epic List

- **Epic 1: Core Foundation** — Agent, Provider, Runtime, Memory, Messages, Session, Context, Execution Strategy (Phase 1-2)
- **Epic 2: Tool System** — Tool trait, derive macros, tool execution (Phase 1)
- **Epic 3: Steering & Structured Output** — Guard-Hint-Track, structured output, Steering::auto() (Phase 2)
- **Epic 4: Providers & Persistence** — OpenAI/Anthropic/Ollama providers, SQLite memory (Phase 2)
- **Epic 5: Advanced Features** — MCP, RAG, Multi-agent, Eval (Phase 3)
- **Epic 6: Examples** — Progressive examples (Phase 2-3)
- **Epic 7: Production Hardening** — AgentOutput, feature flags, observability, error recovery (Phase 3)

---

## Epic 1: Core Foundation

**Goal:** Establish the workspace, core traits, types, runtime loop, and meta-crate so that a basic chat agent works end-to-end with 5 lines of code.

### Story 1.1: Workspace & Crate Scaffolding

As a developer,
I want the Cargo workspace and crate structure set up,
So that I can start implementing core types with proper module organization.

**Acceptance Criteria:**

**Given** a fresh clone of the repository
**When** I run `cargo build`
**Then** all crates compile successfully with no warnings
**And** the workspace structure matches project-context.md specifications
**And** `crates/traitclaw-core/src/lib.rs` exists with module declarations
**And** `crates/traitclaw/src/lib.rs` re-exports traitclaw-core

### Story 1.2: Core Types & Message System

As a developer,
I want Message, CompletionRequest, CompletionResponse and related types defined,
So that providers and the runtime have a common type language.

**Acceptance Criteria:**

**Given** the types module exists
**When** I create a `Message` with role and content
**Then** it serializes to JSON matching OpenAI message format
**And** `MessageRole` enum has System, User, Assistant, Tool variants
**And** `CompletionRequest` contains messages, tools, model, temperature, max_tokens
**And** `CompletionResponse` contains content (text or tool_calls), usage stats
**And** all types implement `Clone`, `Debug`, `Serialize`, `Deserialize`

### Story 1.3: Provider Trait

As a developer,
I want a `Provider` trait that abstracts LLM communication,
So that any LLM can be plugged in behind a common interface.

**Acceptance Criteria:**

**Given** the Provider trait is defined
**When** I implement it for a mock provider
**Then** `complete()` accepts `CompletionRequest` and returns `Result<CompletionResponse>`
**And** `stream()` returns `Result<CompletionStream>`
**And** `model_info()` returns `ModelInfo` with name, tier, context_window, capabilities
**And** `ModelTier` enum has Small, Medium, Large variants
**And** the trait requires `Send + Sync + 'static`

### Story 1.4: Agent Builder & Config

As a developer,
I want an `AgentBuilder` with a fluent API to configure agents,
So that creating an agent is intuitive and discoverable.

**Acceptance Criteria:**

**Given** I use the builder pattern
**When** I call `Agent::builder().model(provider).system("...").build()`
**Then** it returns a configured `Agent` instance
**And** `.build()` returns `Result<Agent>` (error if no provider set)
**And** builder methods accept `impl Into<String>` for string parameters
**And** optional settings have sensible defaults (max_tokens, temperature, etc.)

### Story 1.5: In-Memory Memory & Memory Trait

As a developer,
I want a `Memory` trait with a default in-memory implementation,
So that agents can maintain conversation history out of the box.

**Acceptance Criteria:**

**Given** the Memory trait is defined with 3 layers
**When** I use `InMemoryMemory::new()`
**Then** `messages()` returns conversation history for a session
**And** `append()` adds a message to session history
**And** `get_context/set_context()` manages working memory key-value pairs
**And** `recall/store()` manages long-term memory entries
**And** InMemoryMemory is the default when no memory is configured

### Story 1.6: Agent Runtime Loop

As a developer,
I want the agent runtime loop that orchestrates LLM calls and tool execution,
So that `agent.run("Hello")` produces a complete response.

**Acceptance Criteria:**

**Given** an Agent with a provider and system prompt
**When** I call `agent.run("Hello")`
**Then** it assembles context (system prompt + memory + user message)
**And** sends CompletionRequest to provider
**And** if response is text → returns AgentOutput::Text
**And** if response is tool_calls → executes tools → feeds results back → loops
**And** loop terminates when LLM returns text (not tool calls)
**And** conversation is saved to memory after completion
**And** Guard/Hint/Tracker hook points exist (Noop by default)

### Story 1.7: Error Types & Result Alias

As a developer,
I want well-defined error types for traitclaw-core,
So that errors are descriptive and actionable.

**Acceptance Criteria:**

**Given** the error module exists
**When** an error occurs during agent execution
**Then** it returns a typed `Error` enum variant (not a string)
**And** variants include: Provider, ToolExecution, Memory, Config, Runtime
**And** `type Result<T> = std::result::Result<T, Error>` alias exists
**And** errors implement `std::error::Error` via `thiserror`

### Story 1.8: Streaming Support

As a developer,
I want `agent.stream("Hello")` to return an async stream of response chunks,
So that I can display responses incrementally.

**Acceptance Criteria:**

**Given** an Agent with a provider that supports streaming
**When** I call `agent.stream("Hello")`
**Then** it returns `Result<AgentStream>`
**And** AgentStream implements `Stream<Item = Result<StreamEvent>>`
**And** StreamEvent has variants: TextDelta, ToolCallStart, ToolCallDelta, Done
**And** the stream properly handles tool calls mid-stream (pause stream → execute → resume)

### Story 1.9: Prelude & Meta-crate Re-exports

As a developer,
I want `use traitclaw::prelude::*` to import everything I need,
So that getting started requires minimal import knowledge.

**Acceptance Criteria:**

**Given** the traitclaw meta-crate exists
**When** I add `traitclaw = "0.1"` to my Cargo.toml
**Then** `use traitclaw::prelude::*` imports Agent, AgentBuilder, Provider, Tool, Memory, Message types
**And** the meta-crate feature-gates optional crate re-exports
**And** default features provide a working agent with `openai-compat` provider

### Story 1.10: Session Management

As a developer,
I want to manage multiple conversations with separate session IDs,
So that different users/conversations have isolated memory.

**Acceptance Criteria:**

**Given** the `Memory` trait is extended with session lifecycle methods
**When** I call `agent.session("user-123")`
**Then** it returns an `AgentSession` wrapper bound to that session ID
**And** `session.say("Hello")` uses the bound session ID for memory operations
**And** `agent.session_auto()` creates a new session with UUID and returns `AgentSession`
**And** `agent.run(input)` is backward-compatible (auto-creates ephemeral session)
**And** `runtime.rs` and `streaming.rs` accept `session_id` parameter instead of hardcode `"default"`
**And** `Memory` trait has default impls: `create_session() -> String`, `list_sessions()`, `delete_session(id)`
**And** `InMemoryMemory` implements session lifecycle using internal HashMap keys

### Story 1.11: Context Window Management

As a developer,
I want pluggable context window management to prevent overflow,
So that agents handle long conversations without crashing.

**Acceptance Criteria:**

**Given** a `ContextStrategy` trait is defined in `traitclaw-core/src/traits/`
**When** no strategy is configured
**Then** `SlidingWindowStrategy` is used by default (threshold = 0.85)
**And** it estimates tokens (4 chars ≈ 1 token) and removes oldest non-system messages when over threshold
**And** `AgentState.last_output_truncated` is set to `true` when messages are removed
**And** `AgentBuilder::context_strategy(impl ContextStrategy)` allows custom strategies
**And** `NoopContextStrategy` is available for users who want no automatic management
**And** runtime calls `context_strategy.prepare()` before every LLM call

### Story 1.12: Tool Execution Strategy

As a developer,
I want configurable tool execution (sequential, parallel, or custom),
So that I can optimize for safety or speed depending on use case.

**Acceptance Criteria:**

**Given** an `ExecutionStrategy` trait with `#[async_trait]` is defined
**When** no strategy is configured
**Then** `SequentialStrategy` is used by default (backward-compatible)
**And** `ParallelStrategy { max_concurrency }` runs tool calls concurrently via `tokio::JoinSet`
**And** `AdaptiveStrategy` uses `Tracker::recommended_concurrency()` to decide
**And** Guard checks still run before each tool execution regardless of strategy
**And** `AgentBuilder::execution_strategy(impl ExecutionStrategy)` allows custom strategies
**And** runtime delegates to `strategy.execute_batch()` instead of inline loop

### Story 1.13: Tool Output Processing

As a developer,
I want pluggable tool output processing (truncation, transformation, filtering),
So that I can prevent context overflow and customize tool output handling.

**Acceptance Criteria:**

**Given** an `OutputProcessor` trait is defined (sync)
**When** no processor is configured
**Then** `TruncateProcessor` is used by default (max 10,000 chars)
**And** output exceeding the limit is truncated with `"[output truncated]"` suffix
**And** `NoopProcessor` is available for users who want raw output
**And** `ChainProcessor` allows composing multiple processors in a pipeline
**And** `AgentBuilder::output_processor(impl OutputProcessor)` allows custom processors
**And** runtime applies processor after each tool execution, before adding result to messages

### Story 1.14: Provider Retry with Backoff

As a developer,
I want automatic retry with exponential backoff for transient provider errors,
So that my agent is resilient to temporary API issues.

**Acceptance Criteria:**

**Given** a `RetryProvider` wrapper in `traitclaw-core`
**When** the inner provider returns a transient error (429, 500, 502, 503, 504, timeout)
**Then** it retries up to `max_retries` times (default: 3)
**And** uses exponential backoff starting at `initial_delay` (default: 500ms)
**And** caps delay at `max_delay` (default: 30s)
**And** non-transient errors are propagated immediately without retry
**And** `RetryProvider` implements `Provider` trait (decorator pattern)
**And** `AgentBuilder::with_retry(config)` is a convenience method

---

## Epic 2: Tool System

**Goal:** Enable developers to define tools using derive macros, with automatic JSON Schema generation and type-safe execution.

### Story 2.1: Tool Trait & ErasedTool

As a developer,
I want a `Tool` trait with associated Input/Output types,
So that tools are type-safe and have auto-generated schemas.

**Acceptance Criteria:**

**Given** the Tool trait is defined
**When** I implement it with typed Input/Output
**Then** `schema()` returns a `ToolSchema` with JSON Schema from the Input type
**And** `execute()` takes validated Input and returns typed Output
**And** `ErasedTool` wrapper enables `Vec<Arc<dyn ErasedTool>>` storage in Agent
**And** ErasedTool handles JSON → Input deserialization → execute → Output → JSON

### Story 2.2: #[derive(Tool)] Macro

As a developer,
I want `#[derive(Tool)]` to auto-generate tool boilerplate from a struct,
So that defining tools takes minimal code.

**Acceptance Criteria:**

**Given** a struct annotated with `#[derive(Tool)]`
**When** I compile the code
**Then** it auto-generates `Tool` impl with name, description, schema
**And** `#[tool(description = "...")]` sets the tool description
**And** struct fields become tool parameters with doc comments as descriptions
**And** `#[tool(default = value)]` sets default parameter values
**And** JSON Schema is derived from field types via `schemars`
**And** compile error if struct doesn't have an `execute` method with correct signature

### Story 2.3: Tool Integration in Runtime

As a developer,
I want the agent runtime to automatically discover and execute tools,
So that LLM tool calls are handled transparently.

**Acceptance Criteria:**

**Given** an Agent with tools registered via `.tool(MyTool)` or `.tools([A, B])`
**When** the LLM returns a tool_call response
**Then** the runtime matches the tool by name
**And** deserializes arguments to the tool's Input type
**And** executes the tool
**And** serializes the Output and feeds it back as a tool result message
**And** if tool name not found → returns error message to LLM
**And** if deserialization fails → returns descriptive error to LLM

---

## Epic 3: Steering & Structured Output

**Goal:** Implement Guard-Hint-Track steering system and structured output support.

### Story 3.1: Guard-Hint-Track Trait Integration

As a developer,
I want Guard, Hint, and Tracker traits wired into the runtime,
So that the runtime supports model steering when steering feature is enabled.

**Acceptance Criteria:**

**Given** the Guard/Hint/Tracker traits are defined in core
**When** no steering is configured
**Then** NoopGuard/NoopHint/NoopTracker are used (zero overhead)
**And** when Guards are configured, every Action goes through `guard.check()` before execution
**And** when Hints are configured, `hint.should_trigger()` is checked each iteration
**And** when Tracker is configured, state is updated after each LLM call/tool call
**And** `AgentState` struct tracks: iteration_count, token_usage, context_utilization

### Story 3.2: Built-in Steering Implementations

As a developer,
I want pre-built Guards, Hints, and Tracker implementations,
So that I can enable model steering with `Steering::auto()`.

**Acceptance Criteria:**

**Given** the `traitclaw-steering` crate
**When** I use `.steering(Steering::auto())`
**Then** it auto-configures Guards/Hints/Tracker based on `ModelTier`
**And** `ShellDenyGuard` blocks dangerous shell commands (50+ patterns minimum)
**And** `LoopDetectionGuard` detects repeated identical tool calls
**And** `BudgetHint` warns at 75% token budget
**And** `SystemPromptReminder` re-injects key rules at recency zone
**And** `AdaptiveTracker` adjusts concurrency based on context utilization

### Story 3.3: Structured Output

As a developer,
I want `agent.run_structured::<T>(input)` to return a typed Rust value,
So that LLM output is parsed into a compile-time verified struct.

**Acceptance Criteria:**

**Given** a type T that implements `DeserializeOwned + JsonSchema`
**When** I call `agent.run_structured::<T>("query")`
**Then** if `model_info.supports_structured` is true → uses native `response_format: json_schema`
**And** if not → falls back to injecting JSON schema instructions into system prompt
**And** the response is deserialized into T via `serde_json::from_str`
**And** if deserialization fails → retries with error feedback (up to 3 times)
**And** the JSON Schema is automatically derived from T's `JsonSchema` impl
**And** returns `Result<T>` (not `AgentOutput`)

### Story 3.4: Steering::auto() Facade

As a developer,
I want `Steering::auto()` to auto-configure all steering based on model tier,
So that I get optimal Guard/Hint/Tracker setup with one line of code.

**Acceptance Criteria:**

**Given** the `traitclaw-steering` crate
**When** I call `Agent::builder().provider(p).steering(Steering::auto()).build()`
**Then** `Steering::auto()` returns a marker; `.build()` reads `provider.model_info().tier` to resolve config
**And** Small tier → aggressive: ShellDeny + Loop(3) + Budget(50) + Injection + Workspace, BudgetHint(0.5), Reminder(4), Progress(3)
**And** Medium tier → balanced: ShellDeny + Loop(3) + Budget(50) + Injection, BudgetHint(0.75), Reminder(8), Progress(5)
**And** Large tier → relaxed: ShellDeny + Loop(5) + Budget(100), BudgetHint(0.80), Reminder(15)
**And** `Steering::for_tier(ModelTier::Medium)` allows explicit tier selection
**And** `Steering::custom()` starts empty for manual guard/hint/tracker configuration

---

## Epic 4: Providers & Persistence

**Goal:** Implement real LLM providers and persistent memory so agents work with production services.

### Story 4.1: OpenAI-Compatible Provider

As a developer,
I want an OpenAI-compatible provider that works with any OpenAI API endpoint,
So that I can use OpenAI, Azure OpenAI, local vLLM, etc.

**Acceptance Criteria:**

**Given** `traitclaw-openai-compat` crate
**When** I configure with `openai_compat("http://localhost:8080/v1", "api-key")`
**Then** it sends requests to the specified endpoint
**And** supports `/chat/completions` endpoint
**And** supports tool calling format
**And** supports streaming via SSE
**And** reuses reqwest Client for connection pooling

### Story 4.2: Native OpenAI Provider

As a developer,
I want a native OpenAI provider with full API support,
So that I get the best experience with OpenAI models.

**Acceptance Criteria:**

**Given** `traitclaw-openai` crate with feature `"openai"`
**When** I use `openai("gpt-4o")`
**Then** it connects to `api.openai.com` with proper authentication
**And** supports all GPT-4o/4o-mini models
**And** auto-sets `ModelTier` based on model name (gpt-4o=Large, gpt-4o-mini=Medium)
**And** supports structured output via `response_format: json_schema`

### Story 4.3: Anthropic Provider

As a developer,
I want an Anthropic provider for Claude models,
So that I can use Claude with proper prompt caching support.

**Acceptance Criteria:**

**Given** `traitclaw-anthropic` crate with feature `"anthropic"`
**When** I use `anthropic("claude-sonnet-4-20250514")`
**Then** it sends requests to Anthropic API format (different from OpenAI)
**And** supports tool use in Anthropic format
**And** supports streaming
**And** auto-sets ModelTier (opus=Large, sonnet=Medium, haiku=Small)

### Story 4.4: SQLite Memory Backend

As a developer,
I want persistent memory using SQLite,
So that agent conversations survive restarts.

**Acceptance Criteria:**

**Given** `traitclaw-memory-sqlite` crate with feature `"sqlite"`
**When** I use `.memory(SqliteMemory::new("./agent.db"))`
**Then** messages are persisted to SQLite database
**And** schema: `sessions(id, created_at)`, `messages(session_id, role, content, tool_call_id, created_at)`
**And** schema: `working_memory(session_id, key, value)`, `long_term_memory(id, content, metadata)`
**And** long-term recall uses FTS5 virtual table for text search
**And** implements full `Memory` trait including `create_session`/`list_sessions`/`delete_session`
**And** database schema is auto-created/migrated on `new()`
**And** concurrent access is handled safely
**And** uses `rusqlite` with `bundled` feature

---

## Epic 5: Advanced Features

**Goal:** MCP protocol, RAG pipeline, multi-agent, and evaluation framework.

### Story 5.1: MCP Client Support

As a developer,
I want agents to connect to MCP servers as tool sources,
So that I can use external tool providers.

**Acceptance Criteria:**

**Given** `traitclaw-mcp` crate with feature `"mcp"`
**When** I use `.mcp_server("npx @modelcontextprotocol/server-filesystem")`
**Then** the agent discovers tools from the MCP server
**And** MCP tools are available alongside native tools
**And** tool calls to MCP tools are routed to the MCP server

### Story 5.2: RAG Pipeline

As a developer,
I want a RAG pipeline with Retriever trait,
So that agents can ground responses in external knowledge.

**Acceptance Criteria:**

**Given** `traitclaw-rag` crate with feature `"rag"`
**When** I implement a custom `Retriever`
**Then** retrieved documents are injected into agent context
**And** `GroundingStrategy` controls how documents are used
**And** built-in `KeywordRetriever` provides BM25 search

### Story 5.3: Multi-Agent Orchestration

As a developer,
I want to compose multiple agents into teams,
So that complex tasks can be split across specialized agents.

**Acceptance Criteria:**

**Given** `traitclaw-team` crate with feature `"team"`
**When** I create a team with router + specialist agents
**Then** messages are routed to the appropriate agent
**And** agents can delegate to each other
**And** `VerificationChain` enables generate → verify patterns

### Story 5.4: Evaluation Framework

As a developer,
I want to evaluate agent performance with metrics,
So that I can measure and improve agent quality.

**Acceptance Criteria:**

**Given** `traitclaw-eval` crate with feature `"eval"`
**When** I define an eval suite with test cases
**Then** it runs the agent on each test case
**And** computes metrics (faithfulness, hallucination rate, relevancy)
**And** generates a report with scores

---

## Epic 6: Examples

**Goal:** Progressive examples proving framework design and teaching developers.

### Story 6.1: Hello Agent Example

As a developer,
I want a minimal 5-line agent example,
So that I can validate the entire pipeline end-to-end.

**Acceptance Criteria:**

**Given** `examples/01-hello-agent/` exists
**When** I run the example
**Then** it creates an agent with provider + system prompt and calls `agent.run()`
**And** validates the full pipeline: Builder → Provider → LLM call → Response
**And** README explains every line
**And** example compiles and runs successfully

### Story 6.2: Tool Calling Example

As a developer,
I want a tool calling example with `#[derive(Tool)]`,
So that I can learn how to add tools to an agent.

**Acceptance Criteria:**

**Given** `examples/02-tool-calling/` exists
**When** I run the example
**Then** it defines 2 tools (Calculator + mock WeatherLookup) using `#[derive(Tool)]`
**And** validates: tool schema generation, tool execution, result feedback to LLM
**And** README explains tool definition and registration

### Story 6.3: Streaming Example

As a developer,
I want a streaming example using `agent.stream()`,
So that I can learn how to display incremental responses.

**Acceptance Criteria:**

**Given** `examples/03-streaming/` exists
**When** I run the example
**Then** it uses `agent.stream()` and prints each `TextDelta` as it arrives
**And** validates: SSE parsing, incremental output display

### Story 6.4: Steering Example

As a developer,
I want a steering example showing Guards and Hints,
So that I can learn how to protect and guide my agent.

**Acceptance Criteria:**

**Given** `examples/04-steering/` exists
**When** I run the example
**Then** it demonstrates `ShellDenyGuard` blocking dangerous commands
**And** demonstrates `BudgetHint` warning at threshold
**And** shows both manual guard/hint setup and `Steering::auto()`

### Story 6.5: Miniclaw Showcase

As a developer,
I want a mini OpenClaw-like assistant built with TraitClaw,
So that I can see the framework used in a real application.

**Acceptance Criteria:**

**Given** the `showcase/miniclaw` project
**When** I build and run it
**Then** it provides a working CLI AI assistant
**And** codebase is under 1000 lines
**And** it demonstrates: Agent, Tools, Memory, Steering
**And** README documents progressive enhancement steps

---

## Epic 7: Production Hardening

**Goal:** Make TraitClaw production-ready with enriched output, feature flags, observability, and error resilience.

### Story 7.1: AgentOutput Enrichment

As a developer,
I want richer agent output with usage stats and multiple variants,
So that I can build production applications with proper monitoring.

**Acceptance Criteria:**

**Given** the `AgentOutput` enum
**When** I call `agent.run(input)`
**Then** `AgentOutput` has variants: `Text(String)`, `Structured(Value)`, `Error(String)`
**And** `output.text()` returns `Option<&str>` (not panic on non-text variants)
**And** `output.usage()` returns `RunUsage { tokens, iterations, duration }`

### Story 7.2: Meta-crate Feature Flags

As a developer,
I want `traitclaw` to feature-gate all optional crate re-exports,
So that I control exactly what gets compiled.

**Acceptance Criteria:**

**Given** `traitclaw/Cargo.toml` with feature flags
**When** I add `traitclaw = { features = ["openai", "steering"] }`
**Then** feature `openai-compat` (default) → re-exports `traitclaw-openai-compat`
**And** feature `openai` → re-exports `traitclaw-openai`
**And** feature `anthropic` → re-exports `traitclaw-anthropic`
**And** feature `steering` → re-exports `traitclaw-steering`
**And** feature `sqlite` → re-exports `traitclaw-memory-sqlite`
**And** feature `macros` (default) → re-exports `traitclaw-macros`
**And** default features = `["openai-compat", "macros"]`

### Story 7.3: Observability (tracing spans)

As a developer,
I want structured tracing spans for agent operations,
So that I can integrate with OpenTelemetry and debug production issues.

**Acceptance Criteria:**

**Given** the agent runtime uses `tracing` crate
**When** I configure a tracing subscriber
**Then** spans are emitted for: `agent.run`, each iteration, each tool call, each LLM request
**And** span attributes include: model, session_id, iteration, tokens_used
**And** optional `tracing-opentelemetry` integration is documented

### Story 7.4: Error Recovery & Graceful Degradation

As a developer,
I want the agent to handle failures gracefully without crashing,
So that my application stays resilient.

**Acceptance Criteria:**

**Given** the agent runtime loop
**When** a tool execution fails
**Then** error message is returned to LLM as tool result (not agent crash)
**And** when a Guard panics → `catch_unwind` treats it as Allow + logs warning
**And** when Memory operations fail → log error + continue (don't crash agent)
**And** when provider times out → return friendly error message
