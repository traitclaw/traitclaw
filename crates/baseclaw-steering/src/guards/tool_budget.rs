//! `ToolBudgetGuard` — limits total tool calls per agent run.

use baseclaw_core::traits::guard::{Guard, GuardResult, GuardSeverity};
use baseclaw_core::types::action::Action;
use std::sync::atomic::{AtomicU32, Ordering};

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
