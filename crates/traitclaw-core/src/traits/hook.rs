//! Agent Hook — async lifecycle hooks for observability & interception.
//!
//! The `AgentHook` trait provides lifecycle callbacks that fire at key
//! points during agent execution: before/after LLM calls, before/after
//! tool execution, and on errors.
//!
//! # Hook vs Tracker
//!
//! - **Tracker** — Internal state monitoring for the Steering subsystem
//!   (Guard/Hint auto-configuration). Sync, lightweight, modifies `AgentState`.
//! - **Hook** — External observability and interception. Async, can perform
//!   I/O (send metrics to DataDog, write logs, etc.), does NOT modify `AgentState`.
//!
//! # Architecture Decision
//!
//! Hooks are `async fn` using Rust 1.75+ native `async fn` in traits.
//! This allows non-blocking I/O (HTTP calls to observability services)
//! without blocking the agent loop.
//!
//! # Example
//!
//! ```rust,no_run
//! use traitclaw_core::traits::hook::AgentHook;
//! use std::time::Duration;
//!
//! struct TimingHook;
//!
//! #[async_trait::async_trait]
//! impl AgentHook for TimingHook {
//!     async fn on_provider_end(
//!         &self,
//!         _response: &traitclaw_core::types::completion::CompletionResponse,
//!         duration: Duration,
//!     ) {
//!         println!("LLM call took {duration:?}");
//!     }
//! }
//! ```

use std::time::Duration;

use async_trait::async_trait;

use crate::agent::AgentOutput;
use crate::types::completion::{CompletionRequest, CompletionResponse};

/// The result of a hook's interception decision.
///
/// Returned by [`AgentHook::before_tool_execute`] to allow or block
/// a tool execution.
#[derive(Debug, Clone)]
pub enum HookAction {
    /// Allow the tool to execute normally.
    Continue,
    /// Block the tool execution with the given reason.
    ///
    /// The reason string is returned to the LLM as the tool result,
    /// allowing it to adapt its behavior.
    Block(String),
}

/// Async lifecycle hooks for agent observability and interception.
///
/// All methods have default empty implementations, so you only need
/// to override the hooks you care about.
///
/// Multiple hooks can be registered on a single agent and are called
/// sequentially in registration order.
///
/// # Object Safety
///
/// This trait is object-safe and used as `Vec<Box<dyn AgentHook>>`.
#[async_trait]
pub trait AgentHook: Send + Sync + 'static {
    /// Called when the agent starts processing input.
    async fn on_agent_start(&self, _input: &str) {}

    /// Called when the agent finishes processing.
    async fn on_agent_end(&self, _output: &AgentOutput, _duration: Duration) {}

    /// Called before each LLM call is made.
    async fn on_provider_start(&self, _request: &CompletionRequest) {}

    /// Called after each LLM call completes.
    async fn on_provider_end(&self, _response: &CompletionResponse, _duration: Duration) {}

    /// Called before a tool is executed.
    ///
    /// Return [`HookAction::Block`] to prevent execution. The block
    /// reason is returned to the LLM as the tool result.
    async fn before_tool_execute(
        &self,
        _name: &str,
        _args: &serde_json::Value,
    ) -> HookAction {
        HookAction::Continue
    }

    /// Called after a tool finishes executing.
    async fn after_tool_execute(
        &self,
        _name: &str,
        _result: &str,
        _duration: Duration,
    ) {
    }

    /// Called for each streaming chunk received.
    async fn on_stream_chunk(&self, _chunk: &str) {}

    /// Called when an error occurs during execution.
    async fn on_error(&self, _error: &crate::Error) {}
}

/// Blanket implementation: `Arc<T>` delegates to `T`.
///
/// This enables sharing a hook instance across multiple agents via `Arc`,
/// useful for recording hooks or metrics collectors.
#[async_trait]
impl<T: AgentHook> AgentHook for std::sync::Arc<T> {
    async fn on_agent_start(&self, input: &str) {
        (**self).on_agent_start(input).await;
    }
    async fn on_agent_end(&self, output: &AgentOutput, duration: Duration) {
        (**self).on_agent_end(output, duration).await;
    }
    async fn on_provider_start(&self, request: &CompletionRequest) {
        (**self).on_provider_start(request).await;
    }
    async fn on_provider_end(&self, response: &CompletionResponse, duration: Duration) {
        (**self).on_provider_end(response, duration).await;
    }
    async fn before_tool_execute(&self, name: &str, args: &serde_json::Value) -> HookAction {
        (**self).before_tool_execute(name, args).await
    }
    async fn after_tool_execute(&self, name: &str, result: &str, duration: Duration) {
        (**self).after_tool_execute(name, result, duration).await;
    }
    async fn on_stream_chunk(&self, chunk: &str) {
        (**self).on_stream_chunk(chunk).await;
    }
    async fn on_error(&self, error: &crate::Error) {
        (**self).on_error(error).await;
    }
}

/// A hook that logs all lifecycle events using `tracing`.
///
/// # Example
///
/// ```rust,no_run
/// use traitclaw_core::traits::hook::LoggingHook;
///
/// // let agent = Agent::builder()
/// //     .model(my_provider)
/// //     .hook(LoggingHook::new(tracing::Level::INFO))
/// //     .build()?;
/// ```
pub struct LoggingHook {
    level: tracing::Level,
}

impl LoggingHook {
    /// Create a new logging hook at the given tracing level.
    #[must_use]
    pub fn new(level: tracing::Level) -> Self {
        Self { level }
    }
}

#[async_trait]
impl AgentHook for LoggingHook {
    async fn on_agent_start(&self, input: &str) {
        match self.level {
            tracing::Level::TRACE => tracing::trace!(input_len = input.len(), "Agent starting"),
            tracing::Level::DEBUG => tracing::debug!(input_len = input.len(), "Agent starting"),
            _ => tracing::info!(input_len = input.len(), "Agent starting"),
        }
    }

    async fn on_agent_end(&self, _output: &AgentOutput, duration: Duration) {
        #[allow(clippy::cast_possible_truncation)]
        let ms = duration.as_millis() as u64;
        match self.level {
            tracing::Level::TRACE => tracing::trace!(duration_ms = ms, "Agent completed"),
            tracing::Level::DEBUG => tracing::debug!(duration_ms = ms, "Agent completed"),
            _ => tracing::info!(duration_ms = ms, "Agent completed"),
        }
    }

    async fn on_provider_start(&self, _request: &CompletionRequest) {
        match self.level {
            tracing::Level::TRACE => tracing::trace!("LLM call starting"),
            tracing::Level::DEBUG => tracing::debug!("LLM call starting"),
            _ => tracing::info!("LLM call starting"),
        }
    }

    async fn on_provider_end(&self, response: &CompletionResponse, duration: Duration) {
        #[allow(clippy::cast_possible_truncation)]
        let ms = duration.as_millis() as u64;
        let tokens = response.usage.total_tokens;
        match self.level {
            tracing::Level::TRACE => {
                tracing::trace!(duration_ms = ms, tokens, "LLM call completed")
            }
            tracing::Level::DEBUG => {
                tracing::debug!(duration_ms = ms, tokens, "LLM call completed")
            }
            _ => tracing::info!(duration_ms = ms, tokens, "LLM call completed"),
        }
    }

    async fn before_tool_execute(&self, name: &str, _args: &serde_json::Value) -> HookAction {
        match self.level {
            tracing::Level::TRACE => tracing::trace!(tool = name, "Tool executing"),
            tracing::Level::DEBUG => tracing::debug!(tool = name, "Tool executing"),
            _ => tracing::info!(tool = name, "Tool executing"),
        }
        HookAction::Continue
    }

    async fn after_tool_execute(&self, name: &str, _result: &str, duration: Duration) {
        #[allow(clippy::cast_possible_truncation)]
        let ms = duration.as_millis() as u64;
        match self.level {
            tracing::Level::TRACE => tracing::trace!(tool = name, duration_ms = ms, "Tool done"),
            tracing::Level::DEBUG => tracing::debug!(tool = name, duration_ms = ms, "Tool done"),
            _ => tracing::info!(tool = name, duration_ms = ms, "Tool done"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Verify trait is object-safe
    fn _assert_object_safe(_: &dyn AgentHook) {}

    #[test]
    fn test_hook_action_variants() {
        let cont = HookAction::Continue;
        assert!(matches!(cont, HookAction::Continue));

        let block = HookAction::Block("reason".into());
        assert!(matches!(block, HookAction::Block(r) if r == "reason"));
    }

    #[test]
    fn test_logging_hook_creation() {
        let hook = LoggingHook::new(tracing::Level::INFO);
        assert_eq!(hook.level, tracing::Level::INFO);
    }
}
