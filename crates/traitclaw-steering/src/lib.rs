//! Steering system for the `TraitClaw` AI agent framework.
//!
//! Provides built-in implementations of the Guard-Hint-Track triad
//! to keep agents safe, focused, and efficient without relying on prompts.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use traitclaw_steering::guards::ShellDenyGuard;
//! use traitclaw_steering::hints::BudgetHint;
//! use traitclaw_steering::trackers::AdaptiveTracker;
//! use traitclaw_core::traits::provider::ModelTier;
//!
//! let shell_guard = ShellDenyGuard::default();
//! let budget_hint = BudgetHint::at(0.75);
//! let tracker = AdaptiveTracker::for_tier(&ModelTier::Medium);
//! ```

#![deny(missing_docs)]
#![allow(clippy::redundant_closure)]

pub mod guards;
pub mod hints;
pub mod steering;
pub mod trackers;

pub use steering::Steering;

/// Convenience prelude — import everything at once.
pub mod prelude {
    pub use crate::guards::{
        ContentFilterGuard, LoopDetectionGuard, PromptInjectionGuard, RateLimitGuard,
        ShellDenyGuard, ToolBudgetGuard, WorkspaceBoundaryGuard,
    };
    pub use crate::hints::{BudgetHint, SystemPromptReminder, TeamProgressHint, TruncationHint};
    pub use crate::steering::Steering;
    pub use crate::trackers::AdaptiveTracker;
}
