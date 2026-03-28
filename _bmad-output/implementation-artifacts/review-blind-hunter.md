@[/bmad-review-adversarial-general]
Vui lòng review adversarial đoạn diff sau đây cho version v0.6.0 (Chunk 1 - Core API) với tư cách Blind Hunter:

```diff
diff --git a/crates/traitclaw-core/src/agent.rs b/crates/traitclaw-core/src/agent.rs
index 439e255..11b739a 100644
--- a/crates/traitclaw-core/src/agent.rs
+++ b/crates/traitclaw-core/src/agent.rs
@@ -163,6 +163,43 @@ impl Agent {
         AgentBuilder::new()
     }
 
+    /// Create an agent with just a provider and system prompt.
+    ///
+    /// This is a convenience shorthand equivalent to:
+    /// ```rust,ignore
+    /// Agent::builder()
+    ///     .provider(provider)
+    ///     .system(system)
+    ///     .build()
+    ///     .unwrap()
+    /// ```
+    ///
+    /// All other settings use their defaults (in-memory memory, no tools,
+    /// no guards, etc.). Use [`Agent::builder()`] for full customization.
+    ///
+    /// # Example
+    ///
+    /// ```rust,no_run
+    /// use traitclaw_core::prelude::*;
+    ///
+    /// # fn example(provider: impl traitclaw_core::traits::provider::Provider) {
+    /// let agent = Agent::with_system(provider, "You are a helpful assistant.");
+    /// # }
+    /// ```
+    /// # Panics
+    ///
+    /// This method cannot panic under normal usage — the internal `build()`
+    /// call only fails when no provider is set, and `with_system` always
+    /// provides one.
+    #[must_use]
+    pub fn with_system(provider: impl Provider, system: impl Into<String>) -> Self {
+        Agent::builder()
+            .provider(provider)
+            .system(system)
+            .build()
+            .expect("Agent::with_system is infallible: provider is always set")
+    }
+
     /// Create an agent directly (prefer using `builder()`).
     #[allow(clippy::too_many_arguments)]
     pub(crate) fn new(
@@ -531,4 +568,81 @@ mod tests {
         assert_eq!(out.usage.iterations, 5);
         assert_eq!(out.usage.duration.as_millis(), 500);
     }
+
+    // --- Agent::with_system() tests (Story 1.1) ---
+
+    use crate::types::completion::{CompletionRequest, CompletionResponse, ResponseContent, Usage};
+    use crate::types::model_info::{ModelInfo, ModelTier};
+    use crate::types::stream::CompletionStream;
+    use async_trait::async_trait;
+
+    struct MockProvider {
+        info: ModelInfo,
+    }
+
+    impl MockProvider {
+        fn new() -> Self {
+            Self {
+                info: ModelInfo::new("mock", ModelTier::Small, 4_096, false, false, false),
+            }
+        }
+    }
+
+    #[async_trait]
+    impl crate::traits::provider::Provider for MockProvider {
+        async fn complete(&self, _req: CompletionRequest) -> crate::Result<CompletionResponse> {
+            Ok(CompletionResponse {
+                content: ResponseContent::Text("ok".into()),
+                usage: Usage {
+                    prompt_tokens: 1,
+                    completion_tokens: 1,
+                    total_tokens: 2,
+                },
+            })
+        }
+        async fn stream(&self, _req: CompletionRequest) -> crate::Result<CompletionStream> {
+            unimplemented!()
+        }
+        fn model_info(&self) -> &ModelInfo {
+            &self.info
+        }
+    }
+
+    #[test]
+    fn test_with_system_str_prompt() {
+        // AC #1, #2: with_system accepts &str and creates a valid agent
+        let agent = Agent::with_system(MockProvider::new(), "You are helpful.");
+        assert_eq!(
+            agent.config.system_prompt.as_deref(),
+            Some("You are helpful.")
+        );
+    }
+
+    #[test]
+    fn test_with_system_string_prompt() {
+        // AC #2: with_system accepts String
+        let prompt = String::from("You are a researcher.");
+        let agent = Agent::with_system(MockProvider::new(), prompt);
+        assert_eq!(
+            agent.config.system_prompt.as_deref(),
+            Some("You are a researcher.")
+        );
+    }
+
+    #[test]
+    fn test_with_system_builder_unchanged() {
+        // AC #3: builder API is unchanged (still works)
+        let result = Agent::builder()
+            .provider(MockProvider::new())
+            .system("test")
+            .build();
+        assert!(result.is_ok());
+    }
+
+    #[test]
+    fn test_with_system_provider_configured() {
+        // AC #4: agent has correct provider
+        let agent = Agent::with_system(MockProvider::new(), "test");
+        assert_eq!(agent.provider.model_info().name, "mock");
+    }
 }
diff --git a/crates/traitclaw-core/src/agent_builder.rs b/crates/traitclaw-core/src/agent_builder.rs
index 1d670eb..4e3f216 100644
--- a/crates/traitclaw-core/src/agent_builder.rs
+++ b/crates/traitclaw-core/src/agent_builder.rs
@@ -97,6 +97,16 @@ impl AgentBuilder {
         self
     }
 
+    /// Set the LLM provider from a pre-wrapped `Arc<dyn Provider>`.
+    ///
+    /// Use this when you already hold a shared provider reference
+    /// (e.g., from [`AgentFactory`](crate::factory::AgentFactory)).
+    #[must_use]
+    pub fn provider_arc(mut self, provider: Arc<dyn Provider>) -> Self {
+        self.provider = Some(provider);
+        self
+    }
+
     /// Set the LLM provider — preferred alias for [`.provider()`][Self::provider].
     ///
     /// Enables the idiomatic `Agent::builder().model(provider).system("...").build()` pattern.
diff --git a/crates/traitclaw-core/src/factory.rs b/crates/traitclaw-core/src/factory.rs
new file mode 100644
index 0000000..8b7534a
--- /dev/null
+++ b/crates/traitclaw-core/src/factory.rs
@@ -0,0 +1,242 @@
+//! Agent factory for shared-provider multi-agent creation.
+//!
+//! `AgentFactory` holds a provider configuration and spawns agents from it,
+//! eliminating repeated builder boilerplate when creating multiple agents
+//! from the same provider.
+
+use std::sync::Arc;
+
+use crate::agent::Agent;
+use crate::agent_builder::AgentBuilder;
+use crate::traits::provider::Provider;
+use crate::Result;
+
+/// A factory for creating multiple agents from a shared provider.
+///
+/// `AgentFactory` solves the "N agents from one provider" problem:
+/// instead of repeating `.provider(p)` for each agent, create
+/// a factory once and call [`spawn()`](Self::spawn) with different prompts.
+///
+/// # Example
+///
+/// ```rust,no_run
+/// use traitclaw_core::factory::AgentFactory;
+/// use traitclaw_core::traits::provider::Provider;
+///
+/// # fn example(provider: impl Provider) {
+/// let factory = AgentFactory::new(provider);
+///
+/// let researcher = factory.spawn("You are a researcher.");
+/// let writer = factory.spawn("You are a technical writer.");
+/// let reviewer = factory.spawn("You are a code reviewer.");
+/// // All three agents share the same provider config (via Arc)
+/// # }
+/// ```
+///
+/// ## How It Works
+///
+/// The factory wraps the provider in `Arc<dyn Provider>`, which is
+/// cheaply cloneable. Each [`spawn()`](Self::spawn) call clones the Arc
+/// (incrementing the reference count) and creates a new agent.
+#[derive(Clone)]
+pub struct AgentFactory {
+    provider: Arc<dyn Provider>,
+}
+
+impl std::fmt::Debug for AgentFactory {
+    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
+        f.debug_struct("AgentFactory")
+            .field("model", &self.provider.model_info().name)
+            .finish()
+    }
+}
+
+impl AgentFactory {
+    /// Create a new factory from a provider.
+    ///
+    /// The provider is wrapped in an `Arc` for cheap cloning. Each
+    /// spawned agent shares the same underlying provider instance.
+    #[must_use]
+    pub fn new(provider: impl Provider) -> Self {
+        Self {
+            provider: Arc::new(provider),
+        }
+    }
+
+    /// Create a factory from an already-wrapped `Arc<dyn Provider>`.
+    ///
+    /// Use this when you already hold a shared provider reference.
+    #[must_use]
+    pub fn from_arc(provider: Arc<dyn Provider>) -> Self {
+        Self { provider }
+    }
+
+    /// Spawn an agent with the factory's provider and a system prompt.
+    ///
+    /// Each spawned agent holds its own `Arc` clone of the provider,
+    /// making agents fully independent (cheap reference-counted sharing).
+    ///
+    /// # Example
+    ///
+    /// ```rust,no_run
+    /// use traitclaw_core::factory::AgentFactory;
+    /// use traitclaw_core::traits::provider::Provider;
+    ///
+    /// # fn example(provider: impl Provider) {
+    /// let factory = AgentFactory::new(provider);
+    /// let agent = factory.spawn("You are a helpful assistant.");
+    /// # }
+    /// ```
+    ///
+    /// # Panics
+    ///
+    /// This method cannot panic under normal usage — the internal builder
+    /// always has a valid provider.
+    #[must_use]
+    pub fn spawn(&self, system: impl Into<String>) -> Agent {
+        AgentBuilder::new()
+            .provider_arc(Arc::clone(&self.provider))
+            .system(system)
+            .build()
+            .expect("AgentFactory::spawn is infallible: provider is always set")
+    }
+
+    /// Spawn an agent with custom builder configuration.
+    ///
+    /// Use this escape hatch when you need more than just a system prompt
+    /// (e.g., adding tools, setting memory, configuring hooks).
+    ///
+    /// The closure receives an [`AgentBuilder`] with the factory's provider
+    /// already set. Call builder methods as needed.
+    ///
+    /// # Example
+    ///
+    /// ```rust,no_run
+    /// use traitclaw_core::factory::AgentFactory;
+    /// use traitclaw_core::traits::provider::Provider;
+    ///
+    /// # fn example(provider: impl Provider) -> traitclaw_core::Result<()> {
+    /// let factory = AgentFactory::new(provider);
+    /// let agent = factory.spawn_with(|b| {
+    ///     b.system("You are a researcher with tools.")
+    ///      .max_iterations(10)
+    /// })?;
+    /// # Ok(())
+    /// # }
+    /// ```
+    ///
+    /// # Errors
+    ///
+    /// Returns an error if the builder customization produces an invalid
+    /// agent configuration.
+    pub fn spawn_with(&self, f: impl FnOnce(AgentBuilder) -> AgentBuilder) -> Result<Agent> {
+        let builder = AgentBuilder::new().provider_arc(Arc::clone(&self.provider));
+        f(builder).build()
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::types::completion::{CompletionRequest, CompletionResponse, ResponseContent, Usage};
+    use crate::types::model_info::{ModelInfo, ModelTier};
+    use crate::types::stream::CompletionStream;
+    use async_trait::async_trait;
+
+    #[derive(Clone)]
+    struct MockCloneProvider {
+        info: ModelInfo,
+    }
+
+    impl MockCloneProvider {
+        fn new() -> Self {
+            Self {
+                info: ModelInfo::new("mock-clone", ModelTier::Small, 4_096, false, false, false),
+            }
+        }
+    }
+
+    #[async_trait]
+    impl Provider for MockCloneProvider {
+        async fn complete(&self, _req: CompletionRequest) -> crate::Result<CompletionResponse> {
+            Ok(CompletionResponse {
+                content: ResponseContent::Text("ok".into()),
+                usage: Usage {
+                    prompt_tokens: 1,
+                    completion_tokens: 1,
+                    total_tokens: 2,
+                },
+            })
+        }
+        async fn stream(&self, _req: CompletionRequest) -> crate::Result<CompletionStream> {
+            unimplemented!()
+        }
+        fn model_info(&self) -> &ModelInfo {
+            &self.info
+        }
+    }
+
+    #[test]
+    fn test_factory_new() {
+        let factory = AgentFactory::new(MockCloneProvider::new());
+        assert_eq!(factory.provider.model_info().name, "mock-clone");
+    }
+
+    #[test]
+    fn test_factory_from_arc() {
+        let provider: Arc<dyn Provider> = Arc::new(MockCloneProvider::new());
+        let factory = AgentFactory::from_arc(provider);
+        assert_eq!(factory.provider.model_info().name, "mock-clone");
+    }
+
+    #[test]
+    fn test_factory_spawn_creates_agent_with_system_prompt() {
+        let factory = AgentFactory::new(MockCloneProvider::new());
+        let agent = factory.spawn("You are a researcher.");
+        assert_eq!(
+            agent.config.system_prompt.as_deref(),
+            Some("You are a researcher.")
+        );
+    }
+
+    #[test]
+    fn test_factory_spawn_produces_independent_agents() {
+        let factory = AgentFactory::new(MockCloneProvider::new());
+        let agent_a = factory.spawn("Agent A");
+        let agent_b = factory.spawn("Agent B");
+
+        assert_eq!(agent_a.config.system_prompt.as_deref(), Some("Agent A"));
+        assert_eq!(agent_b.config.system_prompt.as_deref(), Some("Agent B"));
+        // Both have the same provider model
+        assert_eq!(agent_a.provider.model_info().name, "mock-clone");
+        assert_eq!(agent_b.provider.model_info().name, "mock-clone");
+    }
+
+    #[test]
+    fn test_factory_spawn_with_custom_config() {
+        let factory = AgentFactory::new(MockCloneProvider::new());
+        let agent = factory
+            .spawn_with(|b| b.system("Custom").max_iterations(5))
+            .expect("spawn_with should succeed");
+
+        assert_eq!(agent.config.system_prompt.as_deref(), Some("Custom"));
+        assert_eq!(agent.config.max_iterations, 5);
+    }
+
+    #[test]
+    fn test_factory_spawn_with_no_system() {
+        let factory = AgentFactory::new(MockCloneProvider::new());
+        let agent = factory
+            .spawn_with(|b| b.max_iterations(3))
+            .expect("spawn_with without system should succeed");
+
+        assert!(agent.config.system_prompt.is_none());
+    }
+
+    // Compile-time check: AgentFactory is Send + Sync
+    fn _assert_send_sync<T: Send + Sync>() {}
+    #[test]
+    fn test_factory_is_send_sync() {
+        _assert_send_sync::<AgentFactory>();
+    }
+}
diff --git a/crates/traitclaw-core/src/lib.rs b/crates/traitclaw-core/src/lib.rs
index de757b8..3093350 100644
--- a/crates/traitclaw-core/src/lib.rs
+++ b/crates/traitclaw-core/src/lib.rs
@@ -38,7 +38,9 @@ pub mod config;
 pub mod context_managers;
 pub mod default_strategy;
 pub mod error;
+pub mod factory;
 pub mod memory;
+pub mod pool;
 pub mod registries;
 pub mod retry;
 pub(crate) mod runtime;
@@ -106,6 +108,8 @@ pub use memory::in_memory::InMemoryMemory;
 // Re-export agent
 pub use agent::{Agent, AgentOutput, AgentOutputContent, AgentSession, RunUsage};
 pub use agent_builder::AgentBuilder;
+pub use factory::AgentFactory;
+pub use pool::AgentPool;
 
 /// Prelude module for convenient imports.
 ///
@@ -159,6 +163,10 @@ pub mod prelude {
     pub use crate::agent::{Agent, AgentOutput, AgentOutputContent, AgentSession, RunUsage};
     pub use crate::agent_builder::AgentBuilder;
 
+    // v0.6.0: Composition APIs
+    pub use crate::factory::AgentFactory;
+    pub use crate::pool::AgentPool;
+
     // v0.2.0: Strategy & Hook
     pub use crate::default_strategy::DefaultStrategy;
     pub use crate::traits::hook::{AgentHook, HookAction, LoggingHook};
diff --git a/crates/traitclaw-core/src/pool.rs b/crates/traitclaw-core/src/pool.rs
new file mode 100644
index 0000000..da1f96d
--- /dev/null
+++ b/crates/traitclaw-core/src/pool.rs
@@ -0,0 +1,231 @@
+//! Agent pool for managing and executing groups of agents.
+//!
+//! `AgentPool` holds a collection of agents and provides methods for
+//! sequential pipeline execution (output chaining).
+
+use crate::agent::Agent;
+use crate::agent::AgentOutput;
+use crate::Result;
+
+/// A collection of agents for group execution.
+///
+/// `AgentPool` takes ownership of a `Vec<Agent>` and provides
+/// sequential pipeline execution where each agent's output feeds
+/// into the next agent's input.
+///
+/// # Example
+///
+/// ```rust,no_run
+/// use traitclaw_core::pool::AgentPool;
+/// use traitclaw_core::agent::Agent;
+///
+/// # fn example(agents: Vec<Agent>) {
+/// let pool = AgentPool::new(agents);
+/// assert_eq!(pool.len(), 3);
+/// # }
+/// ```
+pub struct AgentPool {
+    agents: Vec<Agent>,
+}
+
+impl std::fmt::Debug for AgentPool {
+    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
+        f.debug_struct("AgentPool")
+            .field("len", &self.agents.len())
+            .finish()
+    }
+}
+
+impl AgentPool {
+    /// Create a new pool from a vector of agents.
+    #[must_use]
+    pub fn new(agents: Vec<Agent>) -> Self {
+        Self { agents }
+    }
+
+    /// Returns the number of agents in the pool.
+    #[must_use]
+    pub fn len(&self) -> usize {
+        self.agents.len()
+    }
+
+    /// Returns `true` if the pool contains no agents.
+    #[must_use]
+    pub fn is_empty(&self) -> bool {
+        self.agents.is_empty()
+    }
+
+    /// Get a reference to an agent by index.
+    ///
+    /// Returns `None` if the index is out of bounds.
+    #[must_use]
+    pub fn get(&self, index: usize) -> Option<&Agent> {
+        self.agents.get(index)
+    }
+
+    /// Run agents sequentially, chaining outputs.
+    ///
+    /// Each agent receives the previous agent's text output as input.
+    /// The first agent receives the provided `input` string.
+    ///
+    /// # Example
+    ///
+    /// ```rust,no_run
+    /// use traitclaw_core::pool::AgentPool;
+    /// use traitclaw_core::agent::Agent;
+    ///
+    /// # async fn example(pool: &AgentPool) -> traitclaw_core::Result<()> {
+    /// let output = pool.run_sequential("Research Rust async patterns").await?;
+    /// println!("Final output: {}", output.text());
+    /// # Ok(())
+    /// # }
+    /// ```
+    ///
+    /// # Errors
+    ///
+    /// Returns an error immediately if any agent in the pipeline fails.
+    /// Earlier agents' outputs are not available on error.
+    pub async fn run_sequential(&self, input: &str) -> Result<AgentOutput> {
+        if self.agents.is_empty() {
+            return Err(crate::Error::Runtime(
+                "AgentPool::run_sequential called on empty pool".into(),
+            ));
+        }
+
+        let mut current_input = input.to_string();
+        let mut last_output: Option<AgentOutput> = None;
+
+        for agent in &self.agents {
+            let output = agent.run(&current_input).await?;
+            current_input = output.text().to_string();
+            last_output = Some(output);
+        }
+
+        // SAFETY: We checked is_empty above, so last_output is always Some
+        Ok(last_output.expect("pool is non-empty"))
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::traits::provider::Provider;
+    use crate::types::completion::{CompletionRequest, CompletionResponse, ResponseContent, Usage};
+    use crate::types::model_info::{ModelInfo, ModelTier};
+    use crate::types::stream::CompletionStream;
+    use async_trait::async_trait;
+    use std::sync::atomic::{AtomicUsize, Ordering};
+    use std::sync::Arc;
+
+    struct EchoProvider {
+        info: ModelInfo,
+        prefix: String,
+        call_count: Arc<AtomicUsize>,
+    }
+
+    impl EchoProvider {
+        fn new(prefix: &str) -> Self {
+            Self {
+                info: ModelInfo::new("echo", ModelTier::Small, 4_096, false, false, false),
+                prefix: prefix.to_string(),
+                call_count: Arc::new(AtomicUsize::new(0)),
+            }
+        }
+    }
+
+    #[async_trait]
+    impl Provider for EchoProvider {
+        async fn complete(&self, req: CompletionRequest) -> crate::Result<CompletionResponse> {
+            self.call_count.fetch_add(1, Ordering::SeqCst);
+            // Echo back the last user message with our prefix
+            let last_msg = req
+                .messages
+                .iter()
+                .rev()
+                .find(|m| m.role == crate::types::message::MessageRole::User)
+                .map(|m| m.content.clone())
+                .unwrap_or_default();
+            Ok(CompletionResponse {
+                content: ResponseContent::Text(format!("[{}] {}", self.prefix, last_msg)),
+                usage: Usage {
+                    prompt_tokens: 1,
+                    completion_tokens: 1,
+                    total_tokens: 2,
+                },
+            })
+        }
+        async fn stream(&self, _req: CompletionRequest) -> crate::Result<CompletionStream> {
+            unimplemented!()
+        }
+        fn model_info(&self) -> &ModelInfo {
+            &self.info
+        }
+    }
+
+    #[test]
+    fn test_pool_new_and_len() {
+        let agents = vec![
+            Agent::with_system(EchoProvider::new("A"), "Agent A"),
+            Agent::with_system(EchoProvider::new("B"), "Agent B"),
+        ];
+        let pool = AgentPool::new(agents);
+        assert_eq!(pool.len(), 2);
+        assert!(!pool.is_empty());
+    }
+
+    #[test]
+    fn test_pool_empty() {
+        let pool = AgentPool::new(vec![]);
+        assert!(pool.is_empty());
+        assert_eq!(pool.len(), 0);
+    }
+
+    #[test]
+    fn test_pool_get() {
+        let agents = vec![
+            Agent::with_system(EchoProvider::new("A"), "Agent A"),
+            Agent::with_system(EchoProvider::new("B"), "Agent B"),
+            Agent::with_system(EchoProvider::new("C"), "Agent C"),
+        ];
+        let pool = AgentPool::new(agents);
+        assert!(pool.get(0).is_some());
+        assert!(pool.get(1).is_some());
+        assert!(pool.get(2).is_some());
+        assert!(pool.get(5).is_none());
+    }
+
+    #[tokio::test]
+    async fn test_pool_run_sequential_single_agent() {
+        let agents = vec![Agent::with_system(EchoProvider::new("Solo"), "Solo agent")];
+        let pool = AgentPool::new(agents);
+        let output = pool.run_sequential("Hello").await.unwrap();
+        assert_eq!(output.text(), "[Solo] Hello");
+    }
+
+    #[tokio::test]
+    async fn test_pool_run_sequential_pipeline() {
+        let agents = vec![
+            Agent::with_system(EchoProvider::new("R"), "Researcher"),
+            Agent::with_system(EchoProvider::new("W"), "Writer"),
+        ];
+        let pool = AgentPool::new(agents);
+        let output = pool.run_sequential("topic").await.unwrap();
+        // First agent: "[R] topic" → Second agent: "[W] [R] topic"
+        assert_eq!(output.text(), "[W] [R] topic");
+    }
+
+    #[tokio::test]
+    async fn test_pool_run_sequential_empty_pool_errors() {
+        let pool = AgentPool::new(vec![]);
+        let result = pool.run_sequential("anything").await;
+        assert!(result.is_err());
+    }
+
+    #[test]
+    fn test_pool_debug() {
+        let pool = AgentPool::new(vec![Agent::with_system(EchoProvider::new("A"), "A")]);
+        let debug = format!("{pool:?}");
+        assert!(debug.contains("AgentPool"));
+        assert!(debug.contains("len: 1"));
+    }
+}
diff --git a/crates/traitclaw-team/src/group_chat.rs b/crates/traitclaw-team/src/group_chat.rs
new file mode 100644
index 0000000..da2f93a
--- /dev/null
+++ b/crates/traitclaw-team/src/group_chat.rs
@@ -0,0 +1,330 @@
+//! Multi-agent group chat with configurable turn-taking and termination.
+//!
+//! Provides [`RoundRobinGroupChat`] for structured multi-turn conversations
+//! where agents take turns in a fixed order, each seeing the full transcript.
+
+use std::fmt;
+
+use traitclaw_core::agent::Agent;
+use traitclaw_core::types::message::{Message, MessageRole};
+
+// ─────────────────────────────────────────────────────────────────────────────
+// Termination Conditions
+// ─────────────────────────────────────────────────────────────────────────────
+
+/// Trait for determining when a group chat should stop.
+///
+/// Implement this trait for custom termination logic (keyword detection,
+/// quality thresholds, consensus detection, etc.).
+pub trait TerminationCondition: Send + Sync {
+    /// Check whether the chat should terminate.
+    ///
+    /// - `round`: the current round number (0-indexed)
+    /// - `messages`: the full conversation transcript so far
+    fn should_terminate(&self, round: usize, messages: &[Message]) -> bool;
+}
+
+/// Terminate after a fixed number of rounds.
+///
+/// # Example
+///
+/// ```rust
+/// use traitclaw_team::group_chat::MaxRoundsTermination;
+///
+/// let term = MaxRoundsTermination::new(6);
+/// ```
+#[derive(Debug, Clone)]
+pub struct MaxRoundsTermination {
+    max_rounds: usize,
+}
+
+impl MaxRoundsTermination {
+    /// Create a termination condition that stops after `max_rounds` rounds.
+    #[must_use]
+    pub fn new(max_rounds: usize) -> Self {
+        Self { max_rounds }
+    }
+}
+
+impl TerminationCondition for MaxRoundsTermination {
+    fn should_terminate(&self, round: usize, _messages: &[Message]) -> bool {
+        round >= self.max_rounds
+    }
+}
+
+// ─────────────────────────────────────────────────────────────────────────────
+// Group Chat Result
+// ─────────────────────────────────────────────────────────────────────────────
+
+/// The result of a group chat session.
+#[derive(Debug, Clone)]
+pub struct GroupChatResult {
+    /// Full conversation transcript in chronological order.
+    pub transcript: Vec<Message>,
+    /// The final message text produced by the last responding agent.
+    pub final_message: String,
+}
+
+// ─────────────────────────────────────────────────────────────────────────────
+// RoundRobinGroupChat
+// ─────────────────────────────────────────────────────────────────────────────
+
+/// A round-robin group chat where agents take turns responding.
+///
+/// Each agent sees the full conversation history and adds its response.
+/// The chat continues until the termination condition is met.
+///
+/// # Example
+///
+/// ```rust,no_run
+/// use traitclaw_team::group_chat::RoundRobinGroupChat;
+/// use traitclaw_core::agent::Agent;
+///
+/// # async fn example(agents: Vec<Agent>) -> traitclaw_core::Result<()> {
+/// let mut chat = RoundRobinGroupChat::new(agents);
+/// let result = chat.run("Discuss the future of AI").await?;
+/// println!("Transcript has {} messages", result.transcript.len());
+/// println!("Final: {}", result.final_message);
+/// # Ok(())
+/// # }
+/// ```
+pub struct RoundRobinGroupChat {
+    agents: Vec<Agent>,
+    termination: Box<dyn TerminationCondition>,
+}
+
+impl fmt::Debug for RoundRobinGroupChat {
+    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
+        f.debug_struct("RoundRobinGroupChat")
+            .field("agents", &self.agents.len())
+            .finish()
+    }
+}
+
+impl RoundRobinGroupChat {
+    /// Create a new group chat with default termination (`n_agents × 3` rounds).
+    ///
+    /// # Panics
+    ///
+    /// This method cannot panic under normal usage.
+    #[must_use]
+    pub fn new(agents: Vec<Agent>) -> Self {
+        let max_rounds = agents.len().saturating_mul(3).max(1);
+        Self {
+            termination: Box::new(MaxRoundsTermination::new(max_rounds)),
+            agents,
+        }
+    }
+
+    /// Set the maximum number of rounds (convenience method).
+    #[must_use]
+    pub fn with_max_rounds(mut self, n: usize) -> Self {
+        self.termination = Box::new(MaxRoundsTermination::new(n));
+        self
+    }
+
+    /// Set a custom termination condition.
+    #[must_use]
+    pub fn with_termination(mut self, t: impl TerminationCondition + 'static) -> Self {
+        self.termination = Box::new(t);
+        self
+    }
+
+    /// Returns the number of agents in the chat.
+    #[must_use]
+    pub fn len(&self) -> usize {
+        self.agents.len()
+    }
+
+    /// Returns `true` if the chat has no agents.
+    #[must_use]
+    pub fn is_empty(&self) -> bool {
+        self.agents.is_empty()
+    }
+
+    /// Run the group chat starting with the given task prompt.
+    ///
+    /// Agents respond in round-robin order, each seeing the full transcript.
+    /// The chat terminates when the termination condition is met.
+    ///
+    /// # Errors
+    ///
+    /// Returns an error if:
+    /// - The agent pool is empty
+    /// - Any agent fails to produce a response
+    pub async fn run(&self, task: &str) -> traitclaw_core::Result<GroupChatResult> {
+        if self.agents.is_empty() {
+            return Err(traitclaw_core::Error::Runtime(
+                "RoundRobinGroupChat::run() called with no agents".into(),
+            ));
+        }
+
+        let mut transcript = vec![Message {
+            role: MessageRole::User,
+            content: task.to_string(),
+            tool_call_id: None,
+        }];
+
+        let n_agents = self.agents.len();
+        let mut round = 0;
+
+        loop {
+            if self.termination.should_terminate(round, &transcript) {
+                break;
+            }
+
+            let agent_idx = round % n_agents;
+            let agent = &self.agents[agent_idx];
+
+            // Build the context: format transcript as a conversation prompt
+            let context = Self::format_transcript(&transcript);
+            let output = agent.run(&context).await?;
+            let response_text = output.text().to_string();
+
+            transcript.push(Message {
+                role: MessageRole::Assistant,
+                content: response_text,
+                tool_call_id: None,
+            });
+
+            round += 1;
+        }
+
+        let final_message = transcript
+            .last()
+            .map(|m| m.content.clone())
+            .unwrap_or_default();
+
+        Ok(GroupChatResult {
+            transcript,
+            final_message,
+        })
+    }
+
+    /// Format transcript messages into a single context string.
+    fn format_transcript(messages: &[Message]) -> String {
+        messages
+            .iter()
+            .map(|m| {
+                let role = match m.role {
+                    MessageRole::User => "User",
+                    MessageRole::Assistant => "Assistant",
+                    MessageRole::System => "System",
+                    MessageRole::Tool => "Tool",
+                    _ => "Unknown",
+                };
+                format!("[{}]: {}", role, m.content)
+            })
+            .collect::<Vec<_>>()
+            .join("\n\n")
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::tests_common::EchoProvider;
+
+    // ── TerminationCondition ────────────────────────────────────────────────
+
+    #[test]
+    fn test_max_rounds_at_boundary() {
+        let term = MaxRoundsTermination::new(3);
+        assert!(!term.should_terminate(0, &[]));
+        assert!(!term.should_terminate(1, &[]));
+        assert!(!term.should_terminate(2, &[]));
+        assert!(term.should_terminate(3, &[]));
+        assert!(term.should_terminate(4, &[]));
+    }
+
+    #[test]
+    fn test_max_rounds_zero() {
+        let term = MaxRoundsTermination::new(0);
+        assert!(term.should_terminate(0, &[]));
+    }
+
+    // ── RoundRobinGroupChat ─────────────────────────────────────────────────
+
+    #[test]
+    fn test_group_chat_new_default_rounds() {
+        let agents = vec![
+            Agent::with_system(EchoProvider::new("A"), "Agent A"),
+            Agent::with_system(EchoProvider::new("B"), "Agent B"),
+        ];
+        let chat = RoundRobinGroupChat::new(agents);
+        assert_eq!(chat.len(), 2);
+        // Default max_rounds = 2 * 3 = 6
+    }
+
+    #[test]
+    fn test_group_chat_with_max_rounds() {
+        let agents = vec![Agent::with_system(EchoProvider::new("A"), "Agent A")];
+        let chat = RoundRobinGroupChat::new(agents).with_max_rounds(10);
+        assert_eq!(chat.len(), 1);
+    }
+
+    #[tokio::test]
+    async fn test_group_chat_run_basic() {
+        let agents = vec![
+            Agent::with_system(EchoProvider::new("R"), "Researcher"),
+            Agent::with_system(EchoProvider::new("W"), "Writer"),
+        ];
+        let chat = RoundRobinGroupChat::new(agents).with_max_rounds(2);
+        let result = chat.run("Discuss Rust").await.unwrap();
+
+        // Initial user message + 2 agent responses = 3 messages
+        assert_eq!(result.transcript.len(), 3);
+        assert!(!result.final_message.is_empty());
+    }
+
+    #[tokio::test]
+    async fn test_group_chat_round_robin_order() {
+        let agents = vec![
+            Agent::with_system(EchoProvider::new("FIRST"), "First"),
+            Agent::with_system(EchoProvider::new("SECOND"), "Second"),
+        ];
+        let chat = RoundRobinGroupChat::new(agents).with_max_rounds(4);
+        let result = chat.run("Test").await.unwrap();
+
+        // 5 messages: 1 user + 4 agent responses
+        assert_eq!(result.transcript.len(), 5);
+        // Check round-robin order by prefix
+        assert!(result.transcript[1].content.contains("[FIRST]"));
+        assert!(result.transcript[2].content.contains("[SECOND]"));
+        assert!(result.transcript[3].content.contains("[FIRST]"));
+        assert!(result.transcript[4].content.contains("[SECOND]"));
+    }
+
+    #[tokio::test]
+    async fn test_group_chat_empty_agents_returns_error() {
+        let chat = RoundRobinGroupChat::new(vec![]);
+        let result = chat.run("Test").await;
+        assert!(result.is_err());
+    }
+
+    #[tokio::test]
+    async fn test_group_chat_custom_termination() {
+        // Custom termination: stop when any message contains "DONE"
+        struct ContainsKeyword;
+        impl TerminationCondition for ContainsKeyword {
+            fn should_terminate(&self, _round: usize, messages: &[Message]) -> bool {
+                messages.iter().any(|m| m.content.contains("DONE"))
+            }
+        }
+
+        let agents = vec![Agent::with_system(EchoProvider::new("DONE"), "Agent")];
+        let chat = RoundRobinGroupChat::new(agents).with_termination(ContainsKeyword);
+        let result = chat.run("Test").await.unwrap();
+
+        // Should stop after first agent response (contains "DONE")
+        // 1 user + 1 agent = 2 messages
+        assert_eq!(result.transcript.len(), 2);
+    }
+
+    #[test]
+    fn test_group_chat_debug() {
+        let chat = RoundRobinGroupChat::new(vec![Agent::with_system(EchoProvider::new("A"), "A")]);
+        let debug = format!("{chat:?}");
+        assert!(debug.contains("RoundRobinGroupChat"));
+    }
+}
diff --git a/crates/traitclaw-team/src/lib.rs b/crates/traitclaw-team/src/lib.rs
index 1c273b7..0e5cc2b 100644
--- a/crates/traitclaw-team/src/lib.rs
+++ b/crates/traitclaw-team/src/lib.rs
@@ -25,15 +25,109 @@
 
 pub mod conditional_router;
 pub mod execution;
+pub mod group_chat;
 pub mod router;
 pub mod team_context;
 
+#[cfg(test)]
+pub(crate) mod tests_common;
+
 use serde::{Deserialize, Serialize};
+use std::sync::Arc;
+use traitclaw_core::traits::provider::Provider;
 
 pub use conditional_router::ConditionalRouter;
 pub use execution::{run_verification_chain, TeamRunner};
 pub use team_context::TeamContext;
 
+/// Create an [`AgentPool`](traitclaw_core::pool::AgentPool) from a [`Team`] and a provider.
+///
+/// Each role's `system_prompt` is used as the agent's system prompt.
+/// Roles without a `system_prompt` cause an error listing all missing roles.
+///
+/// # Example
+///
+/// ```rust
+/// use traitclaw_team::{AgentRole, Team, pool_from_team};
+/// use traitclaw_core::traits::provider::Provider;
+///
+/// # fn example(provider: impl Provider) -> traitclaw_core::Result<()> {
+/// let team = Team::new("content_team")
+///     .add_role(AgentRole::new("researcher", "Research").with_system_prompt("You research topics."))
+///     .add_role(AgentRole::new("writer", "Write").with_system_prompt("You write articles."));
+///
+/// let pool = pool_from_team(&team, provider)?;
+/// assert_eq!(pool.len(), 2);
+/// # Ok(())
+/// # }
+/// ```
+///
+/// # Errors
+///
+/// Returns an error if any role in the team is missing a `system_prompt`.
+pub fn pool_from_team(
+    team: &Team,
+    provider: impl Provider,
+) -> traitclaw_core::Result<traitclaw_core::pool::AgentPool> {
+    // Check for missing system_prompts first
+    let missing: Vec<&str> = team
+        .roles()
+        .iter()
+        .filter(|r| r.system_prompt.is_none())
+        .map(|r| r.name.as_str())
+        .collect();
+
+    if !missing.is_empty() {
+        return Err(traitclaw_core::Error::Config(format!(
+            "Cannot create AgentPool from team '{}': roles missing system_prompt: [{}]",
+            team.name(),
+            missing.join(", ")
+        )));
+    }
+
+    let factory = traitclaw_core::factory::AgentFactory::new(provider);
+    let agents: Vec<traitclaw_core::Agent> = team
+        .roles()
+        .iter()
+        .map(|role| factory.spawn(role.system_prompt.as_ref().expect("checked above")))
+        .collect();
+
+    Ok(traitclaw_core::pool::AgentPool::new(agents))
+}
+
+/// Create an [`AgentPool`](traitclaw_core::pool::AgentPool) from a [`Team`]
+/// using a pre-wrapped `Arc<dyn Provider>`.
+///
+/// Same as [`pool_from_team`] but accepts a shared provider reference.
+pub fn pool_from_team_arc(
+    team: &Team,
+    provider: Arc<dyn Provider>,
+) -> traitclaw_core::Result<traitclaw_core::pool::AgentPool> {
+    let missing: Vec<&str> = team
+        .roles()
+        .iter()
+        .filter(|r| r.system_prompt.is_none())
+        .map(|r| r.name.as_str())
+        .collect();
+
+    if !missing.is_empty() {
+        return Err(traitclaw_core::Error::Config(format!(
+            "Cannot create AgentPool from team '{}': roles missing system_prompt: [{}]",
+            team.name(),
+            missing.join(", ")
+        )));
+    }
+
+    let factory = traitclaw_core::factory::AgentFactory::from_arc(provider);
+    let agents: Vec<traitclaw_core::Agent> = team
+        .roles()
+        .iter()
+        .map(|role| factory.spawn(role.system_prompt.as_ref().expect("checked above")))
+        .collect();
+
+    Ok(traitclaw_core::pool::AgentPool::new(agents))
+}
+
 /// A team of agents working together.
 pub struct Team {
     name: String,
diff --git a/crates/traitclaw-team/src/tests_common.rs b/crates/traitclaw-team/src/tests_common.rs
new file mode 100644
index 0000000..e8b9f4e
--- /dev/null
+++ b/crates/traitclaw-team/src/tests_common.rs
@@ -0,0 +1,53 @@
+//! Shared test utilities for traitclaw-team tests.
+
+use async_trait::async_trait;
+use traitclaw_core::traits::provider::Provider;
+use traitclaw_core::types::completion::{
+    CompletionRequest, CompletionResponse, ResponseContent, Usage,
+};
+use traitclaw_core::types::message::MessageRole;
+use traitclaw_core::types::model_info::{ModelInfo, ModelTier};
+use traitclaw_core::types::stream::CompletionStream;
+
+/// A provider that echoes back the last user message with a prefix.
+pub struct EchoProvider {
+    info: ModelInfo,
+    prefix: String,
+}
+
+impl EchoProvider {
+    /// Create a new echo provider with the given prefix.
+    pub fn new(prefix: &str) -> Self {
+        Self {
+            info: ModelInfo::new("echo", ModelTier::Small, 4_096, false, false, false),
+            prefix: prefix.to_string(),
+        }
+    }
+}
+
+#[async_trait]
+impl Provider for EchoProvider {
+    async fn complete(&self, req: CompletionRequest) -> traitclaw_core::Result<CompletionResponse> {
+        let last_msg = req
+            .messages
+            .iter()
+            .rev()
+            .find(|m| m.role == MessageRole::User)
+            .map(|m| m.content.clone())
+            .unwrap_or_default();
+        Ok(CompletionResponse {
+            content: ResponseContent::Text(format!("[{}] {}", self.prefix, last_msg)),
+            usage: Usage {
+                prompt_tokens: 1,
+                completion_tokens: 1,
+                total_tokens: 2,
+            },
+        })
+    }
+    async fn stream(&self, _req: CompletionRequest) -> traitclaw_core::Result<CompletionStream> {
+        unimplemented!()
+    }
+    fn model_info(&self) -> &ModelInfo {
+        &self.info
+    }
+}

```
