# 09 — Multi-Agent

Compose specialized agents into teams with role-based routing.

## What it does

1. Creates a **Team** with specialized roles (researcher, writer, reviewer)
2. Each role has a custom **system prompt** for behavior
3. Demonstrates **role routing** (find the right agent for a task)
4. Shows the **VerificationChain** pattern (generate → verify → retry)

## Key APIs

```rust
let team = Team::new("pipeline")
    .add_role(AgentRole::new("researcher", "Research topics")
        .with_system_prompt("You are a researcher"));

let chain = VerificationChain::new().with_max_retries(3);
```

## Running

```bash
cargo run  # no API key needed — demonstrates composition patterns
```
