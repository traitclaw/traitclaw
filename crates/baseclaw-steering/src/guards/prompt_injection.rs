//! `PromptInjectionGuard` — detects prompt injection in tool arguments.
//!
//! Checks tool arguments for patterns that try to override system instructions.

use baseclaw_core::traits::guard::{Guard, GuardResult, GuardSeverity};
use baseclaw_core::types::action::Action;

const INJECTION_PATTERNS: &[&str] = &[
    "(?i)ignore\\s+(all\\s+)?(previous|prior|above|earlier)\\s+(instructions?|prompts?|rules?)",
    "(?i)forget\\s+(everything|all)\\s+(you|above)",
    "(?i)you\\s+are\\s+now\\s+(DAN|jailbroken|unrestricted|free)",
    "(?i)(disregard|override|bypass)\\s+(your\\s+)?(safety|content)\\s+(instructions?|guidelines?)",
    "(?i)you\\s+(must|should|will)\\s+now\\s+(ignore|disregard|forget)\\b",
    "(?i)\\[system\\]\\s*:",
    "(?i)<system>",
    "(?i)###\\s*system\\s*###",
    "(?i)print\\s+(your\\s+)?(system\\s+)?(prompt|instructions?)\\b",
    "(?i)reveal\\s+(your\\s+)?(hidden|internal|secret)\\s+(instructions?|prompt)\\b",
    "(?i)special\\s+developer\\s+mode",
    "(?i)sudo\\s+(mode|override|admin)",
];

/// A [`Guard`] that blocks likely prompt injection in tool arguments and shell commands.
pub struct PromptInjectionGuard {
    patterns: Vec<regex::Regex>,
}

impl PromptInjectionGuard {
    /// Create a `PromptInjectionGuard` with the default patterns.
    ///
    /// # Panics
    ///
    /// Panics if any built-in regex is invalid (should never happen).
    #[must_use]
    pub fn new() -> Self {
        let patterns = INJECTION_PATTERNS
            .iter()
            .map(|p| regex::Regex::new(p).expect("Built-in pattern must be valid"))
            .collect();
        Self { patterns }
    }

    fn matches_any(&self, text: &str) -> bool {
        self.patterns.iter().any(|re| re.is_match(text))
    }
}

impl Default for PromptInjectionGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Guard for PromptInjectionGuard {
    fn name(&self) -> &'static str {
        "prompt_injection"
    }

    fn check(&self, action: &Action) -> GuardResult {
        let text = match action {
            Action::ToolCall { arguments, .. } => arguments.to_string(),
            Action::ShellCommand { command } => command.clone(),
            Action::FileWrite { content, .. } => content.clone(),
            _ => return GuardResult::Allow,
        };

        if self.matches_any(&text) {
            GuardResult::Deny {
                reason: "PromptInjectionGuard: possible prompt injection detected in action"
                    .to_string(),
                severity: GuardSeverity::Critical,
            }
        } else {
            GuardResult::Allow
        }
    }
}
