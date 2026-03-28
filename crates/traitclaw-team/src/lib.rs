//! Multi-agent orchestration for the `TraitClaw` AI agent framework.
//!
//! Provides `Team` structs for composing agents, `Router` protocol for
//! message routing, delegation between agents, and `VerificationChain`
//! for generate-then-verify patterns.
//!
//! # Quick Start
//!
//! ```rust
//! use traitclaw_team::{AgentRole, Team, VerificationChain, VerifyResult};
//! use traitclaw_team::router::SequentialRouter;
//!
//! let team = Team::new("research_team")
//!     .add_role(AgentRole::new("researcher", "Research topics in depth"))
//!     .add_role(AgentRole::new("writer", "Write clear summaries"));
//!
//! assert_eq!(team.name(), "research_team");
//! assert_eq!(team.roles().len(), 2);
//! ```

#![deny(missing_docs)]
#![allow(clippy::redundant_closure)]

pub mod conditional_router;
pub mod execution;
pub mod group_chat;
pub mod router;
pub mod team_context;

#[cfg(test)]
pub(crate) mod tests_common;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use traitclaw_core::traits::provider::Provider;

pub use conditional_router::ConditionalRouter;
pub use execution::{run_verification_chain, TeamRunner};
pub use team_context::TeamContext;

/// Create an [`AgentPool`](traitclaw_core::pool::AgentPool) from a [`Team`] and a provider.
///
/// Each role's `system_prompt` is used as the agent's system prompt.
/// Roles without a `system_prompt` cause an error listing all missing roles.
///
/// # Example
///
/// ```rust
/// use traitclaw_team::{AgentRole, Team, pool_from_team};
/// use traitclaw_core::traits::provider::Provider;
///
/// # fn example(provider: impl Provider) -> traitclaw_core::Result<()> {
/// let team = Team::new("content_team")
///     .add_role(AgentRole::new("researcher", "Research").with_system_prompt("You research topics."))
///     .add_role(AgentRole::new("writer", "Write").with_system_prompt("You write articles."));
///
/// let pool = pool_from_team(&team, provider)?;
/// assert_eq!(pool.len(), 2);
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if any role in the team is missing a `system_prompt`.
pub fn pool_from_team(
    team: &Team,
    provider: impl Provider,
) -> traitclaw_core::Result<traitclaw_core::pool::AgentPool> {
    // Check for missing system_prompts first
    let missing: Vec<&str> = team
        .roles()
        .iter()
        .filter(|r| r.system_prompt.is_none())
        .map(|r| r.name.as_str())
        .collect();

    if !missing.is_empty() {
        return Err(traitclaw_core::Error::Config(format!(
            "Cannot create AgentPool from team '{}': roles missing system_prompt: [{}]",
            team.name(),
            missing.join(", ")
        )));
    }

    let factory = traitclaw_core::factory::AgentFactory::new(provider);
    let agents: Vec<traitclaw_core::Agent> = team
        .roles()
        .iter()
        .map(|role| factory.spawn(role.system_prompt.as_ref().expect("checked above")))
        .collect();

    Ok(traitclaw_core::pool::AgentPool::new(agents))
}

/// Create an [`AgentPool`](traitclaw_core::pool::AgentPool) from a [`Team`]
/// using a pre-wrapped `Arc<dyn Provider>`.
///
/// Same as [`pool_from_team`] but accepts a shared provider reference.
pub fn pool_from_team_arc(
    team: &Team,
    provider: Arc<dyn Provider>,
) -> traitclaw_core::Result<traitclaw_core::pool::AgentPool> {
    let missing: Vec<&str> = team
        .roles()
        .iter()
        .filter(|r| r.system_prompt.is_none())
        .map(|r| r.name.as_str())
        .collect();

    if !missing.is_empty() {
        return Err(traitclaw_core::Error::Config(format!(
            "Cannot create AgentPool from team '{}': roles missing system_prompt: [{}]",
            team.name(),
            missing.join(", ")
        )));
    }

    let factory = traitclaw_core::factory::AgentFactory::from_arc(provider);
    let agents: Vec<traitclaw_core::Agent> = team
        .roles()
        .iter()
        .map(|role| factory.spawn(role.system_prompt.as_ref().expect("checked above")))
        .collect();

    Ok(traitclaw_core::pool::AgentPool::new(agents))
}

/// A team of agents working together.
pub struct Team {
    name: String,
    roles: Vec<AgentRole>,
    router: Box<dyn router::Router>,
}

impl Team {
    /// Create a new team with the given name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            roles: Vec::new(),
            router: Box::new(router::SequentialRouter::new()),
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

    /// Set a custom router for this team.
    ///
    /// Default: [`SequentialRouter`](router::SequentialRouter).
    #[must_use]
    pub fn with_router(mut self, router: impl router::Router) -> Self {
        self.router = Box::new(router);
        self
    }

    /// Get a reference to the team's router.
    #[must_use]
    pub fn router(&self) -> &dyn router::Router {
        &*self.router
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
