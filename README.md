<p align="center">
  <h1 align="center">🦀 BaseClaw</h1>
  <p align="center"><strong>A Rust AI Agent Framework — Simple by default, powerful when needed.</strong></p>
  <p align="center">
    <a href="https://crates.io/crates/baseclaw"><img src="https://img.shields.io/crates/v/baseclaw.svg" alt="crates.io"></a>
    <a href="https://docs.rs/baseclaw"><img src="https://docs.rs/baseclaw/badge.svg" alt="docs.rs"></a>
    <a href="https://github.com/baseclaw/baseclaw/actions"><img src="https://github.com/baseclaw/baseclaw/workflows/CI/badge.svg" alt="CI"></a>
    <a href="https://github.com/baseclaw/baseclaw/blob/main/LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg" alt="License"></a>
  </p>
</p>

Build AI agents in Rust with **5 lines of code**. BaseClaw provides type-safe tools, streaming responses, persistent memory, multi-agent teams, and MCP integration — all with zero-cost abstractions and no garbage collector.

```rust
use baseclaw::prelude::*;
use baseclaw_openai_compat::OpenAiCompatProvider;

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

## Why BaseClaw?

| Feature | BaseClaw | TypeScript Frameworks | Other Rust |
|---------|:--------:|:---------------------:|:----------:|
| **Zero-cost abstractions** | ✅ | ❌ GC overhead | Partial |
| **Type-safe tools** | ✅ `#[derive(Tool)]` | ❌ Runtime validation | ❌ Manual |
| **5-line quickstart** | ✅ | ✅ | ❌ Verbose |
| **Streaming** | ✅ First-class | ✅ | Partial |
| **Persistent memory** | ✅ SQLite built-in | ❌ External | ❌ External |
| **Multi-agent teams** | ✅ | Partial | ❌ |
| **MCP integration** | ✅ Native | ✅ | ❌ |
| **Static binary** | ✅ Single binary | ❌ Needs runtime | ✅ |

## Quick Start

```bash
cargo add baseclaw baseclaw-openai-compat tokio anyhow
```

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
use baseclaw_steering::Steering;
let steering = Steering::auto(); // One-liner: guards + hints + tracking
```

### 💾 Persistent Memory

```rust
use baseclaw_memory_sqlite::SqliteMemory;
let memory = SqliteMemory::new("./agent.db")?;
```

### 👥 Multi-Agent Teams

```rust
use baseclaw_team::{Team, AgentRole};
let team = Team::new("research")
    .add_role(AgentRole::new("researcher", "Research topics"))
    .add_role(AgentRole::new("writer", "Write summaries"));
```

### 🔌 MCP Integration

```rust
use baseclaw_mcp::McpServer;
let server = McpServer::stdio("npx", &["-y", "@mcp/server-filesystem", "."]).await?;
```

## Crate Architecture

```
baseclaw                    ← Meta-crate (start here)
├── baseclaw-core           ← Agent, Provider, Tool, Memory traits + runtime
├── baseclaw-macros          ← #[derive(Tool)] proc macro
├── baseclaw-openai-compat   ← OpenAI/Ollama/Groq/vLLM provider
├── baseclaw-openai          ← Native OpenAI provider
├── baseclaw-anthropic       ← Claude provider
├── baseclaw-steering        ← Guards, Hints, Trackers
├── baseclaw-memory-sqlite   ← SQLite persistent memory
├── baseclaw-mcp             ← MCP client (Model Context Protocol)
├── baseclaw-rag             ← RAG pipeline with BM25 retrieval
├── baseclaw-team            ← Multi-agent orchestration
└── baseclaw-eval            ← Evaluation & benchmarking
```

## Feature Flags

```toml
[dependencies]
baseclaw = { version = "0.1", features = ["full"] }
```

| Feature | Crate | Default |
|---------|-------|:-------:|
| `openai-compat` | `baseclaw-openai-compat` | ✅ |
| `macros` | `baseclaw-macros` | ✅ |
| `steering` | `baseclaw-steering` | ❌ |
| `sqlite` | `baseclaw-memory-sqlite` | ❌ |
| `mcp` | `baseclaw-mcp` | ❌ |
| `rag` | `baseclaw-rag` | ❌ |
| `team` | `baseclaw-team` | ❌ |
| `eval` | `baseclaw-eval` | ❌ |
| `full` | All of the above | ❌ |

## Examples

| Example | Description | API Key? |
|---------|-------------|:--------:|
| [01-hello-agent](examples/01-hello-agent) | Minimal 5-line agent | Yes |
| [02-tool-calling](examples/02-tool-calling) | Custom tools with `impl Tool` | Yes |
| [03-streaming](examples/03-streaming) | Real-time streaming responses | Yes |
| [04-steering](examples/04-steering) | Guards, Hints, auto-config | Yes |
| [05-structured-output](examples/05-structured-output) | JSON → Rust types | Yes |
| [06-memory-persistence](examples/06-memory-persistence) | SQLite conversation history | No |
| [07-rag-pipeline](examples/07-rag-pipeline) | Retrieval-Augmented Generation | Optional |
| [08-eval-suite](examples/08-eval-suite) | Agent quality testing | No |
| [09-multi-agent](examples/09-multi-agent) | Team composition | No |
| [10-mcp-client](examples/10-mcp-client) | MCP tool integration | Yes |

```bash
cd examples/01-hello-agent && cargo run
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
