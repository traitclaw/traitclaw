//! Hint trait for contextual guidance injection.
//!
//! Hints inject messages into the conversation at the right time to steer the model.
//! Like GPS recalculation — keep the model on track without blocking execution.

use crate::types::agent_state::AgentState;
use crate::types::message::MessageRole;

/// Where in the conversation to inject the hint message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InjectionPoint {
    /// Add to the system prompt (beginning of conversation).
    SystemPrompt,
    /// Add as a new message before the next LLM call.
    BeforeNextLlmCall,
    /// Add to the end of the conversation (recency zone — model pays most attention here).
    RecencyZone,
    /// Append to a specific tool result.
    AppendToToolResult {
        /// Name of the tool whose result to append to.
        tool_name: String,
    },
}

/// Priority level for hint injection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HintPriority {
    /// Only inject if context is not too long.
    Low,
    /// Inject if token budget allows.
    Normal,
    /// Always inject regardless of budget.
    Critical,
}

/// A hint message to be injected into the conversation.
#[derive(Debug, Clone)]
pub struct HintMessage {
    /// The role of this message (usually System or Assistant).
    pub role: MessageRole,
    /// The content of the hint.
    pub content: String,
    /// How important this hint is.
    pub priority: HintPriority,
}

/// Contextual guidance injection for model steering.
///
/// Hints are checked every iteration and inject messages into the conversation
/// when triggered. They collaborate with the model by providing context and reminders.
///
/// Part of the Guard-Hint-Track steering system.
pub trait Hint: Send + Sync + 'static {
    /// The name of this hint (for logging and tracing).
    fn name(&self) -> &'static str;

    /// Whether this hint should trigger based on current agent state.
    fn should_trigger(&self, state: &AgentState) -> bool;

    /// Generate the hint message to inject.
    fn generate(&self, state: &AgentState) -> HintMessage;

    /// Where in the conversation to inject this hint.
    fn injection_point(&self) -> InjectionPoint;
}

/// No-op hint that never triggers. Used when no hints are configured.
pub struct NoopHint;

impl Hint for NoopHint {
    fn name(&self) -> &'static str {
        "noop"
    }

    fn should_trigger(&self, _state: &AgentState) -> bool {
        false
    }

    fn generate(&self, _state: &AgentState) -> HintMessage {
        HintMessage {
            role: MessageRole::System,
            content: String::new(),
            priority: HintPriority::Low,
        }
    }

    fn injection_point(&self) -> InjectionPoint {
        InjectionPoint::BeforeNextLlmCall
    }
}
