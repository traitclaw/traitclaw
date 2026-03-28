//! OpenAI-compatible provider for the `TraitClaw` AI agent framework.
//!
//! Works with any `POST /v1/chat/completions` endpoint:
//! OpenAI, Ollama, Groq, Mistral, Together AI, vLLM, Azure OpenAI.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use traitclaw_openai_compat::OpenAiCompatProvider;
//!
//! let provider = OpenAiCompatProvider::openai(
//!     "gpt-4o-mini",
//!     std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set"),
//! );
//!
//! // Local Ollama — no auth required:
//! let ollama = OpenAiCompatProvider::ollama("llama3.2");
//! ```

#![deny(missing_docs)]
#![allow(clippy::redundant_closure)]
// `OpenAI` is a proper noun, not a code identifier — suppress doc_markdown for this crate.

mod convert;
mod provider;
pub mod retry;
mod wire;

pub use provider::{OpenAiCompatConfig, OpenAiCompatProvider};
pub use retry::RetryPolicy;
