//! `SystemPromptReminder` — re-injects key instructions periodically.

use traitclaw_core::traits::hint::{Hint, HintMessage, HintPriority, InjectionPoint};
use traitclaw_core::types::agent_state::AgentState;
use traitclaw_core::types::message::MessageRole;

/// A [`Hint`] that re-injects key rules every N iterations.
pub struct SystemPromptReminder {
    every_n: u32,
    rules: Vec<String>,
}

impl SystemPromptReminder {
    /// Create a reminder that fires every `n` iterations.
    #[must_use]
    pub fn every(n: u32) -> Self {
        Self {
            every_n: n,
            rules: Vec::new(),
        }
    }

    /// Add rules to remind the model about.
    #[must_use]
    pub fn rules(mut self, rules: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.rules = rules.into_iter().map(Into::into).collect();
        self
    }
}

impl Hint for SystemPromptReminder {
    fn name(&self) -> &'static str {
        "system_prompt_reminder"
    }

    fn should_trigger(&self, state: &AgentState) -> bool {
        state.iteration_count > 0 && state.iteration_count % self.every_n as usize == 0
    }

    fn generate(&self, _state: &AgentState) -> HintMessage {
        let rules_str = if self.rules.is_empty() {
            "Follow your original system instructions.".to_string()
        } else {
            self.rules
                .iter()
                .enumerate()
                .map(|(i, r)| format!("{}. {r}", i + 1))
                .collect::<Vec<_>>()
                .join("\n")
        };

        HintMessage {
            role: MessageRole::System,
            content: format!("Reminder - key rules:\n{rules_str}"),
            priority: HintPriority::Normal,
        }
    }

    fn injection_point(&self) -> InjectionPoint {
        InjectionPoint::RecencyZone
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use traitclaw_core::types::model_info::ModelTier;

    #[test]
    fn test_triggers_at_correct_interval() {
        let hint = SystemPromptReminder::every(5).rules(["Be concise"]);
        let mut s = AgentState::new(ModelTier::Small, 4096);
        s.iteration_count = 5;
        assert!(hint.should_trigger(&s));
    }

    #[test]
    fn test_does_not_trigger_off_interval() {
        let hint = SystemPromptReminder::every(5).rules(["Be concise"]);
        let mut s = AgentState::new(ModelTier::Small, 4096);
        s.iteration_count = 3;
        assert!(!hint.should_trigger(&s));
    }

    #[test]
    fn test_never_triggers_at_zero() {
        let hint = SystemPromptReminder::every(5);
        let s = AgentState::new(ModelTier::Small, 4096);
        assert!(!hint.should_trigger(&s));
    }

    #[test]
    fn test_message_includes_rules() {
        let hint = SystemPromptReminder::every(5).rules(["Be concise", "No profanity"]);
        let s = AgentState::new(ModelTier::Small, 4096);
        let msg = hint.generate(&s);
        assert!(msg.content.contains("Be concise"));
        assert!(msg.content.contains("No profanity"));
    }
}
