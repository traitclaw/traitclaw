<p align="center">
  <h1 align="center">🦀 TraitClaw</h1>
  <p align="center"><strong>A Rust AI Agent Framework — Simple by default, powerful when needed.</strong></p>
  <p align="center">
    <a href="https://crates.io/crates/traitclaw"><img src="https://img.shields.io/crates/v/traitclaw.svg" alt="crates.io"></a>
    <a href="https://docs.rs/traitclaw"><img src="https://docs.rs/traitclaw/badge.svg" alt="docs.rs"></a>
    <a href="https://github.com/traitclaw/traitclaw/actions"><img src="https://github.com/traitclaw/traitclaw/workflows/CI/badge.svg" alt="CI"></a>
    <a href="https://github.com/traitclaw/traitclaw/blob/main/LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg" alt="License"></a>
    <a href="https://github.com/traitclaw/traitclaw"><img src="https://img.shields.io/badge/MSRV-1.75-blue.svg" alt="MSRV"></a>
  </p>
</p>

Build AI agents in Rust with **5 lines of code**. TraitClaw provides type-safe tools, streaming responses, persistent memory, multi-agent teams, reasoning strategies, and MCP integration — all with zero-cost abstractions and no garbage collector.

```rust
use traitclaw::prelude::*;
use traitclaw_openai_compat::OpenAiCompatProvider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let agent = Agent::builder()
        .provider(OpenAiCompatProvider::openai("gpt-4o-mini", api_key))
        .system("You are a helpful assistant")
        .build()?;

    let output = agent.run("Hello!").await?;
    println!("{}", output.text());
    Ok(())
}
```

## Why TraitClaw?

- 🧩 **Composable Trait Architecture** — 8 core traits (`Provider`, `Tool`, `Memory`, `Guard`, `Hint`, `Tracker`, `ContextManager`, `OutputTransformer`) compose into any agent topology
- 🔧 **Type-Safe Tool Calling** — `#[derive(Tool)]` generates JSON schemas from Rust structs at compile time
- 🧠 **Multi-Strategy Reasoning** — Built-in ReAct, Chain-of-Thought, and MCTS strategies via `AgentStrategy` trait
- 👥 **Multi-Agent Teams** — Teams, routers, delegation, and verification chains out of the box
- 🔌 **MCP Integration** — Native Model Context Protocol client for tool discovery
- 📡 **First-Class Streaming** — `async Stream` with typed events, not string concatenation
- 🔒 **Production Observability** — `tracing` integration for structured logging and OpenTelemetry
- 🦀 **Zero-Cost Abstractions** — No GC, no runtime, single static binary

## Quick Start

```bash
cargo add traitclaw traitclaw-openai-compat tokio anyhow
```

## Feature Matrix

| Category | Feature | Status |
|----------|---------|:------:|
| **Core** | Agent & Builder | ✅ |
| | 8 Core Traits | ✅ |
| | Tool System + `#[derive(Tool)]` | ✅ |
| | Real-Time Streaming | ✅ |
| | Structured Output (JSON → Rust) | ✅ |
| **Reasoning** | ReAct (Think→Act→Observe) | ✅ |
| | Chain-of-Thought | ✅ |
| | Monte Carlo Tree Search | ✅ |
| | Custom Strategies | ✅ |
| **Providers** | OpenAI / GPT | ✅ |
| | Anthropic / Claude | ✅ |
| | OpenAI-Compatible (Ollama, Groq, Mistral, vLLM) | ✅ |
| **Memory** | In-Memory | ✅ |
| | SQLite Persistent | ✅ |
| | Compressed (LLM + Rule-Based) | ✅ |
| **Orchestration** | Multi-Agent Teams | ✅ |
| | Router Strategies (Sequential, Leader, Conditional) | ✅ |
| | Verification Chains | ✅ |
| | Agent Factory & Pool | ✅ |
| **Integration** | MCP Client (stdio + SSE) | ✅ |
| | RAG Pipeline (BM25, Embeddings, Hybrid) | ✅ |
| | Evaluation Framework | ✅ |
| **Safety** | Guards (Loop Detection, Rate Limit, Shell Deny) | ✅ |
| | Hints (Budget, System Prompt Reminder) | ✅ |
| | Trackers (Adaptive) | ✅ |
| **Observability** | `tracing` Integration | ✅ |
| | Lifecycle Hooks | ✅ |
| **Planned** | Benchmarks & Orchestration Strategy | 🔜 v1.1 |
| | Inter-Agent Contracts | 🔜 v1.2 |
| | Retry & Checkpoint Resilience | 🔜 v1.3 |

## Features

### 🔧 Type-Safe Tool Calling

```rust
#[derive(Tool)]
#[tool(description = "Search the web")]
struct WebSearch { query: String }
```

### 🎯 Structured Output

```rust
#[derive(Deserialize, JsonSchema)]
struct Review { title: String, rating: u8 }

let review: Review = agent.run_structured("Review Inception").await?;
```

### 📡 Real-Time Streaming

```rust
let mut stream = agent.stream("Tell me a story");
while let Some(Ok(StreamEvent::TextDelta(text))) = stream.next().await {
    print!("{text}");
}
```

### 🛡️ Steering — Guards, Hints & Trackers

```rust
use traitclaw_steering::Steering;
let steering = Steering::auto(); // One-liner: guards + hints + tracking
```

### 💾 Persistent Memory

```rust
use traitclaw_memory_sqlite::SqliteMemory;
let memory = SqliteMemory::new("./agent.db")?;
```

### 👥 Multi-Agent Teams

```rust
use traitclaw_team::{Team, AgentRole};
let team = Team::new("research")
    .add_role(AgentRole::new("researcher", "Research topics"))
    .add_role(AgentRole::new("writer", "Write summaries"));
```

### 🔌 MCP Integration

```rust
use traitclaw_mcp::McpServer;
let server = McpServer::stdio("npx", &["-y", "@mcp/server-filesystem", "."]).await?;
```

### 🧠 Reasoning Strategies

```rust
use traitclaw_strategies::ReActStrategy;
let strategy = ReActStrategy::builder().max_steps(10).build();
```

## 🏗️ Architecture Overview

TraitClaw follows a layered trait architecture:

```
┌─────────────────────────────────────────────────────────┐
│                    traitclaw (meta-crate)                │
│  Re-exports everything. One dependency, full power.     │
├─────────────────────────────────────────────────────────┤
│  Extension Crates                                       │
│  ┌──────────┐ ┌──────────┐ ┌───────┐ ┌──────┐         │
│  │ steering │ │ sqlite   │ │ mcp   │ │ team │ ...     │
│  └──────────┘ └──────────┘ └───────┘ └──────┘         │
├─────────────────────────────────────────────────────────┤
│  Provider Crates                                        │
│  ┌──────────────┐ ┌────────────┐ ┌──────────────────┐  │
│  │ openai-compat│ │ anthropic  │ │ openai (native)  │  │
│  └──────────────┘ └────────────┘ └──────────────────┘  │
├─────────────────────────────────────────────────────────┤
│  traitclaw-core (foundation)                            │
│  Agent · Provider · Tool · Memory · Guard · Hint ·      │
│  Tracker · ContextManager · OutputTransformer ·          │
│  ExecutionStrategy · AgentStrategy · AgentHook           │
└─────────────────────────────────────────────────────────┘
```

### The Agent Loop

1. **Context Hydration** — Retrieve past dialogue from `Memory`, append user prompt
2. **Provider Generation** — LLM evaluates context, returns text or tool call
3. **Tool Resolution** — Parse arguments, execute Rust function, append result
4. **Recursive Reasoning** — Repeat steps 2–3 until task is complete
5. **Memory Commit** — Save final trajectory back to `Memory`

## ⚙️ Feature Flags

```toml
[dependencies]
traitclaw = { version = "1.0", features = ["full"] }
```

| Feature | Crate | Default |
|---------|-------|:-------:|
| `openai-compat` | `traitclaw-openai-compat` | ✅ |
| `macros` | `traitclaw-macros` | ✅ |
| `steering` | `traitclaw-steering` | ❌ |
| `sqlite` | `traitclaw-memory-sqlite` | ❌ |
| `mcp` | `traitclaw-mcp` | ❌ |
| `rag` | `traitclaw-rag` | ❌ |
| `team` | `traitclaw-team` | ❌ |
| `eval` | `traitclaw-eval` | ❌ |
| `strategies` | `traitclaw-strategies` | ❌ |
| `tiktoken` | Accurate token counting | ❌ |
| `full` | All of the above | ❌ |

## 📦 Crate Ecosystem

| Crate | Purpose |
|-------|---------|
| [`traitclaw`](https://crates.io/crates/traitclaw) | **Meta-crate — start here** |
| [`traitclaw-core`](https://crates.io/crates/traitclaw-core) | Core traits, Agent runtime, Builder |
| [`traitclaw-macros`](https://crates.io/crates/traitclaw-macros) | `#[derive(Tool)]` proc macro |
| [`traitclaw-openai-compat`](https://crates.io/crates/traitclaw-openai-compat) | OpenAI, Ollama, Groq, Mistral, vLLM |
| [`traitclaw-openai`](https://crates.io/crates/traitclaw-openai) | Native OpenAI + Structured Output |
| [`traitclaw-anthropic`](https://crates.io/crates/traitclaw-anthropic) | Anthropic Claude provider |
| [`traitclaw-steering`](https://crates.io/crates/traitclaw-steering) | Guards, Hints, Trackers |
| [`traitclaw-memory-sqlite`](https://crates.io/crates/traitclaw-memory-sqlite) | SQLite persistent memory |
| [`traitclaw-mcp`](https://crates.io/crates/traitclaw-mcp) | MCP client (stdio + SSE) |
| [`traitclaw-rag`](https://crates.io/crates/traitclaw-rag) | RAG pipeline (BM25, embeddings, hybrid) |
| [`traitclaw-team`](https://crates.io/crates/traitclaw-team) | Multi-agent teams & orchestration |
| [`traitclaw-eval`](https://crates.io/crates/traitclaw-eval) | Evaluation & benchmarking |
| [`traitclaw-strategies`](https://crates.io/crates/traitclaw-strategies) | ReAct, CoT, MCTS reasoning |
| [`traitclaw-test-utils`](https://crates.io/crates/traitclaw-test-utils) | Mock providers & test helpers |

## 📚 Examples

| # | Example | Description |
|---|---------|-------------|
| 01 | [hello-agent](https://github.com/traitclaw/traitclaw/tree/main/examples/01-hello-agent) | Minimal 5-line agent |
| 02 | [tool-calling](https://github.com/traitclaw/traitclaw/tree/main/examples/02-tool-calling) | Custom tools with `#[derive(Tool)]` |
| 03 | [streaming](https://github.com/traitclaw/traitclaw/tree/main/examples/03-streaming) | Real-time streaming responses |
| 04 | [steering](https://github.com/traitclaw/traitclaw/tree/main/examples/04-steering) | Guards, Hints & auto-config |
| 05 | [structured-output](https://github.com/traitclaw/traitclaw/tree/main/examples/05-structured-output) | JSON → Rust types |
| 06 | [memory-persistence](https://github.com/traitclaw/traitclaw/tree/main/examples/06-memory-persistence) | SQLite conversation history |
| 10 | [mcp-client](https://github.com/traitclaw/traitclaw/tree/main/examples/10-mcp-client) | MCP tool discovery |
| 11 | [custom-strategy](https://github.com/traitclaw/traitclaw/tree/main/examples/11-custom-strategy) | Build your own AgentStrategy |
| 12 | [lifecycle-hooks](https://github.com/traitclaw/traitclaw/tree/main/examples/12-lifecycle-hooks) | Observability middleware |
| 13 | [custom-router](https://github.com/traitclaw/traitclaw/tree/main/examples/13-custom-router) | Custom team routing logic |
| 14 | [compressed-memory](https://github.com/traitclaw/traitclaw/tree/main/examples/14-compressed-memory) | LLM-powered memory compression |
| 15 | [context-manager](https://github.com/traitclaw/traitclaw/tree/main/examples/15-context-manager) | Custom context window management |
| 16 | [output-transformer](https://github.com/traitclaw/traitclaw/tree/main/examples/16-output-transformer) | Post-process agent output |
| 17 | [dynamic-tools](https://github.com/traitclaw/traitclaw/tree/main/examples/17-dynamic-tools) | Runtime tool registration |
| 18 | [context-managers](https://github.com/traitclaw/traitclaw/tree/main/examples/18-context-managers) | Built-in context managers |
| 19 | [grouped-registry](https://github.com/traitclaw/traitclaw/tree/main/examples/19-grouped-registry) | Grouped tool registry |
| 20 | [mcp-tool-registry](https://github.com/traitclaw/traitclaw/tree/main/examples/20-mcp-tool-registry) | MCP as ToolRegistry |
| 21 | [rag-pipeline](https://github.com/traitclaw/traitclaw/tree/main/examples/21-rag-pipeline) | RAG with BM25 + embeddings |
| 22 | [multi-agent-team](https://github.com/traitclaw/traitclaw/tree/main/examples/22-multi-agent-team) | Team orchestration |
| 23 | [eval-runner](https://github.com/traitclaw/traitclaw/tree/main/examples/23-eval-runner) | Evaluation test suites |
| 24 | [agent-factory](https://github.com/traitclaw/traitclaw/tree/main/examples/24-agent-factory) | Factory pattern for agents |
| 25 | [reasoning-strategies](https://github.com/traitclaw/traitclaw/tree/main/examples/25-reasoning-strategies) | ReAct, CoT, MCTS in action |
| 26 | [observability](https://github.com/traitclaw/traitclaw/tree/main/examples/26-observability) | Tracing & structured logging |

```bash
cd examples/01-hello-agent && cargo run
```

## 🔒 Stability Policies

### Semantic Versioning

TraitClaw follows [Semantic Versioning 2.0.0](https://semver.org/):

| Change Type | Allowed In |
|-------------|-----------|
| Bug fixes | Patch (`1.0.x`) |
| New public types, traits, methods | Minor (`1.x.0`) |
| MSRV bump | Minor (`1.x.0`) |
| Deprecation notices | Minor (`1.x.0`) |
| Removal of deprecated items | Major (`2.0.0`) |
| Trait signature changes | Major (`2.0.0`) |

### MSRV (Minimum Supported Rust Version)

- **Current:** Rust 1.75
- Bumped only in **minor** releases, never in patches
- Always documented in [CHANGELOG.md](https://github.com/traitclaw/traitclaw/blob/main/CHANGELOG.md)
- Maintains a minimum 6-month lag from the latest Rust stable release

### Deprecation Policy

- Deprecated items are marked with `#[deprecated(since = "1.x", note = "Use Y instead")]`
- Items remain available for a minimum of **2 minor versions** after deprecation
- Deprecated items are removed only in the next **major** version (v2.0.0)

## 🗺️ Roadmap

| Version | Codename | Focus |
|---------|----------|-------|
| **v1.0.0** | Production Ready | ✅ Stable API, crates.io publication |
| v1.1.0 | Benchmark & Orchestrate | Benchmarks, orchestration strategy |
| v1.2.0 | Contracts | Inter-agent contracts, typed delegation |
| v1.3.0 | Resilience | Retry, checkpoint, fault tolerance |

## License

Licensed under either of [Apache License, Version 2.0](https://github.com/traitclaw/traitclaw/blob/main/LICENSE-APACHE) or [MIT License](https://github.com/traitclaw/traitclaw/blob/main/LICENSE-MIT) at your option.
