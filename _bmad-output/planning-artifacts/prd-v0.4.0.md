---
stepsCompleted: ["step-01-init.md", "step-02-discovery.md", "step-02b-vision.md", "step-02c-executive-summary.md", "step-03-success.md", "step-04-journeys.md", "step-05-features.md", "step-06-nfr.md", "step-07-ux.md", "step-08-tech.md", "step-09-risks.md", "step-10-compliance.md", "step-11-review.md", "step-12-complete.md"]
inputDocuments: ["planning-artifacts/prd-v0.3.0.md", "project-context.md", "planning-artifacts/epics-v0.3.0.md", "brainstorming/brainstorming-session-2026-03-23-210618.md"]
workflowType: 'prd'
classification:
  projectType: developer_tool
  domain: AI / Developer Tool
  complexity: medium
  projectContext: brownfield
---

# Product Requirements Document — TraitClaw v0.4.0 "Power Tools"

**Author:** Bangvu
**Date:** 2026-03-25
**Version:** 0.4.0
**Depends on:** v0.3.0 (Context Window Rescue)

---

## Executive Summary

TraitClaw v0.4.0 ("Power Tools") completes the extensibility ecosystem started in v0.3.0 and adds intelligent runtime capabilities. While v0.3.0 introduced the foundational traits (`ContextManager`, `OutputTransformer`, `ToolRegistry`), it shipped with minimal built-in implementations. v0.4.0 delivers the **power implementations** that make those traits production-ready: grouped and adaptive tool management, progressive output processing, accurate token counting, and MCP-based tool discovery.

### What Makes This Special

v0.4.0 is a "fill the gaps" release — no new traits, no breaking changes, just powerful implementations of existing extension points. This means:

1. **Zero new API surface** — All implementations plug into traits from v0.3.0. Users who learned `.context_manager()`, `.output_transformer()`, `.tool_registry()` can adopt v0.4.0 features with zero new concepts.
2. **MCP Integration** — `McpToolRegistry` brings Model Context Protocol tool discovery into the framework, positioning TraitClaw as the first Rust agent framework with native MCP tool management.
3. **Production-grade token counting** — `TikTokenCounter` replaces the 4-chars≈1-token approximation with exact OpenAI-compatible tokenization, feature-gated behind `tiktoken` to keep default builds lean.

### Release Philosophy

> Ship powerful defaults, not more abstractions.

v0.3.0 gave users the *hooks*. v0.4.0 gives users the *implementations* so they don't have to build them.

## Project Classification

- **Project Type:** Developer Tool (Rust Framework/Library)
- **Domain:** AI / Developer Tooling
- **Complexity:** Medium
- **Project Context:** Brownfield (building on TraitClaw v0.3.0)

---

## Success Criteria

### User Success

- **Tool management at scale:** A developer with 30+ tools can organize them into named groups and only send relevant schemas to the LLM, saving ~3K tokens per call.
- **Model-aware tooling:** `AdaptiveRegistry` automatically limits tool count based on `ModelTier` (Small=5, Medium=15, Large=all), preventing context overflow without manual configuration.
- **Accurate budgeting:** Token-counting-dependent features (context managers, output transformers) produce accurate results when `tiktoken` feature is enabled.
- **MCP ecosystem access:** A developer can discover and register tools from any MCP-compatible server with a single line of configuration.

### Technical Success

- **Zero regression:** All existing tests pass. No performance degradation on the default path.
- **Zero new traits:** All features implement existing `ToolRegistry`, `OutputTransformer`, or standalone utility types.
- **Minimal dependency impact:** `tiktoken-rs` and MCP-related deps are feature-gated. Default builds unchanged.

### Measurable Outcomes

| Metric | Target | Measurement |
|--------|--------|-------------|
| Tool schema tokens (30 tools, grouped) | 60% reduction vs flat | Benchmark: GroupedRegistry vs SimpleRegistry |
| Token counting accuracy | < 2% error vs tiktoken | Unit test: CharApprox vs TikToken on 1000 samples |
| MCP tool discovery latency | < 500ms for 10 tools | Integration test with mock MCP server |
| Compile time (default features) | < 2% increase | `cargo build --timings` |
| Binary size (default features) | < 1% increase | `cargo bloat` (feature-gated deps add nothing to default) |

---

## Product Scope

### MVP — v0.4.0

1. **`GroupedRegistry`** — Named tool groups with group-level activation/deactivation.
2. **`AdaptiveRegistry`** — Auto-limits active tools based on `ModelTier`.
3. **`ProgressiveTransformer`** — Returns summary first; includes full output only if LLM requests it.
4. **`TikTokenCounter`** — Accurate token counting via `tiktoken-rs` (feature-gated).
5. **`McpToolRegistry`** — Discover and register tools from MCP servers on demand.

### Deferred (v0.5.0+)

- `EmbeddingCompressor` — Semantic importance scoring for context compression.
- `StreamingOutputTransformer` — Process tool output chunks as they arrive.
- Built-in `MctsStrategy` and `ReActStrategy` implementations.

### Vision (Future — v1.0)

- Remove deprecated `ContextStrategy` and `OutputProcessor` traits.
- Visual context budget dashboard for debugging.
- Automatic context strategy selection based on model capabilities.

---

## User Journeys

### Journey 1: "Too many tools clog my context"

**Persona:** Developer building a power-user agent with 30+ tools.

```
Problem → 30 tool schemas = 5K tokens sent every LLM call
→ Discovers GroupedRegistry
→ .tool_registry(GroupedRegistry::new()
    .group("search", [web_search, deep_search])
    .group("code", [read_file, write_file, run_cmd])
    .activate("search"))
→ Only active group schemas sent to LLM
→ Saves ~3K tokens per call
→ Can switch groups mid-conversation via tool calls
```

### Journey 2: "I deploy to different model tiers"

**Persona:** Developer deploying same agent across GPT-4o (large) and GPT-4o-mini (small).

```
Problem → Small models choke on 30 tool schemas, large models handle them fine
→ Discovers AdaptiveRegistry
→ .tool_registry(AdaptiveRegistry::new(tools))
→ Small tier: auto-selects 5 most relevant, Medium: 15, Large: all
→ Same code, different models, optimal tool counts automatically
```

### Journey 3: "Tool outputs are huge but I only need a summary"

**Persona:** Developer whose search tool returns 50KB JSON.

```
Problem → Full JSON eats 12K tokens, but LLM only needs 3 key fields
→ Discovers ProgressiveTransformer
→ .output_transformer(ProgressiveTransformer::new(provider, 500))
→ First: LLM receives 500-char summary
→ If LLM needs more: auto-retrieves full output via __get_full_output tool
→ 90% of the time, summary is enough → massive token savings
```

### Journey 4: "I want to use MCP-compatible tools"

**Persona:** Developer using VSCode/Cursor with MCP tool servers.

```
Need → Connect to MCP servers and use their tools as agent tools
→ Discovers McpToolRegistry
→ .tool_registry(McpToolRegistry::connect("http://localhost:3000"))
→ Tools auto-discovered from MCP server
→ Schemas auto-registered in agent
→ Zero manual tool definition required
```

---

## Feature Requirements

### F1: `GroupedRegistry`

**Priority:** P0 (Must-have)
**Crate:** `traitclaw-core`

```rust
let registry = GroupedRegistry::new()
    .group("search", vec![web_search, deep_search])
    .group("code", vec![read_file, write_file])
    .group("data", vec![query_db, insert_db])
    .activate("search");
```

| Requirement | Detail |
|-------------|--------|
| Implements | `ToolRegistry` trait from v0.3.0 |
| Group management | `.group(name, tools)`, `.activate(name)`, `.deactivate(name)` |
| Multi-group | Multiple groups can be active simultaneously |
| Default | First group activated on creation, or none if `.activate()` not called |
| Interior mutability | `RwLock`-based for runtime group switching |

### F2: `AdaptiveRegistry`

**Priority:** P0 (Must-have)
**Crate:** `traitclaw-core`

```rust
let registry = AdaptiveRegistry::new(all_tools)
    .with_limits(5, 15, usize::MAX);  // small, medium, large
```

| Requirement | Detail |
|-------------|--------|
| Implements | `ToolRegistry` trait from v0.3.0 |
| Auto-selection | Uses `ModelInfo::tier()` to determine limit |
| Default limits | Small=5, Medium=15, Large=unlimited |
| Configurable | `.with_limits(small, medium, large)` |
| Priority ordering | Tools registered first have higher priority |
| Tier access | Reads `ModelTier` from `AgentConfig` or provider's `model_info()` |

### F3: `ProgressiveTransformer`

**Priority:** P1 (Should-have)
**Crate:** `traitclaw-core`

```rust
let transformer = ProgressiveTransformer::new(provider.clone(), 500)
    .with_summary_prompt("Summarize this tool output concisely");
```

| Requirement | Detail |
|-------------|--------|
| Implements | `OutputTransformer` trait from v0.3.0 |
| Two-phase | Phase 1: LLM-generated summary (configurable max length) |
| | Phase 2: Full output available via virtual `__get_full_output` tool |
| Provider | Uses any `Provider` for summarization (configurable) |
| Fallback | If LLM summarization fails, falls back to truncation |
| Prompt | Configurable via `.with_summary_prompt()` |

### F4: `TikTokenCounter`

**Priority:** P1 (Should-have)
**Crate:** `traitclaw-core` (feature-gated: `tiktoken`)

```rust
// In Cargo.toml: traitclaw = { features = ["tiktoken"] }
let counter = TikTokenCounter::for_model("gpt-4o");
let tokens = counter.count_tokens(&messages);
```

| Requirement | Detail |
|-------------|--------|
| Feature gate | Behind `tiktoken` feature flag |
| Dependency | `tiktoken-rs` (only pulled when feature enabled) |
| Model support | Maps model names to tiktoken encodings |
| Integration | Can be injected into `ContextManager` via `estimate_tokens()` override |
| Fallback | Unknown models fall back to `cl100k_base` encoding |

### F5: `McpToolRegistry`

**Priority:** P1 (Should-have)
**Crate:** `traitclaw-mcp`

```rust
let registry = McpToolRegistry::connect("http://localhost:3000").await?;
// or
let registry = McpToolRegistry::from_config(mcp_config).await?;
```

| Requirement | Detail |
|-------------|--------|
| Implements | `ToolRegistry` trait from v0.3.0 |
| Discovery | Connects to MCP server, lists available tools |
| Schema mapping | Maps MCP tool schemas to TraitClaw `ToolSchema` |
| Execution | Routes tool calls through MCP protocol |
| Reconnection | Auto-reconnects on connection loss |
| Multiple servers | Support connecting to multiple MCP servers |
| Crate | Lives in `traitclaw-mcp` (already exists for MCP client) |

---

## Non-Functional Requirements

### Performance

- Zero overhead on default path: `SimpleRegistry` remains the default. New registries only used when explicitly configured.
- `GroupedRegistry` / `AdaptiveRegistry`: `RwLock` read lock = atomic increment (nanoseconds). Write lock only during `activate()`/`deactivate()`.
- `TikTokenCounter`: Tokenization is ~10x slower than char-approx but only runs once per `prepare()` call (not per message).
- `McpToolRegistry`: Network latency for MCP calls is dominated by the MCP server, not the registry.

### Compatibility

- **Backward compatibility:** v0.3.0 code compiles without changes on v0.4.0.
- **MSRV:** Rust 1.75+
- **Semver:** Minor version bump (0.3.0 → 0.4.0). No breaking changes.
- **Feature gates:** `tiktoken` and `mcp-registry` are optional features.

### Documentation

- All public types and methods must have rustdoc with examples.
- Migration guide from v0.3.0 to v0.4.0 (short — no breaking changes, just new features).
- Update existing examples to demonstrate new registries.

### Testing

- Unit tests for all implementations.
- Integration tests for `McpToolRegistry` with mock MCP server.
- Token counting accuracy test: `TikTokenCounter` vs `CharApproxCounter` comparison.
- `GroupedRegistry` group switching under concurrent access.
- `AdaptiveRegistry` with all three `ModelTier` values.

---

## Technical Constraints

### Architecture Decisions

| ADR | Decision | Rationale |
|-----|----------|-----------|
| ADR-9 | `GroupedRegistry` uses `RwLock<HashMap<String, Vec<Arc<dyn ErasedTool>>>>` | Same interior mutability pattern as `DynamicRegistry` from v0.3.0. Consistent API. |
| ADR-10 | `TikTokenCounter` behind feature flag | `tiktoken-rs` adds ~2MB to binary and compile time. Default char-approx is sufficient for 95% of use cases. |
| ADR-11 | `McpToolRegistry` in `traitclaw-mcp` crate | MCP protocol dependency already exists there. Keeps `traitclaw-core` dependency-free. |
| ADR-12 | `ProgressiveTransformer` accepts `Arc<dyn Provider>` | Same pattern as `LlmCompressor` from v0.3.0. Provider-agnostic summarization. |

### Dependency Policy

- **No new required dependencies.**
- `tiktoken-rs` behind `"tiktoken"` feature flag.
- MCP transport deps already in `traitclaw-mcp`.
- `ProgressiveTransformer` uses existing `Provider` trait — no new deps.

### Crate Impact Map

| Crate | Changes |
|-------|---------|
| `traitclaw-core` | Add `GroupedRegistry`, `AdaptiveRegistry`, `ProgressiveTransformer`, `TikTokenCounter` |
| `traitclaw-mcp` | Add `McpToolRegistry` |
| `traitclaw` (meta) | Re-export new types. Add `"tiktoken"` feature flag. |
| New examples | `examples/19-grouped-registry/`, `examples/20-mcp-tool-registry/` |

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| `tiktoken-rs` version compatibility | Low | Medium | Pin version, test against multiple Rust versions |
| MCP protocol changes | Medium | High | Abstract MCP transport behind internal trait. Use `traitclaw-mcp` existing patterns. |
| `ProgressiveTransformer` summary quality | Medium | Medium | Configurable prompt. Fallback to truncation. Users can swap Provider. |
| `AdaptiveRegistry` wrong tool selection | Medium | Medium | Priority-based ordering (first registered = highest priority). Users can override limits. |
| Feature creep | Low | Medium | Strict scope: 5 implementations only. No new traits. |

---

## Release Plan

### Phase 1: Core Implementations (Week 1-2)

- [ ] `GroupedRegistry` with group CRUD + activation
- [ ] `AdaptiveRegistry` with ModelTier-based limits
- [ ] Unit tests for both registries

### Phase 2: Output & Counting (Week 3-4)

- [ ] `ProgressiveTransformer` with 2-phase output
- [ ] `TikTokenCounter` with feature gate + model mapping
- [ ] Unit tests + token accuracy benchmarks

### Phase 3: MCP Integration (Week 5-6)

- [ ] `McpToolRegistry` in `traitclaw-mcp`
- [ ] MCP schema mapping + tool execution routing
- [ ] Integration tests with mock MCP server

### Phase 4: Polish & Release (Week 7-8)

- [ ] Examples: `19-grouped-registry/`, `20-mcp-tool-registry/`
- [ ] Migration guide `docs/migration-v0.3-to-v0.4.md`
- [ ] Update README, version bump, publish
