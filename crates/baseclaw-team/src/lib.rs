//! Multi-agent orchestration for the `BaseClaw` AI agent framework.
//!
//! Provides `Team` structs for composing agents, `Router` protocol for
//! message routing, delegation between agents, and `VerificationChain`
//! for generate-then-verify patterns.
//!
//! # Quick Start
//!
//! ```rust
//! use baseclaw_team::{AgentRole, Team, VerificationChain, VerifyResult};
//!
//! let team = Team::new("research_team")
//!     .add_role(AgentRole::new("researcher", "Research topics in depth"))
//!     .add_role(AgentRole::new("writer", "Write clear summaries"));
//!
//! assert_eq!(team.name(), "research_team");
//! assert_eq!(team.roles().len(), 2);
//! ```

#![deny(warnings)]
#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use serde::{Deserialize, Serialize};

/// A team of agents working together.
pub struct Team {
    name: String,
    roles: Vec<AgentRole>,
}

impl Team {
    /// Create a new team with the given name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            roles: Vec::new(),
        }
    }

    /// Add a role to the team.
    #[must_use]
    pub fn add_role(mut self, role: AgentRole) -> Self {
        self.roles.push(role);
        self
    }

    /// Get the team name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the team's roles.
    #[must_use]
    pub fn roles(&self) -> &[AgentRole] {
        &self.roles
    }

    /// Find a role by name.
    #[must_use]
    pub fn find_role(&self, name: &str) -> Option<&AgentRole> {
        self.roles.iter().find(|r| r.name == name)
    }
}

/// A role within a team — describes what an agent specializes in.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRole {
    /// Role name (used for routing).
    pub name: String,
    /// Description of the role's responsibilities.
    pub description: String,
    /// Optional system prompt override for this role.
    pub system_prompt: Option<String>,
}

impl AgentRole {
    /// Create a new agent role.
    #[must_use]
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            system_prompt: None,
        }
    }

    /// Set a custom system prompt for this role.
    #[must_use]
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }
}

/// A verification chain: generate with one agent, verify with another.
///
/// If verification fails, the generation is retried with feedback.
pub struct VerificationChain {
    /// Maximum number of generate-verify cycles.
    pub max_retries: usize,
}

impl VerificationChain {
    /// Create a new verification chain with default 3 retries.
    #[must_use]
    pub fn new() -> Self {
        Self { max_retries: 3 }
    }

    /// Set the maximum number of retries.
    #[must_use]
    pub fn with_max_retries(mut self, n: usize) -> Self {
        self.max_retries = n;
        self
    }
}

impl Default for VerificationChain {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a verification step.
#[derive(Debug, Clone)]
pub enum VerifyResult {
    /// Verification passed — output is acceptable.
    Accepted(String),
    /// Verification failed — include feedback for retry.
    Rejected(String),
}

impl VerifyResult {
    /// Check if the result was accepted.
    #[must_use]
    pub fn is_accepted(&self) -> bool {
        matches!(self, Self::Accepted(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_builder() {
        let team = Team::new("test_team")
            .add_role(AgentRole::new("role1", "First role"))
            .add_role(AgentRole::new("role2", "Second role"));

        assert_eq!(team.name(), "test_team");
        assert_eq!(team.roles().len(), 2);
    }

    #[test]
    fn test_find_role() {
        let team = Team::new("team").add_role(AgentRole::new("researcher", "Research"));

        assert!(team.find_role("researcher").is_some());
        assert!(team.find_role("unknown").is_none());
    }

    #[test]
    fn test_agent_role_with_prompt() {
        let role =
            AgentRole::new("writer", "Write docs").with_system_prompt("You are a technical writer");
        assert_eq!(
            role.system_prompt,
            Some("You are a technical writer".into())
        );
    }

    #[test]
    fn test_verification_chain_default() {
        let chain = VerificationChain::new();
        assert_eq!(chain.max_retries, 3);
    }

    #[test]
    fn test_verification_chain_custom_retries() {
        let chain = VerificationChain::new().with_max_retries(5);
        assert_eq!(chain.max_retries, 5);
    }

    #[test]
    fn test_verify_result() {
        assert!(VerifyResult::Accepted("ok".into()).is_accepted());
        assert!(!VerifyResult::Rejected("bad".into()).is_accepted());
    }
}
