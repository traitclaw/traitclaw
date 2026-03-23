//! Streaming runtime — returns an async [`Stream`] of [`StreamEvent`]s.

use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;

use crate::agent::Agent;
use crate::types::agent_state::AgentState;
use crate::types::completion::CompletionRequest;
use crate::types::message::{Message, MessageRole};
use crate::types::stream::StreamEvent;
use crate::Result;

/// A pinned, owned stream of [`StreamEvent`]s.
pub type AgentStream =
    std::pin::Pin<Box<dyn tokio_stream::Stream<Item = Result<StreamEvent>> + Send>>;

/// Run the agent and return a streaming response.
pub(crate) fn stream_agent(agent: &Agent, input: String) -> AgentStream {
    let provider = agent.provider.clone();
    let memory = agent.memory.clone();
    let tools = agent.tools.clone();
    let hints = agent.hints.clone();
    let config = agent.config.clone();

    let (tx, rx) = tokio::sync::mpsc::channel::<Result<StreamEvent>>(32);

    tokio::spawn(async move {
        let model_info = provider.model_info();
        let session_id = "default";

        let mut messages = match memory.messages(session_id).await {
            Ok(m) => m,
            Err(e) => {
                let _ = tx.send(Err(e)).await;
                return;
            }
        };

        if let Some(ref sys) = config.system_prompt {
            if messages.is_empty() || messages[0].role != MessageRole::System {
                messages.insert(0, Message::system(sys));
            }
        }

        let user_msg = Message::user(&input);
        messages.push(user_msg.clone());
        if let Err(e) = memory.append(session_id, user_msg).await {
            let _ = tx.send(Err(e)).await;
            return;
        }

        let tool_schemas = tools.iter().map(|t| t.schema()).collect::<Vec<_>>();
        let state = AgentState::new(model_info.tier, model_info.context_window);

        for _iteration in 0..config.max_iterations {
            // Inject hints
            for hint in &hints {
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
                max_tokens: config.max_tokens,
                temperature: config.temperature,
                stream: true,
            };

            let stream = match provider.stream(request).await {
                Ok(s) => s,
                Err(e) => {
                    let _ = tx.send(Err(e)).await;
                    return;
                }
            };

            let done = forward_stream(stream, &tx, session_id, &memory).await;
            if done {
                return;
            }
        }

        let _ = tx
            .send(Err(crate::Error::Runtime(
                "Agent reached maximum iterations".into(),
            )))
            .await;
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
