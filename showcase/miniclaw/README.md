# Miniclaw — Mini AI Assistant Showcase

A complete AI assistant built with BaseClaw in ~150 lines of code.

## Progressive Enhancement

| Step | Feature | Lines Added |
|------|---------|:-----------:|
| 1 | Basic Agent | ~10 |
| 2 | + SQLite Memory | +5 |
| 3 | + Tools (ReadFile, WebSearch) | +30 |
| 4 | + Steering (auto-config) | +3 |
| 5 | Interactive REPL | +30 |

**Total: ~150 lines** (well under the 1000-line budget)

## Features

- 🤖 **Agent** — OpenAI-compatible provider with system prompt
- 🧠 **Memory** — SQLite-backed conversation persistence
- 🔧 **Tools** — File reading + web search (mock)
- 🛡️ **Steering** — Auto-configured guards/hints/tracking
- 💬 **REPL** — Interactive CLI with conversation history

## Running

```bash
export OPENAI_API_KEY="sk-..."
cargo run
```

## Architecture Validation

This showcase validates BaseClaw's DX promise:
- Each feature adds only a few lines
- No boilerplate or hacks needed
- Clean, readable code throughout
