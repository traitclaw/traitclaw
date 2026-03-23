//! Agent — the main entry point for running AI agents.

use std::sync::Arc;

use crate::agent_builder::AgentBuilder;
use crate::config::AgentConfig;
use crate::streaming::AgentStream;
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

/// Output from an agent run.
///
/// This enum is marked `#[non_exhaustive]` — new variants may be added in
/// future releases (e.g., `Image`, `Audio`). Always use a wildcard arm in
/// exhaustive matches.
#[derive(Debug, Clone)]
#[non_exhaustive]
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
    pub(crate) context_strategy: Arc<dyn ContextStrategy>,
    pub(crate) execution_strategy: Arc<dyn ExecutionStrategy>,
    pub(crate) output_processor: Arc<dyn OutputProcessor>,
    pub(crate) config: AgentConfig,
}

impl Agent {
    /// Create a builder for constructing an agent.
    #[must_use]
    pub fn builder() -> AgentBuilder {
        AgentBuilder::new()
    }

    /// Create an agent directly (prefer using `builder()`).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        provider: Arc<dyn Provider>,
        tools: Vec<Arc<dyn ErasedTool>>,
        memory: Arc<dyn Memory>,
        guards: Vec<Arc<dyn Guard>>,
        hints: Vec<Arc<dyn Hint>>,
        tracker: Arc<dyn Tracker>,
        context_strategy: Arc<dyn ContextStrategy>,
        execution_strategy: Arc<dyn ExecutionStrategy>,
        output_processor: Arc<dyn OutputProcessor>,
        config: AgentConfig,
    ) -> Self {
        Self {
            provider,
            tools,
            memory,
            guards,
            hints,
            tracker,
            context_strategy,
            execution_strategy,
            output_processor,
            config,
        }
    }

    /// Create a session bound to a specific session ID.
    ///
    /// The returned [`AgentSession`] routes all memory operations through this
    /// session ID, providing conversation isolation.
    ///
    /// ```rust,no_run
    /// # use baseclaw_core::prelude::*;
    /// # async fn example(agent: &Agent) -> baseclaw_core::Result<()> {
    /// let session = agent.session("user-123");
    /// let output = session.say("Hello!").await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn session(&self, id: impl Into<String>) -> AgentSession<'_> {
        AgentSession {
            agent: self,
            session_id: id.into(),
        }
    }

    /// Create a session with an auto-generated UUID v4 session ID.
    ///
    /// Useful when you want isolated conversations without managing IDs.
    #[must_use]
    pub fn session_auto(&self) -> AgentSession<'_> {
        AgentSession {
            agent: self,
            session_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Run the agent with user input and return the final output.
    ///
    /// This uses the `"default"` session for backward compatibility.
    /// For session isolation, use [`Agent::session()`] or [`Agent::session_auto()`].
    ///
    /// # Errors
    ///
    /// Returns an error if the provider fails, tool execution fails,
    /// memory operations fail, or max iterations are reached.
    pub async fn run(&self, input: &str) -> Result<AgentOutput> {
        crate::runtime::run_agent(self, input, "default").await
    }

    /// Run the agent and return a streaming response.
    ///
    /// This uses the `"default"` session for backward compatibility.
    ///
    /// Returns an [`AgentStream`] that yields [`StreamEvent`]s incrementally,
    /// allowing you to display text as it is generated.
    ///
    /// [`StreamEvent`]: crate::types::stream::StreamEvent
    #[must_use]
    pub fn stream(&self, input: &str) -> AgentStream {
        crate::streaming::stream_agent(self, input.to_string(), "default".to_string())
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
    ///
    /// # Note
    ///
    /// Not yet implemented — will be completed in Story 3.3.
    ///
    /// FIXME(story-3.3): change to `async fn` when the full implementation lands.
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

/// A session-scoped agent wrapper.
///
/// Binds an [`Agent`] to a specific `session_id`, routing all memory
/// operations through that session for conversation isolation.
///
/// Created via [`Agent::session()`] or [`Agent::session_auto()`].
pub struct AgentSession<'a> {
    agent: &'a Agent,
    /// The session ID this session is bound to.
    session_id: String,
}

impl AgentSession<'_> {
    /// Send a message within this session.
    ///
    /// Equivalent to [`Agent::run()`] but uses this session's ID for memory.
    ///
    /// # Errors
    ///
    /// Returns an error if the provider fails, tool execution fails,
    /// memory operations fail, or max iterations are reached.
    pub async fn say(&self, input: &str) -> Result<AgentOutput> {
        crate::runtime::run_agent(self.agent, input, &self.session_id).await
    }

    /// Stream a response within this session.
    ///
    /// Equivalent to [`Agent::stream()`] but uses this session's ID for memory.
    #[must_use]
    pub fn stream(&self, input: &str) -> AgentStream {
        crate::streaming::stream_agent(self.agent, input.to_string(), self.session_id.clone())
    }

    /// Get the session ID.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.session_id
    }
}
