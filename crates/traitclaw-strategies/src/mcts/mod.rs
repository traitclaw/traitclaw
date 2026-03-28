//! Monte Carlo Tree Search reasoning strategy.
//!
//! The MCTS strategy explores multiple reasoning paths in parallel and
//! selects the highest-scoring result using a configurable scoring function.
//!
//! # Example
//!
//! ```no_run
//! use traitclaw_strategies::mcts::MctsStrategy;
//!
//! let strategy = MctsStrategy::builder()
//!     .branches(5)
//!     .max_depth(3)
//!     .build()
//!     .unwrap();
//! ```

mod scoring;
mod strategy;

pub use scoring::ScoringFn;
pub use strategy::MctsStrategy;
pub use strategy::MctsStrategyBuilder;
