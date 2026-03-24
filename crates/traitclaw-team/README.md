# traitclaw-team

[![crates.io](https://img.shields.io/crates/v/traitclaw-team.svg)](https://crates.io/crates/traitclaw-team)
[![docs.rs](https://docs.rs/traitclaw-team/badge.svg)](https://docs.rs/traitclaw-team)

**Multi-agent orchestration for TraitClaw — teams, role routing, delegation, and verification chains.**

Compose multiple agents into teams where each agent has a specialized role. Route messages to the right agent, and set up generate-then-verify pipelines for quality assurance.

## Usage

```rust
use traitclaw_team::{AgentRole, Team, VerificationChain, VerifyResult};

// Define a team with specialized roles
let team = Team::new("content_pipeline")
    .add_role(
        AgentRole::new("researcher", "Research topics in depth")
            .with_system_prompt("You are a thorough researcher"),
    )
    .add_role(
        AgentRole::new("writer", "Write clear summaries")
            .with_system_prompt("You are a skilled writer"),
    )
    .add_role(
        AgentRole::new("reviewer", "Review for accuracy")
            .with_system_prompt("You are a strict editor"),
    );

// Route to a specific role
let researcher = team.find_role("researcher").unwrap();

// Verification chain: generate → verify → retry if rejected
let chain = VerificationChain::new().with_max_retries(3);
```

## Components

| Component | Purpose |
|-----------|---------|
| `Team` | Container for named agent roles |
| `AgentRole` | Role definition with name, description, and optional system prompt |
| `VerificationChain` | Generate-then-verify pipeline with retry logic |
| `VerifyResult` | `Accepted(output)` or `Rejected(feedback)` |

## Patterns

- **Pipeline**: researcher → writer → reviewer (sequential processing)
- **Router**: route queries to the best-matching role
- **Verification**: generate content, verify quality, retry with feedback

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE) or [MIT License](../../LICENSE-MIT) at your option.
