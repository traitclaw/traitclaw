//! Steering system for the `BaseClaw` AI agent framework.
//!
//! Provides built-in implementations of the Guard-Hint-Track triad
//! to keep agents safe, focused, and efficient without relying on prompts.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use baseclaw_steering::guards::ShellDenyGuard;
//! use baseclaw_steering::hints::BudgetHint;
//! use baseclaw_steering::trackers::AdaptiveTracker;
//! use baseclaw_core::traits::provider::ModelTier;
//!
//! let shell_guard = ShellDenyGuard::default();
//! let budget_hint = BudgetHint::at(0.75);
//! let tracker = AdaptiveTracker::for_tier(&ModelTier::Medium);
//! ```

#![deny(warnings)]
#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod guards;
pub mod hints;
pub mod trackers;

/// Convenience prelude — import everything at once.
pub mod prelude {
    pub use crate::guards::{
        LoopDetectionGuard, PromptInjectionGuard, ShellDenyGuard, ToolBudgetGuard,
        WorkspaceBoundaryGuard,
    };
    pub use crate::hints::{BudgetHint, SystemPromptReminder, TeamProgressHint, TruncationHint};
    pub use crate::trackers::AdaptiveTracker;
}
