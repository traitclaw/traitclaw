//! `BudgetHint` — warns the agent as it approaches the token limit.

use baseclaw_core::traits::hint::{Hint, HintMessage, HintPriority, InjectionPoint};
use baseclaw_core::types::agent_state::AgentState;
use baseclaw_core::types::message::MessageRole;

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
