---
stepsCompleted: ["step-01-init.md", "step-02-discovery.md", "step-02b-vision.md", "step-02c-executive-summary.md", "step-03-success.md", "step-04-journeys.md", "step-05-features.md", "step-06-nfr.md", "step-08-tech.md", "step-09-risks.md", "step-12-complete.md"]
inputDocuments: ["planning-artifacts/prd-v0.4.0.md", "project-context.md", "planning-artifacts/epics-v0.4.0.md"]
workflowType: 'prd'
classification:
  projectType: developer_tool
  domain: AI / Developer Tool
  complexity: high
  projectContext: brownfield
---

# Product Requirements Document — TraitClaw v0.5.0 "Ecosystem"

**Author:** Bangvu
**Date:** 2026-03-25
**Version:** 0.5.0
**Depends on:** v0.4.0 (Power Tools)

---

## Executive Summary

TraitClaw v0.5.0 ("Ecosystem") transforms the framework from a core agent runtime into a **full-stack AI agent platform**. While v0.3.0 introduced async traits and v0.4.0 added powerful built-in implementations, the surrounding crates (`traitclaw-rag`, `traitclaw-team`, `traitclaw-eval`) remain scaffolds with types but no execution logic. v0.5.0 delivers the missing execution engines that make these crates production-ready, plus modernizes the provider crates.

### What Makes This Special

1. **RAG Pipeline** — From keyword-only search to embedding-based vector retrieval, document chunking, and agent-integrated context injection.
2. **Multi-Agent Execution** — `Team.run(input)` actually executes agent orchestration. `VerificationChain` runs generate-verify loops. Shared context flows between agents.
3. **Eval Runner** — `EvalRunner.run(agent, suite)` executes evaluation suites. LLM-as-Judge metric enables AI-powered quality assessment.
4. **Provider Modernization** — Extended thinking for Claude, new model constructors (DeepSeek, xAI), retry/backoff for rate limiting.

### Release Philosophy

> Connect the dots. Make every crate executable, not just type-safe.

v0.5.0 is about turning scaffolds into running systems. Every crate should have a `run()` or `execute()` entry point.

## Project Classification

- **Project Type:** Developer Tool (Rust Framework/Library)
- **Domain:** AI / Developer Tooling
- **Complexity:** High
- **Project Context:** Brownfield (building on TraitClaw v0.4.0)

---

## Success Criteria

### User Success

- **RAG in 5 lines:** Developer can add retrieval-augmented generation to any agent with chunking, embedding retrieval, and grounding in ≤ 5 lines of setup.
- **Multi-agent execution:** `team.run("Build me an app").await` orchestrates multiple agents through a router and returns a final result.
- **Eval feedback loop:** Developer runs `eval_runner.run(&agent, &suite).await` and gets a structured report with pass/fail, scores, and LLM judge assessments.
- **Provider reliability:** Rate-limited API calls auto-retry with exponential backoff instead of failing.

### Measurable Outcomes

| Metric | Target | Measurement |
|--------|--------|-------------|
| RAG retrieval accuracy (embedding vs keyword) | ≥ 30% improvement | Benchmark on standard IR dataset |
| Team.run() completion rate | ≥ 95% | Integration test with mock agents |
| Eval suite execution time (10 cases) | < 30s | Benchmark with mock provider |
| Provider retry success rate (429 errors) | ≥ 90% recovery | Integration test with rate-limited mock |

---

## Product Scope

### MVP — v0.5.0

#### Pillar 1: RAG Pipeline (`traitclaw-rag`)
1. **`EmbeddingRetriever`** — Vector similarity search using any embedding provider
2. **Document Chunking** — Fixed-size, sentence-based, recursive text splitting
3. **`HybridRetriever`** — Combined keyword + embedding scoring
4. **New Grounding Strategies** — `CitationStrategy`, `ContextWindowStrategy`
5. **`RagContextManager`** — Agent integration via `ContextManager` trait

#### Pillar 2: Multi-Agent Execution (`traitclaw-team`)
6. **`Team.run()` execution engine** — Actually execute agent orchestration
7. **Agent binding** — Map `AgentRole` → `Agent` instances
8. **`VerificationChain.run()`** — Execute generate-verify-retry loops
9. **`ConditionalRouter`** — Route by content matching / regex
10. **Shared team context** — Agents within a team share state

#### Pillar 3: Eval Runner (`traitclaw-eval`)
11. **`EvalRunner`** — `run(agent, suite)` execution engine
12. **Async `Metric` trait** — Enable LLM-based evaluation
13. **`LlmJudgeMetric`** — AI-powered output quality scoring
14. **`SchemaValidationMetric`** — Verify structured output correctness
15. **`ToolUsageMetric`** — Validate correct tool invocation patterns
16. **Report export** — JSON and CSV output

#### Pillar 4: Provider Modernization
17. **Anthropic: Extended thinking** — `thinking` parameter support for Claude
18. **Anthropic: PDF support** — Document content type handling
19. **Anthropic: Model info update** — Claude 3.5+ model mappings
20. **OpenAI: New constructors** — `deepseek()`, `xai()` convenience functions
21. **OpenAI-compat: Retry/backoff** — Configurable retry policy for 429/5xx errors
22. **Steering: New guards** — `RateLimitGuard`, `ContentFilterGuard`

### Deferred (v0.6.0+)

- Graph-based routing (DAG execution engine)
- Embedding model hosting (local ONNX inference)
- `StreamingOutputTransformer`
- Built-in `MctsStrategy` and `ReActStrategy`
- Visual context budget dashboard

---

## User Journeys

### Journey 1: "I want my agent to use documents as context"

**Persona:** Developer building a customer support agent with a knowledge base.

```
Problem → Agent hallucinates answers without documentation context
→ Creates chunked documents from help center
→ let retriever = EmbeddingRetriever::new(embedding_provider)
→ retriever.add_documents(chunks)
→ agent.context_manager(RagContextManager::new(retriever))
→ User asks question → relevant docs injected into context automatically
→ Agent answers accurately with citations
```

### Journey 2: "I want a team of agents to handle complex tasks"

**Persona:** Developer building a content production pipeline.

```
Problem → Single agent can't research, write, AND verify quality
→ Creates Team with researcher + writer + reviewer roles
→ team.run("Write a technical blog about Rust async").await
→ SequentialRouter: researcher → writer → reviewer
→ VerificationChain: reviewer rejects → writer revises → reviewer accepts
→ Returns polished blog post after multi-agent collaboration
```

### Journey 3: "I want to measure my agent's quality over time"

**Persona:** Developer maintaining a production agent with quality SLAs.

```
Problem → No automated way to detect quality regressions
→ Creates EvalSuite with 50 test cases
→ Adds KeywordMetric + LlmJudgeMetric
→ eval_runner.run(&agent, &suite).await
→ Gets EvalReport: 47/50 passed, avg score 0.92
→ Exports JSON for CI dashboard tracking
→ Detects regression before production deploy
```

---

## Feature Requirements

### F1: EmbeddingRetriever

**Priority:** P0 | **Crate:** `traitclaw-rag`

```rust
let retriever = EmbeddingRetriever::new(embedding_provider)
    .with_similarity_threshold(0.7);
retriever.add_documents(chunked_docs).await?;
let results = retriever.retrieve("How to reset password?", 5).await?;
```

| Requirement | Detail |
|-------------|--------|
| Embedding provider | Accepts any `EmbeddingProvider` trait (new trait) |
| Storage | In-memory vector store (default). Trait-based for external DB. |
| Similarity | Cosine similarity scoring |
| Threshold | Configurable minimum similarity score |

### F2: Document Chunking

**Priority:** P0 | **Crate:** `traitclaw-rag`

```rust
let chunks = FixedSizeChunker::new(500, 50).chunk(&document);
let chunks = SentenceChunker::new(3).chunk(&document);  // 3 sentences per chunk
let chunks = RecursiveChunker::new(1000).chunk(&document);
```

| Requirement | Detail |
|-------------|--------|
| Fixed-size | Split by character count with configurable overlap |
| Sentence | Split by sentence boundaries (`.`, `!`, `?`) |
| Recursive | Split hierarchically (paragraph → sentence → word) |

### F3: Team Execution Engine

**Priority:** P0 | **Crate:** `traitclaw-team`

```rust
let team = Team::new("content_team")
    .bind("researcher", researcher_agent)
    .bind("writer", writer_agent)
    .with_router(SequentialRouter::new());

let result = team.run("Write about Rust async").await?;
```

| Requirement | Detail |
|-------------|--------|
| `team.run()` | Async execution through router pipeline |
| Agent binding | `.bind(role_name, Agent)` connects roles to agents |
| Shared context | `TeamContext` struct passed between agents |
| Max iterations | Configurable safety limit (default: 10) |

### F4: EvalRunner

**Priority:** P0 | **Crate:** `traitclaw-eval`

```rust
let runner = EvalRunner::new()
    .metric(KeywordMetric)
    .metric(LlmJudgeMetric::new(judge_provider))
    .threshold(0.8);

let report = runner.run(&agent, &suite).await?;
println!("{}", report.summary());
report.export_json("results.json")?;
```

| Requirement | Detail |
|-------------|--------|
| Async execution | Runs agent on each test case |
| Multi-metric | Applies all registered metrics to each result |
| Threshold | Configurable pass/fail threshold (default: 0.8) |
| Report | `EvalReport` with summary, per-case scores, export |

### F5: LlmJudgeMetric

**Priority:** P1 | **Crate:** `traitclaw-eval`

```rust
let judge = LlmJudgeMetric::new(judge_provider)
    .with_criteria("accuracy", "Is the response factually accurate?")
    .with_criteria("helpfulness", "Does it address the user's need?");
```

| Requirement | Detail |
|-------------|--------|
| Uses any Provider | Same `Arc<dyn Provider>` pattern as LlmCompressor |
| Multi-criteria | Multiple named evaluation criteria |
| Structured output | Judge returns JSON scores per criteria |
| Async | Must be async (LLM call per evaluation) |

### F6: Provider Retry/Backoff

**Priority:** P1 | **Crate:** `traitclaw-openai-compat`

```rust
let provider = OpenAiCompatProvider::openai("gpt-4o", api_key)
    .with_retry(RetryPolicy::exponential(3, Duration::from_millis(500)));
```

| Requirement | Detail |
|-------------|--------|
| Retry on | 429 (rate limit), 500, 502, 503, 504 |
| Backoff | Exponential with jitter |
| Max retries | Configurable (default: 3) |
| Applies to | Both `complete()` and `stream()` |

### F7: Anthropic Extended Thinking

**Priority:** P1 | **Crate:** `traitclaw-anthropic`

```rust
let provider = AnthropicProvider::new("claude-3-5-sonnet-20241022", api_key)
    .with_extended_thinking(true);
```

| Requirement | Detail |
|-------------|--------|
| Wire format | Add `thinking` parameter to Anthropic request |
| Response parsing | Handle `thinking` content blocks in response |
| Configurable | `.with_extended_thinking(bool)` builder method |

---

## Non-Functional Requirements

### Performance

- RAG: Embedding retrieval O(n) for in-memory store. Acceptable for ≤10K documents.
- Team: Router decisions are O(1). Agent execution is LLM-bound.
- Eval: Parallel test case execution via `tokio::JoinSet` where provider allows.
- Retry: Exponential backoff adds max ~15s total delay for 3 retries.

### Compatibility

- **Backward compatibility:** v0.4.0 code compiles unchanged on v0.5.0.
- **MSRV:** Rust 1.75+
- **Semver:** Minor version bump (0.4.0 → 0.5.0). No breaking changes.
- **New traits:** `EmbeddingProvider` (in `traitclaw-rag`), async `Metric` (in `traitclaw-eval`). Both additive.

### Testing

- Integration tests for full RAG pipeline (chunk → embed → retrieve → ground)
- Integration tests for Team execution with mock agents
- Eval runner tests with mock provider and mock metrics
- Retry tests with simulated 429 responses
- Extended thinking wire format tests

---

## Technical Constraints

### Architecture Decisions

| ADR | Decision | Rationale |
|-----|----------|-----------|
| ADR-13 | `EmbeddingProvider` trait in `traitclaw-rag` | Keeps embedding concerns separate from core. Not all agents need embeddings. |
| ADR-14 | In-memory vector store as default | External vector DB integration via trait. In-memory sufficient for ≤10K docs. |
| ADR-15 | `Team.run()` takes owned agents | Agents consumed by team to prevent concurrent mutation. `Arc<Agent>` for sharing. |
| ADR-16 | Async `Metric` trait replaces sync | Breaking change within `traitclaw-eval` (acceptable — eval crate is pre-1.0 scaffold). |
| ADR-17 | `RetryPolicy` in `traitclaw-openai-compat` | Provider-level retry. Not in core (some providers handle retry server-side). |

### Dependency Policy

- **No new required dependencies in `traitclaw-core`.**
- `traitclaw-rag`: No new external deps for in-memory store. External vector DB drivers behind feature flags.
- `traitclaw-eval`: No new deps (uses existing `Provider` trait for LLM judge).
- `traitclaw-openai-compat`: No new deps (retry logic is pure Rust + tokio).

### Crate Impact Map

| Crate | Changes |
|-------|---------|
| `traitclaw-rag` | `EmbeddingRetriever`, `HybridRetriever`, chunkers, `RagContextManager`, `EmbeddingProvider` trait |
| `traitclaw-team` | `Team.run()`, agent binding, `VerificationChain.run()`, `ConditionalRouter`, `TeamContext` |
| `traitclaw-eval` | `EvalRunner`, async `Metric`, `LlmJudgeMetric`, `SchemaValidationMetric`, `ToolUsageMetric`, report export |
| `traitclaw-anthropic` | Extended thinking, PDF support, model info updates |
| `traitclaw-openai` | `deepseek()`, `xai()` constructors |
| `traitclaw-openai-compat` | `RetryPolicy`, exponential backoff |
| `traitclaw-steering` | `RateLimitGuard`, `ContentFilterGuard` |

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Embedding provider API diversity | High | Medium | `EmbeddingProvider` trait abstracts over OpenAI/Cohere/local. Start with OpenAI format. |
| Team deadlocks (agent A waits for agent B) | Medium | High | Max iteration limit (default: 10). Timeout per agent call. |
| Eval LLM judge consistency | High | Medium | Fixed judge prompt template. Temperature=0. Multiple runs for statistical significance. |
| Scope creep (6 crates × multiple features) | High | High | Phase-gated release. Ship RAG + Team first, then Eval + Providers. |
| Breaking change in async Metric trait | Low | Low | `traitclaw-eval` has near-zero external users. Pre-1.0 allows breaking changes. |

---

## Release Plan

### Phase 1: RAG Pipeline (Week 1-3)

- [ ] `EmbeddingProvider` trait + in-memory vector store
- [ ] Document chunkers (fixed, sentence, recursive)
- [ ] `EmbeddingRetriever` + `HybridRetriever`
- [ ] `RagContextManager` agent integration
- [ ] Grounding strategies: `CitationStrategy`, `ContextWindowStrategy`

### Phase 2: Multi-Agent Execution (Week 4-6)

- [ ] Agent binding: `team.bind(role, agent)`
- [ ] `Team.run()` execution engine
- [ ] `VerificationChain.run()` with generate-verify-retry
- [ ] `ConditionalRouter` with regex matching
- [ ] Shared `TeamContext`

### Phase 3: Eval Runner (Week 7-9)

- [ ] Async `Metric` trait (breaking change for eval crate)
- [ ] `EvalRunner.run(agent, suite)` execution
- [ ] `LlmJudgeMetric`, `SchemaValidationMetric`, `ToolUsageMetric`
- [ ] Report export (JSON, CSV)

### Phase 4: Provider Modernization (Week 10-11)

- [ ] `RetryPolicy` + exponential backoff in openai-compat
- [ ] Anthropic: extended thinking, PDF, model updates
- [ ] OpenAI: `deepseek()`, `xai()` constructors
- [ ] Steering: `RateLimitGuard`, `ContentFilterGuard`

### Phase 5: Polish & Release (Week 12)

- [ ] Examples for RAG, Team, Eval
- [ ] Migration guide `docs/migration-v0.4-to-v0.5.md`
- [ ] Update README, version bump, publish
