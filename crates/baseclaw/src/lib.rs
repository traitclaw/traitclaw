//! # BaseClaw
//!
//! A Rust AI Agent Framework — Simple by default, powerful when needed.
//!
//! This is the main entry point for the BaseClaw framework. It re-exports
//! everything from `baseclaw-core` and `baseclaw-macros`, so you only need
//! one dependency in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! baseclaw = "0.1"
//! ```
//!
//! ## Feature Flags
//!
//! | Feature | Crate | Default |
//! |---------|-------|---------|
//! | `openai-compat` | `baseclaw-openai-compat` | ✅ |
//! | `macros` | `baseclaw-macros` | ✅ |
//! | `steering` | `baseclaw-steering` | ❌ |
//! | `sqlite` | `baseclaw-memory-sqlite` | ❌ |
//! | `mcp` | `baseclaw-mcp` | ❌ |
//! | `rag` | `baseclaw-rag` | ❌ |
//! | `team` | `baseclaw-team` | ❌ |
//! | `eval` | `baseclaw-eval` | ❌ |
//! | `full` | all of the above | ❌ |

#![deny(warnings)]
#![deny(missing_docs)]

// Re-export everything from baseclaw-core
pub use baseclaw_core::*;

// Re-export the derive macro
pub use baseclaw_macros::Tool;

// ── Feature-gated provider re-exports ────────────────────

/// OpenAI-compatible provider (GPT, Ollama, Groq, etc.)
#[cfg(feature = "openai-compat")]
pub use baseclaw_openai_compat as openai_compat;

/// Steering system (Guards, Hints, Trackers)
#[cfg(feature = "steering")]
pub use baseclaw_steering as steering;

/// SQLite-backed persistent memory
#[cfg(feature = "sqlite")]
pub use baseclaw_memory_sqlite as memory_sqlite;

/// Model Context Protocol (MCP) integration
#[cfg(feature = "mcp")]
pub use baseclaw_mcp as mcp;

/// Retrieval-Augmented Generation (RAG)
#[cfg(feature = "rag")]
pub use baseclaw_rag as rag;

/// Multi-agent teams and delegation
#[cfg(feature = "team")]
pub use baseclaw_team as team;

/// Evaluation and benchmarking
#[cfg(feature = "eval")]
pub use baseclaw_eval as eval;
