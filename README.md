<p align="center">
  <h1 align="center">🦀 TraitClaw</h1>
  <p align="center"><strong>A Rust AI Agent Framework — Simple by default, powerful when needed.</strong></p>
  <p align="center">
    <a href="https://crates.io/crates/traitclaw"><img src="https://img.shields.io/crates/v/traitclaw.svg" alt="crates.io"></a>
    <a href="https://docs.rs/traitclaw"><img src="https://docs.rs/traitclaw/badge.svg" alt="docs.rs"></a>
    <a href="https://github.com/traitclaw/traitclaw/actions"><img src="https://github.com/traitclaw/traitclaw/workflows/CI/badge.svg" alt="CI"></a>
    <a href="https://github.com/traitclaw/traitclaw/blob/main/LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg" alt="License"></a>
  </p>
</p>

Build AI agents in Rust with **5 lines of code**. TraitClaw provides type-safe tools, streaming responses, persistent memory, multi-agent teams, and MCP integration — all with zero-cost abstractions and no garbage collector.

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

| Feature | TraitClaw | TypeScript Frameworks | Other Rust |
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
cargo add traitclaw traitclaw-openai-compat tokio anyhow
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

## 🏗️ Core Architecture & Model

TraitClaw is built entirely on Rust **traits**. There is no vendor lock-in, no hidden runtimes, and no bloated abstractions. Everything is swappable.

* **`Agent`**: The orchestrator. It manages the conversation loop, error handling, tool resolution, and streaming.
* **`Provider`**: The LLM backend interface (`impl Provider`). Whether it's OpenAI, Anthropic, or an open-source local model, they all conform to this trait.
* **`Memory`**: The conversation persistence layer (`impl Memory`). It dictates how and where dialogue history is stored.
* **`Tool`**: The functional capability (`impl Tool`). Tools define their schema and execution logic.

Because it relies on dynamic dispatch where flexibility is needed (e.g., `Box<dyn Tool>`) and static typing everywhere else, TraitClaw achieves zero-cost abstractions for the core loop.

### 🔄 The Agent Loop (Core Engine)
The true power of TraitClaw lies in how the `Agent` manages the execution loop securely and autonomously:
1. **Context Hydration**: The agent retrieves past dialogue from `Memory` and appends the user's new prompt.
2. **Provider Generation**: The `Provider` evaluates the context. It can either generate a final text response or request to execute a `Tool`.
3. **Tool Resolution & Execution**: If a tool is requested, the Agent automatically parses the arguments, executes the corresponding Rust function, and appends the `ToolResult` back into the context.
4. **Recursive Reasoning**: The Provider evaluates the tool's result. Steps 2 & 3 repeat until the Provider determines the task is complete.
5. **Memory Commit**: The final trajectory is saved back to `Memory`.

## 🚀 Extensibility

Extending TraitClaw is as simple as implementing a trait.

### Custom Providers
Want to support a proprietary enterprise LLM or `llama.cpp`? Implement the `Provider` trait.
```rust
#[async_trait]
impl Provider for MyCustomProvider {
    async fn generate(&self, messages: &[Message], config: &AgentConfig) -> Result<AgentOutput> { /* ... */ }
    async fn stream(&self, messages: &[Message], config: &AgentConfig) -> Result<BoxStream<'static, Result<StreamEvent>>> { /* ... */ }
}
```

### Custom Memory
Need to scale beyond SQLite? Implement the `Memory` trait for Redis, PostgreSQL, or DynamoDB.
```rust
#[async_trait]
impl Memory for RedisMemory {
    async fn get_messages(&self, session_id: &str) -> Result<Vec<Message>> { /* ... */ }
    async fn add_message(&self, session_id: &str, message: Message) -> Result<()> { /* ... */ }
}
```

### Dynamic Tools
While `#[derive(Tool)]` is great for simple functions, you can manually implement `Tool` to build dynamic schemas (e.g., querying a database schema at runtime to build the tool signature).

---

## 📦 Crates Ecosystem & Status

TraitClaw is modular. You only pay for what you compile. Some crates are currently in the process of being published to **crates.io** (subject to rate-limiting).

| Crate | Purpose | crates.io Status |
|---|---|---|
| **`traitclaw`** | **Meta-crate (Start Here)** | ⏳ Pending _(Rate limited)_ |
| `traitclaw-core` | Core traits, `Agent` runtime, `Tool`, `Memory` | ✅ `v0.1.0` |
| `traitclaw-macros` | `#[derive(Tool)]` proc macro | ✅ `v0.1.0` |
| `traitclaw-openai-compat`| OpenAI, Ollama, Groq, vLLM Provider | ✅ `v0.1.0` |
| `traitclaw-openai` | Native OpenAI Provider | ⏳ Pending _(Rate limited)_ |
| `traitclaw-anthropic` | Claude Provider | ✅ `v0.1.0` |
| `traitclaw-steering` | Guards, Hints, Trackers | ⏳ Pending _(Rate limited)_ |
| `traitclaw-memory-sqlite`| SQLite persistent memory | ✅ `v0.1.0` |
| `traitclaw-mcp` | MCP client (Model Context Protocol) | ⏳ Pending _(Rate limited)_ |
| `traitclaw-rag` | Basic RAG pipeline with BM25 | ⏳ Pending _(Rate limited)_ |
| `traitclaw-team` | Multi-agent orchestration | ⏳ Pending _(Rate limited)_ |
| `traitclaw-eval` | Evaluation & benchmarking suite | ⏳ Pending _(Rate limited)_ |

## ⚙️ Feature Flags

When using the `traitclaw` meta-package, you can opt-in to features:

```toml
[dependencies]
traitclaw = { version = "0.1", features = ["full"] }
```

| Feature | Enables Crate | Default |
|---------|-------|:-------:|
| `openai-compat` | `traitclaw-openai-compat` | ✅ |
| `macros` | `traitclaw-macros` | ✅ |
| `steering` | `traitclaw-steering` | ❌ |
| `sqlite` | `traitclaw-memory-sqlite` | ❌ |
| `mcp` | `traitclaw-mcp` | ❌ |
| `rag` | `traitclaw-rag` | ❌ |
| `team` | `traitclaw-team` | ❌ |
| `eval` | `traitclaw-eval` | ❌ |
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
