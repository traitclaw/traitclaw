# 04 — Steering

Demonstrates Guards, Hints, and the `Steering::auto()` one-liner.

## Features Shown

1. **`Steering::auto()`** — auto-configures guards/hints/tracker based on model tier
2. **Manual Guard** — `ShellDenyGuard` blocking dangerous commands
3. **Full Agent** — agent with steering integrated

## Key Concepts

- **Guards**: Pre-execution filters that block harmful messages
- **Hints**: Context injectors that enrich prompts
- **Tracker**: Budget/usage tracking across turns
- **`Steering::auto()`**: One-liner that picks the right profile for your model tier

## Running

```bash
export OPENAI_API_KEY="sk-..."
cargo run
```
