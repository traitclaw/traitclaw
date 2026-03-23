//! Streaming types for incremental responses.

use std::pin::Pin;

use tokio_stream::Stream;

use crate::Result;

/// Events emitted during streaming.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// A chunk of text from the LLM.
    TextDelta(String),
    /// A tool call has started.
    ToolCallStart {
        /// The tool call ID.
        id: String,
        /// The tool name.
        name: String,
    },
    /// A chunk of tool call arguments.
    ToolCallDelta {
        /// The tool call ID.
        id: String,
        /// Partial arguments JSON.
        arguments_delta: String,
    },
    /// The stream has completed.
    Done,
}

/// A stream of completion events.
pub type CompletionStream = Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>;
