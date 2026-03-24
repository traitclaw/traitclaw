# traitclaw-openai

[![crates.io](https://img.shields.io/crates/v/traitclaw-openai.svg)](https://crates.io/crates/traitclaw-openai)
[![docs.rs](https://docs.rs/traitclaw-openai/badge.svg)](https://docs.rs/traitclaw-openai)

**Native OpenAI provider for TraitClaw with structured output and ergonomic constructors.**

Provides first-class integration with OpenAI's API including GPT-4o, GPT-4o-mini, o1, and future models. Uses native structured output when available.

## Usage

```rust
use traitclaw::prelude::*;
use traitclaw_openai::OpenAiProvider;

let provider = OpenAiProvider::new("gpt-4o-mini", "sk-...");

let agent = Agent::builder()
    .provider(provider)
    .system("You are a helpful assistant")
    .build()?;

let output = agent.run("Hello!").await?;
```

## When to Use This vs `traitclaw-openai-compat`

| | `traitclaw-openai` | `traitclaw-openai-compat` |
|-|:-:|:-:|
| OpenAI-only features | ✅ Native support | Partial |
| Structured output | ✅ Native `response_format` | Schema injection |
| Other backends (Ollama, Groq) | ❌ | ✅ |

Use **this crate** for pure OpenAI deployments. Use `traitclaw-openai-compat` if you need to swap between providers.

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE) or [MIT License](../../LICENSE-MIT) at your option.
