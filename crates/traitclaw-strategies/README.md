# traitclaw-strategies

[![crates.io](https://img.shields.io/crates/v/traitclaw-strategies.svg)](https://crates.io/crates/traitclaw-strategies)
[![docs.rs](https://docs.rs/traitclaw-strategies/badge.svg)](https://docs.rs/traitclaw-strategies)

**Built-in reasoning strategies (ReAct, MCTS, CoT) for the TraitClaw AI agent framework.**

This crate provides three reasoning strategies that implement the `AgentStrategy` trait:

- **ReAct** — Think→Act→Observe reasoning loops with tool use
- **Chain-of-Thought (CoT)** — Structured step-by-step reasoning
- **MCTS** — Monte Carlo Tree Search with parallel branch evaluation

## Feature Flags

All strategies are enabled by default. Use `default-features = false` to selectively enable only the strategies you need.

| Feature | Description |
|---------|-------------|
| `react` | ReAct reasoning strategy |
| `cot`   | Chain-of-Thought reasoning |
| `mcts`  | Monte Carlo Tree Search |

## Usage

```toml
[dependencies]
traitclaw-strategies = "1.0"
```

```rust
use traitclaw_strategies::ReActStrategy;
use traitclaw_core::AgentStrategy;

let strategy = ReActStrategy::builder()
    .max_steps(10)
    .build();
```

## Documentation

- [API Reference (docs.rs)](https://docs.rs/traitclaw-strategies)
- [TraitClaw Guide](https://github.com/traitclaw/traitclaw)
- [Examples](https://github.com/traitclaw/traitclaw/tree/main/examples/25-reasoning-strategies)

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE) or [MIT License](../../LICENSE-MIT) at your option.
