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
//! # Quick Start
//!
//! ```rust,no_run
//! use baseclaw::prelude::*;
//!
//! # async fn example() -> baseclaw::Result<()> {
//! // Create an agent (requires a Provider implementation)
//! // let agent = Agent::builder()
//! //     .model(my_provider)
//! //     .system("You are a helpful assistant")
//! //     .build()?;
//! //
//! // let response = agent.run("Hello!").await?;
//! # Ok(())
//! # }
//! ```

#![deny(warnings)]
#![deny(missing_docs)]

// Re-export everything from baseclaw-core
pub use baseclaw_core::*;

// Re-export the derive macro
pub use baseclaw_macros::Tool;
