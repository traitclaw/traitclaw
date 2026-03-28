//! Chain-of-Thought reasoning strategy.
//!
//! The CoT strategy instructs the LLM to reason step-by-step before
//! producing a final answer, making the reasoning process transparent
//! and auditable.
//!
//! # Example
//!
//! ```no_run
//! use traitclaw_strategies::cot::ChainOfThoughtStrategy;
//!
//! let strategy = ChainOfThoughtStrategy::builder()
//!     .max_steps(5)
//!     .build()
//!     .unwrap();
//! ```

mod strategy;

pub use strategy::ChainOfThoughtStrategy;
pub use strategy::ChainOfThoughtStrategyBuilder;
