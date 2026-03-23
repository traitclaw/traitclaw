//! Agent configuration.

/// Configuration for an Agent instance.
///
/// This struct is `#[non_exhaustive]` — new fields may be added in future
/// releases (e.g., `retry_policy`, `timeout_secs`) without breaking changes.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct AgentConfig {
    /// System prompt for the agent.
    pub system_prompt: Option<String>,

    /// Maximum number of tokens for LLM responses.
    pub max_tokens: Option<u32>,

    /// Temperature for LLM sampling (0.0 - 2.0).
    pub temperature: Option<f32>,

    /// Maximum number of tool call iterations before stopping.
    pub max_iterations: u32,

    /// Maximum token budget for the entire agent run.
    pub token_budget: Option<usize>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            system_prompt: None,
            max_tokens: Some(4096),
            temperature: Some(0.7),
            max_iterations: 20,
            token_budget: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AgentConfig::default();
        assert!(config.system_prompt.is_none());
        assert_eq!(config.max_iterations, 20);
        assert_eq!(config.max_tokens, Some(4096));
        assert!(config.token_budget.is_none());
        assert!((config.temperature.unwrap() - 0.7).abs() < f32::EPSILON);
    }
}
