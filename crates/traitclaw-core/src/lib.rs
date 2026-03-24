//! # `TraitClaw` Core
//!
//! Core traits, types, and runtime for the `TraitClaw` AI Agent Framework.
//!
//! This crate provides the fundamental abstractions for building AI agents:
//!
//! - [`Provider`] — LLM abstraction trait
//! - [`Tool`] — Tool definition trait with auto JSON Schema
//! - [`Memory`] — 3-layer memory system (conversation, working, long-term)
//! - [`Guard`] — Hard boundary checks for model steering
//! - [`Hint`] — Contextual guidance injection
//! - [`Tracker`] — Runtime state monitoring
//! - [`Agent`] — The main agent struct with builder pattern
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use traitclaw_core::prelude::*;
//!
//! # async fn example() -> traitclaw_core::Result<()> {
//! // Agent creation is done via the builder pattern
//! // (requires a Provider implementation)
//! # Ok(())
//! # }
//! ```

#![deny(warnings)]
#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod traits;
pub mod types;

pub mod agent;
pub mod agent_builder;
pub mod config;
pub mod error;
pub mod memory;
pub mod retry;
pub(crate) mod runtime;
pub(crate) mod streaming;

// Re-export core traits at crate root
pub use traits::context_strategy::{ContextStrategy, NoopContextStrategy, SlidingWindowStrategy};
pub use traits::execution_strategy::{
    AdaptiveStrategy, ExecutionStrategy, ParallelStrategy, PendingToolCall, SequentialStrategy,
    ToolResult,
};
pub use traits::guard::{Guard, GuardResult};
pub use traits::hint::{Hint, HintMessage, HintPriority, InjectionPoint};
pub use traits::memory::Memory;
pub use traits::output_processor::{
    ChainProcessor, NoopProcessor, OutputProcessor, TruncateProcessor,
};
pub use traits::provider::Provider;

pub use retry::{RetryConfig, RetryProvider};
pub use traits::tool::{ErasedTool, Tool, ToolSchema};
pub use traits::tracker::Tracker;

// Re-export core types at crate root
pub use types::action::Action;
pub use types::agent_state::AgentState;
pub use types::completion::{CompletionRequest, CompletionResponse, Usage};
pub use types::message::{Message, MessageRole};
pub use types::model_info::{ModelInfo, ModelTier};
pub use types::stream::{CompletionStream, StreamEvent};
pub use types::tool_call::ToolCall;

// Re-export error types
pub use error::{Error, Result};

// Re-export memory implementations
pub use memory::in_memory::InMemoryMemory;

// Re-export agent
pub use agent::{Agent, AgentOutput, AgentOutputContent, AgentSession, RunUsage};
pub use agent_builder::AgentBuilder;

/// Prelude module for convenient imports.
///
/// Use `use traitclaw_core::prelude::*;` for convenient access
/// to the most commonly used types.
///
/// ```rust
/// use traitclaw_core::prelude::*;
/// ```
pub mod prelude {

    pub use crate::traits::context_strategy::{
        ContextStrategy, NoopContextStrategy, SlidingWindowStrategy,
    };
    pub use crate::traits::execution_strategy::{
        ExecutionStrategy, ParallelStrategy, SequentialStrategy,
    };
    pub use crate::traits::guard::{Guard, GuardResult};
    pub use crate::traits::hint::{Hint, HintMessage};
    pub use crate::traits::memory::Memory;
    pub use crate::traits::output_processor::{OutputProcessor, TruncateProcessor};
    pub use crate::traits::provider::{ModelInfo, ModelTier, Provider};
    pub use crate::traits::tool::{ErasedTool, Tool, ToolSchema};
    pub use crate::traits::tracker::Tracker;

    pub use crate::types::action::Action;
    pub use crate::types::agent_state::AgentState;
    pub use crate::types::completion::{CompletionRequest, CompletionResponse};
    pub use crate::types::message::{Message, MessageRole};
    pub use crate::types::stream::{CompletionStream, StreamEvent};
    pub use crate::types::tool_call::ToolCall;

    pub use crate::config::AgentConfig;
    pub use crate::error::{Error, Result};
    pub use crate::memory::in_memory::InMemoryMemory;

    pub use crate::agent::{Agent, AgentOutput, AgentOutputContent, AgentSession, RunUsage};
    pub use crate::agent_builder::AgentBuilder;
}
