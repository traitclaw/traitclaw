//! `LoopDetectionGuard` — detects repetitive identical tool calls.

use traitclaw_core::traits::guard::{Guard, GuardResult, GuardSeverity};
use traitclaw_core::types::action::Action;
use std::collections::VecDeque;
use std::sync::Mutex;

/// A [`Guard`] that blocks repetitive identical tool calls.
pub struct LoopDetectionGuard {
    max_repeats: usize,
    window: usize,
    history: Mutex<VecDeque<(String, String)>>,
}

impl LoopDetectionGuard {
    /// Create a new `LoopDetectionGuard`.
    ///
    /// - `max_repeats` — max times the same `(tool, args)` can appear before blocking
    /// - `window` — number of recent calls to track (sliding window)
    #[must_use]
    pub fn new(max_repeats: usize, window: usize) -> Self {
        Self {
            max_repeats,
            window,
            history: Mutex::new(VecDeque::with_capacity(window)),
        }
    }
}

impl Default for LoopDetectionGuard {
    fn default() -> Self {
        Self::new(3, 20)
    }
}

impl Guard for LoopDetectionGuard {
    fn name(&self) -> &'static str {
        "loop_detection"
    }

    fn check(&self, action: &Action) -> GuardResult {
        let Action::ToolCall { name, arguments } = action else {
            return GuardResult::Allow;
        };

        let key = (name.clone(), arguments.to_string());
        let mut history = self.history.lock().expect("lock poisoned");

        let count = history.iter().filter(|k| *k == &key).count();

        if count >= self.max_repeats {
            return GuardResult::Deny {
                reason: format!(
                    "LoopDetectionGuard: tool '{name}' called with identical args {} times (max {})",
                    count + 1,
                    self.max_repeats
                ),
                severity: GuardSeverity::Medium,
            };
        }

        if history.len() >= self.window {
            history.pop_front();
        }
        history.push_back(key);

        GuardResult::Allow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tool_call(name: &str, args: &str) -> Action {
        Action::ToolCall {
            name: name.into(),
            arguments: serde_json::Value::String(args.into()),
        }
    }

    #[test]
    fn test_allows_first_calls() {
        let guard = LoopDetectionGuard::new(3, 10);
        let action = tool_call("echo", "hello");
        assert!(matches!(guard.check(&action), GuardResult::Allow));
        assert!(matches!(guard.check(&action), GuardResult::Allow));
        assert!(matches!(guard.check(&action), GuardResult::Allow));
    }

    #[test]
    fn test_blocks_after_max_repeats() {
        let guard = LoopDetectionGuard::new(2, 10);
        let action = tool_call("echo", "hello");
        assert!(matches!(guard.check(&action), GuardResult::Allow));
        assert!(matches!(guard.check(&action), GuardResult::Allow));
        assert!(matches!(guard.check(&action), GuardResult::Deny { .. }));
    }

    #[test]
    fn test_different_args_not_a_loop() {
        let guard = LoopDetectionGuard::new(2, 10);
        assert!(matches!(
            guard.check(&tool_call("echo", "a")),
            GuardResult::Allow
        ));
        assert!(matches!(
            guard.check(&tool_call("echo", "b")),
            GuardResult::Allow
        ));
        assert!(matches!(
            guard.check(&tool_call("echo", "c")),
            GuardResult::Allow
        ));
    }

    #[test]
    fn test_allows_non_tool_actions() {
        let guard = LoopDetectionGuard::new(1, 10);
        let action = Action::RawOutput {
            content: "test".into(),
        };
        assert!(matches!(guard.check(&action), GuardResult::Allow));
    }
}
