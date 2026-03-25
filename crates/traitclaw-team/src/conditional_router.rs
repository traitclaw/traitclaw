//! `ConditionalRouter` — content-based routing using regex patterns.
//!
//! Routes messages to named targets based on their content matching
//! configurable regex patterns. Falls back to a configurable default target.
//!
//! # Example
//!
//! ```rust
//! use traitclaw_team::conditional_router::ConditionalRouter;
//! use traitclaw_team::router::{Router, TeamMessage, TeamState};
//!
//! let router = ConditionalRouter::new()
//!     .when("search|find|look up", "researcher")
//!     .when("write|draft|compose", "writer")
//!     .default("general");
//!
//! let state = TeamState::new(vec![
//!     "researcher".into(), "writer".into(), "general".into(),
//! ]);
//!
//! let msg = TeamMessage::new("user", "Please search for Rust benchmarks");
//! let decision = router.route(&msg, &state);
//! // Routes to "researcher" because content contains "search"
//! ```

use regex::Regex;

use crate::router::{AgentId, Router, RoutingDecision, TeamMessage, TeamState};

/// A routing rule: if message content matches `pattern`, route to `target`.
struct RoutingRule {
    pattern: Regex,
    target: AgentId,
}

/// Content-based router using regex pattern matching.
///
/// Rules are evaluated in order. The first matching pattern wins.
/// If no pattern matches, routes to the configured default target.
pub struct ConditionalRouter {
    rules: Vec<RoutingRule>,
    default_target: Option<AgentId>,
}

impl ConditionalRouter {
    /// Create a new empty `ConditionalRouter`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            default_target: None,
        }
    }

    /// Add a routing rule: if message content matches `pattern`, route to `target`.
    ///
    /// Rules are checked in insertion order. The first match wins.
    ///
    /// # Panics
    ///
    /// Panics if `pattern` is not a valid regex.
    #[must_use]
    pub fn when(mut self, pattern: &str, target: impl Into<AgentId>) -> Self {
        let re = Regex::new(pattern).unwrap_or_else(|e| panic!("Invalid regex '{pattern}': {e}"));
        self.rules.push(RoutingRule {
            pattern: re,
            target: target.into(),
        });
        self
    }

    /// Set the fallback target when no pattern matches.
    ///
    /// If no default is set and no pattern matches, returns a no-op `Complete("")`.
    #[must_use]
    pub fn default(mut self, target: impl Into<AgentId>) -> Self {
        self.default_target = Some(target.into());
        self
    }
}

impl Default for ConditionalRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl Router for ConditionalRouter {
    fn route(&self, message: &TeamMessage, _state: &TeamState) -> RoutingDecision {
        let content = &message.content;

        // Check rules in order
        for rule in &self.rules {
            if rule.pattern.is_match(content) {
                return RoutingDecision::SendTo(rule.target.clone());
            }
        }

        // Fallback
        match &self.default_target {
            Some(target) => RoutingDecision::SendTo(target.clone()),
            None => RoutingDecision::Complete(String::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::router::{TeamMessage, TeamState};

    fn state() -> TeamState {
        TeamState::new(vec!["researcher".into(), "writer".into(), "general".into()])
    }

    #[test]
    fn test_conditional_router_matches_first_rule() {
        // AC #9: routes "search" keyword to "researcher"
        let router = ConditionalRouter::new()
            .when("search|find|look up", "researcher")
            .when("write|draft", "writer")
            .default("general");

        let msg = TeamMessage::new("user", "Please search for Rust benchmarks");
        let decision = router.route(&msg, &state());
        assert_eq!(decision, RoutingDecision::SendTo("researcher".into()));
    }

    #[test]
    fn test_conditional_router_second_rule_matches() {
        let router = ConditionalRouter::new()
            .when("search", "researcher")
            .when("write|draft", "writer")
            .default("general");

        let msg = TeamMessage::new("user", "Please draft a blog post");
        let decision = router.route(&msg, &state());
        assert_eq!(decision, RoutingDecision::SendTo("writer".into()));
    }

    #[test]
    fn test_conditional_router_fallback_to_default() {
        let router = ConditionalRouter::new()
            .when("search", "researcher")
            .default("general");

        let msg = TeamMessage::new("user", "Tell me a joke");
        let decision = router.route(&msg, &state());
        assert_eq!(decision, RoutingDecision::SendTo("general".into()));
    }

    #[test]
    fn test_conditional_router_no_match_no_default() {
        let router = ConditionalRouter::new().when("search", "researcher");

        let msg = TeamMessage::new("user", "Tell me a joke");
        let decision = router.route(&msg, &state());
        assert_eq!(decision, RoutingDecision::Complete(String::new()));
    }

    #[test]
    fn test_conditional_router_case_insensitive_via_regex() {
        // Regex (?i) prefix for case-insensitive
        let router = ConditionalRouter::new()
            .when("(?i)search|(?i)find", "researcher")
            .default("general");

        let msg = TeamMessage::new("user", "SEARCH for information");
        let decision = router.route(&msg, &state());
        assert_eq!(decision, RoutingDecision::SendTo("researcher".into()));
    }

    #[test]
    fn test_conditional_router_empty_rules_uses_default() {
        let router = ConditionalRouter::new().default("general");

        let msg = TeamMessage::new("user", "anything at all");
        let decision = router.route(&msg, &state());
        assert_eq!(decision, RoutingDecision::SendTo("general".into()));
    }

    #[test]
    fn test_conditional_router_is_router_trait_object() {
        let r: Box<dyn Router> = Box::new(ConditionalRouter::new().default("fallback"));
        let msg = TeamMessage::new("user", "test");
        let decision = r.route(&msg, &state());
        assert_eq!(decision, RoutingDecision::SendTo("fallback".into()));
    }
}
