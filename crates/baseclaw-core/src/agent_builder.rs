//! Agent builder for fluent configuration.
//!
//! Use [`AgentBuilder`] to construct an [`Agent`] with progressive complexity.

use std::sync::Arc;

use crate::agent::Agent;
use crate::config::AgentConfig;
use crate::traits::guard::{Guard, NoopGuard};
use crate::traits::hint::{Hint, NoopHint};
use crate::traits::memory::Memory;
use crate::traits::provider::Provider;
use crate::traits::tool::ErasedTool;
use crate::traits::tracker::{NoopTracker, Tracker};
use crate::memory::in_memory::InMemoryMemory;
use crate::Result;

/// Builder for constructing an [`Agent`] with a fluent API.
///
/// # Example
///
/// ```rust,no_run
/// use baseclaw_core::prelude::*;
/// use baseclaw_core::agent_builder::AgentBuilder;
///
/// # async fn example() -> baseclaw_core::Result<()> {
/// // Minimal agent (provider required):
/// // let agent = AgentBuilder::new()
/// //     .provider(my_provider)
/// //     .system("You are helpful")
/// //     .build()?;
/// # Ok(())
/// # }
/// ```
pub struct AgentBuilder {
    provider: Option<Arc<dyn Provider>>,
    tools: Vec<Arc<dyn ErasedTool>>,
    memory: Option<Arc<dyn Memory>>,
    guards: Vec<Arc<dyn Guard>>,
    hints: Vec<Arc<dyn Hint>>,
    tracker: Option<Arc<dyn Tracker>>,
    config: AgentConfig,
}

impl AgentBuilder {
    /// Create a new builder with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            provider: None,
            tools: Vec::new(),
            memory: None,
            guards: Vec::new(),
            hints: Vec::new(),
            tracker: None,
            config: AgentConfig::default(),
        }
    }

    /// Set the LLM provider (required).
    #[must_use]
    pub fn provider(mut self, provider: impl Provider) -> Self {
        self.provider = Some(Arc::new(provider));
        self
    }

    /// Set the system prompt.
    #[must_use]
    pub fn system(mut self, prompt: impl Into<String>) -> Self {
        self.config.system_prompt = Some(prompt.into());
        self
    }

    /// Add a tool to the agent.
    #[must_use]
    pub fn tool(mut self, tool: impl ErasedTool) -> Self {
        self.tools.push(Arc::new(tool));
        self
    }

    /// Add multiple tools at once.
    #[must_use]
    pub fn tools<I, T>(mut self, tools: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: ErasedTool,
    {
        for tool in tools {
            self.tools.push(Arc::new(tool));
        }
        self
    }

    /// Set the memory backend.
    #[must_use]
    pub fn memory(mut self, memory: impl Memory) -> Self {
        self.memory = Some(Arc::new(memory));
        self
    }

    /// Add a guard to the agent.
    #[must_use]
    pub fn guard(mut self, guard: impl Guard) -> Self {
        self.guards.push(Arc::new(guard));
        self
    }

    /// Add a hint to the agent.
    #[must_use]
    pub fn hint(mut self, hint: impl Hint) -> Self {
        self.hints.push(Arc::new(hint));
        self
    }

    /// Set the tracker for runtime monitoring.
    #[must_use]
    pub fn tracker(mut self, tracker: impl Tracker) -> Self {
        self.tracker = Some(Arc::new(tracker));
        self
    }

    /// Set the maximum number of tool call iterations.
    #[must_use]
    pub fn max_iterations(mut self, max: u32) -> Self {
        self.config.max_iterations = max;
        self
    }

    /// Set the maximum tokens for LLM responses.
    #[must_use]
    pub fn max_tokens(mut self, max: u32) -> Self {
        self.config.max_tokens = Some(max);
        self
    }

    /// Set the sampling temperature.
    #[must_use]
    pub fn temperature(mut self, temp: f32) -> Self {
        self.config.temperature = Some(temp);
        self
    }

    /// Set the token budget for the entire run.
    #[must_use]
    pub fn token_budget(mut self, budget: usize) -> Self {
        self.config.token_budget = Some(budget);
        self
    }

    /// Build the agent. Returns an error if no provider is configured.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`](crate::Error::Config) if no provider has been set.
    pub fn build(self) -> Result<Agent> {
        let provider = self
            .provider
            .ok_or_else(|| crate::Error::Config("No provider configured. Use .provider() to set one.".into()))?;

        // Default to Noop steering if none configured
        let guards: Vec<Arc<dyn Guard>> = if self.guards.is_empty() {
            vec![Arc::new(NoopGuard)]
        } else {
            self.guards
        };

        let hints: Vec<Arc<dyn Hint>> = if self.hints.is_empty() {
            vec![Arc::new(NoopHint)]
        } else {
            self.hints
        };

        let tracker = self
            .tracker
            .unwrap_or_else(|| Arc::new(NoopTracker));

        // Default to in-memory if no memory configured
        let memory = self
            .memory
            .unwrap_or_else(|| Arc::new(InMemoryMemory::new()));

        Ok(Agent::new(
            provider,
            self.tools,
            memory,
            guards,
            hints,
            tracker,
            self.config,
        ))
    }
}

impl Default for AgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_without_provider_returns_error() {
        let result = AgentBuilder::new()
            .system("You are helpful")
            .build();
        assert!(result.is_err());
    }
}
