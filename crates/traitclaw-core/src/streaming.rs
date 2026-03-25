//! Streaming runtime — returns an async [`Stream`] of [`StreamEvent`]s.

use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

use crate::types::agent_state::AgentState;
use crate::types::completion::CompletionRequest;
use crate::types::message::{Message, MessageRole};
use crate::types::stream::StreamEvent;
use crate::Result;

/// A pinned, owned stream of [`StreamEvent`]s.
pub type AgentStream =
    std::pin::Pin<Box<dyn tokio_stream::Stream<Item = Result<StreamEvent>> + Send>>;

/// Run the agent and return a streaming response via runtime.
pub(crate) fn stream_runtime(
    runtime: crate::traits::strategy::AgentRuntime,
    input: String,
    session_id: String,
) -> AgentStream {
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<StreamEvent>>(32);

    tokio::spawn(async move {
        for hook in &runtime.hooks {
            hook.on_agent_start(&input).await;
        }

        let model_info = runtime.provider.model_info();

        // Memory load is non-fatal — start fresh on failure
        let mut messages = match runtime.memory.messages(&session_id).await {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("Streaming: failed to load memory: {e}");
                Vec::new()
            }
        };

        if let Some(ref sys) = runtime.config.system_prompt {
            if messages.is_empty() || messages[0].role != MessageRole::System {
                messages.insert(0, Message::system(sys));
            }
        }

        let user_msg = Message::user(&input);
        messages.push(user_msg.clone());
        if let Err(e) = runtime.memory.append(&session_id, user_msg).await {
            tracing::warn!("Streaming: failed to save user message to memory: {e}");
        }

        let tool_schemas = runtime.tools.iter().map(|t| t.schema()).collect::<Vec<_>>();
        let state = AgentState::new(model_info.tier, model_info.context_window);

        for _iteration in 0..runtime.config.max_iterations {
            for hint in &runtime.hints {
                if hint.should_trigger(&state) {
                    let hm = hint.generate(&state);
                    messages.push(Message {
                        role: hm.role,
                        content: hm.content,
                        tool_call_id: None,
                    });
                }
            }

            let request = CompletionRequest {
                model: model_info.name.clone(),
                messages: messages.clone(),
                tools: tool_schemas.clone(),
                max_tokens: runtime.config.max_tokens,
                temperature: runtime.config.temperature,
                response_format: None,
                stream: true,
            };

            for hook in &runtime.hooks {
                hook.on_provider_start(&request).await;
            }

            let stream = match runtime.provider.stream(request).await {
                Ok(s) => s,
                Err(e) => {
                    for hook in &runtime.hooks {
                        hook.on_error(&e).await;
                    }
                    let _ = tx.send(Err(e)).await;
                    return;
                }
            };

            let done = forward_stream(stream, &tx, &session_id, &runtime.memory).await;
            if done {
                return;
            }
        }

        let err = crate::Error::Runtime("Agent reached maximum iterations".into());
        for hook in &runtime.hooks {
            hook.on_error(&err).await;
        }
        let _ = tx.send(Err(err)).await;
    });

    Box::pin(ReceiverStream::new(rx))
}

/// Forward stream events to the channel. Returns `true` when the stream is done.
async fn forward_stream(
    mut stream: crate::types::stream::CompletionStream,
    tx: &tokio::sync::mpsc::Sender<Result<StreamEvent>>,
    session_id: &str,
    memory: &std::sync::Arc<dyn crate::traits::memory::Memory>,
) -> bool {
    let mut accumulated_text = String::new();
    let mut has_tool_calls = false;

    while let Some(result) = stream.next().await {
        match result {
            Ok(StreamEvent::TextDelta(delta)) => {
                accumulated_text.push_str(&delta);
                if tx.send(Ok(StreamEvent::TextDelta(delta))).await.is_err() {
                    return true;
                }
            }
            Ok(StreamEvent::Done) => {
                if !accumulated_text.is_empty() {
                    let _ = memory
                        .append(session_id, Message::assistant(&accumulated_text))
                        .await;
                }
                let _ = tx.send(Ok(StreamEvent::Done)).await;
                return true;
            }
            Ok(StreamEvent::ToolCallStart { id, name }) => {
                has_tool_calls = true;
                if tx
                    .send(Ok(StreamEvent::ToolCallStart { id, name }))
                    .await
                    .is_err()
                {
                    return true;
                }
            }
            Ok(StreamEvent::ToolCallDelta {
                id,
                arguments_delta,
            }) => {
                if tx
                    .send(Ok(StreamEvent::ToolCallDelta {
                        id,
                        arguments_delta,
                    }))
                    .await
                    .is_err()
                {
                    return true;
                }
            }
            Err(e) => {
                let _ = tx.send(Err(e)).await;
                return true;
            }
        }
    }

    // Stream ended without Done
    if !has_tool_calls && accumulated_text.is_empty() {
        let _ = tx
            .send(Err(crate::Error::Runtime(
                "Stream ended without content".into(),
            )))
            .await;
        return true;
    }

    false // Tool calls received, continue loop
}

#[cfg(test)]
mod tests {

    use async_trait::async_trait;
    use tokio_stream::StreamExt;

    use crate::agent::Agent;
    use crate::traits::provider::Provider;
    use crate::types::completion::{CompletionRequest, CompletionResponse, ResponseContent, Usage};
    use crate::types::model_info::{ModelInfo, ModelTier};
    use crate::types::stream::{CompletionStream, StreamEvent};

    /// Mock provider that returns streaming events from a predefined list.
    struct StreamingMockProvider {
        info: ModelInfo,
        events: Vec<StreamEvent>,
    }

    impl StreamingMockProvider {
        fn new(events: Vec<StreamEvent>) -> Self {
            Self {
                info: ModelInfo::new("test-stream", ModelTier::Small, 4096, false, true, false),
                events,
            }
        }
    }

    #[async_trait]
    impl Provider for StreamingMockProvider {
        async fn complete(&self, _req: CompletionRequest) -> crate::Result<CompletionResponse> {
            // Not used in streaming tests
            Ok(CompletionResponse {
                content: ResponseContent::Text("fallback".into()),
                usage: Usage::default(),
            })
        }

        async fn stream(&self, _req: CompletionRequest) -> crate::Result<CompletionStream> {
            let (tx, rx) = tokio::sync::mpsc::channel(32);
            let events = self.events.clone();
            tokio::spawn(async move {
                for event in events {
                    if tx.send(Ok(event)).await.is_err() {
                        break;
                    }
                }
            });
            Ok(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)))
        }

        fn model_info(&self) -> &ModelInfo {
            &self.info
        }
    }

    #[tokio::test]
    async fn test_text_stream_yields_text_deltas_ac2_ac3() {
        // AC-2/3: AgentStream yields StreamEvent::TextDelta + Done
        let provider = StreamingMockProvider::new(vec![
            StreamEvent::TextDelta("Hello, ".into()),
            StreamEvent::TextDelta("world!".into()),
            StreamEvent::Done,
        ]);

        let agent = Agent::builder()
            .model(provider)
            .system("You stream")
            .build()
            .unwrap();

        let mut stream = agent.stream("Hi");
        let mut events = vec![];
        while let Some(result) = stream.next().await {
            events.push(result.unwrap());
        }

        assert_eq!(events.len(), 3);
        assert!(matches!(&events[0], StreamEvent::TextDelta(t) if t == "Hello, "));
        assert!(matches!(&events[1], StreamEvent::TextDelta(t) if t == "world!"));
        assert!(matches!(&events[2], StreamEvent::Done));
    }

    #[tokio::test]
    async fn test_done_is_always_last_ac3() {
        // AC-3: Done variant is the last event
        let provider = StreamingMockProvider::new(vec![
            StreamEvent::TextDelta("chunk".into()),
            StreamEvent::Done,
        ]);

        let agent = Agent::builder()
            .model(provider)
            .system("You stream")
            .build()
            .unwrap();

        let mut stream = agent.stream("Hi");
        let mut last_event = None;
        while let Some(result) = stream.next().await {
            last_event = Some(result.unwrap());
        }

        assert!(matches!(last_event, Some(StreamEvent::Done)));
    }

    #[tokio::test]
    async fn test_stream_saves_to_memory_ac1() {
        // AC-1: After streaming completes, accumulated text is saved to memory
        let provider = StreamingMockProvider::new(vec![
            StreamEvent::TextDelta("Saved ".into()),
            StreamEvent::TextDelta("text".into()),
            StreamEvent::Done,
        ]);

        let agent = Agent::builder()
            .model(provider)
            .system("You stream")
            .build()
            .unwrap();

        let mut stream = agent.stream("Save this");
        // Consume the entire stream
        while stream.next().await.is_some() {}

        // Verify memory was saved
        let msgs = agent.memory.messages("default").await.unwrap();
        // Should have user message + assistant accumulated text
        assert!(msgs.iter().any(|m| m.content == "Save this"));
        assert!(msgs.iter().any(|m| m.content == "Saved text"));
    }
}
