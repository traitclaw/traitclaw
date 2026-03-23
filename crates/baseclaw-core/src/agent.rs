//! Agent — the main entry point for running AI agents.

use std::sync::Arc;

use crate::agent_builder::AgentBuilder;
use crate::config::AgentConfig;
use crate::streaming::AgentStream;
use crate::traits::guard::Guard;
use crate::traits::hint::Hint;
use crate::traits::memory::Memory;
use crate::traits::provider::Provider;
use crate::traits::tool::ErasedTool;
use crate::traits::tracker::Tracker;
use crate::Result;

/// Output from an agent run.
#[derive(Debug, Clone)]
pub enum AgentOutput {
    /// The agent returned a text response.
    Text(String),
}

impl AgentOutput {
    /// Get the text content if this is a text output.
    #[must_use]
    pub fn text(&self) -> &str {
        match self {
            Self::Text(t) => t,
        }
    }
}

impl std::fmt::Display for AgentOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(t) => write!(f, "{t}"),
        }
    }
}

/// The main agent struct.
///
/// An agent combines an LLM provider, tools, memory, and steering into
/// a single runtime that processes user input and produces output.
///
/// Use [`Agent::builder()`] to construct an agent.
pub struct Agent {
    pub(crate) provider: Arc<dyn Provider>,
    pub(crate) tools: Vec<Arc<dyn ErasedTool>>,
    pub(crate) memory: Arc<dyn Memory>,
    pub(crate) guards: Vec<Arc<dyn Guard>>,
    pub(crate) hints: Vec<Arc<dyn Hint>>,
    pub(crate) tracker: Arc<dyn Tracker>,
    pub(crate) config: AgentConfig,
}

impl Agent {
    /// Create a builder for constructing an agent.
    #[must_use]
    pub fn builder() -> AgentBuilder {
        AgentBuilder::new()
    }

    /// Create an agent directly (prefer using `builder()`).
    pub(crate) fn new(
        provider: Arc<dyn Provider>,
        tools: Vec<Arc<dyn ErasedTool>>,
        memory: Arc<dyn Memory>,
        guards: Vec<Arc<dyn Guard>>,
        hints: Vec<Arc<dyn Hint>>,
        tracker: Arc<dyn Tracker>,
        config: AgentConfig,
    ) -> Self {
        Self {
            provider,
            tools,
            memory,
            guards,
            hints,
            tracker,
            config,
        }
    }

    /// Run the agent with user input and return the final output.
    ///
    /// This executes the full agent loop: send message to LLM, handle tool
    /// calls, apply Guard/Hint/Track steering, and return the final response.
    ///
    /// # Errors
    ///
    /// Returns an error if the provider fails, tool execution fails,
    /// memory operations fail, or max iterations are reached.
    pub async fn run(&self, input: &str) -> Result<AgentOutput> {
        crate::runtime::run_agent(self, input).await
    }

    /// Run the agent and return a streaming response.
    ///
    /// Returns an [`AgentStream`] that yields [`StreamEvent`]s incrementally,
    /// allowing you to display text as it is generated.
    ///
    /// [`StreamEvent`]: crate::types::stream::StreamEvent
    #[must_use]
    pub fn stream(&self, input: &str) -> AgentStream {
        crate::streaming::stream_agent(self, input.to_string())
    }

    /// Run the agent and return a structured output.
    ///
    /// The LLM is instructed to return JSON matching type `T`'s schema.
    /// If deserialization fails, retries up to 3 times with feedback.
    ///
    /// # Errors
    ///
    /// Returns an error if the provider fails or deserialization fails
    /// after retries.
    pub fn run_structured<T>(&self, _input: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned + schemars::JsonSchema,
    {
        // Will be implemented in Story 3.3
        Err(crate::Error::Runtime(
            "Structured output not yet implemented".into(),
        ))
    }
}
