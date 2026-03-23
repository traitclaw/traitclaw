//! Agent runtime state for the steering system.

use crate::traits::provider::ModelTier;

/// Snapshot of the agent's runtime state.
///
/// Used by Guard, Hint, and Tracker to make steering decisions.
#[derive(Debug, Clone)]
pub struct AgentState {
    /// Current iteration count in the agent loop.
    pub iteration_count: usize,
    /// Total tokens used so far.
    pub token_usage: usize,
    /// Maximum token budget for this run.
    pub token_budget: usize,
    /// Total tokens in the current context.
    pub total_context_tokens: usize,
    /// Model's maximum context window size.
    pub context_window_size: usize,
    /// Whether the last output was truncated.
    pub last_output_truncated: bool,
    /// Whether this is a team/multi-agent task.
    pub is_team_task: bool,
    /// The capability tier of the current model.
    pub model_tier: ModelTier,
}

impl AgentState {
    /// Create a new agent state with defaults.
    #[must_use]
    pub fn new(model_tier: ModelTier, context_window: usize) -> Self {
        Self {
            iteration_count: 0,
            token_usage: 0,
            token_budget: usize::MAX,
            total_context_tokens: 0,
            context_window_size: context_window,
            last_output_truncated: false,
            is_team_task: false,
            model_tier,
        }
    }

    /// Context utilization as a fraction (0.0 - 1.0).
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn context_utilization(&self) -> f32 {
        if self.context_window_size == 0 {
            return 0.0;
        }
        self.total_context_tokens as f32 / self.context_window_size as f32
    }

    /// Budget utilization as a fraction (0.0 - 1.0).
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn budget_utilization(&self) -> f32 {
        if self.token_budget == 0 || self.token_budget == usize::MAX {
            return 0.0;
        }
        self.token_usage as f32 / self.token_budget as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_state_defaults() {
        let state = AgentState::new(ModelTier::Medium, 128_000);
        assert_eq!(state.iteration_count, 0);
        assert_eq!(state.context_window_size, 128_000);
    }

    #[test]
    fn test_context_utilization() {
        let mut state = AgentState::new(ModelTier::Medium, 100);
        state.total_context_tokens = 60;
        assert!((state.context_utilization() - 0.6).abs() < f32::EPSILON);
    }

    #[test]
    fn test_budget_utilization_no_budget() {
        let state = AgentState::new(ModelTier::Medium, 100);
        assert!((state.budget_utilization() - 0.0).abs() < f32::EPSILON);
    }
}
