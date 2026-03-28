# Story 5.1: Create Observability Example

Status: ready-for-dev

## Story

As an agent developer,
I want an end-to-end example demonstrating tracing + event callbacks,
so that I can learn observability features by running working code.

## Acceptance Criteria

1. `examples/26-observability/Cargo.toml` exists with proper workspace-inherited metadata
2. `examples/26-observability/src/main.rs` demonstrates: tracing subscriber setup, `on_event()` callback, and `RunUsage` cost display
3. Running with `RUST_LOG=info cargo run -p example-26-observability` shows structured tracing output
4. Event callback prints each `AgentEvent` variant with descriptive formatting
5. `RunUsage.estimated_cost_usd` is displayed at the end of the run
6. Example includes inline comments explaining each observability feature
7. Example compiles and runs against a mock/fake provider (no API key needed for CI)

## Tasks / Subtasks

- [ ] Task 1: Create example crate (AC: #1)
  - [ ] Create `examples/26-observability/Cargo.toml`
  - [ ] Add dependencies: `traitclaw`, `tokio`, `tracing`, `tracing-subscriber`
  - [ ] Add to root `Cargo.toml` workspace members
- [ ] Task 2: Implement main.rs (AC: #2, #3, #4, #5, #6)
  - [ ] Initialize `tracing_subscriber` with env filter
  - [ ] Create agent with mock provider (or OpenAI with env var check)
  - [ ] Register `on_event()` callback that prints events
  - [ ] Configure `with_pricing()` for cost tracking
  - [ ] Run agent and display output + usage
  - [ ] Add comments explaining each section
- [ ] Task 3: Verify (AC: #7)
  - [ ] `cargo build -p example-26-observability`
  - [ ] Run and verify tracing output appears
  - [ ] Run and verify event callback fires

## Dev Notes

### Example Structure

```rust
//! # Observability Example
//!
//! Demonstrates TraitClaw v0.8.0 observability features:
//! - Structured tracing with `tracing` crate
//! - Event callbacks via `on_event()`
//! - Cost tracking via `RunUsage`

use traitclaw::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter("traitclaw=info")
        .init();

    // 2. Build agent with observability features
    let agent = Agent::builder()
        .provider(/* mock or real */)
        .system("You are a helpful assistant.")
        .on_event(|event| {
            match event {
                AgentEvent::LlmStart { model } => {
                    println!("📡 Calling LLM: {model}");
                }
                AgentEvent::LlmEnd { model, prompt_tokens, completion_tokens, duration_ms } => {
                    println!("✅ LLM responded: {model} ({prompt_tokens}+{completion_tokens} tokens, {duration_ms}ms)");
                }
                AgentEvent::ToolCall { tool_name, .. } => {
                    println!("🔧 Tool call: {tool_name}");
                }
                AgentEvent::ToolResult { tool_name, success, duration_ms } => {
                    println!("📦 Tool result: {tool_name} (success={success}, {duration_ms}ms)");
                }
                _ => {} // #[non_exhaustive]
            }
        })
        .with_pricing(traitclaw::default_pricing())
        .build()?;

    // 3. Run agent
    let output = agent.run("What is 2+2?").await?;

    // 4. Display results
    println!("\n--- Results ---");
    println!("Response: {output}");
    println!("Tokens: {}", output.usage.tokens);
    println!("Cost: ${:.6}", output.usage.estimated_cost_usd);

    Ok(())
}
```

### Existing Example Pattern

Reference `examples/01-basic-agent/Cargo.toml` for proper workspace example structure:
```toml
[package]
name = "example-26-observability"
version = "0.1.0"
edition.workspace = true
publish = false

[dependencies]
traitclaw = { path = "../../crates/traitclaw" }
tokio = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

### Mock Provider for CI

The example should work without API keys for CI testing. Options:
1. Use `MockProvider` from test-utils (if made visible to examples)
2. Create a simple inline mock just for the example
3. Check env var and fallback to mock

Recommend option 2 (inline mock) since `traitclaw-test-utils` is dev-only.

### References

- [_bmad-output/planning-artifacts/architecture-v0.8.0.md](file:///Users/admin/Desktop/Projects/traitclaw/_bmad-output/planning-artifacts/architecture-v0.8.0.md) — Project Structure
- FR25 in PRD v0.8.0
- Existing examples in `examples/` directory for pattern reference

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
