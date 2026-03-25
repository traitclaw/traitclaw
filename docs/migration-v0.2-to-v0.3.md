# Migration Guide: v0.2.0 → v0.3.0 "Context Window Rescue"

## TL;DR

**No breaking changes.** All v0.2.0 code compiles and runs unchanged on v0.3.0.

v0.3.0 adds three async trait replacements for better context management,
output processing, and dynamic tool management. All are opt-in — existing
agents use bridged defaults automatically.

## What Changed

| v0.2.0 (deprecated) | v0.3.0 (new) | Why |
|---------------------|-------------|-----|
| `ContextStrategy` (sync) | `ContextManager` (async) | Enables LLM-powered compression |
| `OutputProcessor` (sync) | `OutputTransformer` (async) | Context-aware, per-tool processing |
| `Vec<Arc<dyn ErasedTool>>` | `ToolRegistry` trait | Runtime tool add/remove/toggle |

## 1. ContextManager — Async Context Window Management

**Before (v0.2.0):**

```rust
use traitclaw::prelude::*;

// Sync ContextStrategy — still works in v0.3.0
let agent = Agent::builder()
    .model(provider)
    .context_strategy(SlidingWindowStrategy::with_threshold(0.85))
    .build()?;
```

**After (v0.3.0):**

```rust
use traitclaw::prelude::*;
use async_trait::async_trait;

struct SummarizingManager { /* ... */ }

#[async_trait]
impl ContextManager for SummarizingManager {
    async fn prepare(
        &self,
        messages: &mut Vec<Message>,
        context_window: usize,
        state: &mut AgentState,
    ) {
        // Can call LLMs, do async token counting, etc.
    }
}

let agent = Agent::builder()
    .model(provider)
    .context_manager(SummarizingManager::new())
    .build()?;
```

The old `.context_strategy()` still works — your `ContextStrategy` impl
is automatically bridged to `ContextManager` via a blanket impl.

## 2. OutputTransformer — Context-Aware Output Processing

**Before (v0.2.0):**

```rust
// Sync, no context awareness
let agent = Agent::builder()
    .model(provider)
    .output_processor(TruncateProcessor::with_limit(5000))
    .build()?;
```

**After (v0.3.0):**

```rust
use traitclaw::BudgetAwareTruncator;

// Context-aware: halves limit when context is > 80% full
let agent = Agent::builder()
    .model(provider)
    .output_transformer(BudgetAwareTruncator::new(10_000, 0.8))
    .build()?;
```

Built-in transformers:
- `BudgetAwareTruncator` — context-aware truncation
- `JsonExtractor` — extracts JSON from verbose output
- `TransformerChain` — compose multiple transformers

## 3. ToolRegistry — Dynamic Tool Management

**Before (v0.2.0):**

```rust
// Tools fixed at construction time
let agent = Agent::builder()
    .model(provider)
    .tool(SearchTool)
    .tool(CalcTool)
    .build()?;
```

**After (v0.3.0):**

```rust
use traitclaw::DynamicRegistry;

let registry = DynamicRegistry::new();
registry.register(Arc::new(SearchTool));
registry.register(Arc::new(CalcTool));

let agent = Agent::builder()
    .model(provider)
    .tool(SearchTool)
    .tool(CalcTool)
    .tool_registry(registry.clone())
    .build()?;

// Later: toggle tools at runtime
registry.set_enabled("search", false);
registry.register(Arc::new(NewTool));
```

## 4. Token Counting Utilities

New standalone utility for token estimation:

```rust
use traitclaw::CharApproxCounter;

let counter = CharApproxCounter::new(4); // 4 chars ≈ 1 token
let tokens = counter.count(&messages);
```

## Deprecation Timeline

| Trait | Status | Removal |
|-------|--------|---------|
| `ContextStrategy` | Deprecated, auto-bridged | v1.0.0 |
| `OutputProcessor` | Deprecated, auto-bridged | v1.0.0 |
| `.context_strategy()` builder | Works, prefer `.context_manager()` | v1.0.0 |
| `.output_processor()` builder | Works, prefer `.output_transformer()` | v1.0.0 |
