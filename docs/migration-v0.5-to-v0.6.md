# Migration Guide: v0.5.0 → v0.6.0 "Composition"

> **Date:** March 2026
> **Status:** ✅ Zero Breaking Changes — all v0.5.0 code compiles without modification.

## Overview

v0.6.0 is a **purely additive release**. Every new API is a new function, struct, or method.
No existing types, methods, or traits have been changed or removed.

## What's New

| API | Location | Purpose |
|-----|----------|---------|
| `Agent::with_system(provider, system)` | `traitclaw-core` | One-line agent creation |
| `AgentFactory::new(provider)` | `traitclaw-core` | Factory for spawning multiple agents |
| `AgentPool::new(agents)` | `traitclaw-core` | Sequential output-chaining pipeline |
| `pool_from_team(&team, provider)` | `traitclaw-team` | Convert a `Team` to an `AgentPool` |
| `RoundRobinGroupChat::new(agents)` | `traitclaw-team` | Multi-turn round-robin group chat |
| `TerminationCondition` trait | `traitclaw-team` | Pluggable chat termination logic |

---

## API Changes (Additive Only)

### 1. `Agent::with_system()` — shorthand for single agents

**Before (v0.5.0):** Still works unchanged.
```rust
let agent = Agent::builder()
    .provider(my_provider)
    .system("You are a helpful assistant.")
    .build()?;
```

**After (v0.6.0) — optional shorthand:**
```rust
let agent = Agent::with_system(my_provider, "You are a helpful assistant.");
```

### 2. `AgentFactory` — repeated agent creation from one provider

**Before (v0.5.0):**
```rust
let researcher = Agent::builder().provider(p1).system("Research").build()?;
let writer     = Agent::builder().provider(p2).system("Write").build()?;
// p1 and p2 must be independent instances
```

**After (v0.6.0) — optional factory pattern:**
```rust
let factory = AgentFactory::new(my_provider); // provider wrapped in Arc once
let researcher = factory.spawn("You are a researcher.");
let writer     = factory.spawn("You are a writer.");
```

For custom configuration, use `spawn_with()`:
```rust
let agent = factory.spawn_with(|b| b.system("Custom").max_iterations(5))?;
```

### 3. `AgentPool` — sequential pipeline execution

**Before (v0.5.0):** Use `TeamRunner` with closures for pipelines.

**After (v0.6.0) — direct agent pipelines:**
```rust
let pool = AgentPool::new(vec![researcher, writer, editor]);
let final_output = pool.run_sequential("Write an article about Rust").await?;
```

### 4. `pool_from_team()` — bridge Team → AgentPool

**After (v0.6.0):**
```rust
use traitclaw_team::pool_from_team;

let pool = pool_from_team(&team, my_provider)?;
// Each role's system_prompt becomes the agent's system prompt
let output = pool.run_sequential("task input").await?;
```

> **Note:** All roles must have a `system_prompt` set (via `.with_system_prompt()`).

### 5. `RoundRobinGroupChat` — multi-turn agent collaboration

New in v0.6.0. No v0.5.0 equivalent.
```rust
use traitclaw_team::group_chat::RoundRobinGroupChat;

let mut chat = RoundRobinGroupChat::new(vec![alice, bob])
    .with_max_rounds(6);

let result = chat.run("Discuss the pros and cons of async Rust.").await?;
println!("Transcript: {} messages", result.transcript.len());
println!("Final: {}", result.final_message);
```

---

## Optional Adoption

**You don't have to adopt any new APIs.** All v0.5.0 builder patterns continue to work.

Recommended incremental adoption path:
1. Use `Agent::with_system()` for new simple agents — reduces boilerplate
2. Use `AgentFactory` when you need multiple agents with the same base provider
3. Use `AgentPool` when you need output-chaining pipelines
4. Use `RoundRobinGroupChat` when you need multi-turn collaborative agents

---

## Prelude Changes

Two new types are available via `use traitclaw_core::prelude::*`:
- `AgentFactory`
- `AgentPool`

No existing prelude items were removed.

---

## Backward Compatibility Verification

```sh
# All 20 existing examples compile without modification:
cargo build -p hello-agent -p tool-calling -p multi-agent-team  # ✅

# All existing tests pass (0 regressions):
cargo test -p traitclaw-core -p traitclaw-team  # ✅ 316 tests
```
