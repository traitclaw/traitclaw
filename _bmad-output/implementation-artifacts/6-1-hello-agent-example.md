# Story 6.1: Hello Agent Example

Status: ready-for-dev

## Story

As a developer,
I want a minimal 5-line agent example,
So that I can validate the entire pipeline end-to-end.

## Acceptance Criteria

1. **Given** `examples/01-hello-agent/` exists **When** I run the example **Then** it creates an agent with provider + system prompt and calls `agent.run()`
2. **And** validates the full pipeline: Builder → Provider → LLM call → Response
3. **And** README explains every line
4. **And** example compiles and runs successfully

## Tasks / Subtasks

- [ ] Task 1: Create `examples/01-hello-agent/` with Cargo.toml and main.rs
- [ ] Task 2: Implement 5-line agent (AC: 1, 2)
- [ ] Task 3: Write README explaining every line (AC: 3)
- [ ] Task 4: Verify compilation and run (AC: 4)

## Dev Notes

### Target Code
```rust
use traitclaw::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let agent = Agent::builder()
        .model(openai("gpt-4o-mini"))
        .system("You are a helpful assistant")
        .build()?;

    let response = agent.run("Hello!").await?;
    println!("{}", response.text().unwrap_or("No response"));
    Ok(())
}
```

### References
- [Source: _bmad-output/architecture.md#7 Developer Experience - Level 1]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log
