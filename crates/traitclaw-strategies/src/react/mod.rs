//! ReAct reasoning strategy (Think→Act→Observe loops).
//!
//! The ReAct strategy implements an autonomous reasoning loop where the agent:
//! 1. **Thinks** about the problem
//! 2. **Acts** by calling tools
//! 3. **Observes** tool outputs
//! 4. Repeats until it has enough information to produce an **Answer**
//!
//! # Example
//!
//! ```no_run
//! use traitclaw_strategies::react::ReActStrategy;
//!
//! let strategy = ReActStrategy::builder()
//!     .max_iterations(10)
//!     .build()
//!     .unwrap();
//! ```

mod strategy;

pub use strategy::ReActStrategy;
pub use strategy::ReActStrategyBuilder;
