# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-03-29

### Added
- `CHANGELOG.md`, `CONTRIBUTING.md` ecosystem files
- `LICENSE-MIT` and `LICENSE-APACHE` at repository root
- Complete `Cargo.toml` metadata for all 14 crates (keywords, categories, docs.rs config)
- Sub-crate `README.md` files for crates.io display
- `#![deny(missing_docs)]` enforced across all library crates
- Roadmap section in README (v1.1–v1.3 plans)

### Changed
- **API frozen** under Semantic Versioning — no breaking changes until v2.0.0
- MSRV set to 1.75 (documented in Cargo.toml and README)
- Root README overhauled with badges, Feature Matrix, Architecture diagram, all 23 examples

## [0.9.0] - 2026-03-28

### Removed
- **Breaking:** `ContextStrategy` trait (replaced by `ContextManager` in v0.3.0)
- **Breaking:** `OutputProcessor` trait (replaced by `OutputTransformer` in v0.3.0)

### Changed
- Prelude enriched with all commonly-used types
- Builder error messages standardized for consistency
- Test suite refactored to use `traitclaw-test-utils` shared mocks

### Fixed
- `DefaultStrategy` error handling for empty tool call responses
- `CompressedMemory` cache invalidation on session boundary
- `LeaderRouter` parsing edge cases with malformed LLM output

## [0.8.0] - 2026-03-28

### Added
- `BudgetAwareTruncator` — token-budget-aware output truncation
- `TransformerChain` — composable output transformer pipeline
- `DynamicRegistry` — runtime tool registration/deregistration
- `AdaptiveRegistry` — tier-based tool filtering by model capability
- `ProgressiveTransformer` — incremental output processing

### Changed
- Quality foundation improvements across core abstractions
- Improved observability for context management pipeline

## [0.7.0] - 2026-03-27

### Added
- `traitclaw-strategies` crate with three reasoning strategies:
  - **ReAct** — Think→Act→Observe reasoning loops with tool use
  - **Chain-of-Thought (CoT)** — Structured step-by-step reasoning
  - **MCTS** — Monte Carlo Tree Search with parallel branch evaluation
- `ThoughtStep` observability type for strategy introspection
- `StrategyEvent` streaming for real-time strategy progress
- Feature flags (`react`, `cot`, `mcts`) for selective compilation

## [0.6.0] - 2026-03-26

### Added
- `AgentFactory` — shared-provider agent construction pattern
- `AgentPool` — named agent pool with round-robin and lookup
- Comprehensive examples (24–26): agent-factory, reasoning-strategies, observability
- Backward compatibility with all v0.5.0 APIs (zero breaking changes)
- `traitclaw-team` crate enhancements:
  - `ConditionalRouter` for rule-based routing
  - `TeamRunner` for managed team execution
  - `pool_from_team` and `pool_from_team_arc` convenience constructors

## [0.5.0] - 2026-03-25

### Added
- `traitclaw-eval` crate — evaluation framework with `EvalSuite`, `TestCase`, `Metric`
  - Built-in metrics: `KeywordMetric`, `LengthRelevancyMetric`
  - `LlmJudgeMetric` for LLM-as-judge evaluation
  - `EvalRunner` for async evaluation execution
  - JSON/CSV export via `EvalReportExport`
- `traitclaw-rag` crate — RAG pipeline
  - `Retriever` trait, `KeywordRetriever` (BM25-style)
  - `EmbeddingRetriever`, `HybridRetriever` for advanced retrieval
  - `Chunker` trait with `FixedSizeChunker`, `SentenceChunker`, `RecursiveChunker`
  - `RagContextManager` for automatic grounding

## [0.4.0] - 2026-03-25

### Added
- `traitclaw-mcp` crate — Model Context Protocol client
  - `McpServer` with stdio and SSE transport
  - `McpToolRegistry` for automatic tool discovery
  - `MultiServerMcpRegistry` for multi-server aggregation
- `traitclaw-steering` crate — Guard/Hint/Tracker system
  - Guards: `ShellDenyGuard`, `LoopDetectionGuard`, `RateLimitGuard`, `ToolBudgetGuard`, `ContentFilterGuard`, `PromptInjectionGuard`, `WorkspaceBoundaryGuard`
  - Hints: `BudgetHint`, `SystemPromptReminder`, `TruncationHint`, `TeamProgressHint`
  - Trackers: `AdaptiveTracker`
  - `Steering::auto()` one-liner configuration
- `GroupedRegistry` — organize tools by category

## [0.3.0] - 2026-03-25

### Added
- `ContextManager` async trait — custom context window management
- `OutputTransformer` async trait — post-processing agent output
- Built-in context managers: `RuleBasedCompressor`, `LlmCompressor`, `TieredCompressor`
- Built-in transformers: `JsonExtractor`, `FullOutputRetriever`
- Blanket implementations for backward compatibility with sync traits

## [0.2.0] - 2026-03-24

### Added
- `AgentStrategy` trait — pluggable execution strategies
- `AgentHook` trait — lifecycle hooks for observability
- `DefaultStrategy` — standard agent execution loop
- `LoggingHook` — built-in tracing-based hook
- `CompressedMemory` — LLM-powered conversation compression
- Streaming support via `CompletionStream` and `StreamEvent`
- `RetryProvider` — automatic retry with exponential backoff

### Fixed
- `DefaultStrategy` error handling for edge cases
- `CompressedMemory` cache invalidation

## [0.1.0] - 2026-03-24

### Added
- Initial release of the TraitClaw AI Agent Framework
- `traitclaw-core` — `Agent`, `AgentBuilder`, `Provider`, `Tool`, `ErasedTool`, `Memory`, `Message` types
- `traitclaw-macros` — `#[derive(Tool)]` proc macro for type-safe tool generation
- `traitclaw-openai-compat` — OpenAI-compatible provider (GPT, Ollama, Groq, Mistral, vLLM)
- `traitclaw-openai` — Native OpenAI provider with structured output
- `traitclaw-anthropic` — Anthropic Claude provider
- `traitclaw-memory-sqlite` — SQLite-backed persistent memory with FTS5
- `traitclaw-test-utils` — Mock providers and test helpers
- `traitclaw` — Meta-crate with feature flags
- InMemoryMemory for quick prototyping
- 10 getting-started examples

[1.0.0]: https://github.com/traitclaw/traitclaw/releases/tag/v1.0.0
[0.9.0]: https://github.com/traitclaw/traitclaw/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/traitclaw/traitclaw/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/traitclaw/traitclaw/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/traitclaw/traitclaw/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/traitclaw/traitclaw/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/traitclaw/traitclaw/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/traitclaw/traitclaw/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/traitclaw/traitclaw/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/traitclaw/traitclaw/releases/tag/v0.1.0
