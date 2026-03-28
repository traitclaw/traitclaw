# traitclaw-test-utils

[![crates.io](https://img.shields.io/crates/v/traitclaw-test-utils.svg)](https://crates.io/crates/traitclaw-test-utils)
[![docs.rs](https://docs.rs/traitclaw-test-utils/badge.svg)](https://docs.rs/traitclaw-test-utils)

**Shared test utilities for the TraitClaw AI Agent Framework.**

Provides reusable mock implementations and helpers for testing agents without hitting real LLM APIs:

- **`MockProvider`** — Deterministic LLM provider returning pre-defined responses
- **`MockMemory`** — In-memory session-based memory backend
- **`EchoTool`** — Tool that echoes its input for tool-calling tests
- **`FailTool`** — Tool that always returns an error
- **`make_runtime`** — One-call `AgentRuntime` setup for strategy tests

## Usage

```toml
[dev-dependencies]
traitclaw-test-utils = "1.0"
```

```rust
use traitclaw_test_utils::provider::MockProvider;
use traitclaw_test_utils::runtime::make_runtime;

let runtime = make_runtime(MockProvider::text("hello"), vec![]);
// Use runtime with any AgentStrategy for deterministic testing
```

## Documentation

- [API Reference (docs.rs)](https://docs.rs/traitclaw-test-utils)
- [TraitClaw Guide](https://github.com/traitclaw/traitclaw)
- [Examples](https://github.com/traitclaw/traitclaw/tree/main/examples)

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE) or [MIT License](../../LICENSE-MIT) at your option.
