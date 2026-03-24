//! `ToolBudgetGuard` — limits total tool calls per agent run.

use std::sync::atomic::{AtomicU32, Ordering};
use traitclaw_core::traits::guard::{Guard, GuardResult, GuardSeverity};
use traitclaw_core::types::action::Action;

/// A [`Guard`] that hard-caps the total number of tool calls in a run.
pub struct ToolBudgetGuard {
    max_calls: u32,
    calls_made: AtomicU32,
}

impl ToolBudgetGuard {
    /// Create a `ToolBudgetGuard` with a given maximum.
    #[must_use]
    pub fn new(max_calls: u32) -> Self {
        Self {
            max_calls,
            calls_made: AtomicU32::new(0),
        }
    }

    /// Reset the call counter (e.g. between agent runs).
    pub fn reset(&self) {
        self.calls_made.store(0, Ordering::Relaxed);
    }
}

impl Default for ToolBudgetGuard {
    fn default() -> Self {
        Self::new(50)
    }
}

impl Guard for ToolBudgetGuard {
    fn name(&self) -> &'static str {
        "tool_budget"
    }

    fn check(&self, action: &Action) -> GuardResult {
        let Action::ToolCall { name, .. } = action else {
            return GuardResult::Allow;
        };

        let used = self.calls_made.fetch_add(1, Ordering::Relaxed);

        if used >= self.max_calls {
            GuardResult::Deny {
                reason: format!(
                    "ToolBudgetGuard: budget exhausted ({}/{} used). \
                     Tool '{name}' blocked. Please summarize progress and respond to the user.",
                    used, self.max_calls
                ),
                severity: GuardSeverity::Critical,
            }
        } else {
            GuardResult::Allow
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tool_action() -> Action {
        Action::ToolCall {
            name: "test".into(),
            arguments: serde_json::Value::Null,
        }
    }

    #[test]
    fn test_allows_within_budget() {
        let guard = ToolBudgetGuard::new(3);
        assert!(matches!(guard.check(&tool_action()), GuardResult::Allow));
        assert!(matches!(guard.check(&tool_action()), GuardResult::Allow));
        assert!(matches!(guard.check(&tool_action()), GuardResult::Allow));
    }

    #[test]
    fn test_blocks_after_budget_exhausted() {
        let guard = ToolBudgetGuard::new(2);
        assert!(matches!(guard.check(&tool_action()), GuardResult::Allow));
        assert!(matches!(guard.check(&tool_action()), GuardResult::Allow));
        assert!(matches!(
            guard.check(&tool_action()),
            GuardResult::Deny { .. }
        ));
    }

    #[test]
    fn test_reset_restores_budget() {
        let guard = ToolBudgetGuard::new(1);
        assert!(matches!(guard.check(&tool_action()), GuardResult::Allow));
        assert!(matches!(
            guard.check(&tool_action()),
            GuardResult::Deny { .. }
        ));
        guard.reset();
        assert!(matches!(guard.check(&tool_action()), GuardResult::Allow));
    }
}
