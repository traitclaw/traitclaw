# Contributing to TraitClaw

Thank you for your interest in contributing to TraitClaw! This guide will help you get started.

## Getting Started

1. **Fork & clone** the repository:
   ```bash
   git clone https://github.com/<your-username>/traitclaw.git
   cd traitclaw
   ```

2. **Build** the workspace:
   ```bash
   cargo build --workspace
   ```

3. **Run tests** to verify your setup:
   ```bash
   cargo test --workspace
   ```

## Filing Issues

- **Bug reports** — Include: Rust version (`rustc --version`), OS, minimal reproduction, expected vs actual behavior
- **Feature requests** — Describe the use case, proposed API surface, and which crate it belongs to
- **Questions** — Use GitHub Discussions instead of Issues

## Pull Request Process

1. **Branch** from `main` using descriptive names: `feat/add-redis-memory`, `fix/streaming-panic`, `docs/improve-examples`
2. **Keep PRs focused** — one feature or fix per PR
3. **All CI checks must pass** before merge (see below)
4. **Add tests** for new functionality
5. **Update documentation** — doc comments, README, CHANGELOG if applicable

## Code Style

TraitClaw uses standard Rust formatting and linting:

```bash
# Format — must produce no changes
cargo fmt --all --check

# Lint — must produce no warnings
cargo clippy --workspace --all-targets -- -D warnings

# Docs — must produce no warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

The workspace enforces `clippy::pedantic` at warn level with specific lints allowed. See the root `Cargo.toml` `[workspace.lints.clippy]` section for the full list.

## Testing

```bash
# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p traitclaw-core

# Run a specific test
cargo test -p traitclaw-core test_agent_builder
```

- All public APIs must have tests
- Use `traitclaw-test-utils` for mock providers and tools
- Integration tests that require API keys should be `#[ignore]`

## Crate Structure

```
crates/
├── traitclaw-core/          Core traits, Agent, Builder, runtime
├── traitclaw-macros/         #[derive(Tool)] proc macro
├── traitclaw-openai/         Native OpenAI provider
├── traitclaw-anthropic/      Anthropic Claude provider
├── traitclaw-openai-compat/  OpenAI-compatible provider (Ollama, Groq, etc.)
├── traitclaw-steering/       Guards, Hints, Trackers
├── traitclaw-memory-sqlite/  SQLite persistent memory
├── traitclaw-mcp/            Model Context Protocol client
├── traitclaw-rag/            RAG pipeline
├── traitclaw-team/           Multi-agent orchestration
├── traitclaw-eval/           Evaluation framework
├── traitclaw-strategies/     Reasoning strategies (ReAct, CoT, MCTS)
├── traitclaw-test-utils/     Shared test mocks
└── traitclaw/                Meta-crate (re-exports everything)
```

**Where does my change go?**

| Change type | Target crate |
|-------------|-------------|
| New core trait | `traitclaw-core` |
| New provider | New crate: `traitclaw-{provider}` |
| New tool feature | `traitclaw-core` (trait) or `traitclaw-macros` (derive) |
| New steering component | `traitclaw-steering` |
| New memory backend | New crate: `traitclaw-memory-{backend}` |
| New example | `examples/NN-descriptive-name/` |

## Commit Convention

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(core): add retry configuration to AgentBuilder
fix(anthropic): handle empty content in streaming response
docs(readme): add RAG pipeline example
refactor(team): extract router logic into separate module
test(strategies): add MCTS convergence tests
chore(ci): update Rust toolchain to 1.78
```

**Types:** `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `perf`
**Scopes:** crate name without `traitclaw-` prefix (e.g., `core`, `openai`, `team`)

## MSRV Policy

The Minimum Supported Rust Version is **1.75**. Do not use language features or library APIs introduced after this version without discussion.

## License

By contributing, you agree that your contributions will be licensed under the same dual license as the project: MIT OR Apache-2.0.
