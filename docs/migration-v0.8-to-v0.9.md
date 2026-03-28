# Migrating from v0.8.0 to v0.9.0

## Overview

v0.9.0 is a **hardening release** that removes all deprecated types from v0.2.0/v0.3.0.
If you've already migrated to the modern async traits (`ContextManager`, `OutputTransformer`),
**this is a zero-effort upgrade.** Otherwise, follow the quick-fix table below — most projects
migrate in under 5 minutes.

## Breaking Changes

### 1. `ContextStrategy` → `ContextManager`

The sync `ContextStrategy` trait and its implementations have been removed.
Use `ContextManager` (async, introduced in v0.3.0) instead.

| Removed | Replacement |
|---------|-------------|
| `ContextStrategy` trait | `ContextManager` trait |
| `NoopContextStrategy` | (default — no explicit impl needed) |
| `SlidingWindowStrategy` | `RuleBasedCompressor` (default) |
| `AgentBuilder::context_strategy()` | `AgentBuilder::context_manager()` |
| `AgentRuntime.context_strategy` | `AgentRuntime.context_manager` |

#### Before (v0.8.0)

```rust
use traitclaw_core::prelude::*;

// Sync trait — no longer exists
impl ContextStrategy for MyStrategy {
    fn prepare(&self, messages: &mut Vec<Message>, context_window: usize, state: &mut AgentState) {
        // sync implementation
    }
}

let agent = Agent::builder()
    .provider(my_provider)
    .context_strategy(MyStrategy)
    .build()?;
```

#### After (v0.9.0)

```rust
use traitclaw_core::prelude::*;
use async_trait::async_trait;

// Async trait — the modern API
#[async_trait]
impl ContextManager for MyStrategy {
    async fn prepare(&self, messages: &mut Vec<Message>, context_window: usize, state: &mut AgentState) {
        // async implementation (can call LLMs, do I/O, etc.)
    }
}

let agent = Agent::builder()
    .provider(my_provider)
    .context_manager(MyStrategy)
    .build()?;
```

---

### 2. `OutputProcessor` → `OutputTransformer`

The sync `OutputProcessor` trait and its implementations have been removed.
Use `OutputTransformer` (async, introduced in v0.3.0) instead.

| Removed | Replacement |
|---------|-------------|
| `OutputProcessor` trait | `OutputTransformer` trait |
| `NoopProcessor` | (default — no explicit impl needed) |
| `TruncateProcessor` | `BudgetAwareTruncator` (default) |
| `ChainProcessor` | `TransformerChain` |
| `AgentBuilder::output_processor()` | `AgentBuilder::output_transformer()` |
| `AgentRuntime.output_processor` | `AgentRuntime.output_transformer` |

#### Before (v0.8.0)

```rust
use traitclaw_core::prelude::*;

impl OutputProcessor for MyProcessor {
    fn process(&self, output: String) -> String {
        // sync, no context
        output.chars().take(1000).collect()
    }
}

let agent = Agent::builder()
    .provider(my_provider)
    .output_processor(MyProcessor)
    .build()?;
```

#### After (v0.9.0)

```rust
use traitclaw_core::prelude::*;
use async_trait::async_trait;

#[async_trait]
impl OutputTransformer for MyProcessor {
    async fn transform(&self, output: String, tool_name: &str, state: &AgentState) -> String {
        // async, context-aware (tool_name + state available)
        output.chars().take(1000).collect()
    }
}

let agent = Agent::builder()
    .provider(my_provider)
    .output_transformer(MyProcessor)
    .build()?;
```

---

### 3. `AgentRuntime` Field Changes

If you implement a custom `AgentStrategy`, the `AgentRuntime` struct no longer contains:

| Removed Field | Use Instead |
|---------------|-------------|
| `runtime.context_strategy` | `runtime.context_manager` |
| `runtime.output_processor` | `runtime.output_transformer` |

---

## Search-and-Replace Quick Fix

Run these in your project to handle the most common renames.
Uses `sed -i.bak` for cross-platform compatibility (Linux + macOS):

```bash
# Trait renames
sed -i.bak 's/ContextStrategy/ContextManager/g' src/**/*.rs
sed -i.bak 's/OutputProcessor/OutputTransformer/g' src/**/*.rs

# Builder method renames  
sed -i.bak 's/\.context_strategy(/\.context_manager(/g' src/**/*.rs
sed -i.bak 's/\.output_processor(/\.output_transformer(/g' src/**/*.rs

# Implementation renames
sed -i.bak 's/SlidingWindowStrategy/RuleBasedCompressor/g' src/**/*.rs
sed -i.bak 's/TruncateProcessor/BudgetAwareTruncator/g' src/**/*.rs
sed -i.bak 's/ChainProcessor/TransformerChain/g' src/**/*.rs
sed -i.bak 's/NoopContextStrategy/RuleBasedCompressor/g' src/**/*.rs
sed -i.bak 's/NoopProcessor/BudgetAwareTruncator/g' src/**/*.rs

# Clean up backup files
find src -name '*.bak' -delete
```

> **⚠️ Review your changes after running:** These are broad text replacements that may
> rename unintended identifiers (e.g., variable names or module paths). Always run
> `git diff` to verify the changes are correct before committing.

Then update trait implementations:
- `fn prepare(...)` → `async fn prepare(...)`
- `fn process(&self, output: String) -> String` → `async fn transform(&self, output: String, tool_name: &str, state: &AgentState) -> String`

---

## Prelude Changes

### Removed from prelude

| Type | Reason |
|------|--------|
| `ContextStrategy` | Deprecated sync trait removed |
| `NoopContextStrategy` | Deprecated impl removed |
| `SlidingWindowStrategy` | Deprecated impl removed |
| `OutputProcessor` | Deprecated sync trait removed |
| `TruncateProcessor` | Deprecated impl removed |

### Added to prelude

| Type | Purpose |
|------|---------|
| `CompressedMemory` | LLM-compressed memory backend |
| `RetryConfig` | Provider retry configuration |
| `RetryProvider` | Automatic retry wrapper for providers |
| `DynamicRegistry` | Runtime tool registration/deregistration |

---

## Non-Breaking Changes

### Improved Builder Error Messages

Error messages now follow the format: `"[Component]: [what happened]. Use .[method]() to fix."`

```rust
// v0.8.0
// Error: "No provider configured. Use .provider() to set one."

// v0.9.0
// Error: "AgentBuilder: no provider configured. Use .provider(my_provider) before .build()"
```

---

## Verification

```bash
# All examples compile
cargo build --workspace

# All tests pass
cargo test --workspace

# No lint warnings
cargo clippy --workspace --all-targets -- -D warnings

# Documentation builds cleanly
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```
