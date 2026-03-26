//! Multi-agent group chat with configurable turn-taking and termination.
//!
//! Provides [`RoundRobinGroupChat`] for structured multi-turn conversations
//! where agents take turns in a fixed order, each seeing the full transcript.

use std::fmt;

use traitclaw_core::agent::Agent;
use traitclaw_core::types::message::{Message, MessageRole};

// ─────────────────────────────────────────────────────────────────────────────
// Termination Conditions
// ─────────────────────────────────────────────────────────────────────────────

/// Trait for determining when a group chat should stop.
///
/// Implement this trait for custom termination logic (keyword detection,
/// quality thresholds, consensus detection, etc.).
pub trait TerminationCondition: Send + Sync {
    /// Check whether the chat should terminate.
    ///
    /// - `round`: the current round number (0-indexed)
    /// - `messages`: the full conversation transcript so far
    fn should_terminate(&self, round: usize, messages: &[Message]) -> bool;
}

/// Terminate after a fixed number of rounds.
///
/// # Example
///
/// ```rust
/// use traitclaw_team::group_chat::MaxRoundsTermination;
///
/// let term = MaxRoundsTermination::new(6);
/// ```
#[derive(Debug, Clone)]
pub struct MaxRoundsTermination {
    max_rounds: usize,
}

impl MaxRoundsTermination {
    /// Create a termination condition that stops after `max_rounds` rounds.
    #[must_use]
    pub fn new(max_rounds: usize) -> Self {
        Self { max_rounds }
    }
}

impl TerminationCondition for MaxRoundsTermination {
    fn should_terminate(&self, round: usize, _messages: &[Message]) -> bool {
        round >= self.max_rounds
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Group Chat Result
// ─────────────────────────────────────────────────────────────────────────────

/// The result of a group chat session.
#[derive(Debug, Clone)]
pub struct GroupChatResult {
    /// Full conversation transcript in chronological order.
    pub transcript: Vec<Message>,
    /// The final message text produced by the last responding agent.
    pub final_message: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// RoundRobinGroupChat
// ─────────────────────────────────────────────────────────────────────────────

/// A round-robin group chat where agents take turns responding.
///
/// Each agent sees the full conversation history and adds its response.
/// The chat continues until the termination condition is met.
///
/// # Example
///
/// ```rust,no_run
/// use traitclaw_team::group_chat::RoundRobinGroupChat;
/// use traitclaw_core::agent::Agent;
///
/// # async fn example(agents: Vec<Agent>) -> traitclaw_core::Result<()> {
/// let mut chat = RoundRobinGroupChat::new(agents);
/// let result = chat.run("Discuss the future of AI").await?;
/// println!("Transcript has {} messages", result.transcript.len());
/// println!("Final: {}", result.final_message);
/// # Ok(())
/// # }
/// ```
pub struct RoundRobinGroupChat {
    agents: Vec<Agent>,
    termination: Box<dyn TerminationCondition>,
}

impl fmt::Debug for RoundRobinGroupChat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RoundRobinGroupChat")
            .field("agents", &self.agents.len())
            .finish()
    }
}

impl RoundRobinGroupChat {
    /// Create a new group chat with default termination (`n_agents × 3` rounds).
    ///
    /// # Panics
    ///
    /// This method cannot panic under normal usage.
    #[must_use]
    pub fn new(agents: Vec<Agent>) -> Self {
        let max_rounds = agents.len().saturating_mul(3).max(1);
        Self {
            termination: Box::new(MaxRoundsTermination::new(max_rounds)),
            agents,
        }
    }

    /// Set the maximum number of rounds (convenience method).
    #[must_use]
    pub fn with_max_rounds(mut self, n: usize) -> Self {
        self.termination = Box::new(MaxRoundsTermination::new(n));
        self
    }

    /// Set a custom termination condition.
    #[must_use]
    pub fn with_termination(mut self, t: impl TerminationCondition + 'static) -> Self {
        self.termination = Box::new(t);
        self
    }

    /// Returns the number of agents in the chat.
    #[must_use]
    pub fn len(&self) -> usize {
        self.agents.len()
    }

    /// Returns `true` if the chat has no agents.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }

    /// Run the group chat starting with the given task prompt.
    ///
    /// Agents respond in round-robin order, each seeing the full transcript.
    /// The chat terminates when the termination condition is met.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The agent pool is empty
    /// - Any agent fails to produce a response
    pub async fn run(&self, task: &str) -> traitclaw_core::Result<GroupChatResult> {
        if self.agents.is_empty() {
            return Err(traitclaw_core::Error::Runtime(
                "RoundRobinGroupChat::run() called with no agents".into(),
            ));
        }

        let mut transcript = vec![Message {
            role: MessageRole::User,
            content: task.to_string(),
            tool_call_id: None,
        }];

        let n_agents = self.agents.len();
        let mut round = 0;

        loop {
            if self.termination.should_terminate(round, &transcript) {
                break;
            }

            let agent_idx = round % n_agents;
            let agent = &self.agents[agent_idx];

            // Build the context: format transcript as a conversation prompt
            let context = Self::format_transcript(&transcript);
            let output = agent.run(&context).await?;
            let response_text = output.text().to_string();

            transcript.push(Message {
                role: MessageRole::Assistant,
                content: response_text,
                tool_call_id: None,
            });

            round += 1;
        }

        let final_message = transcript
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_default();

        Ok(GroupChatResult {
            transcript,
            final_message,
        })
    }

    /// Format transcript messages into a single context string.
    fn format_transcript(messages: &[Message]) -> String {
        messages
            .iter()
            .map(|m| {
                let role = match m.role {
                    MessageRole::User => "User",
                    MessageRole::Assistant => "Assistant",
                    MessageRole::System => "System",
                    MessageRole::Tool => "Tool",
                    _ => "Unknown",
                };
                format!("[{}]: {}", role, m.content)
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests_common::EchoProvider;

    // ── TerminationCondition ────────────────────────────────────────────────

    #[test]
    fn test_max_rounds_at_boundary() {
        let term = MaxRoundsTermination::new(3);
        assert!(!term.should_terminate(0, &[]));
        assert!(!term.should_terminate(1, &[]));
        assert!(!term.should_terminate(2, &[]));
        assert!(term.should_terminate(3, &[]));
        assert!(term.should_terminate(4, &[]));
    }

    #[test]
    fn test_max_rounds_zero() {
        let term = MaxRoundsTermination::new(0);
        assert!(term.should_terminate(0, &[]));
    }

    // ── RoundRobinGroupChat ─────────────────────────────────────────────────

    #[test]
    fn test_group_chat_new_default_rounds() {
        let agents = vec![
            Agent::with_system(EchoProvider::new("A"), "Agent A"),
            Agent::with_system(EchoProvider::new("B"), "Agent B"),
        ];
        let chat = RoundRobinGroupChat::new(agents);
        assert_eq!(chat.len(), 2);
        // Default max_rounds = 2 * 3 = 6
    }

    #[test]
    fn test_group_chat_with_max_rounds() {
        let agents = vec![Agent::with_system(EchoProvider::new("A"), "Agent A")];
        let chat = RoundRobinGroupChat::new(agents).with_max_rounds(10);
        assert_eq!(chat.len(), 1);
    }

    #[tokio::test]
    async fn test_group_chat_run_basic() {
        let agents = vec![
            Agent::with_system(EchoProvider::new("R"), "Researcher"),
            Agent::with_system(EchoProvider::new("W"), "Writer"),
        ];
        let chat = RoundRobinGroupChat::new(agents).with_max_rounds(2);
        let result = chat.run("Discuss Rust").await.unwrap();

        // Initial user message + 2 agent responses = 3 messages
        assert_eq!(result.transcript.len(), 3);
        assert!(!result.final_message.is_empty());
    }

    #[tokio::test]
    async fn test_group_chat_round_robin_order() {
        let agents = vec![
            Agent::with_system(EchoProvider::new("FIRST"), "First"),
            Agent::with_system(EchoProvider::new("SECOND"), "Second"),
        ];
        let chat = RoundRobinGroupChat::new(agents).with_max_rounds(4);
        let result = chat.run("Test").await.unwrap();

        // 5 messages: 1 user + 4 agent responses
        assert_eq!(result.transcript.len(), 5);
        // Check round-robin order by prefix
        assert!(result.transcript[1].content.contains("[FIRST]"));
        assert!(result.transcript[2].content.contains("[SECOND]"));
        assert!(result.transcript[3].content.contains("[FIRST]"));
        assert!(result.transcript[4].content.contains("[SECOND]"));
    }

    #[tokio::test]
    async fn test_group_chat_empty_agents_returns_error() {
        let chat = RoundRobinGroupChat::new(vec![]);
        let result = chat.run("Test").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_group_chat_custom_termination() {
        // Custom termination: stop when any message contains "DONE"
        struct ContainsKeyword;
        impl TerminationCondition for ContainsKeyword {
            fn should_terminate(&self, _round: usize, messages: &[Message]) -> bool {
                messages.iter().any(|m| m.content.contains("DONE"))
            }
        }

        let agents = vec![Agent::with_system(EchoProvider::new("DONE"), "Agent")];
        let chat = RoundRobinGroupChat::new(agents).with_termination(ContainsKeyword);
        let result = chat.run("Test").await.unwrap();

        // Should stop after first agent response (contains "DONE")
        // 1 user + 1 agent = 2 messages
        assert_eq!(result.transcript.len(), 2);
    }

    #[test]
    fn test_group_chat_debug() {
        let chat = RoundRobinGroupChat::new(vec![Agent::with_system(EchoProvider::new("A"), "A")]);
        let debug = format!("{chat:?}");
        assert!(debug.contains("RoundRobinGroupChat"));
    }
}
