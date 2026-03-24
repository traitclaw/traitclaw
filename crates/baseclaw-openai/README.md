# baseclaw-openai

[![crates.io](https://img.shields.io/crates/v/baseclaw-openai.svg)](https://crates.io/crates/baseclaw-openai)
[![docs.rs](https://docs.rs/baseclaw-openai/badge.svg)](https://docs.rs/baseclaw-openai)

**Native OpenAI provider for BaseClaw with structured output and ergonomic constructors.**

Provides first-class integration with OpenAI's API including GPT-4o, GPT-4o-mini, o1, and future models. Uses native structured output when available.

## Usage

```rust
use baseclaw::prelude::*;
use baseclaw_openai::OpenAiProvider;

let provider = OpenAiProvider::new("gpt-4o-mini", "sk-...");

let agent = Agent::builder()
    .provider(provider)
    .system("You are a helpful assistant")
    .build()?;

let output = agent.run("Hello!").await?;
```

## When to Use This vs `baseclaw-openai-compat`

| | `baseclaw-openai` | `baseclaw-openai-compat` |
|-|:-:|:-:|
| OpenAI-only features | ✅ Native support | Partial |
| Structured output | ✅ Native `response_format` | Schema injection |
| Other backends (Ollama, Groq) | ❌ | ✅ |

Use **this crate** for pure OpenAI deployments. Use `baseclaw-openai-compat` if you need to swap between providers.

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE) or [MIT License](../../LICENSE-MIT) at your option.
