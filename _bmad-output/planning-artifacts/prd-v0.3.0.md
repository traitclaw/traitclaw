---
stepsCompleted: ["step-01-init.md", "step-02-discovery.md", "step-02b-vision.md", "step-02c-executive-summary.md", "step-03-success.md", "step-04-journeys.md", "step-05-features.md", "step-06-nfr.md", "step-07-ux.md", "step-08-tech.md", "step-09-risks.md", "step-10-compliance.md", "step-11-review.md", "step-12-complete.md"]
inputDocuments: ["brainstorming/v030-context-window-rescue.md", "project-context.md", "architecture.md", "planning-artifacts/prd.md", "planning-artifacts/epics-v0.2.0.md"]
workflowType: 'prd'
classification:
  projectType: developer_tool
  domain: AI / Developer Tool
  complexity: medium
  projectContext: brownfield
---

# Product Requirements Document — TraitClaw v0.3.0 "Context Window Rescue"

**Author:** Bangvu
**Date:** 2026-03-25
**Version:** 0.3.0
**Depends on:** v0.2.0 (The Openness Update)

---

## Executive Summary

TraitClaw v0.3.0 ("Context Window Rescue") addresses the #1 production pain point for AI agents: **context window overflow**. When agents run beyond 5–10 iterations, conversation history, tool outputs, and tool schemas collectively consume the context window, degrading response quality or triggering hard failures. This release introduces three new pluggable traits — `ContextManager`, `OutputTransformer`, and `ToolRegistry` — that give developers precise, extensible control over what enters the LLM's context window.

### What Makes This Special

v0.3.0 follows the same evolution pattern established in v0.2.0: **new async traits with blanket implementations for backward compatibility**. Every `ContextStrategy` implementation from v0.1.0 automatically becomes a `ContextManager`. Every `OutputProcessor` automatically becomes an `OutputTransformer`. The existing `Vec<Arc<dyn ErasedTool>>` is transparently wrapped in a `SimpleRegistry`. **Zero breaking changes** — all v0.2.0 code compiles and runs unchanged on v0.3.0. Old traits are deprecated with clear migration messages and will only be removed in v1.0.0.

The core architectural insight is that context overflow has three independent sources — history, tool output, and tool schemas — each requiring a different optimization strategy. By providing separate, composable traits for each source, developers can mix and match: e.g., LLM-powered history compression + budget-aware output truncation + grouped tool schemas.

## Project Classification

- **Project Type:** Developer Tool (Rust Framework/Library)
- **Domain:** AI / Developer Tooling
- **Complexity:** Medium
- **Project Context:** Brownfield (building on TraitClaw v0.2.0)

---

## Success Criteria

### User Success

- **Context survival:** An agent with `TieredCompressor` runs 50+ iterations on a 128K context window model without overflow.
- **Zero-disruption upgrade:** Existing v0.2.0 code compiles and runs on v0.3.0 without modification. All deprecated traits have blanket impls bridging to new traits.
- **Progressive adoption:** A developer can adopt one new trait at a time — ContextManager first, OutputTransformer second, ToolRegistry third — without needing all three simultaneously.

### Business Success

- **Ecosystem adoption:** 50% of TraitClaw users adopt at least one v0.3.0 trait within 3 months.
- **Community engagement:** At least 3 community-contributed implementations (custom compressors, transformers, or registries) within 6 months.

### Technical Success

- **Zero regression:** All existing tests pass. No performance degradation on the default path.
- **Minimal API surface:** Exactly 3 new traits added to `traitclaw-core`.
- **No new mandatory dependencies:** `tiktoken-rs` is feature-gated. All LLM-powered implementations accept any `Provider`.

### Measurable Outcomes

| Metric | Target | Measurement |
|--------|--------|-------------|
| Context utilization | 40% reduction in wasted tokens | Benchmark: same task with/without v0.3.0 traits |
| Max iterations (128K model) | 50+ iterations without overflow | Integration test |
| Compile time (default features) | < 3% increase | `cargo build --timings` |
| Binary size (default features) | < 2% increase | `cargo bloat` |
| API surface | 3 new traits | Manual count |

---

## Product Scope

### MVP — v0.3.0

1. **`ContextManager` trait** — Async pluggable context management replacing sync `ContextStrategy`.
2. **Built-in context managers** — `SlidingWindowManager`, `RuleBasedCompressor`, `LlmCompressor`, `TieredCompressor`.
3. **Token-accurate counting** — `TikTokenCounter` (feature-gated) and `CharApproxCounter` (default).
4. **`OutputTransformer` trait** — Async context-aware tool output processing replacing sync `OutputProcessor`.
5. **Built-in output transformers** — `TruncateTransformer`, `BudgetAwareTransformer`, `JsonFieldExtractor`, `ChainTransformer`.
6. **`ToolRegistry` trait** — Dynamic tool management with activation/deactivation replacing flat `Vec`.
7. **Built-in tool registries** — `SimpleRegistry` (default, zero overhead), `DynamicRegistry`, `GroupedRegistry`, `AdaptiveRegistry`.
8. **Integration** — All three traits wired into `DefaultStrategy` runtime loop.
9. **Migration guide & examples** — `docs/migration-v0.2-to-v0.3.md`, 3 new examples.

### Growth Features (Post-MVP — v0.4.0+)

- `McpToolRegistry` — Discover and register tools from MCP servers on demand.
- `EmbeddingCompressor` — Use embeddings for semantic importance scoring.
- `StreamingOutputTransformer` — Process tool output chunks as they arrive.
- Built-in `MctsStrategy` and `ReActStrategy` implementations.

### Vision (Future — v1.0)

- Remove deprecated `ContextStrategy` and `OutputProcessor` traits.
- Visual context budget dashboard for debugging.
- Automatic context strategy selection based on model capabilities.

---

## User Journeys

### Journey 1: "My agent forgets important context"

**Persona:** Developer running long-lived chat agents that lose critical information.

```
Problem → Agent drops early messages after 10+ iterations
→ Discovers ContextManager trait in docs
→ Plugs in: `.context_manager(LlmCompressor::new(openai("gpt-4o-mini")))`
→ Old messages are LLM-summarized into a concise system message
→ Agent retains key information across 50+ iterations
→ Cost: ~$0.001 per compression call
```

### Journey 2: "Tool outputs waste my context window"

**Persona:** Developer whose agent calls APIs returning large JSON blobs.

```
Problem → Search tool returns 50KB JSON, eating 12K tokens per call
→ Discovers OutputTransformer trait
→ Plugs in: `.output_transformer(BudgetAwareTransformer::default())`
→ When budget is plentiful, full output is kept
→ When budget drops below 40%, output is aggressively summarized
→ Agent runs twice as many iterations before hitting limits
```

### Journey 3: "Too many tools clog my context"

**Persona:** Developer building a power-user agent with 30+ tools.

```
Problem → 30 tool schemas = 5K tokens sent every LLM call
→ Discovers ToolRegistry trait
→ Uses GroupedRegistry: `.tool_registry(GroupedRegistry::new()
    .group("search", [web_search, deep_search])
    .group("code", [read_file, write_file, run_cmd])
    .activate("search"))`
→ Only active group schemas sent to LLM
→ Saves ~3K tokens per call
```

### Journey 4: "I have a custom compression strategy"

**Persona:** AI researcher with a proprietary context compression algorithm.

```
Need → Custom compression using their fine-tuned summarization model
→ Implements `ContextManager` trait (3 methods)
→ `.context_manager(MyCustomCompressor::new(my_model))`
→ TraitClaw calls their compressor at the right point in the loop
→ Zero framework modification required
```

---

## Feature Requirements

### F1: `ContextManager` Trait

**Priority:** P0 (Must-have)
**Crate:** `traitclaw-core`

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

| Requirement | Detail |
|-------------|--------|
| Async | `prepare()` is async to support LLM-powered compression |
| Backward compat | Blanket impl: `impl<T: ContextStrategy> ContextManager for T` |
| Deprecation | `ContextStrategy` deprecated in v0.3.0, removed in v1.0.0 |
| Default | `SlidingWindowManager` (wraps existing `SlidingWindowStrategy` via blanket impl) |
| Builder | `Agent::builder().context_manager(impl ContextManager)` |
| Token counting | `estimate_tokens()` has default impl, overridable per implementation |

### F2: Built-in Context Managers

**Priority:** P0 (Must-have)
**Crate:** `traitclaw-core`

| Implementation | Description |
|---------------|-------------|
| `SlidingWindowManager` | Existing behavior via blanket impl (backward compat) |
| `RuleBasedCompressor` | Importance scoring: system=1.0, recent=0.9, tool_results=0.7, old=0.3 |
| `LlmCompressor` | Uses a `Provider` to summarize old messages. Configurable prompt. |
| `TieredCompressor` | Chains: keep recent N → rule-compress mid → LLM-summarize old |

### F3: Token Counting

**Priority:** P1 (Should-have)
**Crate:** `traitclaw-core` (feature-gated)

| Implementation | Description |
|---------------|-------------|
| `CharApproxCounter` | Default: 4 chars ≈ 1 token. Zero dependencies. |
| `TikTokenCounter` | Uses `tiktoken-rs`. Behind `"tiktoken"` feature flag. |

### F4: `OutputTransformer` Trait

**Priority:** P0 (Must-have)
**Crate:** `traitclaw-core`

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

| Requirement | Detail |
|-------------|--------|
| Async | `transform()` is async to support LLM-powered extraction |
| Context-aware | Receives `AgentState` for token budget decisions |
| Tool-aware | Receives `tool_name` for tool-specific processing |
| Backward compat | Blanket impl: `impl<T: OutputProcessor> OutputTransformer for T` |
| Deprecation | `OutputProcessor` deprecated in v0.3.0, removed in v1.0.0 |
| Builder | `Agent::builder().output_transformer(impl OutputTransformer)` |

### F5: Built-in Output Transformers

**Priority:** P0 (Must-have)
**Crate:** `traitclaw-core`

| Implementation | Description |
|---------------|-------------|
| `TruncateTransformer` | Wraps existing `TruncateProcessor` (backward compat default) |
| `BudgetAwareTransformer` | Adapts truncation: >80% budget=full, 40-80%=5K, <40%=1K |
| `JsonFieldExtractor` | Extracts specified JSON paths from tool output |
| `ChainTransformer` | Composes multiple transformers in sequence |
| `ProgressiveTransformer` | Returns summary first; provides full on LLM request |

### F6: `ToolRegistry` Trait

**Priority:** P1 (Should-have)
**Crate:** `traitclaw-core`

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

| Requirement | Detail |
|-------------|--------|
| Sync | Read operations are sync (hot path, every LLM call) |
| Interior mutability | Write operations use `&self` + `RwLock` in mutable registries |
| Default | `SimpleRegistry` — all tools active, immutable, zero overhead |
| Builder | `Agent::builder().tool_registry(impl ToolRegistry)` |
| Backward compat | `AgentBuilder::tool(T)` still works (adds to internal `SimpleRegistry`) |

### F7: Built-in Tool Registries

**Priority:** P1 (Should-have)
**Crate:** `traitclaw-core`

| Implementation | Description |
|---------------|-------------|
| `SimpleRegistry` | Default. All tools always active. Immutable. Zero overhead. |
| `DynamicRegistry` | Runtime `register()`/`deactivate()` via `RwLock`. |
| `GroupedRegistry` | Named groups with group-level activation. |
| `AdaptiveRegistry` | Auto-limits based on `ModelTier` (Small=5, Medium=15, Large=all). |

### F8: Integration & Runtime

**Priority:** P0 (Must-have)
**Crate:** `traitclaw-core`

- Wire all three traits into `DefaultStrategy` runtime loop.
- Wire into streaming path.
- `AgentRuntime` stores `Arc<dyn ContextManager>`, `Arc<dyn OutputTransformer>`, `Arc<dyn ToolRegistry>`.

### F9: Migration Guide & Examples

**Priority:** P0 (Must-have)

| Deliverable | Description |
|------------|-------------|
| `docs/migration-v0.2-to-v0.3.md` | No breaking changes. Incremental adoption guide. |
| `examples/15-context-manager/` | LlmCompressor, TieredCompressor demo. |
| `examples/16-output-transformer/` | BudgetAwareTransformer, JsonFieldExtractor demo. |
| `examples/17-tool-registry/` | GroupedRegistry, DynamicRegistry demo. |

---

## Non-Functional Requirements

### Performance

- Zero overhead on default path: `SimpleRegistry` + `SlidingWindowManager` (via blanket impl) + `TruncateTransformer` (via blanket impl) = identical to v0.2.0.
- `LlmCompressor` adds one LLM call per compression event (not per iteration).
- `RwLock` in `DynamicRegistry`: read lock = atomic increment (nanoseconds). Write lock only during `register()`/`deactivate()`.

### Compatibility

- **Backward compatibility:** v0.2.0 code compiles without changes on v0.3.0.
- **MSRV:** Rust 1.75+
- **Semver:** Minor version bump (0.2.0 → 0.3.0). No breaking changes.
- **Deprecation policy:** Old traits deprecated in v0.3.0, removed in v1.0.0 only.

### Documentation

- All public traits and methods must have rustdoc with examples.
- Migration guide from v0.2.0 to v0.3.0.
- Architecture Decision Records for all 3 new traits.

### Testing

- Unit tests for all default implementations.
- Integration tests demonstrating custom ContextManager/OutputTransformer/ToolRegistry.
- Backward compatibility test: v0.2.0 code compiles unmodified.
- Long conversation stress test: 50+ iterations without overflow.
- Token counting accuracy test: tiktoken vs char-approx comparison.

---

## Technical Constraints

### Architecture Decisions

| ADR | Decision | Rationale |
|-----|----------|-----------|
| ADR-5 | Async `ContextManager` with blanket impl for sync `ContextStrategy` | LLM-powered compression needs async. Blanket impl = zero breaking changes. |
| ADR-6 | Async `OutputTransformer` with blanket impl for sync `OutputProcessor` | Same pattern as ADR-5. Context-aware processing needs `AgentState`. |
| ADR-7 | `ToolRegistry` with `&self` + interior mutability for writes | Runtime mutation via `register()`/`deactivate()` requires `&self` (Arc-wrapped). `SimpleRegistry` default is immutable = zero overhead. |
| ADR-8 | Deprecation in v0.3.0, removal in v1.0.0 | Breaking changes only in major versions. Blanket impls have zero runtime cost. |

### Dependency Policy

- **No new required dependencies.**
- `tiktoken-rs` is behind feature flag `"tiktoken"` (optional dep).
- All LLM-powered implementations (LlmCompressor, ProgressiveTransformer) accept any `Provider` — no provider lock-in.

### Crate Impact Map

| Crate | Changes |
|-------|---------|
| `traitclaw-core` | Add `ContextManager`, `OutputTransformer`, `ToolRegistry` traits + all built-in implementations. Deprecate `ContextStrategy`, `OutputProcessor`. |
| `traitclaw` (meta) | Re-export new traits. Add `"tiktoken"` feature flag. |
| New examples | `15-context-manager/`, `16-output-transformer/`, `17-tool-registry/` |

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| LlmCompressor quality varies by model | Medium | Medium | Configurable summary prompt. Recommend `gpt-4o-mini` in docs. Allow users to implement custom compression. |
| Async ContextManager adds complexity | Low | Medium | Blanket impl means sync users see no change. Async is only needed for LLM-powered features. |
| DynamicRegistry RwLock contention | Very Low | Low | Writes are extremely rare (1-2x per session). Reads use shared lock (< 1ns overhead). |
| Token counting accuracy | Medium | Low | TikTokenCounter is accurate. CharApprox is clearly documented as approximate. Default is sufficient for budget decisions. |
| Feature creep across 3 pillars | Medium | High | Ship ContextManager + OutputTransformer as P0. ToolRegistry as P1 — can slip to v0.3.1 if needed. |

---

## Release Plan

### Phase 1: Core Traits (Week 1-2)

- [ ] Define `ContextManager` trait + blanket impl for `ContextStrategy`
- [ ] Implement token counting (`CharApproxCounter`, `TikTokenCounter`)
- [ ] Define `OutputTransformer` trait + blanket impl for `OutputProcessor`
- [ ] Deprecate `ContextStrategy` and `OutputProcessor` with clear messages

### Phase 2: Built-in Implementations (Week 3-4)

- [ ] Implement `RuleBasedCompressor`, `LlmCompressor`, `TieredCompressor`
- [ ] Implement `BudgetAwareTransformer`, `JsonFieldExtractor`, `ChainTransformer`
- [ ] Define `ToolRegistry` trait
- [ ] Implement `SimpleRegistry`, `DynamicRegistry`, `GroupedRegistry`, `AdaptiveRegistry`

### Phase 3: Integration (Week 5-6)

- [ ] Wire all 3 traits into `DefaultStrategy` runtime loop
- [ ] Wire into streaming path
- [ ] Update `AgentRuntime`, `Agent`, and `AgentBuilder`
- [ ] Integration tests + backward compat tests

### Phase 4: Examples & Docs (Week 7-8)

- [ ] Create `examples/15-context-manager/`
- [ ] Create `examples/16-output-transformer/`
- [ ] Create `examples/17-tool-registry/`
- [ ] Write migration guide, ADRs, update README
- [ ] Version bump + publish
