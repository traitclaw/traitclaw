//! `Steering` facade — one-line auto-configuration of Guard-Hint-Track.

use traitclaw_core::traits::guard::Guard;
use traitclaw_core::traits::hint::Hint;
use traitclaw_core::traits::provider::ModelTier;
use traitclaw_core::traits::tracker::Tracker;

use crate::guards::{LoopDetectionGuard, ShellDenyGuard, ToolBudgetGuard};
use crate::hints::{BudgetHint, SystemPromptReminder};
use crate::trackers::AdaptiveTracker;

/// Pre-configured steering based on model tier.
///
/// The recommended entry point: `Steering::auto()` or `Steering::for_tier(tier)`.
///
/// # Example
///
/// ```rust,no_run
/// use traitclaw_steering::Steering;
/// use traitclaw_core::traits::provider::ModelTier;
///
/// let s = Steering::for_tier(ModelTier::Medium);
/// assert_eq!(s.guard_count(), 3);
/// assert_eq!(s.hint_count(), 2);
/// assert!(s.has_tracker());
/// ```
pub struct Steering {
    tier: Option<ModelTier>,
    guards: Vec<Box<dyn Guard>>,
    hints: Vec<Box<dyn Hint>>,
    tracker: Option<Box<dyn Tracker>>,
}

impl Steering {
    /// Auto-configure steering — resolved at build-time from the provider's model tier.
    ///
    /// Returns a marker; call `resolve(tier)` when the tier is known.
    #[must_use]
    pub fn auto() -> Self {
        Self {
            tier: None,
            guards: vec![],
            hints: vec![],
            tracker: None,
        }
    }

    /// Explicitly configure steering for a specific model tier.
    #[must_use]
    pub fn for_tier(tier: ModelTier) -> Self {
        let mut s = Self::auto();
        s.apply_tier(tier);
        s.tier = Some(tier);
        s
    }

    /// Start with an empty configuration for manual guard/hint/tracker setup.
    #[must_use]
    pub fn custom() -> Self {
        Self {
            tier: None,
            guards: vec![],
            hints: vec![],
            tracker: None,
        }
    }

    /// Resolve auto-configuration using the given model tier.
    ///
    /// If this `Steering` was created with `for_tier()` or `custom()`, this is a no-op.
    pub fn resolve(&mut self, tier: ModelTier) {
        if self.tier.is_none() && self.guards.is_empty() {
            self.apply_tier(tier);
            self.tier = Some(tier);
        }
    }

    /// Add a custom guard.
    #[must_use]
    pub fn guard(mut self, guard: impl Guard + 'static) -> Self {
        self.guards.push(Box::new(guard));
        self
    }

    /// Add a custom hint.
    #[must_use]
    pub fn hint(mut self, hint: impl Hint + 'static) -> Self {
        self.hints.push(Box::new(hint));
        self
    }

    /// Set a custom tracker.
    #[must_use]
    pub fn tracker(mut self, tracker: impl Tracker + 'static) -> Self {
        self.tracker = Some(Box::new(tracker));
        self
    }

    /// Consume and return the configured guards.
    #[must_use]
    pub fn guards(self) -> Vec<Box<dyn Guard>> {
        self.guards
    }

    /// Consume and return the configured hints.
    #[must_use]
    pub fn into_hints(self) -> Vec<Box<dyn Hint>> {
        self.hints
    }

    /// Consume and return the configured tracker.
    #[must_use]
    pub fn into_tracker(self) -> Option<Box<dyn Tracker>> {
        self.tracker
    }

    /// Get the number of configured guards.
    #[must_use]
    pub fn guard_count(&self) -> usize {
        self.guards.len()
    }

    /// Get the number of configured hints.
    #[must_use]
    pub fn hint_count(&self) -> usize {
        self.hints.len()
    }

    /// Whether a tracker is configured.
    #[must_use]
    pub fn has_tracker(&self) -> bool {
        self.tracker.is_some()
    }

    fn apply_tier(&mut self, tier: ModelTier) {
        match tier {
            // Small: aggressive protection
            ModelTier::Small => {
                self.guards.push(Box::new(ShellDenyGuard::default()));
                self.guards.push(Box::new(LoopDetectionGuard::new(3, 20)));
                self.guards.push(Box::new(ToolBudgetGuard::new(50)));
                self.hints.push(Box::new(BudgetHint::at(0.50)));
                self.hints.push(Box::new(SystemPromptReminder::every(4)));
                self.tracker = Some(Box::new(AdaptiveTracker::for_tier(&tier)));
            }
            // Large: relaxed
            ModelTier::Large => {
                self.guards.push(Box::new(ShellDenyGuard::default()));
                self.guards.push(Box::new(LoopDetectionGuard::new(5, 30)));
                self.guards.push(Box::new(ToolBudgetGuard::new(100)));
                self.hints.push(Box::new(BudgetHint::at(0.80)));
                self.hints.push(Box::new(SystemPromptReminder::every(15)));
                self.tracker = Some(Box::new(AdaptiveTracker::for_tier(&tier)));
            }
            // Medium (default): balanced
            _ => {
                self.guards.push(Box::new(ShellDenyGuard::default()));
                self.guards.push(Box::new(LoopDetectionGuard::new(3, 20)));
                self.guards.push(Box::new(ToolBudgetGuard::new(50)));
                self.hints.push(Box::new(BudgetHint::at(0.75)));
                self.hints.push(Box::new(SystemPromptReminder::every(8)));
                self.tracker = Some(Box::new(AdaptiveTracker::for_tier(&tier)));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_tier_aggressive_ac2() {
        let s = Steering::for_tier(ModelTier::Small);
        // AC-2: ShellDeny + Loop(3) + Budget(50) = 3 guards
        assert_eq!(s.guard_count(), 3);
        // AC-2: BudgetHint(0.5) + Reminder(4) = 2 hints
        assert_eq!(s.hint_count(), 2);
        assert!(s.has_tracker());
    }

    #[test]
    fn test_medium_tier_balanced_ac3() {
        let s = Steering::for_tier(ModelTier::Medium);
        assert_eq!(s.guard_count(), 3);
        assert_eq!(s.hint_count(), 2);
        assert!(s.has_tracker());
    }

    #[test]
    fn test_large_tier_relaxed_ac4() {
        let s = Steering::for_tier(ModelTier::Large);
        // AC-4: ShellDeny + Loop(5) + Budget(100) = 3 guards
        assert_eq!(s.guard_count(), 3);
        // AC-4: BudgetHint(0.80) + Reminder(15) = 2 hints
        assert_eq!(s.hint_count(), 2);
        assert!(s.has_tracker());
    }

    #[test]
    fn test_auto_empty_before_resolve() {
        let s = Steering::auto();
        assert_eq!(s.guard_count(), 0);
        assert_eq!(s.hint_count(), 0);
        assert!(!s.has_tracker());
    }

    #[test]
    fn test_auto_resolve_ac1() {
        let mut s = Steering::auto();
        s.resolve(ModelTier::Medium);
        assert_eq!(s.guard_count(), 3);
        assert_eq!(s.hint_count(), 2);
        assert!(s.has_tracker());
    }

    #[test]
    fn test_for_tier_ac5() {
        let s = Steering::for_tier(ModelTier::Large);
        assert_eq!(s.guard_count(), 3);
    }

    #[test]
    fn test_custom_starts_empty_ac6() {
        let s = Steering::custom();
        assert_eq!(s.guard_count(), 0);
        assert_eq!(s.hint_count(), 0);
        assert!(!s.has_tracker());
    }

    #[test]
    fn test_custom_manual_config() {
        let s = Steering::custom()
            .guard(ShellDenyGuard::default())
            .hint(BudgetHint::at(0.9));
        assert_eq!(s.guard_count(), 1);
        assert_eq!(s.hint_count(), 1);
        assert!(!s.has_tracker());
    }
}
