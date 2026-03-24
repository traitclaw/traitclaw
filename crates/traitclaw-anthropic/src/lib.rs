//! Anthropic provider for the `TraitClaw` AI agent framework.
//!
//! Supports the Claude family of models via the Anthropic Messages API.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use traitclaw_anthropic::AnthropicProvider;
//!
//! let provider = AnthropicProvider::new(
//!     "claude-3-5-sonnet-20241022",
//!     std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY must be set"),
//! );
//! ```

#![deny(warnings)]
#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::doc_markdown)] // proper nouns (Anthropic, Claude)

mod convert;
mod provider;
mod wire;

pub use provider::AnthropicProvider;
