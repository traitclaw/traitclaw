# Migrating from v0.7.0 to v0.8.0

## Overview

v0.8.0 is a **fully backward-compatible** infrastructure release.
**Zero breaking changes.** All existing code compiles without modification.

## What's New

### 1. Shared Test Utilities (`traitclaw-test-utils`)

A new dev-only crate providing reusable test infrastructure:

- `MockProvider` — deterministic LLM mock (text, tool calls, errors)
- `MockMemory` — in-memory session storage for testing
- `EchoTool`, `FailTool` — tool testing helpers
- `make_runtime()` — one-call test runtime setup

```toml
# Cargo.toml
[dev-dependencies]
traitclaw-test-utils = { path = "../traitclaw-test-utils" }
```

### 2. Structured Runtime Tracing

TraitClaw now instruments all core operations with the `tracing` crate and
[OpenTelemetry GenAI semantic conventions](https://opentelemetry.io/docs/specs/semconv/gen-ai/):

| Span | Target | Key Attributes |
|------|--------|----------------|
| `gen_ai.chat` | `traitclaw::llm` | `gen_ai.system`, `gen_ai.request.model`, `gen_ai.usage.input_tokens`, `gen_ai.usage.output_tokens` |
| `tool.call` | `traitclaw::tool` | `tool.name`, `tool.success` |
| `guard.check` | `traitclaw::guard` | `guard.name`, `guard.result` |
| Hint injection | `traitclaw::hint` | `hint_name` (debug level) |

**No new dependencies required.** The `tracing` crate (already a transitive dependency)
provides zero-cost instrumentation when no subscriber is configured.

### 3. CI/CD Pipeline

GitHub Actions workflow with 4 parallel jobs:
- `fmt` — formatting check
- `clippy` — lint enforcement
- `test` — full workspace tests
- `docs` — documentation build with `-D warnings`

## Zero Breaking Changes

The following are **explicitly unchanged**:

| API | Status |
|-----|--------|
| All trait signatures (`Provider`, `Tool`, `Memory`, `Guard`, `Hint`, `Tracker`) | ✅ Unchanged |
| `Agent::builder()` API | ✅ Unchanged |
| `AgentOutput` structure | ✅ Unchanged |
| `AgentStrategy` trait | ✅ Unchanged |
| `RunUsage` fields | ✅ Unchanged |
| All hook interfaces (`AgentHook`) | ✅ Unchanged |

## Migrating Tests (Optional)

### Before (v0.7.0 — inline mocks)

```rust
// Each test file defines its own MockProvider...
struct MockProvider { responses: Vec<String> }

#[async_trait]
impl Provider for MockProvider {
    async fn complete(&self, _req: CompletionRequest) -> Result<CompletionResponse> {
        // boilerplate...
    }
    fn model_info(&self) -> ModelInfo { /* ... */ }
}
```

### After (v0.8.0 — shared utils)

```rust
use traitclaw_test_utils::provider::MockProvider;
use traitclaw_test_utils::runtime::make_runtime;

// One-liner mock provider
let provider = MockProvider::text("Hello!");

// Or full runtime in one call
let rt = make_runtime(MockProvider::text("Hello!"), vec![]);
```

## Adding Observability (Optional)

### Console output (development)

```rust
use tracing_subscriber::EnvFilter;

tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::new("traitclaw=info"))
    .init();

// All TraitClaw spans now appear in console automatically
let output = agent.run("Hello").await?;
```

### OpenTelemetry / Langfuse (production)

```rust
use tracing_opentelemetry;
use tracing_subscriber::layer::SubscriberExt;

let tracer = opentelemetry_otlp::new_pipeline()
    .with_endpoint("https://your-langfuse.com/api/public/otel")
    .install_batch()?;
let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
tracing_subscriber::registry().with(otel_layer).init();

// All TraitClaw spans export to Langfuse automatically
// Langfuse parses gen_ai.* attributes into its LLM dashboard
```

### Component-level filtering

```bash
RUST_LOG=traitclaw=info          # All spans
RUST_LOG=traitclaw::llm=debug    # Only LLM calls
RUST_LOG=traitclaw::tool=info    # Only tool executions
RUST_LOG=traitclaw::guard=info   # Only guard checks
RUST_LOG=traitclaw=debug         # Everything including hints
```

## Verification

```bash
# All examples compile
cargo build --workspace

# All tests pass
cargo test --workspace

# No lint warnings
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Documentation builds cleanly
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```
