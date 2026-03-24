# traitclaw-steering

[![crates.io](https://img.shields.io/crates/v/traitclaw-steering.svg)](https://crates.io/crates/traitclaw-steering)
[![docs.rs](https://docs.rs/traitclaw-steering/badge.svg)](https://docs.rs/traitclaw-steering)

**Built-in Guards, Hints, and Trackers for the TraitClaw AI agent framework.**

Control agent behavior with declarative safety rails, contextual hints, and automatic usage tracking — all configurable in one line.

## Concepts

| Component | Purpose |
|-----------|---------|
| **Guards** | Safety constraints (e.g., max tokens, content filtering, rate limiting) |
| **Hints** | Contextual guidance injected into the prompt (e.g., tone, format) |
| **Trackers** | Usage monitoring (tokens, latency, cost estimation) |

## Usage

```rust
use traitclaw_steering::Steering;

// One-liner: sane defaults for guards, hints, and tracking
let steering = Steering::auto();

// Or customize
let steering = Steering::builder()
    .guard(MaxTokensGuard::new(4096))
    .hint(ToneHint::new("professional"))
    .tracker(TokenTracker::new())
    .build();
```

## When to Use

- **Production agents** — add safety rails without changing agent logic
- **Cost management** — track and limit token usage
- **Tone control** — enforce consistent assistant behavior

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE) or [MIT License](../../LICENSE-MIT) at your option.
