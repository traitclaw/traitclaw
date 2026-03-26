//! Agent — the main entry point for running AI agents.

use std::sync::Arc;

use crate::agent_builder::AgentBuilder;
use crate::config::AgentConfig;
use crate::streaming::AgentStream;
use crate::traits::context_manager::ContextManager;
#[allow(deprecated)]
use crate::traits::context_strategy::ContextStrategy;
use crate::traits::execution_strategy::ExecutionStrategy;
use crate::traits::guard::Guard;
use crate::traits::hint::Hint;
use crate::traits::hook::AgentHook;
use crate::traits::memory::Memory;
#[allow(deprecated)]
use crate::traits::output_processor::OutputProcessor;
use crate::traits::output_transformer::OutputTransformer;
use crate::traits::provider::Provider;
use crate::traits::strategy::{AgentRuntime, AgentStrategy};
use crate::traits::tool::ErasedTool;
use crate::traits::tool_registry::ToolRegistry;
use crate::traits::tracker::Tracker;
use crate::types::message::Message;
use crate::Result;

/// Usage statistics from an agent run.
#[derive(Debug, Clone, Default)]
pub struct RunUsage {
    /// Total tokens consumed across all LLM calls.
    pub tokens: usize,
    /// Number of agent loop iterations.
    pub iterations: usize,
    /// Wall-clock duration of the run.
    pub duration: std::time::Duration,
}

/// Output from an agent run.
///
/// This struct is marked `#[non_exhaustive]` — new fields may be added in
/// future releases without breaking changes.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct AgentOutput {
    /// The response content.
    pub content: AgentOutputContent,
    /// Usage statistics for this run.
    pub usage: RunUsage,
}

/// The content type of an agent output.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum AgentOutputContent {
    /// The agent returned a text response.
    Text(String),
    /// The agent returned a structured JSON response.
    Structured(serde_json::Value),
    /// The agent encountered an error.
    Error(String),
}

impl AgentOutput {
    /// Create a text output with usage.
    #[must_use]
    pub fn text_with_usage(text: String, usage: RunUsage) -> Self {
        Self {
            content: AgentOutputContent::Text(text),
            usage,
        }
    }

    /// Get the text content if this is a text output.
    ///
    /// Returns an empty string for `Structured` and `Error` variants.
    /// Use [`structured()`](Self::structured) or [`Display`] for those.
    #[must_use]
    pub fn text(&self) -> &str {
        match &self.content {
            AgentOutputContent::Text(t) => t,
            _ => "",
        }
    }

    /// Get the error message if this is an error output.
    #[must_use]
    pub fn error_message(&self) -> Option<&str> {
        match &self.content {
            AgentOutputContent::Error(e) => Some(e),
            _ => None,
        }
    }

    /// Get the structured JSON value if this is a structured output.
    #[must_use]
    pub fn structured(&self) -> Option<&serde_json::Value> {
        match &self.content {
            AgentOutputContent::Structured(v) => Some(v),
            _ => None,
        }
    }

    /// Check if this is an error output.
    #[must_use]
    pub fn is_error(&self) -> bool {
        matches!(&self.content, AgentOutputContent::Error(_))
    }
}

impl std::fmt::Display for AgentOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.content {
            AgentOutputContent::Text(t) => write!(f, "{t}"),
            AgentOutputContent::Structured(v) => write!(f, "{v}"),
            AgentOutputContent::Error(e) => write!(f, "Error: {e}"),
        }
    }
}

/// The main agent struct.
///
/// An agent combines an LLM provider, tools, memory, and steering into
/// a single runtime that processes user input and produces output.
///
/// Use [`Agent::builder()`] to construct an agent.
#[allow(deprecated)]
pub struct Agent {
    pub(crate) provider: Arc<dyn Provider>,
    pub(crate) tools: Vec<Arc<dyn ErasedTool>>,
    pub(crate) memory: Arc<dyn Memory>,
    pub(crate) guards: Vec<Arc<dyn Guard>>,
    pub(crate) hints: Vec<Arc<dyn Hint>>,
    pub(crate) tracker: Arc<dyn Tracker>,
    pub(crate) context_manager: Arc<dyn ContextManager>,
    pub(crate) context_strategy: Arc<dyn ContextStrategy>,
    pub(crate) execution_strategy: Arc<dyn ExecutionStrategy>,
    pub(crate) output_transformer: Arc<dyn OutputTransformer>,
    pub(crate) output_processor: Arc<dyn OutputProcessor>,
    pub(crate) tool_registry: Arc<dyn ToolRegistry>,
    pub(crate) strategy: Box<dyn AgentStrategy>,
    pub(crate) hooks: Vec<Arc<dyn AgentHook>>,
    pub(crate) config: AgentConfig,
}

impl std::fmt::Debug for Agent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Agent")
            .field("model", &self.provider.model_info().name)
            .field("tools", &self.tools.len())
            .field("guards", &self.guards.len())
            .field("hints", &self.hints.len())
            .field("hooks", &self.hooks.len())
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

#[allow(deprecated)]
impl Agent {
    /// Create a builder for constructing an agent.
    #[must_use]
    pub fn builder() -> AgentBuilder {
        AgentBuilder::new()
    }

    /// Create an agent with just a provider and system prompt.
    ///
    /// This is a convenience shorthand equivalent to:
    /// ```rust,ignore
    /// Agent::builder()
    ///     .provider(provider)
    ///     .system(system)
    ///     .build()
    ///     .unwrap()
    /// ```
    ///
    /// All other settings use their defaults (in-memory memory, no tools,
    /// no guards, etc.). Use [`Agent::builder()`] for full customization.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use traitclaw_core::prelude::*;
    ///
    /// # fn example(provider: impl traitclaw_core::traits::provider::Provider) {
    /// let agent = Agent::with_system(provider, "You are a helpful assistant.");
    /// # }
    /// ```
    /// # Panics
    ///
    /// This method cannot panic under normal usage — the internal `build()`
    /// call only fails when no provider is set, and `with_system` always
    /// provides one.
    #[must_use]
    pub fn with_system(provider: impl Provider, system: impl Into<String>) -> Self {
        Agent::builder()
            .provider(provider)
            .system(system)
            .build()
            .expect("Agent::with_system is infallible: provider is always set")
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
        context_manager: Arc<dyn ContextManager>,
        context_strategy: Arc<dyn ContextStrategy>,
        execution_strategy: Arc<dyn ExecutionStrategy>,
        output_transformer: Arc<dyn OutputTransformer>,
        output_processor: Arc<dyn OutputProcessor>,
        tool_registry: Arc<dyn ToolRegistry>,
        strategy: Box<dyn AgentStrategy>,
        hooks: Vec<Arc<dyn AgentHook>>,
        config: AgentConfig,
    ) -> Self {
        Self {
            provider,
            tools,
            memory,
            guards,
            hints,
            tracker,
            context_manager,
            context_strategy,
            execution_strategy,
            output_transformer,
            output_processor,
            tool_registry,
            strategy,
            hooks,
            config,
        }
    }

    /// Build an [`AgentRuntime`] from this agent's components.
    ///
    /// The runtime is passed to the strategy for execution.
    fn to_runtime(&self) -> AgentRuntime {
        AgentRuntime {
            provider: Arc::clone(&self.provider),
            tools: self.tools.clone(),
            memory: Arc::clone(&self.memory),
            guards: self.guards.clone(),
            hints: self.hints.clone(),
            tracker: Arc::clone(&self.tracker),
            context_manager: Arc::clone(&self.context_manager),
            context_strategy: Arc::clone(&self.context_strategy),
            execution_strategy: Arc::clone(&self.execution_strategy),
            output_transformer: Arc::clone(&self.output_transformer),
            output_processor: Arc::clone(&self.output_processor),
            tool_registry: Arc::clone(&self.tool_registry),
            hooks: self.hooks.clone(),
            config: self.config.clone(),
        }
    }

    /// Create a session bound to a specific session ID.
    ///
    /// The returned [`AgentSession`] routes all memory operations through this
    /// session ID, providing conversation isolation.
    ///
    /// ```rust,no_run
    /// # use traitclaw_core::prelude::*;
    /// # async fn example(agent: &Agent) -> traitclaw_core::Result<()> {
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
        let runtime = self.to_runtime();
        self.strategy.execute(&runtime, input, "default").await
    }

    /// Run the agent and return a streaming response.
    ///
    /// This uses the `"default"` session for backward compatibility.
    ///
    /// Returns an [`AgentStream`] that yields [`StreamEvent`]s incrementally,
    /// providing real-time output from the LLM.
    ///
    /// [`AgentStream`]: crate::streaming::AgentStream
    /// [`StreamEvent`]: crate::types::stream::StreamEvent
    #[must_use]
    pub fn stream(&self, input: &str) -> AgentStream {
        self.stream_with_session(input, "default")
    }

    /// Run the agent and return a structured output.
    ///
    /// The LLM is instructed to return JSON matching type `T`'s schema.
    /// If deserialization fails, retries up to 3 times with feedback.
    ///
    /// When the provider's model supports native structured-output
    /// (`model_info.supports_structured == true`), the `response_format`
    /// is set on the `CompletionRequest` for guaranteed valid JSON.
    /// Otherwise, schema instructions are injected into the system prompt.
    ///
    /// # ⚠️ Stateless Mode
    ///
    /// This method calls the provider directly, **bypassing** the agent
    /// runtime loop. Memory, guards, hints, context strategy, and usage
    /// tracking are not used. Use `run()` for full agent behavior.
    ///
    /// # Errors
    ///
    /// Returns an error if the provider fails or deserialization fails
    /// after retries.
    pub async fn run_structured<T>(&self, input: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned + schemars::JsonSchema,
    {
        let model_info = self.provider.model_info();
        let schema = schemars::schema_for!(T);
        let schema_json = serde_json::to_value(&schema)
            .map_err(|e| crate::Error::Runtime(format!("Failed to serialize schema: {e}")))?;

        let uses_native = model_info.supports_structured;

        let mut messages = vec![];
        if let Some(ref system_prompt) = self.config.system_prompt {
            messages.push(Message::system(system_prompt));
        }

        // If the model doesn't support native structured output, inject schema instructions
        if !uses_native {
            let schema_str = serde_json::to_string_pretty(&schema_json)
                .unwrap_or_else(|_| schema_json.to_string());
            messages.push(Message::system(format!(
                "You MUST respond ONLY with valid JSON matching this schema:\n```json\n{schema_str}\n```\nDo NOT include any text before or after the JSON."
            )));
        }

        messages.push(Message::user(input));

        let max_retries = 3;
        let mut last_error = String::new();

        for attempt in 0..=max_retries {
            if attempt > 0 {
                // Add retry feedback
                messages.push(Message::system(format!(
                    "Your previous response was not valid JSON. Error: {last_error}\n\
                     Please try again. Respond ONLY with valid JSON."
                )));
            }

            let response_format = if uses_native {
                Some(crate::types::completion::ResponseFormat::JsonSchema {
                    json_schema: schema_json.clone(),
                })
            } else {
                None
            };

            let request = crate::types::completion::CompletionRequest {
                model: model_info.name.clone(),
                messages: messages.clone(),
                tools: vec![],
                max_tokens: self.config.max_tokens,
                temperature: self.config.temperature,
                response_format,
                stream: false,
            };

            let response = self.provider.complete(request).await?;

            let text = match response.content {
                crate::types::completion::ResponseContent::Text(t) => t,
                crate::types::completion::ResponseContent::ToolCalls(_) => {
                    last_error = "Model returned tool calls instead of JSON".into();
                    messages.push(Message::assistant("[tool calls returned]"));
                    continue;
                }
            };

            match serde_json::from_str::<T>(&text) {
                Ok(value) => return Ok(value),
                Err(e) => {
                    last_error = format!("{e}");
                    messages.push(Message::assistant(&text));
                }
            }
        }

        Err(crate::Error::Runtime(format!(
            "Structured output failed after {max_retries} retries. Last error: {last_error}"
        )))
    }

    /// Internal stream implementation supporting custom session IDs.
    pub(crate) fn stream_with_session(&self, input: &str, session_id: &str) -> AgentStream {
        let runtime = crate::traits::strategy::AgentRuntime {
            provider: Arc::clone(&self.provider),
            tools: self.tools.clone(),
            memory: Arc::clone(&self.memory),
            guards: self.guards.clone(),
            hints: self.hints.clone(),
            tracker: Arc::clone(&self.tracker),
            context_manager: Arc::clone(&self.context_manager),
            context_strategy: Arc::clone(&self.context_strategy),
            execution_strategy: Arc::clone(&self.execution_strategy),
            output_transformer: Arc::clone(&self.output_transformer),
            output_processor: Arc::clone(&self.output_processor),
            tool_registry: Arc::clone(&self.tool_registry),
            config: self.config.clone(),
            hooks: self.hooks.clone(),
        };

        self.strategy.stream(&runtime, input, session_id)
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
        let runtime = self.agent.to_runtime();
        self.agent
            .strategy
            .execute(&runtime, input, &self.session_id)
            .await
    }

    /// Execute the agent loop with a custom session ID, returning a stream.
    ///
    /// Equivalent to [`Agent::stream()`] but uses this session's ID for memory.
    #[must_use]
    pub fn stream(&self, input: &str) -> AgentStream {
        self.agent.stream_with_session(input, &self.session_id)
    }

    /// Get the session ID.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.session_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_usage_default() {
        let u = RunUsage::default();
        assert_eq!(u.tokens, 0);
        assert_eq!(u.iterations, 0);
        assert_eq!(u.duration, std::time::Duration::ZERO);
    }

    #[test]
    fn test_text_output() {
        let out = AgentOutput::text_with_usage("Hello".into(), RunUsage::default());
        assert_eq!(out.text(), "Hello");
        assert!(!out.is_error());
        assert!(out.structured().is_none());
        assert!(out.error_message().is_none());
    }

    #[test]
    fn test_text_returns_empty_for_structured() {
        let out = AgentOutput {
            content: AgentOutputContent::Structured(serde_json::json!({"key": "val"})),
            usage: RunUsage::default(),
        };
        assert_eq!(out.text(), "");
        assert!(out.structured().is_some());
        assert_eq!(out.structured().unwrap()["key"], "val");
    }

    #[test]
    fn test_text_returns_empty_for_error() {
        let out = AgentOutput {
            content: AgentOutputContent::Error("boom".into()),
            usage: RunUsage::default(),
        };
        assert_eq!(out.text(), "");
        assert!(out.is_error());
        assert_eq!(out.error_message(), Some("boom"));
    }

    #[test]
    fn test_display_text() {
        let out = AgentOutput::text_with_usage("hi".into(), RunUsage::default());
        assert_eq!(format!("{out}"), "hi");
    }

    #[test]
    fn test_display_structured() {
        let out = AgentOutput {
            content: AgentOutputContent::Structured(serde_json::json!(42)),
            usage: RunUsage::default(),
        };
        assert_eq!(format!("{out}"), "42");
    }

    #[test]
    fn test_display_error() {
        let out = AgentOutput {
            content: AgentOutputContent::Error("fail".into()),
            usage: RunUsage::default(),
        };
        assert_eq!(format!("{out}"), "Error: fail");
    }

    #[test]
    fn test_usage_carried_through() {
        let usage = RunUsage {
            tokens: 100,
            iterations: 5,
            duration: std::time::Duration::from_millis(500),
        };
        let out = AgentOutput::text_with_usage("x".into(), usage);
        assert_eq!(out.usage.tokens, 100);
        assert_eq!(out.usage.iterations, 5);
        assert_eq!(out.usage.duration.as_millis(), 500);
    }

    // --- Agent::with_system() tests (Story 1.1) ---

    use crate::types::completion::{CompletionRequest, CompletionResponse, ResponseContent, Usage};
    use crate::types::model_info::{ModelInfo, ModelTier};
    use crate::types::stream::CompletionStream;
    use async_trait::async_trait;

    struct MockProvider {
        info: ModelInfo,
    }

    impl MockProvider {
        fn new() -> Self {
            Self {
                info: ModelInfo::new("mock", ModelTier::Small, 4_096, false, false, false),
            }
        }
    }

    #[async_trait]
    impl crate::traits::provider::Provider for MockProvider {
        async fn complete(&self, _req: CompletionRequest) -> crate::Result<CompletionResponse> {
            Ok(CompletionResponse {
                content: ResponseContent::Text("ok".into()),
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
    fn test_with_system_str_prompt() {
        // AC #1, #2: with_system accepts &str and creates a valid agent
        let agent = Agent::with_system(MockProvider::new(), "You are helpful.");
        assert_eq!(
            agent.config.system_prompt.as_deref(),
            Some("You are helpful.")
        );
    }

    #[test]
    fn test_with_system_string_prompt() {
        // AC #2: with_system accepts String
        let prompt = String::from("You are a researcher.");
        let agent = Agent::with_system(MockProvider::new(), prompt);
        assert_eq!(
            agent.config.system_prompt.as_deref(),
            Some("You are a researcher.")
        );
    }

    #[test]
    fn test_with_system_builder_unchanged() {
        // AC #3: builder API is unchanged (still works)
        let result = Agent::builder()
            .provider(MockProvider::new())
            .system("test")
            .build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_with_system_provider_configured() {
        // AC #4: agent has correct provider
        let agent = Agent::with_system(MockProvider::new(), "test");
        assert_eq!(agent.provider.model_info().name, "mock");
    }
}
