# traitclaw-core

[![crates.io](https://img.shields.io/crates/v/traitclaw-core.svg)](https://crates.io/crates/traitclaw-core)
[![docs.rs](https://docs.rs/traitclaw-core/badge.svg)](https://docs.rs/traitclaw-core)

**Core traits, types, and runtime for the TraitClaw AI Agent Framework.**

This crate provides the foundational building blocks that all TraitClaw crates depend on. Most users should use the [`traitclaw`](https://crates.io/crates/traitclaw) meta-crate instead of depending on this directly.

## Core Abstractions

| Trait/Type | Purpose |
|-----------|---------|
| `Agent` | Orchestrates LLM calls, tool execution, and streaming |
| `AgentBuilder` | Fluent builder for configuring agents |
| `Provider` | Abstract LLM backend (OpenAI, Anthropic, Ollama, etc.) |
| `Tool` / `ErasedTool` | Type-safe tool definitions with JSON schema generation |
| `Memory` | Conversation persistence, working memory, long-term recall |
| `CompletionResponse` | Structured response from LLM completions |
| `StreamEvent` | Event types for real-time streaming |
| `Message` | Chat messages with role (system, user, assistant, tool) |

## Usage

```rust
use traitclaw_core::prelude::*;

let agent = Agent::builder()
    .provider(my_provider)
    .system("You are a helpful assistant")
    .tool(MyTool)
    .build()?;

// Single-turn completion
let output = agent.run("Hello").await?;

// Structured JSON output
let data: MyStruct = agent.run_structured("Analyze this").await?;

// Streaming
let stream = agent.stream("Tell me a story");
```

## Key Design Decisions

- **Trait-based providers** — any LLM backend implements `Provider`
- **Type-erased tools** — tools are type-safe at definition, erased at runtime for dynamic dispatch
- **Async-first** — all I/O operations are async with `tokio`
- **Zero-copy where possible** — references and slices over allocations

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE) or [MIT License](../../LICENSE-MIT) at your option.
