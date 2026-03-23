//! `AnthropicProvider` — implements [`Provider`] for the Anthropic Messages API.
//!
//! Supports Claude 3.5 Sonnet, Claude 3.5 Haiku, Claude 3 Opus, and other Claude models.
//!
//! # Example
//!
//! ```rust,no_run
//! use baseclaw_anthropic::AnthropicProvider;
//!
//! let provider = AnthropicProvider::new(
//!     "claude-3-5-sonnet-20241022",
//!     std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY must be set"),
//! );
//! ```

use async_trait::async_trait;
use baseclaw_core::traits::provider::{ModelInfo, ModelTier, Provider};
use baseclaw_core::types::completion::{CompletionRequest, CompletionResponse};
use baseclaw_core::types::stream::StreamEvent;
use baseclaw_core::{Error, Result};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::Stream;

use crate::convert::{from_wire, to_wire};
use crate::wire::{
    MessagesResponse, StreamContentBlock, StreamDelta, StreamEvent as AnthropicEvent,
    ANTHROPIC_BASE, ANTHROPIC_VERSION,
};

/// Provider for the Anthropic Messages API.
pub struct AnthropicProvider {
    api_key: String,
    #[allow(dead_code)]
    model: String,
    client: reqwest::Client,
    model_info: ModelInfo,
}

impl AnthropicProvider {
    /// Create a new `AnthropicProvider`.
    ///
    /// # Panics
    ///
    /// Panics if the `reqwest` HTTP client cannot be built (extremely unlikely in practice).
    #[must_use]
    pub fn new(model: impl Into<String>, api_key: impl Into<String>) -> Self {
        let model = model.into();
        let model_info = infer_model_info(&model);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            api_key: api_key.into(),
            model,
            client,
            model_info,
        }
    }

    fn messages_url() -> String {
        format!("{ANTHROPIC_BASE}/messages")
    }

    fn add_headers(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        builder
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let wire = to_wire(request);
        tracing::debug!(
            model = wire.model.as_str(),
            "Sending Anthropic completion request"
        );

        let builder = self.client.post(Self::messages_url()).json(&wire);
        let builder = self.add_headers(builder);

        let http_resp = builder
            .send()
            .await
            .map_err(|e| Error::provider(format!("HTTP error: {e}")))?;

        let status = http_resp.status();
        if !status.is_success() {
            let body = http_resp.text().await.unwrap_or_default();
            return Err(Error::provider(format!(
                "Anthropic API error {status}: {body}"
            )));
        }

        let resp: MessagesResponse = http_resp
            .json()
            .await
            .map_err(|e| Error::provider(format!("Parse error: {e}")))?;

        Ok(from_wire(resp))
    }

    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<std::pin::Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>> {
        let mut wire = to_wire(request);
        wire.stream = true;

        tracing::debug!(
            model = wire.model.as_str(),
            "Sending Anthropic streaming request"
        );

        let builder = self.client.post(Self::messages_url()).json(&wire);
        let builder = self.add_headers(builder);

        let http_resp = builder
            .send()
            .await
            .map_err(|e| Error::provider(format!("HTTP stream error: {e}")))?;

        let status = http_resp.status();
        if !status.is_success() {
            let body = http_resp.text().await.unwrap_or_default();
            return Err(Error::provider(format!(
                "Anthropic API stream error {status}: {body}"
            )));
        }

        let (tx, rx) = tokio::sync::mpsc::channel::<Result<StreamEvent>>(64);
        let byte_stream = http_resp.bytes_stream();

        tokio::spawn(parse_anthropic_sse(byte_stream, tx));

        Ok(Box::pin(ReceiverStream::new(rx)))
    }

    fn model_info(&self) -> &ModelInfo {
        &self.model_info
    }
}

/// Parse Anthropic SSE stream and emit `StreamEvent`s.
async fn parse_anthropic_sse(
    mut byte_stream: impl tokio_stream::Stream<Item = reqwest::Result<bytes::Bytes>> + Send + Unpin,
    tx: tokio::sync::mpsc::Sender<Result<StreamEvent>>,
) {
    use tokio_stream::StreamExt;

    let mut buffer = String::new();
    // Track tool_call blocks: index → (id, name, accumulated_json)
    let mut tool_blocks: std::collections::HashMap<u32, (String, String)> =
        std::collections::HashMap::new();

    while let Some(chunk) = byte_stream.next().await {
        let bytes = match chunk {
            Ok(b) => b,
            Err(e) => {
                let _ = tx
                    .send(Err(Error::provider(format!("Stream read error: {e}"))))
                    .await;
                return;
            }
        };

        buffer.push_str(&String::from_utf8_lossy(&bytes));

        // Process complete SSE lines (Anthropic sends `event:` + `data:` pairs)
        loop {
            let Some(newline) = buffer.find('\n') else {
                break;
            };
            let line = buffer[..newline].trim().to_string();
            buffer.drain(..=newline);

            // Only process `data:` lines — skip `event:` type lines
            if !line.starts_with("data: ") {
                continue;
            }

            let data = &line["data: ".len()..];
            if data == "[DONE]" {
                let _ = tx.send(Ok(StreamEvent::Done)).await;
                return;
            }

            let event: AnthropicEvent = match serde_json::from_str(data) {
                Ok(e) => e,
                Err(_) => continue,
            };

            match event {
                AnthropicEvent::ContentBlockStart {
                    index,
                    content_block,
                } => match content_block {
                    StreamContentBlock::ToolUse { id, name } => {
                        tool_blocks.insert(index, (id.clone(), name.clone()));
                        if tx
                            .send(Ok(StreamEvent::ToolCallStart { id, name }))
                            .await
                            .is_err()
                        {
                            return;
                        }
                    }
                    StreamContentBlock::Text { .. } => {}
                },

                AnthropicEvent::ContentBlockDelta { index, delta } => match delta {
                    StreamDelta::TextDelta { text } => {
                        if !text.is_empty()
                            && tx.send(Ok(StreamEvent::TextDelta(text))).await.is_err()
                        {
                            return;
                        }
                    }
                    StreamDelta::InputJsonDelta { partial_json } => {
                        if let Some((id, _)) = tool_blocks.get(&index) {
                            let id = id.clone();
                            if tx
                                .send(Ok(StreamEvent::ToolCallDelta {
                                    id,
                                    arguments_delta: partial_json,
                                }))
                                .await
                                .is_err()
                            {
                                return;
                            }
                        }
                    }
                },

                AnthropicEvent::MessageStop => {
                    let _ = tx.send(Ok(StreamEvent::Done)).await;
                    return;
                }

                AnthropicEvent::ContentBlockStop { .. }
                | AnthropicEvent::MessageDelta { .. }
                | AnthropicEvent::Other => {}
            }
        }
    }

    let _ = tx.send(Ok(StreamEvent::Done)).await;
}

/// Infer `ModelInfo` from an Anthropic model name.
fn infer_model_info(model: &str) -> ModelInfo {
    let m = model.to_lowercase();

    let tier = if m.contains("opus") || m.contains("claude-3-5-sonnet") {
        ModelTier::Large
    } else if m.contains("haiku") {
        ModelTier::Small
    } else {
        ModelTier::Medium
    };

    ModelInfo::new(
        model,
        tier,
        200_000, // All Claude models have 200k context
        true,
        true, // All Claude 3 models support vision
        m.contains("claude-3-5-sonnet"),
    )
}
