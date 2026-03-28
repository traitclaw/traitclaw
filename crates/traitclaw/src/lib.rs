//! # TraitClaw
//!
//! A Rust AI Agent Framework — Simple by default, powerful when needed.
//!
//! This is the main entry point for the TraitClaw framework. It re-exports
//! everything from `traitclaw-core` and `traitclaw-macros`, so you only need
//! one dependency in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! traitclaw = "1.0"
//! ```
//!
//! ## Feature Flags
//!
//! | Feature | Crate | Default |
//! |---------|-------|---------|
//! | `openai-compat` | `traitclaw-openai-compat` | ✅ |
//! | `macros` | `traitclaw-macros` | ✅ |
//! | `steering` | `traitclaw-steering` | ❌ |
//! | `sqlite` | `traitclaw-memory-sqlite` | ❌ |
//! | `mcp` | `traitclaw-mcp` | ❌ |
//! | `rag` | `traitclaw-rag` | ❌ |
//! | `team` | `traitclaw-team` | ❌ |
//! | `eval` | `traitclaw-eval` | ❌ |
//! | `strategies` | `traitclaw-strategies` | ❌ |
//! | `full` | all of the above | ❌ |

#![deny(missing_docs)]

// Re-export everything from traitclaw-core
pub use traitclaw_core::*;

// Re-export the derive macro
pub use traitclaw_macros::Tool;

// ── Feature-gated provider re-exports ────────────────────

/// OpenAI-compatible provider (GPT, Ollama, Groq, etc.)
#[cfg(feature = "openai-compat")]
pub use traitclaw_openai_compat as openai_compat;

/// Steering system (Guards, Hints, Trackers)
#[cfg(feature = "steering")]
pub use traitclaw_steering as steering;

/// SQLite-backed persistent memory
#[cfg(feature = "sqlite")]
pub use traitclaw_memory_sqlite as memory_sqlite;

/// Model Context Protocol (MCP) integration
#[cfg(feature = "mcp")]
pub use traitclaw_mcp as mcp;

/// Retrieval-Augmented Generation (RAG)
#[cfg(feature = "rag")]
pub use traitclaw_rag as rag;

/// Multi-agent teams and delegation
#[cfg(feature = "team")]
pub use traitclaw_team as team;

/// Evaluation and benchmarking
#[cfg(feature = "eval")]
pub use traitclaw_eval as eval;

/// Reasoning strategies (ReAct, CoT, MCTS)
#[cfg(feature = "strategies")]
pub use traitclaw_strategies as strategies;
