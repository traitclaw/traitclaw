//! `BudgetHint` — warns the agent as it approaches the token limit.

use traitclaw_core::traits::hint::{Hint, HintMessage, HintPriority, InjectionPoint};
use traitclaw_core::types::agent_state::AgentState;
use traitclaw_core::types::message::MessageRole;

/// A [`Hint`] that fires when token utilization crosses a threshold.
pub struct BudgetHint {
    threshold: f32,
}

impl BudgetHint {
    /// Create a hint that fires when utilization reaches `threshold` (0.0–1.0).
    #[must_use]
    pub fn at(threshold: f32) -> Self {
        Self { threshold }
    }
}

impl Default for BudgetHint {
    fn default() -> Self {
        Self::at(0.75)
    }
}

impl Hint for BudgetHint {
    fn name(&self) -> &'static str {
        "budget_hint"
    }

    fn should_trigger(&self, state: &AgentState) -> bool {
        state.context_utilization() >= self.threshold
    }

    fn generate(&self, state: &AgentState) -> HintMessage {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let pct = (state.context_utilization() * 100.0).max(0.0).round() as u32;
        HintMessage {
            role: MessageRole::System,
            content: format!(
                "Budget Warning: You have used approximately {pct}% of your context window. \
                 Begin wrapping up: prioritize completing the most important tasks, \
                 avoid starting new large operations, and prepare a concise summary for the user."
            ),
            priority: HintPriority::Critical,
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

    fn state_with_utilization(total_ctx: usize, window: usize) -> AgentState {
        let mut s = AgentState::new(ModelTier::Small, window);
        s.total_context_tokens = total_ctx;
        s
    }

    #[test]
    fn test_triggers_at_threshold() {
        let hint = BudgetHint::at(0.75);
        let state = state_with_utilization(800, 1000);
        assert!(hint.should_trigger(&state));
    }

    #[test]
    fn test_does_not_trigger_below_threshold() {
        let hint = BudgetHint::at(0.75);
        let state = state_with_utilization(500, 1000);
        assert!(!hint.should_trigger(&state));
    }

    #[test]
    fn test_message_contains_percentage() {
        let hint = BudgetHint::at(0.75);
        let state = state_with_utilization(800, 1000);
        let msg = hint.generate(&state);
        assert!(msg.content.contains("80%"));
    }
}
