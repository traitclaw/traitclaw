//! `AdaptiveTracker` — monitors agent behavior and exposes tier-based config.

use baseclaw_core::traits::provider::ModelTier;
use baseclaw_core::traits::tracker::Tracker;
use baseclaw_core::types::agent_state::AgentState;
use baseclaw_core::types::completion::CompletionResponse;
use serde_json::Value;

/// Configuration driven by model tier.
#[derive(Debug, Clone)]
pub struct TierConfig {
    /// How often to inject hints (every N iterations).
    pub hint_frequency: u32,
    /// Maximum context utilization before throttling.
    pub context_throttle: f32,
    /// How often to re-inject system prompt (every N iterations).
    pub system_remind_frequency: u32,
    /// Recommended max concurrent operations for this tier.
    pub concurrency: usize,
}

impl TierConfig {
    /// Config for `Small` tier models (Haiku, Phi, 7B/8B).
    #[must_use]
    pub fn small() -> Self {
        Self {
            hint_frequency: 3,
            context_throttle: 0.50,
            system_remind_frequency: 4,
            concurrency: 1,
        }
    }

    /// Config for `Medium` tier models (Sonnet, 4o-mini, Mixtral).
    #[must_use]
    pub fn medium() -> Self {
        Self {
            hint_frequency: 6,
            context_throttle: 0.60,
            system_remind_frequency: 8,
            concurrency: 3,
        }
    }

    /// Config for `Large` tier models (Opus, GPT-4o, 70B).
    #[must_use]
    pub fn large() -> Self {
        Self {
            hint_frequency: 12,
            context_throttle: 0.80,
            system_remind_frequency: 15,
            concurrency: usize::MAX,
        }
    }

    /// Derive config from a `ModelTier`.
    #[must_use]
    pub fn for_tier(tier: &ModelTier) -> Self {
        match tier {
            ModelTier::Small => Self::small(),
            ModelTier::Medium => Self::medium(),
            ModelTier::Large => Self::large(),
        }
    }
}

/// A [`Tracker`] that monitors the agent and exposes tier-based configuration.
pub struct AdaptiveTracker {
    config: TierConfig,
}

impl AdaptiveTracker {
    /// Create an `AdaptiveTracker` for a specific model tier.
    #[must_use]
    pub fn for_tier(tier: &ModelTier) -> Self {
        Self {
            config: TierConfig::for_tier(tier),
        }
    }

    /// Get the resolved tier configuration.
    #[must_use]
    pub fn config(&self) -> &TierConfig {
        &self.config
    }
}

impl Tracker for AdaptiveTracker {
    fn on_iteration(&self, state: &mut AgentState) {
        tracing::debug!(
            iteration = state.iteration_count,
            "AdaptiveTracker: iteration start"
        );
    }

    fn on_tool_call(&self, name: &str, _args: &Value, state: &mut AgentState) {
        tracing::debug!(tool = name, "AdaptiveTracker: tool call");

        if state.context_utilization() >= self.config.context_throttle {
            tracing::warn!(
                utilization = %(format!("{:.0}%", state.context_utilization() * 100.0)),
                "AdaptiveTracker: context utilization exceeds throttle threshold"
            );
        }
    }

    fn on_llm_response(&self, response: &CompletionResponse, _state: &mut AgentState) {
        tracing::debug!(
            completion_tokens = response.usage.completion_tokens,
            "AdaptiveTracker: LLM response received"
        );
    }

    fn recommended_concurrency(&self, _state: &AgentState) -> usize {
        self.config.concurrency
    }
}
