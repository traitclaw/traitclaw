//! `TruncationHint` and `TeamProgressHint`.

use traitclaw_core::traits::hint::{Hint, HintMessage, HintPriority, InjectionPoint};
use traitclaw_core::types::agent_state::AgentState;
use traitclaw_core::types::message::MessageRole;

/// A [`Hint`] that fires when context truncation has occurred.
pub struct TruncationHint;

impl Hint for TruncationHint {
    fn name(&self) -> &'static str {
        "truncation_hint"
    }

    fn should_trigger(&self, state: &AgentState) -> bool {
        state.last_output_truncated
    }

    fn generate(&self, _state: &AgentState) -> HintMessage {
        HintMessage {
            role: MessageRole::System,
            content: "Context Note: Some earlier messages were removed to fit the context window. \
                      If you need information from earlier in the conversation, ask the user to restate it. \
                      Do not assume any missing context.".to_string(),
            priority: HintPriority::Normal,
        }
    }

    fn injection_point(&self) -> InjectionPoint {
        InjectionPoint::BeforeNextLlmCall
    }
}

/// A [`Hint`] that reminds the agent to report progress every N iterations.
pub struct TeamProgressHint {
    every_n: u32,
}

impl TeamProgressHint {
    /// Create a hint that fires every `n` iterations.
    #[must_use]
    pub fn every(n: u32) -> Self {
        Self { every_n: n }
    }
}

impl Default for TeamProgressHint {
    fn default() -> Self {
        Self::every(5)
    }
}

impl Hint for TeamProgressHint {
    fn name(&self) -> &'static str {
        "team_progress_hint"
    }

    fn should_trigger(&self, state: &AgentState) -> bool {
        state.iteration_count > 0 && state.iteration_count % self.every_n as usize == 0
    }

    fn generate(&self, state: &AgentState) -> HintMessage {
        HintMessage {
            role: MessageRole::System,
            content: format!(
                "Progress Check (iteration {}): Briefly summarize what you have completed \
                 so far and what remains. Keep it to 2-3 sentences.",
                state.iteration_count
            ),
            priority: HintPriority::Low,
        }
    }

    fn injection_point(&self) -> InjectionPoint {
        InjectionPoint::RecencyZone
    }
}
