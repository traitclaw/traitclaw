//! Agent builder for fluent configuration.
//!
//! Use [`AgentBuilder`] to construct an [`Agent`] with progressive complexity.

use std::sync::Arc;

use crate::agent::Agent;
use crate::config::AgentConfig;
use crate::context_managers::RuleBasedCompressor;
use crate::default_strategy::DefaultStrategy;
use crate::memory::in_memory::InMemoryMemory;
use crate::traits::context_manager::ContextManager;
use crate::traits::execution_strategy::{ExecutionStrategy, SequentialStrategy};
use crate::traits::guard::{Guard, NoopGuard};
use crate::traits::hint::{Hint, NoopHint};
use crate::traits::hook::AgentHook;
use crate::traits::memory::Memory;
use crate::traits::output_transformer::OutputTransformer;
use crate::traits::provider::Provider;
use crate::traits::strategy::AgentStrategy;
use crate::traits::tool::ErasedTool;
use crate::traits::tool_registry::{SimpleRegistry, ToolRegistry};
use crate::traits::tracker::{NoopTracker, Tracker};
use crate::transformers::BudgetAwareTruncator;
use crate::Result;

/// Builder for constructing an [`Agent`] with a fluent API.
///
/// # Example
///
/// ```rust,no_run
/// use traitclaw_core::prelude::*;
/// use traitclaw_core::agent_builder::AgentBuilder;
///
/// # async fn example() -> traitclaw_core::Result<()> {
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
    context_manager: Option<Arc<dyn ContextManager>>,
    execution_strategy: Option<Arc<dyn ExecutionStrategy>>,
    output_transformer: Option<Arc<dyn OutputTransformer>>,
    tool_registry: Option<Arc<dyn ToolRegistry>>,
    strategy: Option<Box<dyn AgentStrategy>>,
    hooks: Vec<Arc<dyn AgentHook>>,
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
            context_manager: None,
            execution_strategy: None,
            output_transformer: None,
            tool_registry: None,
            strategy: None,
            hooks: Vec::new(),
            config: AgentConfig::default(),
        }
    }

    /// Set the LLM provider (required).
    ///
    /// Prefer [`.model()`][Self::model] for the idiomatic fluent-API usage.
    /// Both methods are equivalent; `.model()` matches the `agent.model()` pattern
    /// described in the architecture docs.
    #[must_use]
    pub fn provider(mut self, provider: impl Provider) -> Self {
        self.provider = Some(Arc::new(provider));
        self
    }

    /// Set the LLM provider from a pre-wrapped `Arc<dyn Provider>`.
    ///
    /// Use this when you already hold a shared provider reference
    /// (e.g., from [`AgentFactory`](crate::factory::AgentFactory)).
    #[must_use]
    pub fn provider_arc(mut self, provider: Arc<dyn Provider>) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Set the LLM provider — preferred alias for [`.provider()`][Self::provider].
    ///
    /// Enables the idiomatic `Agent::builder().model(provider).system("...").build()` pattern.
    #[must_use]
    pub fn model(self, provider: impl Provider) -> Self {
        self.provider(provider)
    }

    /// Wrap the configured provider with automatic retry and exponential backoff.
    ///
    /// Must be called **after** `.provider()` or `.model()`.
    /// Uses [`RetryProvider`](crate::retry::RetryProvider) internally.
    #[must_use]
    pub fn with_retry(mut self, config: crate::retry::RetryConfig) -> Self {
        if let Some(inner) = self.provider.take() {
            self.provider = Some(Arc::new(crate::retry::RetryProvider::new(inner, config)));
        } else {
            tracing::warn!("with_retry() called before provider() — retry config will be ignored");
        }
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

    /// Add a pre-wrapped `Arc<dyn ErasedTool>` directly.
    ///
    /// Use this when you already hold a shared tool instance that you want
    /// to attach to multiple agents without cloning the underlying value.
    #[must_use]
    pub fn tool_arc(mut self, tool: Arc<dyn ErasedTool>) -> Self {
        self.tools.push(tool);
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

    /// Add multiple pre-wrapped `Arc<dyn ErasedTool>` instances at once.
    #[must_use]
    pub fn tools_arc<I>(mut self, tools: I) -> Self
    where
        I: IntoIterator<Item = Arc<dyn ErasedTool>>,
    {
        self.tools.extend(tools);
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

    /// Set the async context window manager.
    ///
    /// Supports LLM-powered compression and accurate token counting.
    /// Default: `RuleBasedCompressor`.
    #[must_use]
    pub fn context_manager(mut self, manager: impl ContextManager + 'static) -> Self {
        self.context_manager = Some(Arc::new(manager));
        self
    }

    /// Set the tool execution strategy.
    ///
    /// Default: [`SequentialStrategy`] (one at a time).
    /// Use [`ParallelStrategy`](crate::traits::execution_strategy::ParallelStrategy) for concurrent execution.
    #[must_use]
    pub fn execution_strategy(mut self, strategy: impl ExecutionStrategy + 'static) -> Self {
        self.execution_strategy = Some(Arc::new(strategy));
        self
    }

    /// Set the async tool output transformer.
    ///
    /// Supports context-aware, async tool output processing.
    /// Default: `BudgetAwareTruncator` (10,000 chars).
    #[must_use]
    pub fn output_transformer(mut self, transformer: impl OutputTransformer + 'static) -> Self {
        self.output_transformer = Some(Arc::new(transformer));
        self
    }

    /// Set the dynamic tool registry (v0.3.0).
    ///
    /// Enables runtime tool activation/deactivation.
    /// Default: `SimpleRegistry` wrapping configured tools.
    #[must_use]
    pub fn tool_registry(mut self, registry: impl ToolRegistry + 'static) -> Self {
        self.tool_registry = Some(Arc::new(registry));
        self
    }

    /// Set the agent execution strategy.
    ///
    /// Default: [`DefaultStrategy`] (preserves v0.1.0 loop behavior).
    /// Implement [`AgentStrategy`] for custom reasoning architectures.
    #[must_use]
    pub fn strategy(mut self, strategy: impl AgentStrategy) -> Self {
        self.strategy = Some(Box::new(strategy));
        self
    }

    /// Add a lifecycle hook for observability and interception.
    ///
    /// Multiple hooks can be registered and are called sequentially.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use traitclaw_core::traits::hook::LoggingHook;
    ///
    /// # fn example() {
    /// // Agent::builder()
    /// //     .model(my_provider)
    /// //     .hook(LoggingHook::new(tracing::Level::INFO))
    /// //     .build()
    /// # }
    /// ```
    #[must_use]
    pub fn hook(mut self, hook: impl AgentHook) -> Self {
        self.hooks.push(Arc::new(hook));
        self
    }

    /// Build the agent. Returns an error if no provider is configured.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`](crate::Error::Config) if no provider has been set.
    pub fn build(self) -> Result<Agent> {
        let provider = self.provider.ok_or_else(|| {
            crate::Error::Config(
                "AgentBuilder: no provider configured. Use .provider(my_provider) before .build()"
                    .into(),
            )
        })?;

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

        let tracker = self.tracker.unwrap_or_else(|| Arc::new(NoopTracker));

        let default_ctx = RuleBasedCompressor::default();
        // context_manager defaults to RuleBasedCompressor
        let context_manager: Arc<dyn ContextManager> = self
            .context_manager
            .unwrap_or_else(|| Arc::new(default_ctx));

        let execution_strategy = self
            .execution_strategy
            .unwrap_or_else(|| Arc::new(SequentialStrategy));

        let default_out = BudgetAwareTruncator::default();
        // output_transformer defaults to BudgetAwareTruncator
        let output_transformer: Arc<dyn OutputTransformer> = self
            .output_transformer
            .unwrap_or_else(|| Arc::new(default_out));

        // tool_registry defaults to SimpleRegistry wrapping configured tools
        let tool_registry: Arc<dyn ToolRegistry> = self
            .tool_registry
            .unwrap_or_else(|| Arc::new(SimpleRegistry::new(self.tools.clone())));

        // Default to in-memory if no memory configured
        let memory = self
            .memory
            .unwrap_or_else(|| Arc::new(InMemoryMemory::new()));

        let strategy = self.strategy.unwrap_or_else(|| Box::new(DefaultStrategy));

        Ok(Agent::new(
            provider,
            self.tools,
            memory,
            guards,
            hints,
            tracker,
            context_manager,
            execution_strategy,
            output_transformer,
            tool_registry,
            strategy,
            self.hooks,
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
    use crate::types::completion::{CompletionRequest, CompletionResponse, ResponseContent, Usage};
    use crate::types::model_info::{ModelInfo, ModelTier};
    use crate::types::stream::CompletionStream;
    use async_trait::async_trait;

    struct FakeProvider {
        info: ModelInfo,
    }

    impl FakeProvider {
        fn new() -> Self {
            Self {
                info: ModelInfo::new("fake", ModelTier::Small, 4_096, false, false, false),
            }
        }
    }

    #[async_trait]
    impl Provider for FakeProvider {
        async fn complete(&self, _req: CompletionRequest) -> crate::Result<CompletionResponse> {
            Ok(CompletionResponse {
                content: ResponseContent::Text("ok".into()),
                usage: Usage {
                    prompt_tokens: 1,
                    completion_tokens: 1,
                    total_tokens: 2,
                },
            })
        }
        async fn stream(&self, _req: CompletionRequest) -> crate::Result<CompletionStream> {
            unimplemented!()
        }
        fn model_info(&self) -> &ModelInfo {
            &self.info
        }
    }

    #[test]
    fn test_builder_without_provider_returns_error() {
        // AC-2: .build() errors if no provider set
        let result = AgentBuilder::new().system("You are helpful").build();
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_model_alias_ac1() {
        // AC-1: Agent::builder().model(provider).system("...").build() succeeds
        let result = Agent::builder()
            .model(FakeProvider::new())
            .system("You are helpful")
            .build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_builder_accepts_str_and_string_ac3() {
        // AC-3: system/other string params accept &str and String
        let result_str = Agent::builder()
            .model(FakeProvider::new())
            .system("literal")
            .build();
        let result_string = Agent::builder()
            .model(FakeProvider::new())
            .system("owned".to_string())
            .build();
        assert!(result_str.is_ok());
        assert!(result_string.is_ok());
    }

    #[test]
    fn test_defaults_ac4() {
        // AC-4: optional settings have sensible defaults
        let config = AgentConfig::default();
        assert_eq!(
            config.max_iterations, 20,
            "default max_iterations should be 20"
        );
        assert_eq!(
            config.max_tokens,
            Some(4096),
            "default max_tokens should be 4096"
        );
        assert!(
            (config.temperature.unwrap_or(0.0) - 0.7).abs() < f32::EPSILON,
            "default temperature should be 0.7"
        );
    }
}
