//! Runtime construction helpers for strategy tests.
//!
//! [`make_runtime`] constructs a fully-configured [`AgentRuntime`]
//! with sensible defaults, reducing test setup to a single function call.
//!
//! # Example
//!
//! ```rust
//! use traitclaw_test_utils::provider::MockProvider;
//! use traitclaw_test_utils::runtime::make_runtime;
//!
//! let rt = make_runtime(MockProvider::text("ok"), vec![]);
//! // rt is ready for strategy.execute(&rt, "input", "session-1")
//! ```

use std::sync::Arc;

use async_trait::async_trait;

use traitclaw_core::config::AgentConfig;
use traitclaw_core::traits::context_manager::ContextManager;
use traitclaw_core::traits::output_transformer::OutputTransformer;
use traitclaw_core::traits::provider::Provider;
use traitclaw_core::traits::strategy::AgentRuntime;
use traitclaw_core::traits::tool::ErasedTool;
use traitclaw_core::traits::tool_registry::SimpleRegistry;
use traitclaw_core::traits::tracker::Tracker;
use traitclaw_core::types::agent_state::AgentState;
use traitclaw_core::types::completion::CompletionResponse;
use traitclaw_core::types::message::Message;

use crate::memory::MockMemory;

// ── Noop Implementations (pub(crate) only) ──────────────────────────────

pub(crate) struct NoopTracker;

impl Tracker for NoopTracker {
    fn on_iteration(&self, _state: &mut AgentState) {}
    fn on_tool_call(&self, _name: &str, _args: &serde_json::Value, _state: &mut AgentState) {}
    fn on_llm_response(&self, _response: &CompletionResponse, _state: &mut AgentState) {}
    fn recommended_concurrency(&self, _state: &AgentState) -> usize {
        usize::MAX
    }
}

pub(crate) struct NoopContextManager;

#[async_trait]
impl ContextManager for NoopContextManager {
    async fn prepare(
        &self,
        _messages: &mut Vec<Message>,
        _context_window: usize,
        _state: &mut AgentState,
    ) {
    }
}

pub(crate) struct NoopOutputTransformer;

#[async_trait]
impl OutputTransformer for NoopOutputTransformer {
    async fn transform(&self, output: String, _tool_name: &str, _state: &AgentState) -> String {
        output
    }
}

// ── Public API ───────────────────────────────────────────────────────────

/// Create a minimal [`AgentRuntime`] with the given provider and tools.
///
/// All other components use default no-op implementations:
/// - Memory: [`MockMemory`]
/// - Tracker: no-op
/// - Context/Output: pass-through
/// - Guards/Hints/Hooks: empty
/// - Tool execution: sequential
/// - Config: [`AgentConfig::default()`]
///
/// # Example
///
/// ```rust
/// use std::sync::Arc;
/// use traitclaw_test_utils::provider::MockProvider;
/// use traitclaw_test_utils::runtime::make_runtime;
///
/// let rt = make_runtime(MockProvider::text("hello"), vec![]);
/// assert_eq!(rt.config.max_iterations, 20);
/// ```
pub fn make_runtime(
    provider: impl Provider + 'static,
    tools: Vec<Arc<dyn ErasedTool>>,
) -> AgentRuntime {
    make_runtime_with_config(provider, tools, AgentConfig::default())
}

/// Create a minimal [`AgentRuntime`] with custom [`AgentConfig`].
///
/// Same as [`make_runtime`], but accepts a custom config for tests
/// that need non-default settings (e.g., `max_turns`, `model`).
///
/// # Example
///
/// ```rust
/// use std::sync::Arc;
/// use traitclaw_core::config::AgentConfig;
/// use traitclaw_test_utils::provider::MockProvider;
/// use traitclaw_test_utils::runtime::make_runtime_with_config;
///
/// let mut config = AgentConfig::default();
/// config.max_iterations = 5;
/// let rt = make_runtime_with_config(MockProvider::text("ok"), vec![], config);
/// assert_eq!(rt.config.max_iterations, 5);
/// ```
pub fn make_runtime_with_config(
    provider: impl Provider + 'static,
    tools: Vec<Arc<dyn ErasedTool>>,
    config: AgentConfig,
) -> AgentRuntime {
    AgentRuntime {
        provider: Arc::new(provider),
        tools: tools.clone(),
        memory: Arc::new(MockMemory::new()),
        guards: vec![],
        hints: vec![],
        tracker: Arc::new(NoopTracker),
        context_manager: Arc::new(NoopContextManager),
        execution_strategy: Arc::new(traitclaw_core::SequentialStrategy),
        output_transformer: Arc::new(NoopOutputTransformer),
        tool_registry: Arc::new(SimpleRegistry::new(tools)),
        config,
        hooks: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::MockProvider;

    #[test]
    fn test_make_runtime_returns_valid_runtime() {
        let rt = make_runtime(MockProvider::text("ok"), vec![]);
        assert!(rt.guards.is_empty());
        assert!(rt.hints.is_empty());
        assert!(rt.hooks.is_empty());
        assert!(rt.tools.is_empty());
    }

    #[test]
    fn test_make_runtime_uses_default_config() {
        let rt = make_runtime(MockProvider::text("ok"), vec![]);
        assert_eq!(rt.config.max_iterations, 20);
    }

    #[test]
    fn test_make_runtime_with_config_applies_custom_config() {
        let mut config = AgentConfig::default();
        config.max_iterations = 3;
        let rt = make_runtime_with_config(MockProvider::text("ok"), vec![], config);
        assert_eq!(rt.config.max_iterations, 3);
    }

    #[test]
    fn test_make_runtime_with_tools() {
        use crate::tools::EchoTool;

        let echo: Arc<dyn ErasedTool> = Arc::new(EchoTool);
        let rt = make_runtime(MockProvider::text("ok"), vec![echo]);
        assert_eq!(rt.tools.len(), 1);
    }
}
