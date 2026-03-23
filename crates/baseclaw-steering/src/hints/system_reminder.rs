//! `SystemPromptReminder` — re-injects key instructions periodically.

use baseclaw_core::traits::hint::{Hint, HintMessage, HintPriority, InjectionPoint};
use baseclaw_core::types::agent_state::AgentState;
use baseclaw_core::types::message::MessageRole;

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
