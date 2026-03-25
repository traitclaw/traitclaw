---
stepsCompleted: ["step-01-validate-prerequisites", "step-02-design-epics", "step-03-create-stories", "step-04-final-validation"]
inputDocuments:
  - planning-artifacts/prd-v0.5.0.md
  - planning-artifacts/prd-v0.4.0.md
  - project-context.md
---

# TraitClaw v0.5.0 — Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for TraitClaw v0.5.0 "Ecosystem", decomposing 22 features across 7 crates into implementable stories. v0.5.0 transforms scaffold crates (rag, team, eval) into executable systems and modernizes provider crates.

## Requirements Inventory

### Functional Requirements

FR1: Implement `EmbeddingRetriever` with `EmbeddingProvider` trait, in-memory vector store, cosine similarity search.

FR2: Implement document chunkers: `FixedSizeChunker`, `SentenceChunker`, `RecursiveChunker`.

FR3: Implement `HybridRetriever` combining keyword + embedding scoring with configurable weights.

FR4: Implement `RagContextManager` integrating retrieval pipeline with the `ContextManager` trait.

FR5: Implement grounding strategies: `CitationStrategy`, `ContextWindowStrategy`.

FR6: Implement `Team.run()` execution engine with agent binding and router-driven orchestration.

FR7: Implement `VerificationChain.run()` with generate-verify-retry loop.

FR8: Implement `ConditionalRouter` with regex/content-based routing.

FR9: Implement shared `TeamContext` for inter-agent state.

FR10: Implement `EvalRunner.run(agent, suite)` execution with async `Metric` trait.

FR11: Implement `LlmJudgeMetric`, `SchemaValidationMetric`, `ToolUsageMetric`.

FR12: Implement report export (JSON, CSV).

FR13: Implement `RetryPolicy` with exponential backoff in `traitclaw-openai-compat`.

FR14: Implement Anthropic extended thinking and PDF support.

FR15: Add `deepseek()`, `xai()` constructors to `traitclaw-openai`.

FR16: Add `RateLimitGuard`, `ContentFilterGuard` to `traitclaw-steering`.

### FR Coverage Map

| FR | Epic | Description |
|----|------|-------------|
| FR1-FR2 | Epic 17: RAG Foundation | Embedding retrieval + chunking |
| FR3-FR5 | Epic 18: RAG Integration | Hybrid retrieval + agent integration |
| FR6-FR9 | Epic 19: Multi-Agent Execution | Team.run(), VerificationChain, routing |
| FR10-FR12 | Epic 20: Eval Runner | Execution engine + metrics + export |
| FR13-FR15 | Epic 21: Provider Modernization | Retry, extended thinking, new constructors |
| FR16 | Epic 22: Steering Enhancements | New guards |
| — | Epic 23: Documentation & Examples | Guides, examples, polish |

## Epic List

- **Epic 17: RAG Foundation** — EmbeddingProvider trait, chunkers, EmbeddingRetriever (Phase 1)
- **Epic 18: RAG Integration** — HybridRetriever, grounding strategies, RagContextManager (Phase 1)
- **Epic 19: Multi-Agent Execution** — Team.run(), VerificationChain, ConditionalRouter, TeamContext (Phase 2)
- **Epic 20: Eval Runner** — Async Metric, EvalRunner, LlmJudge, report export (Phase 3)
- **Epic 21: Provider Modernization** — RetryPolicy, extended thinking, new constructors (Phase 4)
- **Epic 22: Steering Enhancements** — RateLimitGuard, ContentFilterGuard (Phase 4)
- **Epic 23: Documentation & Examples** — Migration guide, RAG/Team/Eval examples (Phase 5)

> **Note:** Epic numbering continues from v0.4.0 (Epics 13-16).

---

## Epic 17: RAG Foundation

**Goal:** Developers can chunk documents and retrieve them using embedding-based vector search.

**FRs:** FR1, FR2 | **Crate:** `traitclaw-rag`

### Story 17.1: EmbeddingProvider Trait & Document Chunking

As a developer building a RAG pipeline,
I want to split large documents into chunks suitable for embedding,
So that retrieval operates on focused, granular text segments.

**Acceptance Criteria:**

1. `EmbeddingProvider` async trait is defined: `async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f64>>>`
2. `FixedSizeChunker::new(chunk_size, overlap)` splits text by character count with overlap
3. `SentenceChunker::new(sentences_per_chunk)` splits by sentence boundaries (`.`, `!`, `?`)
4. `RecursiveChunker::new(max_chunk_size)` splits hierarchically (paragraph → sentence → character)
5. All chunkers implement `Chunker` trait: `fn chunk(&self, text: &str) -> Vec<String>`
6. Unit test: 1000-char text → FixedSizeChunker(200, 50) produces ≥5 chunks with 50-char overlap
7. Unit test: 10-sentence text → SentenceChunker(3) produces 4 chunks
8. Unit test: empty input → all chunkers return empty Vec

### Story 17.2: EmbeddingRetriever Implementation

As a developer with a document knowledge base,
I want to search documents using semantic similarity,
So that I retrieve contextually relevant results beyond keyword matching.

**Acceptance Criteria:**

1. `EmbeddingRetriever::new(embedding_provider)` creates a vector retriever
2. `.with_similarity_threshold(0.7)` sets minimum cosine similarity filter
3. `.add_documents(docs).await` embeds and stores documents
4. `.retrieve(query, limit).await` embeds query, performs cosine similarity search, returns top-k
5. Implements the existing `Retriever` trait from `traitclaw-rag`
6. In-memory vector store (Vec of (embedding, Document) pairs)
7. Unit test: 10 docs → query returns ≤ limit results sorted by similarity desc
8. Unit test: threshold 0.9 → fewer results than threshold 0.5
9. Unit test: `add_documents` calls `embed()` exactly once with all texts

---

## Epic 18: RAG Integration

**Goal:** Developers can combine retrieval methods and inject retrieved context into agent conversations automatically.

**FRs:** FR3, FR4, FR5 | **Crate:** `traitclaw-rag`

### Story 18.1: HybridRetriever & Grounding Strategies

As a developer who needs both keyword and semantic search,
I want to combine them with configurable weighting,
So that I get the best of both retrieval approaches.

**Acceptance Criteria:**

1. `HybridRetriever::new(keyword_retriever, embedding_retriever)` creates a combined retriever
2. `.with_weights(keyword_weight, embedding_weight)` configures scoring ratio (default: 0.3 / 0.7)
3. Merges results from both retrievers, normalizes scores, re-ranks
4. Implements the existing `Retriever` trait
5. `CitationStrategy` grounding: formats docs as `[1] content (Source: doc_id)`
6. `ContextWindowStrategy::new(max_tokens)` limits total injected context to a token budget
7. Unit test: hybrid retriever returns results from both keyword and embedding sources
8. Unit test: ContextWindowStrategy truncates when context exceeds budget

### Story 18.2: RagContextManager

As a developer who wants automatic document injection,
I want retrieved documents injected into my agent's context,
So that RAG works transparently without manual plumbing.

**Acceptance Criteria:**

1. `RagContextManager::new(retriever)` creates a context manager backed by any `Retriever`
2. `.with_grounding(strategy)` configures how retrieved docs are formatted (default: PrependStrategy)
3. `.with_max_docs(n)` limits number of injected documents
4. Implements `ContextManager` trait from `traitclaw-core`
5. On `prepare()`: extracts last user message → retrieves docs → prepends grounded context as system message
6. Unit test: mock retriever returns 3 docs → context has grounding prefix
7. Unit test: no relevant docs → context unchanged

---

## Epic 19: Multi-Agent Execution

**Goal:** Developers can orchestrate multiple agents through routers and verification chains with actual code execution, not just types.

**FRs:** FR6, FR7, FR8, FR9 | **Crate:** `traitclaw-team`

### Story 19.1: Agent Binding & Team Execution Engine

As a developer with a team of specialized agents,
I want to execute orchestrated multi-agent workflows,
So that complex tasks are broken down and handled by the right agent.

**Acceptance Criteria:**

1. `team.bind("role_name", agent)` connects an `AgentRole` to an actual `Agent` instance
2. `team.run("input text").await` executes the orchestration pipeline
3. Router is called iteratively: route → send to agent → get response → route again
4. `RoutingDecision::Complete(output)` terminates the loop and returns the result
5. `RoutingDecision::SendTo(id)` invokes the bound agent's `complete()` method
6. `.with_max_iterations(n)` prevents infinite loops (default: 10)
7. Integration test: 2-agent sequential team → researcher gets input, writer gets researcher output, returns final
8. Integration test: max_iterations exceeded → returns error

### Story 19.2: VerificationChain Execution & ConditionalRouter

As a developer who needs output quality validation,
I want generate-verify-retry loops and content-based routing,
So that my agent team produces verified, high-quality outputs.

**Acceptance Criteria:**

1. `VerificationChain.run(generator, verifier, input).await` executes the loop
2. Generator produces output → Verifier returns `VerifyResult::Accepted` or `Rejected(feedback)`
3. On rejection: generator retries with feedback appended (up to `max_retries`)
4. On acceptance: returns the accepted output
5. `ConditionalRouter::new()` with `.when(pattern, target)` routes by regex
6. `.default(target)` sets fallback routing target
7. Unit test: verifier accepts on 2nd try → result contains retry feedback
8. Unit test: all retries exhausted → returns error with last output
9. Unit test: ConditionalRouter matches "search" → routes to "researcher"

### Story 19.3: Shared TeamContext

As a developer whose agents need shared state,
I want agents in a team to read/write shared context,
So that information flows between agents without manual message passing.

**Acceptance Criteria:**

1. `TeamContext` struct with `get(key)` and `set(key, value)` methods
2. Values are `serde_json::Value` for flexibility
3. `Team.run()` passes `&TeamContext` to each agent invocation
4. Agents can read context set by previous agents in the pipeline
5. `RwLock`-based for concurrent access safety
6. Unit test: agent A sets key → agent B reads same key → value matches

---

## Epic 20: Eval Runner

**Goal:** Developers can execute evaluation suites against agents, score outputs with LLM judges, and export structured reports.

**FRs:** FR10, FR11, FR12 | **Crate:** `traitclaw-eval`

### Story 20.1: Async Metric Trait & EvalRunner

As a developer testing agent quality,
I want to run evaluation suites and get structured reports,
So that I can measure quality regressions before production deploy.

**Acceptance Criteria:**

1. `Metric` trait updated to async: `async fn score(&self, ...) -> f64`
2. `KeywordMetric` and `LengthRelevancyMetric` updated to async (trivial — no actual await)
3. `EvalRunner::new()` with `.metric(m)` and `.threshold(t)` builder
4. `runner.run(&agent, &suite).await` executes agent on each test case
5. Each test case: send `input` to agent, collect response, score with all metrics
6. Returns `EvalReport` with per-case scores and pass/fail
7. Integration test: 3 test cases with KeywordMetric → report with 3 results
8. Unit test: threshold 0.8 → case scoring 0.7 marked as failed

### Story 20.2: LlmJudgeMetric & Specialized Metrics

As a developer who needs AI-powered quality assessment,
I want LLM-based and structured validation metrics,
So that evaluation goes beyond keyword matching.

**Acceptance Criteria:**

1. `LlmJudgeMetric::new(provider)` creates an async LLM-based scorer
2. `.with_criteria("name", "prompt")` adds named evaluation criteria
3. Judge calls provider.complete() with evaluation prompt → parses score 0.0-1.0
4. `SchemaValidationMetric::new(schema)` validates that agent output matches a JSON schema
5. `ToolUsageMetric::new(expected_tools)` checks if agent called expected tools
6. Unit test: mock provider judge returns 0.85 → metric score = 0.85
7. Unit test: schema validation passes for valid JSON, fails for invalid

### Story 20.3: Report Export

As a developer integrating evals into CI/CD,
I want to export evaluation reports as JSON and CSV,
So that I can track quality metrics in dashboards.

**Acceptance Criteria:**

1. `report.export_json(path)?` writes structured JSON report
2. `report.export_csv(path)?` writes tabular CSV with columns: case_id, metric, score, passed
3. `report.summary()` returns formatted text summary (existing, enhanced)
4. Unit test: export_json → valid JSON parseable back to `EvalReport`
5. Unit test: export_csv → valid CSV with header + one row per case×metric

---

## Epic 21: Provider Modernization

**Goal:** Provider crates support retry/backoff, modern model features (extended thinking), and new provider shortcuts.

**FRs:** FR13, FR14, FR15 | **Crates:** `traitclaw-openai-compat`, `traitclaw-anthropic`, `traitclaw-openai`

### Story 21.1: RetryPolicy & Exponential Backoff

As a developer hitting API rate limits,
I want auto-retry with backoff,
So that transient failures don't crash my agent.

**Acceptance Criteria:**

1. `RetryPolicy` struct: `max_retries`, `initial_delay`, `max_delay`, `jitter`
2. `RetryPolicy::exponential(max_retries, initial_delay)` constructor
3. `.with_retry(policy)` builder on `OpenAiCompatProvider`
4. Retries on HTTP 429, 500, 502, 503, 504
5. Exponential backoff: delay × 2^attempt with optional jitter
6. Applies to both `complete()` and `stream()` methods
7. Unit test: mock 429 → retry → 200 → succeeds
8. Unit test: max_retries exceeded → returns last error

### Story 21.2: Anthropic Extended Thinking & Model Updates

As a developer using Claude's advanced features,
I want extended thinking and updated model mappings,
So that I can leverage Claude's full capabilities.

**Acceptance Criteria:**

1. `.with_extended_thinking(true)` builder on `AnthropicProvider`
2. Wire format: adds `thinking: { type: "enabled", budget_tokens: N }` to request
3. Response parsing handles `thinking` content blocks (stored in metadata, not in main content)
4. `infer_model_info` updated for Claude 3.5 Sonnet v2, Claude 4 (when available)
5. PDF/document content type support in message conversion
6. Unit test: extended thinking request includes `thinking` parameter
7. Unit test: response with thinking block → thinking stored separately from content

### Story 21.3: New Provider Constructors

As a developer using DeepSeek or xAI,
I want convenience constructors,
So that setup is one line.

**Acceptance Criteria:**

1. `deepseek(model)` reads `DEEPSEEK_API_KEY` from env, base URL `https://api.deepseek.com/v1`
2. `xai(model)` reads `XAI_API_KEY` from env, base URL `https://api.x.ai/v1`
3. `infer_model_info` updated with DeepSeek and xAI model mappings
4. All constructors re-exported from `traitclaw-openai`
5. Unit test: `deepseek("deepseek-chat")` creates correct config
6. Unit test: `xai("grok-2")` creates correct config

---

## Epic 22: Steering Enhancements

**Goal:** Steering system gains rate-limit awareness and content safety guards.

**FRs:** FR16 | **Crate:** `traitclaw-steering`

### Story 22.1: RateLimitGuard & ContentFilterGuard

As a developer managing API costs,
I want guards that prevent excessive API calls and filter unsafe content,
So that my agent stays within budget and safety bounds.

**Acceptance Criteria:**

1. `RateLimitGuard::new(max_calls_per_minute)` limits agent loop iterations by time window
2. When limit exceeded, guard returns `GuardAction::Halt` with rate limit message
3. `ContentFilterGuard::new()` with `.deny_pattern(regex)` blocks outputs matching patterns
4. Default patterns: common injection attempts, PII patterns (email, phone, SSN)
5. `.with_custom_patterns(vec![...])` allows user-defined blocklist
6. Unit test: 60 calls/min limit → 61st call halted
7. Unit test: output containing email address → filtered by default PII pattern

---

## Epic 23: Documentation & Examples

**Goal:** Developers can learn v0.5.0 features through guides and runnable examples.

**FRs:** — | **All crates**

### Story 23.1: Migration Guide v0.4 → v0.5

As a developer upgrading from v0.4.0,
I want a clear migration guide,
So that I understand what's new and handle any eval trait changes.

**Acceptance Criteria:**

1. `docs/migration-v0.4-to-v0.5.md` is created
2. Documents the async `Metric` trait breaking change in `traitclaw-eval`
3. Shows RAG pipeline setup example
4. Shows Team.run() example
5. Shows RetryPolicy configuration

### Story 23.2: RAG Pipeline Example

As a developer,
I want `examples/21-rag-pipeline/` demonstrating retrieval-augmented generation,
So that I can learn how to add document knowledge to my agent.

**Acceptance Criteria:**

1. `examples/21-rag-pipeline/` created with `Cargo.toml` and `src/main.rs`
2. Demonstrates: chunk documents → embed → retrieve → ground → agent answer
3. Uses `KeywordRetriever` (no API key needed for example)
4. Compiles and runs successfully

### Story 23.3: Multi-Agent Team Example

As a developer,
I want `examples/22-multi-agent-team/` demonstrating team orchestration,
So that I can learn how to build multi-agent workflows.

**Acceptance Criteria:**

1. `examples/22-multi-agent-team/` created with `Cargo.toml` and `src/main.rs`
2. Demonstrates: define team → bind agents → run → get result
3. Shows SequentialRouter and VerificationChain patterns

### Story 23.4: Eval Runner Example

As a developer,
I want `examples/23-eval-runner/` demonstrating quality evaluation,
So that I can learn how to test my agent's output quality.

**Acceptance Criteria:**

1. `examples/23-eval-runner/` created with `Cargo.toml` and `src/main.rs`
2. Demonstrates: create suite → add cases → run with metrics → print report
3. Shows KeywordMetric and report summary

---

## Implementation Order (Dependency Graph)

```
Phase 1 (Week 1-3): RAG
  Epic 17: Story 17.1 → 17.2
  Epic 18: Story 18.1 → 18.2 (depends on Epic 17)

Phase 2 (Week 4-6): Multi-Agent
  Epic 19: Story 19.1 → 19.2 → 19.3

Phase 3 (Week 7-9): Eval
  Epic 20: Story 20.1 → 20.2 → 20.3

Phase 4 (Week 10-11): Providers & Steering
  Epic 21: Stories 21.1, 21.2, 21.3 (parallel)
  Epic 22: Story 22.1

Phase 5 (Week 12): Documentation
  Epic 23: Stories 23.1-23.4 (parallel, depends on all prior)
```

**Critical Path:** Epic 17 → Epic 18 (RAG dependency chain). All other pillars are independent.

**Total:** 7 epics, 18 stories
