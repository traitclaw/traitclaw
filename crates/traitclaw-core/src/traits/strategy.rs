//! Agent Strategy — pluggable reasoning loop.
//!
//! The `AgentStrategy` trait allows you to replace the default agent execution
//! loop with a custom reasoning architecture (e.g., MCTS, ReAct, Chain-of-Thought).
//!
//! # Architecture Decision
//!
//! Dispatch is dynamic (`Box<dyn AgentStrategy>`) because LLM latency
//! (200-2000ms) dwarfs vtable overhead (nanoseconds). This simplifies
//! the API: strategies can be swapped at runtime without recompilation.
//!
//! # Example
//!
//! ```rust,no_run
//! use traitclaw_core::traits::strategy::AgentStrategy;
//! use traitclaw_core::agent::AgentOutput;
//!
//! struct MctsStrategy { /* config */ }
//!
//! #[async_trait::async_trait]
//! impl AgentStrategy for MctsStrategy {
//!     async fn execute(
//!         &self,
//!         runtime: &traitclaw_core::traits::strategy::AgentRuntime,
//!         input: &str,
//!         session_id: &str,
//!     ) -> traitclaw_core::Result<AgentOutput> {
//!         // Your custom reasoning loop here
//!         todo!()
//!     }
//! }
//! ```

use std::sync::Arc;

use async_trait::async_trait;

use crate::agent::AgentOutput;
use crate::config::AgentConfig;
use crate::traits::context_strategy::ContextStrategy;
use crate::traits::execution_strategy::ExecutionStrategy;
use crate::traits::guard::Guard;
use crate::traits::hint::Hint;
use crate::traits::memory::Memory;
use crate::traits::output_processor::OutputProcessor;
use crate::traits::provider::Provider;
use crate::traits::tool::ErasedTool;
use crate::traits::tracker::Tracker;
use crate::Result;

/// Runtime context provided to strategies.
///
/// Exposes all agent components needed to execute a reasoning loop,
/// without exposing the strategy itself (avoiding recursion).
pub struct AgentRuntime {
    /// The LLM provider.
    pub provider: Arc<dyn Provider>,
    /// Registered tools.
    pub tools: Vec<Arc<dyn ErasedTool>>,
    /// Memory backend.
    pub memory: Arc<dyn Memory>,
    /// Active guards.
    pub guards: Vec<Arc<dyn Guard>>,
    /// Active hints.
    pub hints: Vec<Arc<dyn Hint>>,
    /// Runtime tracker.
    pub tracker: Arc<dyn Tracker>,
    /// Context window strategy.
    pub context_strategy: Arc<dyn ContextStrategy>,
    /// Tool execution strategy (sequential/parallel).
    pub execution_strategy: Arc<dyn ExecutionStrategy>,
    /// Tool output processor.
    pub output_processor: Arc<dyn OutputProcessor>,
    /// Agent configuration.
    pub config: AgentConfig,
    /// Lifecycle hooks.
    pub hooks: Vec<Arc<dyn super::hook::AgentHook>>,
}

/// Pluggable agent execution strategy.
///
/// Implement this trait to define a custom reasoning loop. The default
/// implementation ([`DefaultStrategy`]) preserves the v0.1.0 behavior.
///
/// # Object Safety
///
/// This trait is object-safe and used as `Box<dyn AgentStrategy>`.
#[async_trait]
pub trait AgentStrategy: Send + Sync + 'static {
    /// Execute the agent reasoning loop.
    ///
    /// Receives the full `AgentRuntime` with all agent components and
    /// the user's input. Returns the final `AgentOutput`.
    async fn execute(
        &self,
        runtime: &AgentRuntime,
        input: &str,
        session_id: &str,
    ) -> Result<AgentOutput>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Verify trait is object-safe
    fn _assert_object_safe(_: &dyn AgentStrategy) {}

    // Verify AgentRuntime is Send + Sync
    fn _assert_send_sync(_: &(dyn AgentStrategy + Send + Sync)) {}
}
