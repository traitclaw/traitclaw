//! Tool execution strategies for configurable concurrency.
//!
//! By default, tools are executed sequentially. Use [`ParallelStrategy`] for
//! concurrent execution or [`AdaptiveStrategy`] to let the [`Tracker`] decide.
//!
//! [`Tracker`]: crate::traits::tracker::Tracker

use std::sync::Arc;

use async_trait::async_trait;

use crate::traits::guard::{Guard, GuardResult};
use crate::traits::tool::ErasedTool;
use crate::traits::tracker::Tracker;
use crate::types::action::Action;
use crate::types::agent_state::AgentState;
use crate::types::tool_call::ToolCall;

/// A pending tool call to be executed by a strategy.
#[derive(Debug, Clone)]
pub struct PendingToolCall {
    /// Unique identifier for the tool call.
    pub id: String,
    /// Name of the tool to invoke.
    pub name: String,
    /// JSON arguments for the tool.
    pub arguments: serde_json::Value,
}

impl From<&ToolCall> for PendingToolCall {
    fn from(tc: &ToolCall) -> Self {
        Self {
            id: tc.id.clone(),
            name: tc.name.clone(),
            arguments: tc.arguments.clone(),
        }
    }
}

/// The result of executing a single tool call.
#[derive(Debug, Clone)]
pub struct ToolResult {
    /// ID of the tool call this result corresponds to.
    pub id: String,
    /// Output string (may be an error message if execution failed).
    pub output: String,
}

/// Trait for pluggable tool execution strategies.
///
/// Implementations control how a batch of tool calls are executed —
/// sequentially, in parallel, or with custom logic.
#[async_trait]
pub trait ExecutionStrategy: Send + Sync {
    /// Execute a batch of tool calls and return results.
    async fn execute_batch(
        &self,
        calls: Vec<PendingToolCall>,
        tools: &[Arc<dyn ErasedTool>],
        guards: &[Arc<dyn Guard>],
        state: &AgentState,
    ) -> Vec<ToolResult>;
}

// ───────────────────────────── Sequential ─────────────────────────────

/// Execute tool calls one at a time in order.
///
/// This is the default strategy — safe and predictable.
pub struct SequentialStrategy;

#[async_trait]
impl ExecutionStrategy for SequentialStrategy {
    async fn execute_batch(
        &self,
        calls: Vec<PendingToolCall>,
        tools: &[Arc<dyn ErasedTool>],
        guards: &[Arc<dyn Guard>],
        _state: &AgentState,
    ) -> Vec<ToolResult> {
        let mut results = Vec::with_capacity(calls.len());
        for call in calls {
            let output = execute_single(&call, tools, guards).await;
            results.push(ToolResult {
                id: call.id,
                output,
            });
        }
        results
    }
}

// ───────────────────────────── Parallel ─────────────────────────────

/// Execute tool calls concurrently with bounded concurrency.
pub struct ParallelStrategy {
    /// Maximum number of concurrent tool executions.
    pub max_concurrency: usize,
}

impl ParallelStrategy {
    /// Create a parallel strategy with the given concurrency limit.
    #[must_use]
    pub fn new(max_concurrency: usize) -> Self {
        Self {
            max_concurrency: max_concurrency.max(1),
        }
    }
}

#[async_trait]
impl ExecutionStrategy for ParallelStrategy {
    async fn execute_batch(
        &self,
        calls: Vec<PendingToolCall>,
        tools: &[Arc<dyn ErasedTool>],
        guards: &[Arc<dyn Guard>],
        _state: &AgentState,
    ) -> Vec<ToolResult> {
        use tokio::sync::Semaphore;

        let semaphore = Arc::new(Semaphore::new(self.max_concurrency));
        let tools = Arc::new(tools.to_vec());
        let guards = Arc::new(guards.to_vec());

        // P3 fix: pre-clone call IDs before moving calls into spawned tasks,
        // so we can attribute errors correctly if a task panics.
        let call_ids: Vec<String> = calls.iter().map(|c| c.id.clone()).collect();
        let mut handles = Vec::with_capacity(calls.len());

        for call in calls {
            let sem = semaphore.clone();
            let tools = tools.clone();
            let guards = guards.clone();

            handles.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.expect("semaphore closed");
                let output = execute_single(&call, &tools, &guards).await;
                ToolResult {
                    id: call.id,
                    output,
                }
            }));
        }

        let mut results = Vec::with_capacity(handles.len());
        for (i, handle) in handles.into_iter().enumerate() {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(ToolResult {
                    id: call_ids[i].clone(),
                    output: format!("Error: task panicked: {e}"),
                }),
            }
        }
        results
    }
}

// ───────────────────────────── Adaptive ─────────────────────────────

/// Adaptive strategy that queries [`Tracker::recommended_concurrency()`] to
/// decide whether to run sequentially or in parallel.
pub struct AdaptiveStrategy {
    tracker: Arc<dyn Tracker>,
}

impl AdaptiveStrategy {
    /// Create an adaptive strategy that uses the given tracker.
    #[must_use]
    pub fn new(tracker: Arc<dyn Tracker>) -> Self {
        Self { tracker }
    }
}

#[async_trait]
impl ExecutionStrategy for AdaptiveStrategy {
    async fn execute_batch(
        &self,
        calls: Vec<PendingToolCall>,
        tools: &[Arc<dyn ErasedTool>],
        guards: &[Arc<dyn Guard>],
        state: &AgentState,
    ) -> Vec<ToolResult> {
        let concurrency = self.tracker.recommended_concurrency(state);
        if concurrency <= 1 {
            SequentialStrategy
                .execute_batch(calls, tools, guards, state)
                .await
        } else {
            ParallelStrategy::new(concurrency)
                .execute_batch(calls, tools, guards, state)
                .await
        }
    }
}

// ───────────────────────────── Helpers ─────────────────────────────

/// Execute a single tool call with guard checks.
async fn execute_single(
    call: &PendingToolCall,
    tools: &[Arc<dyn ErasedTool>],
    guards: &[Arc<dyn Guard>],
) -> String {
    let action = Action::ToolCall {
        name: call.name.clone(),
        arguments: call.arguments.clone(),
    };

    // Guard checks
    for guard in guards {
        match guard.check(&action) {
            GuardResult::Allow => {}
            GuardResult::Deny { reason, .. } => {
                return format!("Error: Action blocked by guard: {reason}");
            }
            GuardResult::Sanitize { warning, .. } => {
                tracing::info!(guard = guard.name(), "Guard sanitized: {warning}");
            }
        }
    }

    // Find and execute tool
    if let Some(tool) = tools.iter().find(|t| t.name() == call.name) {
        match tool.execute_json(call.arguments.clone()).await {
            Ok(output) => serde_json::to_string(&output)
                .unwrap_or_else(|e| format!("Error serializing output: {e}")),
            Err(e) => format!("Error executing tool: {e}"),
        }
    } else {
        let available: Vec<_> = tools.iter().map(|t| t.name().to_string()).collect();
        format!(
            "Error: Tool '{}' not found. Available: {}",
            call.name,
            available.join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::guard::NoopGuard;

    struct AddTool;

    #[async_trait]
    impl ErasedTool for AddTool {
        fn name(&self) -> &'static str {
            "add"
        }
        fn description(&self) -> &'static str {
            "Adds two numbers"
        }
        fn schema(&self) -> crate::traits::tool::ToolSchema {
            crate::traits::tool::ToolSchema {
                name: "add".into(),
                description: "add".into(),
                parameters: serde_json::json!({}),
            }
        }
        async fn execute_json(
            &self,
            _args: serde_json::Value,
        ) -> std::result::Result<serde_json::Value, crate::Error> {
            Ok(serde_json::json!("result"))
        }
    }

    fn make_calls(n: usize) -> Vec<PendingToolCall> {
        (0..n)
            .map(|i| PendingToolCall {
                id: format!("call-{i}"),
                name: "add".into(),
                arguments: serde_json::json!({}),
            })
            .collect()
    }

    #[tokio::test]
    async fn test_sequential_executes_in_order() {
        let strategy = SequentialStrategy;
        let tools: Vec<Arc<dyn ErasedTool>> = vec![Arc::new(AddTool)];
        let guards: Vec<Arc<dyn Guard>> = vec![Arc::new(NoopGuard)];

        let state = AgentState::new(crate::types::model_info::ModelTier::Small, 4096);

        let results = strategy
            .execute_batch(make_calls(3), &tools, &guards, &state)
            .await;

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].id, "call-0");
        assert_eq!(results[1].id, "call-1");
        assert_eq!(results[2].id, "call-2");
        // All should succeed
        for r in &results {
            assert!(!r.output.starts_with("Error"), "unexpected: {}", r.output);
        }
    }

    #[tokio::test]
    async fn test_parallel_executes_concurrently() {
        let strategy = ParallelStrategy::new(4);
        let tools: Vec<Arc<dyn ErasedTool>> = vec![Arc::new(AddTool)];
        let guards: Vec<Arc<dyn Guard>> = vec![Arc::new(NoopGuard)];

        let state = AgentState::new(crate::types::model_info::ModelTier::Small, 4096);

        let results = strategy
            .execute_batch(make_calls(5), &tools, &guards, &state)
            .await;

        assert_eq!(results.len(), 5);
        for r in &results {
            assert!(!r.output.starts_with("Error"), "unexpected: {}", r.output);
        }
    }

    #[tokio::test]
    async fn test_guard_blocks_propagate() {
        use crate::traits::guard::{Guard, GuardResult};

        struct DenyGuard;
        impl Guard for DenyGuard {
            fn name(&self) -> &'static str {
                "deny"
            }
            fn check(&self, _action: &Action) -> GuardResult {
                GuardResult::Deny {
                    reason: "blocked".into(),
                    severity: crate::traits::guard::GuardSeverity::High,
                }
            }
        }

        let strategy = SequentialStrategy;
        let tools: Vec<Arc<dyn ErasedTool>> = vec![Arc::new(AddTool)];
        let guards: Vec<Arc<dyn Guard>> = vec![Arc::new(DenyGuard)];

        let state = AgentState::new(crate::types::model_info::ModelTier::Small, 4096);

        let results = strategy
            .execute_batch(make_calls(1), &tools, &guards, &state)
            .await;

        assert_eq!(results.len(), 1);
        assert!(results[0].output.contains("blocked"));
    }
}
