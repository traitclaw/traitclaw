# traitclaw

[![crates.io](https://img.shields.io/crates/v/traitclaw.svg)](https://crates.io/crates/traitclaw)
[![docs.rs](https://docs.rs/traitclaw/badge.svg)](https://docs.rs/traitclaw)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/traitclaw/traitclaw)

**A Rust AI Agent Framework — Simple by default, powerful when needed.**

`traitclaw` is the meta-crate entry point for the TraitClaw framework. Add one dependency and you're ready to build AI agents with type-safe tools, streaming, memory, and multi-agent orchestration.

## Quick Start

```toml
[dependencies]
traitclaw = "0.1"
traitclaw-openai-compat = "0.1"
tokio = { version = "1", features = ["full"] }
```

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

## Feature Flags

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
| `full` | All of the above | ❌ |

Enable everything:

```toml
traitclaw = { version = "0.1", features = ["full"] }
```

## What's Included

This crate re-exports:

- **`traitclaw-core`** — `Agent`, `Provider`, `Tool`, `Memory` traits and runtime
- **`traitclaw-macros`** — `#[derive(Tool)]` proc macro
- Feature-gated access to all extension crates

## License

Licensed under either of [Apache License, Version 2.0](../LICENSE-APACHE) or [MIT License](../LICENSE-MIT) at your option.
