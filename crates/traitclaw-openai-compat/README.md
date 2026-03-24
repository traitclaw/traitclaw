# traitclaw-openai-compat

[![crates.io](https://img.shields.io/crates/v/traitclaw-openai-compat.svg)](https://crates.io/crates/traitclaw-openai-compat)
[![docs.rs](https://docs.rs/traitclaw-openai-compat/badge.svg)](https://docs.rs/traitclaw-openai-compat)

**OpenAI-compatible provider for TraitClaw — works with OpenAI, Ollama, Groq, Mistral, vLLM, Azure, and any OpenAI-compatible API.**

This is the most versatile provider in the TraitClaw ecosystem. If your LLM exposes an OpenAI-compatible `/v1/chat/completions` endpoint, this crate will work.

## Supported Backends

| Backend | Constructor |
|---------|------------|
| **OpenAI** (GPT-4o, GPT-4o-mini) | `OpenAiCompatProvider::openai("gpt-4o-mini", key)` |
| **Ollama** (local models) | `OpenAiCompatProvider::new("llama3.2", "", "http://localhost:11434/v1")` |
| **Groq** (Mixtral, LLaMA) | `OpenAiCompatProvider::new("mixtral-8x7b", key, "https://api.groq.com/openai/v1")` |
| **Azure OpenAI** | `OpenAiCompatProvider::new("gpt-4", key, azure_url)` |
| **vLLM** | `OpenAiCompatProvider::new("model", "", "http://localhost:8000/v1")` |
| **Together AI** | `OpenAiCompatProvider::new("model", key, "https://api.together.xyz/v1")` |

## Usage

```rust
use traitclaw::prelude::*;
use traitclaw_openai_compat::OpenAiCompatProvider;

// OpenAI
let provider = OpenAiCompatProvider::openai("gpt-4o-mini", "sk-...");

// Any OpenAI-compatible endpoint
let provider = OpenAiCompatProvider::new(
    "llama3.2",
    "",
    "http://localhost:11434/v1",
);

let agent = Agent::builder()
    .provider(provider)
    .system("You are a helpful assistant")
    .build()?;
```

## Features

- ✅ Chat completions (`/v1/chat/completions`)
- ✅ Streaming with SSE
- ✅ Tool calling / function calling
- ✅ Structured output (JSON mode)
- ✅ Configurable base URL for any endpoint

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE) or [MIT License](../../LICENSE-MIT) at your option.
