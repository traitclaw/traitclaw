//! # TraitClaw Strategies
//!
//! Built-in reasoning strategies for the TraitClaw AI agent framework.
//!
//! This crate provides three reasoning strategies that implement the
//! [`AgentStrategy`](traitclaw_core::AgentStrategy) trait:
//!
//! - **ReAct** (`react` feature): Think→Act→Observe reasoning loops with tool use
//! - **Chain-of-Thought** (`cot` feature): Structured step-by-step reasoning
//! - **MCTS** (`mcts` feature): Monte Carlo Tree Search with parallel branch evaluation
//!
//! All strategies are enabled by default. Use `default-features = false` to
//! selectively enable only the strategies you need.
//!
//! # Feature Flags
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `react` | ReAct reasoning strategy (Think→Act→Observe) |
//! | `cot`   | Chain-of-Thought reasoning strategy |
//! | `mcts`  | Monte Carlo Tree Search strategy |

#![deny(missing_docs)]
#![allow(clippy::redundant_closure)]

pub mod common;
pub mod streaming;

#[cfg(feature = "react")]
pub mod react;

#[cfg(feature = "mcts")]
pub mod mcts;

#[cfg(feature = "cot")]
pub mod cot;

// Top-level re-exports for convenience
pub use common::ThoughtStep;
pub use streaming::StrategyEvent;

#[cfg(feature = "react")]
pub use react::ReActStrategy;

#[cfg(feature = "cot")]
pub use cot::ChainOfThoughtStrategy;

#[cfg(feature = "mcts")]
pub use mcts::MctsStrategy;
