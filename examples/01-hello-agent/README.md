# 01 — Hello Agent

The simplest possible TraitClaw agent in ~5 lines of code.

## What it does

1. Creates an **OpenAI-compatible provider** pointed at `gpt-4o-mini`
2. Builds an **Agent** with a system prompt
3. Calls `agent.run()` and prints the response

## Line-by-line explanation

```rust
let provider = OpenAiCompatProvider::openai("gpt-4o-mini", api_key);
```
> Creates a provider for any OpenAI-compatible API (OpenAI, Groq, Together, etc.)

```rust
let agent = Agent::builder()
    .provider(provider)
    .system("You are a friendly assistant. Keep responses brief.")
    .build()?;
```
> The builder pattern configures the agent. `.provider()` sets the LLM backend, `.system()` sets behavior.

```rust
let output = agent.run("Hello! What is TraitClaw?").await?;
println!("{}", output.text());
```
> `.run()` sends a single-turn message and returns an `AgentOutput`. `.text()` extracts the text content as `&str`.

## Running

```bash
export OPENAI_API_KEY="sk-..."
cargo run
```
