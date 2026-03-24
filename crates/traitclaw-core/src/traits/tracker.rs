//! Tracker trait for runtime state monitoring.
//!
//! Trackers silently follow the model, monitoring runtime state and feeding
//! signals to Guard and Hint systems. Part of the Guard-Hint-Track steering system.

use serde_json::Value;

use crate::types::agent_state::AgentState;
use crate::types::completion::CompletionResponse;

/// Runtime state monitor for the steering system.
///
/// Trackers observe agent behavior without interfering. They update
/// [`AgentState`] which Guards and Hints use for their decisions.
pub trait Tracker: Send + Sync + 'static {
    /// Called at the start of each iteration.
    fn on_iteration(&self, state: &mut AgentState);

    /// Called after a tool is invoked.
    fn on_tool_call(&self, name: &str, args: &Value, state: &mut AgentState);

    /// Called after the LLM responds.
    fn on_llm_response(&self, response: &CompletionResponse, state: &mut AgentState);

    /// Recommended concurrency level based on current state.
    /// Returns the max number of concurrent operations.
    fn recommended_concurrency(&self, state: &AgentState) -> usize;
}

/// No-op tracker that does nothing. Used when no tracker is configured.
pub struct NoopTracker;

impl Tracker for NoopTracker {
    fn on_iteration(&self, _state: &mut AgentState) {}

    fn on_tool_call(&self, _name: &str, _args: &Value, _state: &mut AgentState) {}

    fn on_llm_response(&self, _response: &CompletionResponse, _state: &mut AgentState) {}

    fn recommended_concurrency(&self, _state: &AgentState) -> usize {
        usize::MAX
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::agent_state::AgentState;
    use crate::types::model_info::ModelTier;

    #[test]
    fn test_noop_tracker_recommended_concurrency() {
        let tracker = NoopTracker;
        let state = AgentState::new(ModelTier::Small, 4096);
        assert_eq!(tracker.recommended_concurrency(&state), usize::MAX);
    }

    #[test]
    fn test_noop_tracker_callbacks_do_nothing() {
        let tracker = NoopTracker;
        let mut state = AgentState::new(ModelTier::Small, 4096);
        let initial_iter = state.iteration_count;

        // These should be genuine no-ops
        tracker.on_iteration(&mut state);
        tracker.on_tool_call("test", &serde_json::json!({}), &mut state);

        // State should NOT be modified by NoopTracker
        assert_eq!(state.iteration_count, initial_iter);
    }
}
