//! Router — pluggable multi-agent message routing.
//!
//! The `Router` trait defines how messages flow between agents in a team.
//! Implement it to build custom workflows: sequential pipelines, leader-follower,
//! state machines, or graph-based orchestrations.
//!
//! # Architecture Decision
//!
//! The Router is a simple trait with no graph dependency. Complex routing
//! topologies are expressed through `RoutingDecision` variants rather than
//! requiring a DAG library. This keeps the core minimal while enabling
//! arbitrary routing logic.
//!
//! # Example
//!
//! ```rust
//! use traitclaw_team::router::*;
//!
//! let router = SequentialRouter::new();
//!
//! let mut state = TeamState::new(vec![
//!     "researcher".to_string(),
//!     "writer".to_string(),
//! ]);
//!
//! let msg = TeamMessage::new("user", "Write a report on AI");
//! let decision = router.route(&msg, &state);
//! assert!(matches!(decision, RoutingDecision::SendTo(ref id) if id == "researcher"));
//! ```

use serde::{Deserialize, Serialize};

/// A unique identifier for an agent within a team.
pub type AgentId = String;

/// A message exchanged between agents in a team.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMessage {
    /// The sender agent ID (or "user" for external input).
    pub sender: String,
    /// The message content.
    pub content: String,
    /// Optional metadata (e.g., routing hints, priority).
    pub metadata: serde_json::Value,
}

impl TeamMessage {
    /// Create a new team message.
    #[must_use]
    pub fn new(sender: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            sender: sender.into(),
            content: content.into(),
            metadata: serde_json::Value::Null,
        }
    }

    /// Create a message with metadata.
    #[must_use]
    pub fn with_metadata(
        sender: impl Into<String>,
        content: impl Into<String>,
        metadata: serde_json::Value,
    ) -> Self {
        Self {
            sender: sender.into(),
            content: content.into(),
            metadata,
        }
    }
}

/// The current state of the team orchestration.
#[derive(Debug, Clone)]
pub struct TeamState {
    /// Ordered list of agent IDs in this team.
    pub agents: Vec<AgentId>,
    /// Full message history.
    pub message_history: Vec<TeamMessage>,
    /// Current iteration/round counter.
    pub iteration: usize,
    /// Index of the current agent (for sequential routing).
    pub current_index: usize,
}

impl TeamState {
    /// Create a new team state with the given agent list.
    #[must_use]
    pub fn new(agents: Vec<AgentId>) -> Self {
        Self {
            agents,
            message_history: Vec::new(),
            iteration: 0,
            current_index: 0,
        }
    }

    /// Record a message in the history.
    pub fn record_message(&mut self, msg: TeamMessage) {
        self.message_history.push(msg);
    }

    /// Advance to the next iteration.
    pub fn next_iteration(&mut self) {
        self.iteration += 1;
    }
}

/// A routing decision made by a Router.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoutingDecision {
    /// Send the message to a specific agent.
    SendTo(AgentId),
    /// Broadcast the message to all agents.
    Broadcast,
    /// The orchestration is complete; return this final output.
    Complete(String),
}

/// Pluggable multi-agent message router.
///
/// Implement this trait to define custom routing logic for teams.
///
/// # Object Safety
///
/// This trait is object-safe and used as `Box<dyn Router>`.
pub trait Router: Send + Sync + 'static {
    /// Decide where to route a message given the current team state.
    fn route(&self, message: &TeamMessage, state: &TeamState) -> RoutingDecision;
}

/// Routes messages round-robin through all agents in order.
///
/// The first message goes to agents\[0\], the second to agents\[1\], etc.
/// Completes after all agents have responded once.
pub struct SequentialRouter;

impl SequentialRouter {
    /// Create a new sequential router.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for SequentialRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl Router for SequentialRouter {
    fn route(&self, _message: &TeamMessage, state: &TeamState) -> RoutingDecision {
        if state.current_index < state.agents.len() {
            RoutingDecision::SendTo(state.agents[state.current_index].clone())
        } else {
            // All agents have responded — use the last message as output
            let output = state
                .message_history
                .last()
                .map_or_else(|| String::new(), |m| m.content.clone());
            RoutingDecision::Complete(output)
        }
    }
}

/// Routes all messages through a designated leader agent.
///
/// The leader receives every message first, decides which specialist
/// handles each subtask, and synthesizes the final output.
pub struct LeaderRouter {
    leader_id: AgentId,
}

impl LeaderRouter {
    /// Create a new leader router with the given leader agent ID.
    #[must_use]
    pub fn new(leader_id: impl Into<AgentId>) -> Self {
        Self {
            leader_id: leader_id.into(),
        }
    }

    /// Get the leader agent ID.
    #[must_use]
    pub fn leader_id(&self) -> &str {
        &self.leader_id
    }
}

impl Router for LeaderRouter {
    fn route(&self, message: &TeamMessage, state: &TeamState) -> RoutingDecision {
        // If the leader hasn't seen this message yet, send to leader
        if message.sender != self.leader_id {
            return RoutingDecision::SendTo(self.leader_id.clone());
        }

        // Leader has responded — check if it's a delegation or completion
        // Convention: if leader's message starts with "@agent_name:" it's delegation
        // Otherwise, it's the final synthesized output
        if let Some(target) = message.content.strip_prefix('@') {
            let mut parts = target.splitn(2, ':');
            if let (Some(agent_id), Some(_)) = (parts.next(), parts.next()) {
                let agent_id = agent_id.trim();
                if state.agents.iter().any(|a| a == agent_id) {
                    return RoutingDecision::SendTo(agent_id.to_string());
                } else {
                    tracing::warn!("Leader delegated to unknown agent: {}", agent_id);
                    return RoutingDecision::Complete(format!(
                        "Error: Leader delegated to unknown agent '{}'",
                        agent_id
                    ));
                }
            }
        }

        RoutingDecision::Complete(message.content.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Verify trait is object-safe
    fn _assert_object_safe(_: &dyn Router) {}

    #[test]
    fn test_sequential_router_round_robin() {
        let router = SequentialRouter::new();
        let mut state = TeamState::new(vec!["agent_a".into(), "agent_b".into()]);

        let msg = TeamMessage::new("user", "Hello");

        // First: routes to agent_a
        let d1 = router.route(&msg, &state);
        assert_eq!(d1, RoutingDecision::SendTo("agent_a".into()));

        // Advance
        state.current_index = 1;
        let d2 = router.route(&msg, &state);
        assert_eq!(d2, RoutingDecision::SendTo("agent_b".into()));

        // All done
        state.current_index = 2;
        state
            .message_history
            .push(TeamMessage::new("agent_b", "Final output"));
        let d3 = router.route(&msg, &state);
        assert_eq!(d3, RoutingDecision::Complete("Final output".into()));
    }

    #[test]
    fn test_sequential_router_empty_complete() {
        let router = SequentialRouter::new();
        let state = TeamState::new(vec![]);

        let msg = TeamMessage::new("user", "Hello");
        let decision = router.route(&msg, &state);
        assert_eq!(decision, RoutingDecision::Complete(String::new()));
    }

    #[test]
    fn test_leader_router_delegates_to_leader() {
        let router = LeaderRouter::new("leader");
        let state = TeamState::new(vec!["leader".into(), "writer".into(), "coder".into()]);

        // User message → goes to leader
        let msg = TeamMessage::new("user", "Build me an app");
        assert_eq!(
            router.route(&msg, &state),
            RoutingDecision::SendTo("leader".into())
        );
    }

    #[test]
    fn test_leader_router_delegation_syntax() {
        let router = LeaderRouter::new("leader");
        let state = TeamState::new(vec!["leader".into(), "writer".into(), "coder".into()]);

        // Leader delegates with @agent: syntax
        let msg = TeamMessage::new("leader", "@writer: Write the docs");
        assert_eq!(
            router.route(&msg, &state),
            RoutingDecision::SendTo("writer".into())
        );
    }

    #[test]
    fn test_leader_router_final_output() {
        let router = LeaderRouter::new("leader");
        let state = TeamState::new(vec!["leader".into(), "writer".into()]);

        // Leader returns final output (no @ prefix)
        let msg = TeamMessage::new("leader", "Here is the final summary.");
        assert_eq!(
            router.route(&msg, &state),
            RoutingDecision::Complete("Here is the final summary.".into())
        );
    }

    #[test]
    fn test_team_message_with_metadata() {
        let msg =
            TeamMessage::with_metadata("user", "Hello", serde_json::json!({"priority": "high"}));
        assert_eq!(msg.sender, "user");
        assert_eq!(msg.content, "Hello");
        assert_eq!(msg.metadata["priority"], "high");
    }

    #[test]
    fn test_team_state_record_message() {
        let mut state = TeamState::new(vec!["a".into()]);
        assert!(state.message_history.is_empty());

        state.record_message(TeamMessage::new("user", "test"));
        assert_eq!(state.message_history.len(), 1);
    }

    #[test]
    fn test_team_state_iteration() {
        let mut state = TeamState::new(vec![]);
        assert_eq!(state.iteration, 0);
        state.next_iteration();
        assert_eq!(state.iteration, 1);
    }

    #[test]
    fn test_leader_delegates_to_nonexistent_agent() {
        // Edge case: leader delegates to an agent not in the team
        // Should fall through to Complete (agent_id not found in state.agents)
        let router = LeaderRouter::new("leader");
        let state = TeamState::new(vec!["leader".into(), "writer".into()]);

        let msg = TeamMessage::new("leader", "@ghost: Do something");
        assert_eq!(
            router.route(&msg, &state),
            RoutingDecision::Complete("Error: Leader delegated to unknown agent 'ghost'".into()),
            "delegation to non-member should return an error message"
        );
    }

    #[test]
    fn test_leader_message_with_at_but_no_colon() {
        // Edge case: "@writer please help" has no colon — split(':').next()
        // returns "writer please help", which doesn't match any agent
        let router = LeaderRouter::new("leader");
        let state = TeamState::new(vec!["leader".into(), "writer".into()]);

        let msg = TeamMessage::new("leader", "@writer please help");
        // "writer please help" != "writer", so this is treated as Complete
        assert_eq!(
            router.route(&msg, &state),
            RoutingDecision::Complete("@writer please help".into()),
            "@ without colon should not match agent name"
        );
    }
}
