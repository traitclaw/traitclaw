//! Shared test utilities for strategy tests.
//!
//! Provides mock implementations of `Provider`, `Memory`, and helper
//! functions to create `AgentRuntime` instances for deterministic testing.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_trait::async_trait;

use traitclaw_core::config::AgentConfig;
use traitclaw_core::traits::context_manager::ContextManager;
use traitclaw_core::traits::memory::MemoryEntry;
use traitclaw_core::traits::output_transformer::OutputTransformer;
use traitclaw_core::traits::provider::Provider;
use traitclaw_core::traits::strategy::AgentRuntime;
use traitclaw_core::traits::tool::ErasedTool;
use traitclaw_core::traits::tool_registry::SimpleRegistry;
use traitclaw_core::traits::tracker::Tracker;
use traitclaw_core::types::agent_state::AgentState;
use traitclaw_core::types::completion::{
    CompletionRequest, CompletionResponse, ResponseContent, Usage,
};
use traitclaw_core::types::message::Message;
use traitclaw_core::types::model_info::{ModelInfo, ModelTier};
use traitclaw_core::types::stream::CompletionStream;
use traitclaw_core::types::tool_call::ToolCall;
use traitclaw_core::{Memory, Result};

// ── Mock Provider ────────────────────────────────────────────────────────

/// Deterministic mock provider that returns responses in sequence.
pub(crate) struct MockProvider {
    pub(crate) info: ModelInfo,
    pub(crate) responses: Vec<CompletionResponse>,
    pub(crate) call_idx: AtomicUsize,
}

impl MockProvider {
    /// Create a provider that always returns a single text response.
    pub(crate) fn text(text: &str) -> Self {
        Self {
            info: ModelInfo::new("test-model", ModelTier::Small, 4096, false, false, false),
            responses: vec![CompletionResponse {
                content: ResponseContent::Text(text.into()),
                usage: Usage {
                    prompt_tokens: 10,
                    completion_tokens: 5,
                    total_tokens: 15,
                },
            }],
            call_idx: AtomicUsize::new(0),
        }
    }

    /// Create a provider with explicit responses sequence.
    pub(crate) fn with_responses(responses: Vec<CompletionResponse>) -> Self {
        Self {
            info: ModelInfo::new("test-model", ModelTier::Small, 4096, true, false, false),
            responses,
            call_idx: AtomicUsize::new(0),
        }
    }

    /// Create a provider that returns tool calls on first call, then text.
    pub(crate) fn tool_then_text(tool_calls: Vec<ToolCall>, final_text: &str) -> Self {
        Self::with_responses(vec![
            CompletionResponse {
                content: ResponseContent::ToolCalls(tool_calls),
                usage: Usage {
                    prompt_tokens: 10,
                    completion_tokens: 5,
                    total_tokens: 15,
                },
            },
            CompletionResponse {
                content: ResponseContent::Text(final_text.into()),
                usage: Usage {
                    prompt_tokens: 10,
                    completion_tokens: 5,
                    total_tokens: 15,
                },
            },
        ])
    }

    /// Create a provider that always returns tool calls (never text).
    pub(crate) fn always_tool_calls(tool_calls: Vec<ToolCall>) -> Self {
        Self {
            info: ModelInfo::new("test-model", ModelTier::Small, 4096, true, false, false),
            responses: vec![CompletionResponse {
                content: ResponseContent::ToolCalls(tool_calls),
                usage: Usage {
                    prompt_tokens: 10,
                    completion_tokens: 5,
                    total_tokens: 15,
                },
            }],
            call_idx: AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl Provider for MockProvider {
    async fn complete(&self, _req: CompletionRequest) -> Result<CompletionResponse> {
        let idx = self.call_idx.fetch_add(1, Ordering::SeqCst);
        Ok(self.responses[idx.min(self.responses.len() - 1)].clone())
    }
    async fn stream(&self, _req: CompletionRequest) -> Result<CompletionStream> {
        unimplemented!()
    }
    fn model_info(&self) -> &ModelInfo {
        &self.info
    }
}

// ── Mock Memory ──────────────────────────────────────────────────────────

pub(crate) struct MockMemory {
    messages: tokio::sync::Mutex<Vec<Message>>,
}

impl MockMemory {
    pub(crate) fn new() -> Self {
        Self {
            messages: tokio::sync::Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl Memory for MockMemory {
    async fn messages(&self, _session_id: &str) -> Result<Vec<Message>> {
        Ok(self.messages.lock().await.clone())
    }
    async fn append(&self, _session_id: &str, message: Message) -> Result<()> {
        self.messages.lock().await.push(message);
        Ok(())
    }
    async fn get_context(
        &self,
        _session_id: &str,
        _key: &str,
    ) -> Result<Option<serde_json::Value>> {
        Ok(None)
    }
    async fn set_context(
        &self,
        _session_id: &str,
        _key: &str,
        _value: serde_json::Value,
    ) -> Result<()> {
        Ok(())
    }
    async fn recall(&self, _query: &str, _limit: usize) -> Result<Vec<MemoryEntry>> {
        Ok(vec![])
    }
    async fn store(&self, _entry: MemoryEntry) -> Result<()> {
        Ok(())
    }
}

// ── Mock Noop Implementations ────────────────────────────────────────────

struct NoopTracker;

impl Tracker for NoopTracker {
    fn on_iteration(&self, _state: &mut AgentState) {}
    fn on_tool_call(&self, _name: &str, _args: &serde_json::Value, _state: &mut AgentState) {}
    fn on_llm_response(&self, _response: &CompletionResponse, _state: &mut AgentState) {}
    fn recommended_concurrency(&self, _state: &AgentState) -> usize {
        usize::MAX
    }
}

struct NoopContextManager;

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

struct NoopOutputTransformer;

#[async_trait]
impl OutputTransformer for NoopOutputTransformer {
    async fn transform(&self, output: String, _tool_name: &str, _state: &AgentState) -> String {
        output
    }
}

// ── Runtime Builder ──────────────────────────────────────────────────────

/// Create a minimal `AgentRuntime` with the given provider and tools.
#[allow(deprecated)]
pub(crate) fn make_runtime(
    provider: impl Provider + 'static,
    tools: Vec<Arc<dyn ErasedTool>>,
) -> AgentRuntime {
    AgentRuntime {
        provider: Arc::new(provider),
        tools: tools.clone(),
        memory: Arc::new(MockMemory::new()),
        guards: vec![],
        hints: vec![],
        tracker: Arc::new(NoopTracker),
        context_manager: Arc::new(NoopContextManager),
        context_strategy: Arc::new(traitclaw_core::NoopContextStrategy),
        execution_strategy: Arc::new(traitclaw_core::SequentialStrategy),
        output_transformer: Arc::new(NoopOutputTransformer),
        output_processor: Arc::new(traitclaw_core::NoopProcessor),
        tool_registry: Arc::new(SimpleRegistry::new(tools)),
        config: AgentConfig::default(),
        hooks: vec![],
    }
}
