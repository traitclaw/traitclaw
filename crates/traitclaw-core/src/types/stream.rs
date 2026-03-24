//! Streaming types for incremental responses.

use std::pin::Pin;

use tokio_stream::Stream;

use crate::Result;

/// Events emitted during streaming.
///
/// This enum is `#[non_exhaustive]` — new events (e.g., `Usage`, `Metadata`)
/// may be added in future releases. Always include a wildcard arm when matching.
#[derive(Debug, Clone)]
#[non_exhaustive]
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
    ///
    /// Providers MUST emit `Done` as the final event. Consumers may use
    /// `Done` to flush accumulated state (e.g. save to memory).
    Done,
}

/// A stream of completion events.
pub type CompletionStream = Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_delta() {
        let e = StreamEvent::TextDelta("chunk".into());
        assert!(format!("{e:?}").contains("TextDelta"));
    }

    #[test]
    fn test_tool_call_start() {
        let e = StreamEvent::ToolCallStart {
            id: "call_1".into(),
            name: "search".into(),
        };
        let dbg = format!("{e:?}");
        assert!(dbg.contains("call_1"));
        assert!(dbg.contains("search"));
    }

    #[test]
    fn test_tool_call_delta() {
        let e = StreamEvent::ToolCallDelta {
            id: "call_1".into(),
            arguments_delta: "{\"q\":".into(),
        };
        assert!(format!("{e:?}").contains("ToolCallDelta"));
    }

    #[test]
    fn test_done() {
        let e = StreamEvent::Done;
        assert!(format!("{e:?}").contains("Done"));
    }

    #[test]
    fn test_stream_event_clone() {
        let e = StreamEvent::TextDelta("abc".into());
        let c = e.clone();
        assert!(format!("{c:?}").contains("abc"));
    }
}

