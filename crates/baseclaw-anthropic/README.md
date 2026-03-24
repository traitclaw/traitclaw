# baseclaw-anthropic

[![crates.io](https://img.shields.io/crates/v/baseclaw-anthropic.svg)](https://crates.io/crates/baseclaw-anthropic)
[![docs.rs](https://docs.rs/baseclaw-anthropic/badge.svg)](https://docs.rs/baseclaw-anthropic)

**Anthropic Claude provider for BaseClaw — Claude 3.5 Sonnet, Claude 3 Opus, Claude 3 Haiku.**

Native integration with Anthropic's Messages API for the BaseClaw AI agent framework.

## Supported Models

| Model | Use Case |
|-------|----------|
| `claude-sonnet-4-20250514` | Best balance of intelligence and speed |
| `claude-3-opus-20240229` | Maximum capability |
| `claude-3-haiku-20240307` | Fastest and most affordable |

## Usage

```rust
use baseclaw::prelude::*;
use baseclaw_anthropic::AnthropicProvider;

let provider = AnthropicProvider::new("claude-sonnet-4-20250514", "sk-ant-...");

let agent = Agent::builder()
    .provider(provider)
    .system("You are a helpful assistant")
    .build()?;

let output = agent.run("Explain quantum computing").await?;
```

## Features

- ✅ Messages API (`/v1/messages`)
- ✅ Streaming with SSE
- ✅ Tool use / function calling
- ✅ System prompts
- ✅ Multi-turn conversations

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE) or [MIT License](../../LICENSE-MIT) at your option.
