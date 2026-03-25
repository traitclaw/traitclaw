# Migration Guide: v0.1.0 → v0.2.0

## TL;DR

**No breaking changes.** All v0.1.0 code compiles and runs unchanged on v0.2.0.

v0.2.0 adds four new extension points. All are opt-in — existing agents use
sensible defaults automatically.

## New Features

### 1. AgentStrategy — Custom Reasoning Loops

**Before (v0.1.0):** The agent loop was hardcoded inside `Agent::run()`.

**After (v0.2.0):** The loop is encapsulated in `DefaultStrategy`. You can swap it:

```rust
use traitclaw::prelude::*;

// v0.1.0 code — still works, uses DefaultStrategy automatically
let agent = Agent::builder()
    .model(my_provider)
    .system("You are helpful")
    .build()?;

// v0.2.0 — plug in a custom strategy
let agent = Agent::builder()
    .model(my_provider)
    .system("You are helpful")
    .strategy(MyReActStrategy::new())
    .build()?;
```

### 2. AgentHook — Lifecycle Observability & Interception

```rust
use traitclaw::prelude::*;

// Add observability
let agent = Agent::builder()
    .model(my_provider)
    .hook(LoggingHook::new(tracing::Level::INFO))
    .hook(MyMetricsHook::new())  // multiple hooks supported
    .build()?;
```

Hooks fire at: `on_agent_start/end`, `on_provider_start/end`,
`before/after_tool_execute`, `on_stream_chunk`, `on_error`.

`before_tool_execute` can return `HookAction::Block("reason")` to prevent
tool execution — useful for security policies.

### 3. Router — Multi-Agent Routing

```rust
use traitclaw_team::{Team, AgentRole};
use traitclaw_team::router::{LeaderRouter, SequentialRouter};

// v0.1.0 — still works
let team = Team::new("my_team")
    .add_role(AgentRole::new("writer", "Writes content"));

// v0.2.0 — plug in custom routing
let team = Team::new("my_team")
    .add_role(AgentRole::new("leader", "Orchestrates"))
    .add_role(AgentRole::new("writer", "Writes content"))
    .with_router(LeaderRouter::new("leader"));
```

### 4. CompressedMemory — Context Window Management

```rust
use traitclaw::prelude::*;
use traitclaw_core::memory::compressed::CompressedMemory;

// Wrap any memory with automatic compression
let memory = CompressedMemory::new(
    InMemoryMemory::new(),
    20,  // compress when > 20 messages
    5,   // keep last 5 uncompressed
);

let agent = Agent::builder()
    .model(my_provider)
    .memory(memory)
    .build()?;
```

## Dependency Changes

No new mandatory dependencies. All new features use existing dependencies
(`async-trait`, `serde`, `tracing`).
