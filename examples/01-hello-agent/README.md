# 01 — Hello Agent

The simplest possible BaseClaw agent in ~5 lines of code.

## What it does

1. Creates an **OpenAI-compatible provider** pointed at `gpt-4o-mini`
2. Builds an **Agent** with a system prompt
3. Calls `agent.run()` and prints the response

## Line-by-line explanation

```rust
let provider = OpenAiCompatProvider::new("gpt-4o-mini", api_key, url);
```
> Creates a provider for any OpenAI-compatible API (OpenAI, Groq, Together, etc.)

```rust
let agent = Agent::builder()
    .provider(provider)
    .system_prompt("You are a friendly assistant.")
    .build()?;
```
> The builder pattern configures the agent. `.provider()` sets the LLM backend, `.system_prompt()` sets behavior.

```rust
let response = agent.run("Hello!").await?;
println!("{}", response.text().unwrap_or("No response"));
```
> `.run()` sends a single-turn message and returns a `CompletionResponse`. `.text()` extracts the text content.

## Running

```bash
export OPENAI_API_KEY="sk-..."
cargo run
```
