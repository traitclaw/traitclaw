//! Agent pool for managing and executing groups of agents.
//!
//! `AgentPool` holds a collection of agents and provides methods for
//! sequential pipeline execution (output chaining).

use crate::agent::Agent;
use crate::agent::AgentOutput;
use crate::Result;

/// A collection of agents for group execution.
///
/// `AgentPool` takes ownership of a `Vec<Agent>` and provides
/// sequential pipeline execution where each agent's output feeds
/// into the next agent's input.
///
/// # Example
///
/// ```rust,no_run
/// use traitclaw_core::pool::AgentPool;
/// use traitclaw_core::agent::Agent;
///
/// # fn example(agents: Vec<Agent>) {
/// let pool = AgentPool::new(agents);
/// assert_eq!(pool.len(), 3);
/// # }
/// ```
pub struct AgentPool {
    agents: Vec<Agent>,
}

impl std::fmt::Debug for AgentPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentPool")
            .field("len", &self.agents.len())
            .finish()
    }
}

impl AgentPool {
    /// Create a new pool from a vector of agents.
    #[must_use]
    pub fn new(agents: Vec<Agent>) -> Self {
        Self { agents }
    }

    /// Returns the number of agents in the pool.
    #[must_use]
    pub fn len(&self) -> usize {
        self.agents.len()
    }

    /// Returns `true` if the pool contains no agents.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }

    /// Get a reference to an agent by index.
    ///
    /// Returns `None` if the index is out of bounds.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&Agent> {
        self.agents.get(index)
    }

    /// Run agents sequentially, chaining outputs.
    ///
    /// Each agent receives the previous agent's text output as input.
    /// The first agent receives the provided `input` string.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use traitclaw_core::pool::AgentPool;
    /// use traitclaw_core::agent::Agent;
    ///
    /// # async fn example(pool: &AgentPool) -> traitclaw_core::Result<()> {
    /// let output = pool.run_sequential("Research Rust async patterns").await?;
    /// println!("Final output: {}", output.text());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error immediately if any agent in the pipeline fails.
    /// Earlier agents' outputs are not available on error.
    pub async fn run_sequential(&self, input: &str) -> Result<AgentOutput> {
        if self.agents.is_empty() {
            return Err(crate::Error::Runtime(
                "AgentPool::run_sequential called on empty pool".into(),
            ));
        }

        let mut current_input = input.to_string();
        let mut last_output: Option<AgentOutput> = None;

        for agent in &self.agents {
            let output = agent.run(&current_input).await?;
            current_input = output.text().to_string();
            last_output = Some(output);
        }

        // SAFETY: We checked is_empty above, so last_output is always Some
        Ok(last_output.expect("pool is non-empty"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::provider::Provider;
    use crate::types::completion::{CompletionRequest, CompletionResponse, ResponseContent, Usage};
    use crate::types::model_info::{ModelInfo, ModelTier};
    use crate::types::stream::CompletionStream;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct EchoProvider {
        info: ModelInfo,
        prefix: String,
        call_count: Arc<AtomicUsize>,
    }

    impl EchoProvider {
        fn new(prefix: &str) -> Self {
            Self {
                info: ModelInfo::new("echo", ModelTier::Small, 4_096, false, false, false),
                prefix: prefix.to_string(),
                call_count: Arc::new(AtomicUsize::new(0)),
            }
        }
    }

    #[async_trait]
    impl Provider for EchoProvider {
        async fn complete(&self, req: CompletionRequest) -> crate::Result<CompletionResponse> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            // Echo back the last user message with our prefix
            let last_msg = req
                .messages
                .iter()
                .rev()
                .find(|m| m.role == crate::types::message::MessageRole::User)
                .map(|m| m.content.clone())
                .unwrap_or_default();
            Ok(CompletionResponse {
                content: ResponseContent::Text(format!("[{}] {}", self.prefix, last_msg)),
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
    fn test_pool_new_and_len() {
        let agents = vec![
            Agent::with_system(EchoProvider::new("A"), "Agent A"),
            Agent::with_system(EchoProvider::new("B"), "Agent B"),
        ];
        let pool = AgentPool::new(agents);
        assert_eq!(pool.len(), 2);
        assert!(!pool.is_empty());
    }

    #[test]
    fn test_pool_empty() {
        let pool = AgentPool::new(vec![]);
        assert!(pool.is_empty());
        assert_eq!(pool.len(), 0);
    }

    #[test]
    fn test_pool_get() {
        let agents = vec![
            Agent::with_system(EchoProvider::new("A"), "Agent A"),
            Agent::with_system(EchoProvider::new("B"), "Agent B"),
            Agent::with_system(EchoProvider::new("C"), "Agent C"),
        ];
        let pool = AgentPool::new(agents);
        assert!(pool.get(0).is_some());
        assert!(pool.get(1).is_some());
        assert!(pool.get(2).is_some());
        assert!(pool.get(5).is_none());
    }

    #[tokio::test]
    async fn test_pool_run_sequential_single_agent() {
        let agents = vec![Agent::with_system(EchoProvider::new("Solo"), "Solo agent")];
        let pool = AgentPool::new(agents);
        let output = pool.run_sequential("Hello").await.unwrap();
        assert_eq!(output.text(), "[Solo] Hello");
    }

    #[tokio::test]
    async fn test_pool_run_sequential_pipeline() {
        let agents = vec![
            Agent::with_system(EchoProvider::new("R"), "Researcher"),
            Agent::with_system(EchoProvider::new("W"), "Writer"),
        ];
        let pool = AgentPool::new(agents);
        let output = pool.run_sequential("topic").await.unwrap();
        // First agent: "[R] topic" → Second agent: "[W] [R] topic"
        assert_eq!(output.text(), "[W] [R] topic");
    }

    #[tokio::test]
    async fn test_pool_run_sequential_empty_pool_errors() {
        let pool = AgentPool::new(vec![]);
        let result = pool.run_sequential("anything").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_pool_debug() {
        let pool = AgentPool::new(vec![Agent::with_system(EchoProvider::new("A"), "A")]);
        let debug = format!("{pool:?}");
        assert!(debug.contains("AgentPool"));
        assert!(debug.contains("len: 1"));
    }
}
