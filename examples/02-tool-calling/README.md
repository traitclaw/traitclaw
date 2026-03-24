# 02 — Tool Calling

Demonstrates how to define and use tools with BaseClaw agents.

## Tools Defined

| Tool | Input | Description |
|------|-------|-------------|
| `Calculator` | `expression: String` | Evaluate math expressions |
| `WeatherLookup` | `city: String` | Get weather for a city (mock) |

## Key Concepts

1. **Define a tool** by implementing the `Tool` trait with typed `Input`/`Output`
2. **Register tools** using `.tool(Calculator)` on the agent builder
3. **Tool schemas** are auto-generated from `#[derive(JsonSchema)]` on the input type
4. The LLM decides **when to call tools** based on the user's prompt

## Running

```bash
export OPENAI_API_KEY="sk-..."
cargo run
```
