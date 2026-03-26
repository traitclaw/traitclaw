//! Agent factory for shared-provider multi-agent creation.
//!
//! `AgentFactory` holds a provider configuration and spawns agents from it,
//! eliminating repeated builder boilerplate when creating multiple agents
//! from the same provider.

use std::sync::Arc;

use crate::agent::Agent;
use crate::agent_builder::AgentBuilder;
use crate::traits::provider::Provider;
use crate::Result;

/// A factory for creating multiple agents from a shared provider.
///
/// `AgentFactory` solves the "N agents from one provider" problem:
/// instead of repeating `.provider(p)` for each agent, create
/// a factory once and call [`spawn()`](Self::spawn) with different prompts.
///
/// # Example
///
/// ```rust,no_run
/// use traitclaw_core::factory::AgentFactory;
/// use traitclaw_core::traits::provider::Provider;
///
/// # fn example(provider: impl Provider) {
/// let factory = AgentFactory::new(provider);
///
/// let researcher = factory.spawn("You are a researcher.");
/// let writer = factory.spawn("You are a technical writer.");
/// let reviewer = factory.spawn("You are a code reviewer.");
/// // All three agents share the same provider config (via Arc)
/// # }
/// ```
///
/// ## How It Works
///
/// The factory wraps the provider in `Arc<dyn Provider>`, which is
/// cheaply cloneable. Each [`spawn()`](Self::spawn) call clones the Arc
/// (incrementing the reference count) and creates a new agent.
#[derive(Clone)]
pub struct AgentFactory {
    provider: Arc<dyn Provider>,
}

impl std::fmt::Debug for AgentFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentFactory")
            .field("model", &self.provider.model_info().name)
            .finish()
    }
}

impl AgentFactory {
    /// Create a new factory from a provider.
    ///
    /// The provider is wrapped in an `Arc` for cheap cloning. Each
    /// spawned agent shares the same underlying provider instance.
    #[must_use]
    pub fn new(provider: impl Provider) -> Self {
        Self {
            provider: Arc::new(provider),
        }
    }

    /// Create a factory from an already-wrapped `Arc<dyn Provider>`.
    ///
    /// Use this when you already hold a shared provider reference.
    #[must_use]
    pub fn from_arc(provider: Arc<dyn Provider>) -> Self {
        Self { provider }
    }

    /// Spawn an agent with the factory's provider and a system prompt.
    ///
    /// Each spawned agent holds its own `Arc` clone of the provider,
    /// making agents fully independent (cheap reference-counted sharing).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use traitclaw_core::factory::AgentFactory;
    /// use traitclaw_core::traits::provider::Provider;
    ///
    /// # fn example(provider: impl Provider) {
    /// let factory = AgentFactory::new(provider);
    /// let agent = factory.spawn("You are a helpful assistant.");
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// This method cannot panic under normal usage — the internal builder
    /// always has a valid provider.
    #[must_use]
    pub fn spawn(&self, system: impl Into<String>) -> Agent {
        AgentBuilder::new()
            .provider_arc(Arc::clone(&self.provider))
            .system(system)
            .build()
            .expect("AgentFactory::spawn is infallible: provider is always set")
    }

    /// Spawn an agent with custom builder configuration.
    ///
    /// Use this escape hatch when you need more than just a system prompt
    /// (e.g., adding tools, setting memory, configuring hooks).
    ///
    /// The closure receives an [`AgentBuilder`] with the factory's provider
    /// already set. Call builder methods as needed.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use traitclaw_core::factory::AgentFactory;
    /// use traitclaw_core::traits::provider::Provider;
    ///
    /// # fn example(provider: impl Provider) -> traitclaw_core::Result<()> {
    /// let factory = AgentFactory::new(provider);
    /// let agent = factory.spawn_with(|b| {
    ///     b.system("You are a researcher with tools.")
    ///      .max_iterations(10)
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the builder customization produces an invalid
    /// agent configuration.
    pub fn spawn_with(&self, f: impl FnOnce(AgentBuilder) -> AgentBuilder) -> Result<Agent> {
        let builder = AgentBuilder::new().provider_arc(Arc::clone(&self.provider));
        f(builder).build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::completion::{CompletionRequest, CompletionResponse, ResponseContent, Usage};
    use crate::types::model_info::{ModelInfo, ModelTier};
    use crate::types::stream::CompletionStream;
    use async_trait::async_trait;

    #[derive(Clone)]
    struct MockCloneProvider {
        info: ModelInfo,
    }

    impl MockCloneProvider {
        fn new() -> Self {
            Self {
                info: ModelInfo::new("mock-clone", ModelTier::Small, 4_096, false, false, false),
            }
        }
    }

    #[async_trait]
    impl Provider for MockCloneProvider {
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
    fn test_factory_new() {
        let factory = AgentFactory::new(MockCloneProvider::new());
        assert_eq!(factory.provider.model_info().name, "mock-clone");
    }

    #[test]
    fn test_factory_from_arc() {
        let provider: Arc<dyn Provider> = Arc::new(MockCloneProvider::new());
        let factory = AgentFactory::from_arc(provider);
        assert_eq!(factory.provider.model_info().name, "mock-clone");
    }

    #[test]
    fn test_factory_spawn_creates_agent_with_system_prompt() {
        let factory = AgentFactory::new(MockCloneProvider::new());
        let agent = factory.spawn("You are a researcher.");
        assert_eq!(
            agent.config.system_prompt.as_deref(),
            Some("You are a researcher.")
        );
    }

    #[test]
    fn test_factory_spawn_produces_independent_agents() {
        let factory = AgentFactory::new(MockCloneProvider::new());
        let agent_a = factory.spawn("Agent A");
        let agent_b = factory.spawn("Agent B");

        assert_eq!(agent_a.config.system_prompt.as_deref(), Some("Agent A"));
        assert_eq!(agent_b.config.system_prompt.as_deref(), Some("Agent B"));
        // Both have the same provider model
        assert_eq!(agent_a.provider.model_info().name, "mock-clone");
        assert_eq!(agent_b.provider.model_info().name, "mock-clone");
    }

    #[test]
    fn test_factory_spawn_with_custom_config() {
        let factory = AgentFactory::new(MockCloneProvider::new());
        let agent = factory
            .spawn_with(|b| b.system("Custom").max_iterations(5))
            .expect("spawn_with should succeed");

        assert_eq!(agent.config.system_prompt.as_deref(), Some("Custom"));
        assert_eq!(agent.config.max_iterations, 5);
    }

    #[test]
    fn test_factory_spawn_with_no_system() {
        let factory = AgentFactory::new(MockCloneProvider::new());
        let agent = factory
            .spawn_with(|b| b.max_iterations(3))
            .expect("spawn_with without system should succeed");

        assert!(agent.config.system_prompt.is_none());
    }

    // Compile-time check: AgentFactory is Send + Sync
    fn _assert_send_sync<T: Send + Sync>() {}
    #[test]
    fn test_factory_is_send_sync() {
        _assert_send_sync::<AgentFactory>();
    }
}
