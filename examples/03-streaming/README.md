# 03 — Streaming

Real-time incremental responses using `agent.stream()`.

## What it does

1. Creates an agent configured as a storyteller
2. Calls `agent.stream()` instead of `agent.run()`
3. Iterates over `TextDelta` chunks using `tokio_stream::StreamExt`
4. Prints each chunk immediately as it arrives

## Key Concepts

- `agent.stream()` returns a `Stream<Item = Result<StreamDelta>>`
- Each `StreamDelta` contains partial text via `.text()`
- Flush stdout after each chunk for smooth real-time display

## Running

```bash
export OPENAI_API_KEY="sk-..."
cargo run
```
