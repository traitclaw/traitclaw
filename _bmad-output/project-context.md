---
project_name: 'TraitClaw'
user_name: 'Bangvu'
date: '2026-03-23'
sections_completed: ['technology_stack', 'language_rules', 'framework_rules', 'testing_rules', 'code_quality', 'workflow_rules', 'critical_rules']
existing_patterns_found: 0
---

# Project Context for AI Agents

_This file contains critical rules and patterns that AI agents must follow when implementing code in the TraitClaw project. Focus on unobvious details that agents might otherwise miss._

---

## Technology Stack & Versions

- **Language:** Rust (edition 2021, MSRV 1.75+)
- **Async runtime:** tokio 1.x (features: full)
- **HTTP client:** reqwest 0.12+ (features: json, stream)
- **Serialization:** serde 1.x + serde_json 1.x
- **JSON Schema:** schemars 0.8+
- **Logging/Tracing:** tracing 0.1+ + tracing-subscriber
- **Error handling:** thiserror 1.x (library errors), anyhow (examples only)
- **Proc macros:** syn 2.x + quote 1.x + proc-macro2
- **Streaming:** tokio-stream 0.1+, async-stream
- **Validation:** validator 0.18+ (optional, for tool input validation)
- **CLI (examples):** clap 4.x (derive feature)
- **Testing:** cargo test (built-in), tokio::test for async, mockall for mocks

### Workspace Structure

```
traitclaw/
├── Cargo.toml              # workspace root
├── crates/
│   ├── traitclaw/            # meta-crate (re-exports + feature flags)
│   ├── traitclaw-core/       # core traits, types, runtime
│   ├── traitclaw-macros/     # proc macros (#[derive(Tool)], #[tool])
│   ├── traitclaw-openai/     # OpenAI provider
│   ├── traitclaw-openai-compat/  # OpenAI-compatible provider
│   ├── traitclaw-anthropic/  # Anthropic provider
│   ├── traitclaw-ollama/     # Ollama provider
│   ├── traitclaw-memory-sqlite/  # SQLite memory backend
│   ├── traitclaw-steering/   # Guard-Hint-Track implementations
│   ├── traitclaw-rag/        # RAG pipeline
│   ├── traitclaw-mcp/        # MCP protocol
│   ├── traitclaw-team/       # Multi-agent orchestration
│   ├── traitclaw-workflow/   # Graph-based workflows
│   ├── traitclaw-eval/       # Evaluation framework
│   └── traitclaw-server/     # HTTP/WS server
├── examples/
│   ├── 01-hello-agent/
│   ├── 02-tool-calling/
│   └── ...
└── showcase/
    └── miniclaw/
```

### Crate Internal Structure

#### traitclaw-core (core traits, types, runtime)

```
crates/traitclaw-core/
├── Cargo.toml
└── src/
    ├── lib.rs                  # pub mod + pub use only
    ├── agent.rs                # Agent struct
    ├── agent_builder.rs        # AgentBuilder
    ├── config.rs               # AgentConfig
    ├── runtime.rs              # Agent loop engine
    │
    ├── traits/                 # All core traits
    │   ├── provider.rs         # Provider trait
    │   ├── tool.rs             # Tool trait + ErasedTool
    │   ├── memory.rs           # Memory trait (3 layers)
    │   ├── guard.rs            # Guard trait
    │   ├── hint.rs             # Hint trait
    │   └── tracker.rs          # Tracker trait
    │
    ├── types/                  # Shared types
    │   ├── message.rs          # Message, MessageRole
    │   ├── completion.rs       # CompletionRequest/Response
    │   ├── tool_call.rs        # ToolCall, ToolSchema
    │   ├── action.rs           # Action enum (for Guard)
    │   ├── agent_state.rs      # AgentState (for Hint/Track)
    │   └── stream.rs           # CompletionStream
    │
    ├── error.rs                # Error enum + Result alias
    │
    └── memory/                 # Default implementations
        └── in_memory.rs        # InMemoryMemory (default, zero deps)
```

#### traitclaw-macros (proc macros)

```
crates/traitclaw-macros/
├── Cargo.toml
└── src/
    ├── lib.rs                  # proc_macro entry points
    ├── tool_derive.rs          # #[derive(Tool)] implementation
    └── tool_fn.rs              # #[tool] attribute macro
```

#### traitclaw (meta-crate — re-exports only)

```
crates/traitclaw/
├── Cargo.toml                 # feature flags defined here
└── src/
    └── lib.rs                  # re-exports only: pub use traitclaw_core::*;
```

#### traitclaw-steering (Guard-Hint-Track implementations)

```
crates/traitclaw-steering/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── steering.rs             # Steering::auto() convenience
    ├── guards/
    │   ├── shell_deny.rs
    │   ├── prompt_injection.rs
    │   ├── loop_detection.rs
    │   ├── tool_budget.rs
    │   └── workspace_boundary.rs
    ├── hints/
    │   ├── budget.rs
    │   ├── truncation.rs
    │   ├── system_prompt_reminder.rs
    │   └── team_progress.rs
    └── tracker/
        └── adaptive.rs         # AdaptiveTracker
```

#### Provider crates (same pattern for all)

```
crates/traitclaw-openai/         # same pattern for anthropic, ollama, etc.
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── provider.rs             # OpenAiProvider impl
    ├── types.rs                # API request/response types
    └── stream.rs               # SSE stream parsing
```

### Structural Rules

| Rule | Detail |
|------|--------|
| **1 file = 1 concept** | `provider.rs` only contains `Provider` trait + directly related types |
| **Max 300 lines/file** | Exceeding → split into sub-module directory |
| **`traits/` directory** | All core traits live here, re-exported from `lib.rs` |
| **`types/` directory** | Shared types/enums, re-exported from `lib.rs` |
| **`prelude.rs`** | Contains `pub use` of most commonly used types for `use traitclaw::prelude::*` |
| **No `mod.rs`** | Use `module_name.rs` + `module_name/` directory pattern (Rust 2018+) |
| **Unit tests in-file** | `#[cfg(test)] mod tests` at bottom of each file |
| **Integration tests** | `crates/<crate>/tests/` — test cross-module behavior |

---

## Critical Implementation Rules

### Rust-Specific Rules

- **All public items MUST have doc comments** (`///`). No exceptions for traits, structs, enums, and public functions.
- **Use `#[must_use]` on Result-returning functions** that callers should not ignore.
- **Prefer `impl Trait` in function signatures** over explicit generic bounds when the trait is used only once.
- **Use `Arc<dyn Trait>` for trait objects stored in structs.** Prefer `impl Trait` for function parameters.
- **Error types:** Each crate defines its own error enum using `thiserror`. Core errors re-exported from `traitclaw-core::error`.
- **No `unwrap()` or `expect()` in library code.** Only allowed in tests and examples. Use `?` operator everywhere else.
- **No `println!()` in library code.** Use `tracing::info!()`, `tracing::debug!()`, etc.
- **Async functions:** All IO-bound operations must be async. CPU-bound operations should use `tokio::task::spawn_blocking`.
- **`Send + Sync + 'static` bounds** required on all trait objects that will be stored in Agent struct.
- **Feature flags:** Never import optional dependencies unconditionally. Always gate with `#[cfg(feature = "...")]`.

### Naming Conventions

| Item | Convention | Example |
|------|-----------|---------|
| Crate names | `traitclaw-*` (kebab-case) | `traitclaw-core` |
| Module files | `snake_case.rs` | `agent_builder.rs` |
| Structs/Enums | `PascalCase` | `AgentBuilder`, `GuardResult` |
| Traits | `PascalCase` (noun/adjective) | `Provider`, `Tool`, `Memory` |
| Functions | `snake_case` | `run_agent()`, `build_request()` |
| Constants | `SCREAMING_SNAKE_CASE` | `DEFAULT_MAX_TOKENS` |
| Type parameters | Single uppercase or descriptive | `T`, `P: Provider` |
| Feature flags | `kebab-case` | `"openai"`, `"sqlite"` |

### Module Organization

- **`lib.rs`** only contains `pub mod` declarations and `pub use` re-exports. No logic.
- **`mod.rs`** is forbidden. Use `module_name.rs` + `module_name/` directory pattern.
- **Public API** must be explicitly re-exported from crate root. Internal modules are `pub(crate)`.
- **One concept per file.** A trait and its implementations can share a file only if short (<100 lines).

### Error Handling Pattern

```rust
// Each crate has its own Error enum:
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),

    #[error("Tool execution failed: {tool_name}: {message}")]
    ToolExecution { tool_name: String, message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// Always use Result<T> with crate-level alias:
pub type Result<T> = std::result::Result<T, Error>;
```

---

### Testing Rules

#### Test Organization

- **Unit tests:** In the same file as the code, inside `#[cfg(test)] mod tests { ... }`.
- **Integration tests:** In `tests/` directory at crate root. One file per feature/flow.
- **Doc tests:** Every public function must have a doc test with a working example.
- **Test fixtures:** Shared test helpers go in `tests/common/mod.rs` or a `test-utils` module.

#### Test Naming

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Pattern: test_{function}_{scenario}_{expected}
    #[test]
    fn test_builder_without_model_returns_error() { ... }

    #[tokio::test]
    async fn test_agent_run_with_tools_calls_tool() { ... }

    #[test]
    fn test_guard_blocks_dangerous_command() { ... }
}
```

#### Test Requirements

- **All public traits must have a mock implementation** for testing (use `mockall` or manual mocks).
- **All public functions must have at least:**
  - 1 happy-path test
  - 1 error-path test
  - 1 edge-case test (empty input, max values, etc.)
- **Async tests use `#[tokio::test]`**, never block_on() manually.
- **No external API calls in tests.** Mock all Provider/HTTP calls.
- **Test isolation:** Each test must be independent. No shared mutable state between tests.
- **Snapshot tests** for JSON Schema output from `#[derive(Tool)]` macro.

#### Coverage

- **Target: 80%+ line coverage** for `traitclaw-core`.
- **Proc macros (`traitclaw-macros`):** Test via integration tests (compile-test crate behavior, not proc-macro internals).
- **Provider crates:** Test serialization/deserialization of API requests/responses. Mock HTTP.

---

### Code Quality & Style Rules

#### Formatting & Linting

```toml
# All crates must pass:
# cargo fmt --all --check        (no formatting issues)
# cargo clippy --all-targets     (no warnings)
# cargo test --all               (all tests pass)
# cargo doc --no-deps            (docs build clean)
```

- **`#![deny(warnings)]`** in all `lib.rs` files. No suppressed warnings without documented reason.
- **`#![deny(missing_docs)]`** in `traitclaw-core` and `traitclaw` meta-crate.
- **Clippy pedantic mode** enabled for core crates: `#![warn(clippy::pedantic)]`.

#### Documentation Standards

- **Every public trait:** Must have a doc comment explaining purpose, lifecycle, and example usage.
- **Every public struct field:** Must have a doc comment.
- **Every crate:** Must have a `//! # Crate Name` doc comment in `lib.rs` with overview + example.
- **Examples in docs must compile.** Use `# ` prefix to hide boilerplate in doc tests.
- **Link related items** using `[`TypeName`]` intra-doc links.

#### Code Style

- **Max line length:** 100 characters (rustfmt default).
- **Imports:** Group in order: std → external crates → workspace crates → local modules. Separate groups with blank line.
- **Builder pattern:** All complex structs use builder pattern. Builder returns `Result` from `build()`.
- **Prefer `&str` over `String`** in function parameters. Accept `impl Into<String>` for builder methods.

---

### Development Workflow Rules

#### Git Conventions

- **Branch naming:** `feat/description`, `fix/description`, `refactor/description`
- **Commit messages:** Conventional Commits format: `feat(core): add Agent builder pattern`
  - Prefixes: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `ci`
  - Scope = crate name: `(core)`, `(macros)`, `(openai)`, `(steering)`
- **One logical change per commit.** Don't mix features and refactors.

#### CI Checks (all must pass)

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --all-features
cargo doc --all --no-deps
```

#### Release

- **Workspace versioning:** All crates share the same version.
- **Changelog:** Maintain CHANGELOG.md following Keep a Changelog format.
- **Semver:** Strictly follow Semantic Versioning. Breaking trait changes = major version.

---

### Critical Don't-Miss Rules

#### Anti-Patterns to AVOID

- ❌ **Never put logic in `lib.rs` or `mod.rs`** — only `pub mod` and `pub use`.
- ❌ **Never use `Box<dyn Trait>` when `Arc<dyn Trait>` is needed** — all trait objects in Agent must be `Arc` for `Send + Sync + Clone`.
- ❌ **Never add optional dependencies without feature gating** — every optional dep must be behind a feature flag.
- ❌ **Never use `tokio::time::sleep` for testing timeouts** — use `tokio::time::timeout` instead.
- ❌ **Never expose internal types in public API** — use re-exports from crate root.
- ❌ **Never use `String` for error messages in Result** — always use typed errors with `thiserror`.
- ❌ **Never add `anyhow` as a dependency in library crates** — `anyhow` is for applications/examples only.

#### Security Rules

- **All user input from LLM must be validated** before use in tool execution.
- **File paths must be sanitized** — no path traversal (`..`), no absolute paths outside workspace.
- **Shell commands from tools must go through Guard system** when steering is enabled.
- **API keys and secrets:** Never log, never include in error messages, never serialize.

#### Performance Rules

- **Avoid cloning large structs.** Use `Arc` for shared ownership.
- **Streaming responses:** Use `Stream` trait, never collect full response in memory first.
- **Token counting:** Cache token counts, don't recount on every iteration.
- **Connection pooling:** reqwest Client must be reused (stored in Provider), never created per request.
