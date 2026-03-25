//! OpenAI provider for the `TraitClaw` AI agent framework.
//!
//! Provides ergonomic constructor functions that read API keys from environment
//! variables, plus native OpenAI features like structured output.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use traitclaw_openai::openai;
//!
//! // Reads OPENAI_API_KEY from the environment automatically
//! let provider = openai("gpt-4o-mini");
//! ```
//!
//! # Structured Output
//!
//! ```rust,no_run
//! use traitclaw_openai::{openai, structured::StructuredOutputProvider};
//! use schemars::JsonSchema;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize, JsonSchema)]
//! struct Report { summary: String, confidence: f32 }
//!
//! let provider = StructuredOutputProvider::<Report>::new(openai("gpt-4o-mini"), "report");
//! ```

#![deny(warnings)]
#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::doc_markdown)] // proper nouns: OpenAI, Groq, etc.

mod models;
pub mod structured;

pub use models::{azure_openai, custom, deepseek, groq, mistral, ollama, openai, together, xai};
pub use traitclaw_openai_compat::{OpenAiCompatConfig, OpenAiCompatProvider};
