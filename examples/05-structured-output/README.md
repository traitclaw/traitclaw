# 05 — Structured Output

Get type-safe JSON responses from LLMs using Rust structs.

## What it does

1. Defines Rust structs with `#[derive(Deserialize, JsonSchema)]`
2. Calls `agent.run_structured::<MovieReview>()` — the framework auto-generates JSON schema
3. The LLM response is parsed directly into your Rust type

## Key concept

```rust
#[derive(Debug, Deserialize, JsonSchema)]
struct MovieReview {
    title: String,
    rating: u8,
    summary: String,
}

let review: MovieReview = agent.run_structured("Review Inception").await?;
```

The framework handles schema injection and JSON parsing. If the model supports native structured output, it uses `response_format`; otherwise, schema instructions are injected into the prompt.

## Running

```bash
export OPENAI_API_KEY="sk-..."
cargo run
```
