# baseclaw

[![crates.io](https://img.shields.io/crates/v/baseclaw.svg)](https://crates.io/crates/baseclaw)
[![docs.rs](https://docs.rs/baseclaw/badge.svg)](https://docs.rs/baseclaw)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/baseclaw/baseclaw)

**A Rust AI Agent Framework — Simple by default, powerful when needed.**

`baseclaw` is the meta-crate entry point for the BaseClaw framework. Add one dependency and you're ready to build AI agents with type-safe tools, streaming, memory, and multi-agent orchestration.

## Quick Start

```toml
[dependencies]
baseclaw = "0.1"
baseclaw-openai-compat = "0.1"
tokio = { version = "1", features = ["full"] }
```

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

## Feature Flags

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

Enable everything:

```toml
baseclaw = { version = "0.1", features = ["full"] }
```

## What's Included

This crate re-exports:

- **`baseclaw-core`** — `Agent`, `Provider`, `Tool`, `Memory` traits and runtime
- **`baseclaw-macros`** — `#[derive(Tool)]` proc macro
- Feature-gated access to all extension crates

## License

Licensed under either of [Apache License, Version 2.0](../LICENSE-APACHE) or [MIT License](../LICENSE-MIT) at your option.
