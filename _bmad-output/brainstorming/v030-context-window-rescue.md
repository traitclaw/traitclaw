---
stepsCompleted: [1, 2, 3]
inputDocuments:
  - architecture.md
  - project-context.md
  - epics.md
  - brainstorming/brainstorming-session-2026-03-23-210618.md
session_topic: 'v0.3.0 Context Window Rescue — ContextManager, ToolRegistry, OutputTransformer'
session_goals: 'Define epic scope, stories, and acceptance criteria for v0.3.0'
selected_approach: 'ai-recommended'
techniques_used: ['first_principles', 'codebase_analysis', 'pattern_extraction', 'tradeoff_analysis']
ideas_generated: [30+]
context_file: ''
---

# v0.3.0 Epic — Context Window Rescue

> **Codename:** Context Window Rescue  
> **Theme:** Giải cứu Context Window bằng ContextManager, ToolRegistry và OutputTransformer  
> **Version:** v0.3.0  
> **Depends on:** v0.2.0 (Openness Update)

---

## Problem Statement

In production agent loops, context window fills up from 3 sources:

| Source | Current solution | Problem |
|--------|-----------------|---------|
| Conversation history | `SlidingWindowStrategy` drops oldest messages | Lossy — no summarization, loses important context |
| Tool outputs | `TruncateProcessor` cuts at 10K chars | Dumb — no semantic extraction, wastes tokens on irrelevant data |
| Tool schemas | All schemas sent every LLM call | Wasteful — 20+ tools = 2K+ tokens of schemas, most unused |

**Result:** Agents hit context limits after 5-10 iterations even with GPT-4o (128K).

---

## Architecture Decision: Unified Extensibility Pattern

All 3 pillars follow the same evolution pattern established in v0.2.0:

```
New async Trait → blanket impl for old sync Trait → inject via Builder
```

| New Trait (v0.3.0) | Replaces (deprecated) | Blanket impl |
|---------------------|----------------------|-------------|
| `ContextManager` (async) | `ContextStrategy` (sync) | `impl<T: ContextStrategy> ContextManager for T` |
| `OutputTransformer` (async) | `OutputProcessor` (sync) | `impl<T: OutputProcessor> OutputTransformer for T` |
| `ToolRegistry` (sync, interior mut) | `Vec<Arc<dyn ErasedTool>>` | `SimpleRegistry` wraps existing Vec |

**Zero breaking changes** — all v0.2.0 code compiles unchanged.

**Deprecation timeline:**
- v0.3.0: New traits introduced, old traits deprecated with `#[deprecated]`
- v1.0.0: Old traits removed (breaking changes belong in major versions)

---

## Epic 8: Context Window Rescue

### Story 8.1: ContextManager Trait

As a developer,  
I want a pluggable async context management trait,  
So that I can control how conversation history is compressed before each LLM call.

**Acceptance Criteria:**

**Given** the `ContextManager` trait is defined in `traitclaw-core/src/traits/context_manager.rs`  
**When** I implement it for a custom type  
**Then** `prepare()` is async and accepts `&mut Vec<Message>`, `context_window: usize`, `&mut AgentState`  
**And** `estimate_tokens()` returns token count for a message list  
**And** a blanket impl converts any `ContextStrategy` into `ContextManager` automatically  
**And** `ContextStrategy` is marked `#[deprecated(since = "0.3.0", note = "Use ContextManager. Will be removed in v1.0.0")]`  
**And** `AgentBuilder::context_manager(impl ContextManager)` accepts the new trait  
**And** `AgentBuilder::context_strategy()` still works (wraps via blanket impl)  
**And** `DefaultStrategy` calls `context_manager.prepare().await` instead of sync `context_strategy.prepare()`  
**And** the trait requires `Send + Sync`

```rust
#[async_trait]
pub trait ContextManager: Send + Sync {
    async fn prepare(
        &self,
        messages: &mut Vec<Message>,
        context_window: usize,
        state: &mut AgentState,
    ) -> Result<()>;

    fn estimate_tokens(&self, messages: &[Message]) -> usize {
        messages.iter().map(|m| m.content.len() / 4 + 1).sum()
    }
}
```

---

### Story 8.2: Built-in Context Managers

As a developer,  
I want pre-built context managers for common compression strategies,  
So that I get intelligent context management without writing custom logic.

**Acceptance Criteria:**

**Given** the `ContextManager` trait exists  
**When** I use built-in managers  
**Then** `SlidingWindowManager` is available (= current SlidingWindowStrategy, adapted via blanket impl)  
**And** `RuleBasedCompressor` extracts key information using importance scoring:
  - System messages = weight 1.0 (never remove)
  - Recent messages (last N) = weight 0.9
  - Messages with tool results = weight 0.7
  - Older conversation messages = weight 0.3
  - Compress lowest-weight messages first  
**And** `LlmCompressor` uses a `Provider` to summarize old messages into a concise system message  
**And** `LlmCompressor::new(provider)` accepts any Provider for the summarization call  
**And** `LlmCompressor` has configurable `summary_prompt` template  
**And** `TieredCompressor` chains: recent (keep as-is) → mid (rule-based compress) → old (LLM summarize)  
**And** all managers properly set `AgentState::last_output_truncated` when messages are removed/compressed

```rust
// Usage examples:
Agent::builder()
    .context_manager(SlidingWindowManager::new(0.85))    // simple
    .build();

Agent::builder()
    .context_manager(LlmCompressor::new(openai("gpt-4o-mini")))  // LLM-powered
    .build();

Agent::builder()
    .context_manager(TieredCompressor::new()
        .keep_recent(10)
        .rule_compress_after(20)
        .llm_summarize_after(50, openai("gpt-4o-mini")))
    .build();
```

---

### Story 8.3: Token-Accurate Counting

As a developer,  
I want accurate token counting instead of the "4 chars ≈ 1 token" approximation,  
So that context management decisions are precise.

**Acceptance Criteria:**

**Given** the `ContextManager` trait has `estimate_tokens()` with a default implementation  
**When** I want accurate counting  
**Then** `TikTokenCounter` uses `tiktoken-rs` for OpenAI-compatible token counting  
**And** `CharApproxCounter` uses the current 4-char approximation (default, zero deps)  
**And** `ContextManager::estimate_tokens()` has a default impl using `CharApproxCounter`  
**And** `AgentState` tracks `estimated_tokens_used` updated after each `prepare()` call  
**And** token counter is configurable per-provider via `ModelInfo::token_counter`  
**And** `tiktoken-rs` is behind feature flag `"tiktoken"` (optional dep)

---

### Story 8.4: OutputTransformer Trait

As a developer,  
I want a context-aware async tool output transformer,  
So that tool outputs are intelligently processed based on remaining token budget.

**Acceptance Criteria:**

**Given** the `OutputTransformer` trait is defined in `traitclaw-core/src/traits/output_transformer.rs`  
**When** I implement it for a custom type  
**Then** `transform()` is async and receives `tool_name`, `output: String`, and `&AgentState`  
**And** a blanket impl converts any `OutputProcessor` into `OutputTransformer` automatically  
**And** `OutputProcessor` is marked `#[deprecated(since = "0.3.0", note = "Use OutputTransformer. Will be removed in v1.0.0")]`  
**And** `AgentBuilder::output_transformer(impl OutputTransformer)` accepts the new trait  
**And** `AgentBuilder::output_processor()` still works (wraps via blanket impl)  
**And** `DefaultStrategy` calls `output_transformer.transform().await` after each tool execution  
**And** the trait requires `Send + Sync`

```rust
#[async_trait]
pub trait OutputTransformer: Send + Sync {
    async fn transform(
        &self,
        tool_name: &str,
        output: String,
        state: &AgentState,
    ) -> Result<String>;
}
```

---

### Story 8.5: Built-in Output Transformers

As a developer,  
I want pre-built output transformers for common processing strategies,  
So that tool outputs don't waste context window tokens.

**Acceptance Criteria:**

**Given** the `OutputTransformer` trait exists  
**When** I use built-in transformers  
**Then** `TruncateTransformer` wraps existing `TruncateProcessor` behavior (default)  
**And** `BudgetAwareTransformer` adapts truncation based on remaining token budget:
  - Budget > 80% → keep full output
  - Budget 40-80% → truncate to 5K chars
  - Budget < 40% → aggressive truncate to 1K chars  
**And** `JsonFieldExtractor::new(fields)` extracts only specified JSON paths from tool output  
**And** `LlmSummarizer::new(provider)` uses LLM to extract relevant information from tool output  
**And** `ChainTransformer::new(vec![...])` composes multiple transformers in order  
**And** `ProgressiveTransformer` returns summary first; if LLM asks "show full output" → provides complete output on next call

```rust
Agent::builder()
    .output_transformer(BudgetAwareTransformer::default())
    .build();

Agent::builder()
    .output_transformer(ChainTransformer::new(vec![
        Box::new(JsonFieldExtractor::new(["name", "status", "error"])),
        Box::new(BudgetAwareTransformer::default()),
    ]))
    .build();
```

---

### Story 8.6: ToolRegistry Trait

As a developer,  
I want a pluggable tool registry with dynamic activation/deactivation,  
So that only relevant tool schemas are sent to the LLM to save context tokens.

**Acceptance Criteria:**

**Given** the `ToolRegistry` trait is defined in `traitclaw-core/src/traits/tool_registry.rs`  
**When** I implement it for a custom type  
**Then** `active_schemas()` returns schemas only for currently active tools  
**And** `find(name)` finds a tool by name (active or inactive)  
**And** `register(tool)` adds a tool at runtime → returns `Result<()>`  
**And** `deactivate(name)` / `activate(name)` toggle tool availability  
**And** `all_tools()` returns all registered tools regardless of active status  
**And** `SimpleRegistry` is the default (all tools always active, immutable, zero overhead)  
**And** `DynamicRegistry` supports runtime mutation via `RwLock` (opt-in)  
**And** `AgentBuilder::tool_registry(impl ToolRegistry)` accepts the new trait  
**And** `AgentBuilder::tool(T)` still works → adds to internal `SimpleRegistry`  
**And** `DefaultStrategy` calls `registry.active_schemas()` instead of `tools.iter().map(schema)`  
**And** the trait requires `Send + Sync`

```rust
pub trait ToolRegistry: Send + Sync {
    fn active_schemas(&self) -> Vec<ToolSchema>;
    fn active_tools(&self) -> Vec<Arc<dyn ErasedTool>>;
    fn find(&self, name: &str) -> Option<Arc<dyn ErasedTool>>;
    fn all_tools(&self) -> Vec<Arc<dyn ErasedTool>>;
    fn register(&self, tool: Arc<dyn ErasedTool>) -> Result<()>;
    fn deactivate(&self, name: &str) -> Result<()>;
    fn activate(&self, name: &str) -> Result<()>;
}
```

---

### Story 8.7: Built-in Tool Registries

As a developer,  
I want pre-built tool registries for common tool management patterns,  
So that I can optimize tool schema overhead with minimal code.

**Acceptance Criteria:**

**Given** the `ToolRegistry` trait exists  
**When** I use built-in registries  
**Then** `SimpleRegistry` wraps a `Vec<Arc<dyn ErasedTool>>` (all active, immutable, default)  
**And** `DynamicRegistry` supports runtime `register()`/`deactivate()` via `RwLock`  
**And** `GroupedRegistry` organizes tools into named groups with group-level activation  
**And** `AdaptiveRegistry` auto-limits active tools based on `ModelTier`:
  - Small → max 5 tools
  - Medium → max 15 tools
  - Large → unlimited  
**And** `SimpleRegistry::register()` returns `Err("immutable registry")` (clear error, not panic)

```rust
Agent::builder()
    .tool_registry(GroupedRegistry::new()
        .group("search", [WebSearch, DeepSearch])
        .group("code", [ReadFile, WriteFile, RunCmd])
        .activate("search"))
    .build();

Agent::builder()
    .tool_registry(AdaptiveRegistry::for_tier(ModelTier::Small, all_tools))
    .build();
```

---

### Story 8.8: Integration in DefaultStrategy

As a developer,  
I want all three new traits wired into the agent runtime loop,  
So that context management, output transformation, and tool selection work seamlessly.

**Acceptance Criteria:**

**Given** the `DefaultStrategy` runtime loop  
**When** I run an agent with the new traits configured  
**Then** the loop sequence is:
  1. Load context (memory + system prompt + user message)
  2. `tool_registry.active_schemas()` → only send active tool schemas
  3. inject hints
  4. **`context_manager.prepare().await`** → async context management
  5. Build `CompletionRequest` with pruned messages + active schemas
  6. LLM call
  7. If tool calls → execute tools → **`output_transformer.transform().await`** → inject result
  8. Loop back to step 2  
**And** `AgentRuntime` struct stores `Arc<dyn ContextManager>`, `Arc<dyn OutputTransformer>`, `Arc<dyn ToolRegistry>`  
**And** streaming path (`DefaultStrategy::stream()`) also uses the new traits  
**And** hooks fire at appropriate points (before/after compression, before/after transform)

---

### Story 8.9: Migration Guide & Examples

As a developer,  
I want a migration guide and examples for the new traits,  
So that I can upgrade from v0.2.0 without confusion.

**Acceptance Criteria:**

**Given** the v0.3.0 release  
**When** I read `docs/migration-v0.2-to-v0.3.md`  
**Then** it explains: no breaking changes, all v0.2.0 code works unchanged  
**And** shows how to adopt each new trait incrementally  
**And** `examples/15-context-manager/` demonstrates `LlmCompressor` and `TieredCompressor`  
**And** `examples/16-output-transformer/` demonstrates `BudgetAwareTransformer` and `JsonFieldExtractor`  
**And** `examples/17-tool-registry/` demonstrates `GroupedRegistry` and `DynamicRegistry`  
**And** deprecation warnings include clear migration hints

---

## FR Coverage Map (v0.3.0 additions)

| FR | Story | Description |
|----|-------|-------------|
| FR16 (Context window management) | 8.1, 8.2, 8.3 | Async ContextManager with LLM compression |
| FR18 (Tool output processing) | 8.4, 8.5 | Context-aware OutputTransformer |
| FR2 (Tool calling) | 8.6, 8.7 | Dynamic ToolRegistry |
| NFR1 (Zero-cost abstractions) | All | Blanket impls, SimpleRegistry = zero overhead |

---

## Implementation Order

```
Story 8.1 (ContextManager trait)
  ↓
Story 8.3 (Token counting)
  ↓
Story 8.2 (Built-in context managers)
  ↓
Story 8.4 (OutputTransformer trait)
  ↓
Story 8.5 (Built-in transformers)
  ↓
Story 8.6 (ToolRegistry trait)
  ↓
Story 8.7 (Built-in registries)
  ↓
Story 8.8 (Integration in DefaultStrategy)
  ↓
Story 8.9 (Migration guide & examples)
```

---

## Verification Plan

### Automated Tests
- Unit tests for each trait implementation
- Integration tests: agent with LlmCompressor + BudgetAwareTransformer + GroupedRegistry
- Backward-compat tests: v0.2.0 code compiles and runs unchanged
- Token counting accuracy tests (tiktoken vs char-approx)
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all --all-features`

### Manual Verification
- Run all 17 examples end-to-end
- Benchmark: context window usage with vs without new traits
- Long conversation test (~50 iterations) to verify compression prevents overflow

---

*Brainstorming session: 2026-03-25 | Participant: Bangvu | Facilitator: AI*
*Session concluded with 9 stories across 3 pillars.*
