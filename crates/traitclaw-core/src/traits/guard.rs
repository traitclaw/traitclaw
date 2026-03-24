//! Guard trait for hard boundary enforcement.
//!
//! Guards check actions **before** they execute and can block dangerous behavior.
//! Guard checks must be fast (microseconds) — they run on every action.

use crate::types::action::Action;

/// Result of a guard check.
#[derive(Debug, Clone)]
pub enum GuardResult {
    /// Action is allowed to proceed.
    Allow,
    /// Action is blocked. The agent will receive the reason as feedback.
    Deny {
        /// Why this action was blocked.
        reason: String,
        /// How severe this violation is.
        severity: GuardSeverity,
    },
    /// Action is modified (sanitized) before proceeding.
    Sanitize {
        /// The sanitized action to use instead.
        modified_action: Action,
        /// Warning message about the sanitization.
        warning: String,
    },
}

/// Severity level of a guard violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardSeverity {
    /// Critical — log and block immediately.
    Critical,
    /// High — block and alert developer.
    High,
    /// Medium — block and suggest alternative.
    Medium,
}

/// Hard boundary check for agent actions.
///
/// Guards run **before** every action (tool call, shell command, file write, etc.)
/// and can block or sanitize dangerous behavior. They are the first layer of defense
/// in the Guard-Hint-Track steering system.
///
/// Guard checks must NOT be async — they must complete in microseconds.
pub trait Guard: Send + Sync + 'static {
    /// The name of this guard (for logging and tracing).
    fn name(&self) -> &'static str;

    /// Check whether an action should be allowed.
    ///
    /// This method is called before every agent action. Return [`GuardResult::Allow`]
    /// to let the action proceed, or [`GuardResult::Deny`] to block it.
    fn check(&self, action: &Action) -> GuardResult;
}

/// No-op guard that allows everything. Used when no guards are configured.
pub struct NoopGuard;

impl Guard for NoopGuard {
    fn name(&self) -> &'static str {
        "noop"
    }

    fn check(&self, _action: &Action) -> GuardResult {
        GuardResult::Allow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noop_guard_allows_everything() {
        let guard = NoopGuard;
        let action = Action::RawOutput {
            content: "test".into(),
        };
        assert!(matches!(guard.check(&action), GuardResult::Allow));
    }
}
