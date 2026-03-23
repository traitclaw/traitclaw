//! The `OpenAiCompatProvider` — implements [`Provider`] for any OpenAI-compatible endpoint.
//!
//! # Supported endpoints
//!
//! - **OpenAI** — `https://api.openai.com/v1`
//! - **Ollama** — `http://localhost:11434/v1`
//! - **Groq** — `https://api.groq.com/openai/v1`
//! - **Mistral** — `https://api.mistral.ai/v1`
//! - **Together AI** — `https://api.together.xyz/v1`
//! - **vLLM** — `http://localhost:8000/v1`
//! - **Azure OpenAI** — `https://{resource}.openai.azure.com/openai/deployments/{deployment}`
//!
//! # Example
//!
//! ```rust,no_run
//! use baseclaw_openai_compat::{OpenAiCompatProvider, OpenAiCompatConfig};
//!
//! let provider = OpenAiCompatProvider::openai("gpt-4o-mini", std::env::var("OPENAI_API_KEY").unwrap());
//!
//! // Or against a local Ollama instance:
//! let provider = OpenAiCompatProvider::ollama("llama3.2");
//! ```

use async_trait::async_trait;
use baseclaw_core::traits::provider::{ModelInfo, ModelTier, Provider};
use baseclaw_core::types::completion::{CompletionRequest, CompletionResponse};
use baseclaw_core::types::stream::StreamEvent;
use baseclaw_core::{Error, Result};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::Stream;

use crate::convert::{from_wire, to_wire};
use crate::wire::{ChatResponse, StreamChunk};

// Known base URLs
const OPENAI_BASE: &str = "https://api.openai.com/v1";
const OLLAMA_BASE: &str = "http://localhost:11434/v1";
const GROQ_BASE: &str = "https://api.groq.com/openai/v1";

/// Configuration for an `OpenAiCompatProvider`.
#[derive(Debug, Clone)]
pub struct OpenAiCompatConfig {
    /// Base URL for the endpoint (without trailing `/`)
    pub base_url: String,
    /// API key (Bearer token). Empty string for Ollama and unauthenticated endpoints.
    pub api_key: String,
    /// Model name as expected by the endpoint.
    pub model: String,
    /// Optional `model_info` override. Defaults to heuristic inference.
    pub model_info: Option<ModelInfo>,
}

/// OpenAI-compatible [`Provider`] implementation.
///
/// Works with any endpoint following the `POST /v1/chat/completions` format.
pub struct OpenAiCompatProvider {
    config: OpenAiCompatConfig,
    client: reqwest::Client,
    model_info: ModelInfo,
}

impl OpenAiCompatProvider {
    /// Create a provider for **OpenAI** APIs.
    #[must_use]
    pub fn openai(model: impl Into<String>, api_key: impl Into<String>) -> Self {
        let model = model.into();
        Self::new(OpenAiCompatConfig {
            base_url: OPENAI_BASE.to_string(),
            api_key: api_key.into(),
            model_info: Some(infer_model_info(&model, OPENAI_BASE)),
            model,
        })
    }

    /// Create a provider for **Ollama** (local, no auth).
    #[must_use]
    pub fn ollama(model: impl Into<String>) -> Self {
        let model = model.into();
        Self::new(OpenAiCompatConfig {
            base_url: OLLAMA_BASE.to_string(),
            api_key: String::new(),
            model_info: Some(infer_model_info(&model, OLLAMA_BASE)),
            model,
        })
    }

    /// Create a provider for **Groq**.
    #[must_use]
    pub fn groq(model: impl Into<String>, api_key: impl Into<String>) -> Self {
        let model = model.into();
        Self::new(OpenAiCompatConfig {
            base_url: GROQ_BASE.to_string(),
            api_key: api_key.into(),
            model_info: Some(infer_model_info(&model, GROQ_BASE)),
            model,
        })
    }

    /// Create a provider for any OpenAI-compatible endpoint.
    ///
    /// # Panics
    ///
    /// Panics if the `reqwest` HTTP client cannot be built (extremely unlikely in practice).
    #[must_use]
    pub fn new(config: OpenAiCompatConfig) -> Self {
        let model_info = config
            .model_info
            .clone()
            .unwrap_or_else(|| infer_model_info(&config.model, &config.base_url));

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            config,
            client,
            model_info,
        }
    }

    fn chat_url(&self) -> String {
        format!("{}/chat/completions", self.config.base_url)
    }

    fn add_auth(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if self.config.api_key.is_empty() {
            builder
        } else {
            builder.bearer_auth(&self.config.api_key)
        }
    }
}

#[async_trait]
impl Provider for OpenAiCompatProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let wire = to_wire(request);
        tracing::debug!(model = wire.model.as_str(), "Sending completion request");

        let builder = self.client.post(self.chat_url()).json(&wire);
        let builder = self.add_auth(builder);

        let http_resp = builder
            .send()
            .await
            .map_err(|e| Error::provider(format!("HTTP error: {e}")))?;

        let status = http_resp.status();
        if !status.is_success() {
            let body = http_resp.text().await.unwrap_or_default();
            return Err(Error::provider(format!("API error {status}: {body}")));
        }

        let chat_resp: ChatResponse = http_resp
            .json()
            .await
            .map_err(|e| Error::provider(format!("Parse error: {e}")))?;

        from_wire(chat_resp)
    }

    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<std::pin::Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>> {
        let mut wire = to_wire(request);
        wire.stream = true;

        tracing::debug!(model = wire.model.as_str(), "Sending streaming request");

        let builder = self.client.post(self.chat_url()).json(&wire);
        let builder = self.add_auth(builder);

        let http_resp = builder
            .send()
            .await
            .map_err(|e| Error::provider(format!("HTTP stream error: {e}")))?;

        let status = http_resp.status();
        if !status.is_success() {
            let body = http_resp.text().await.unwrap_or_default();
            return Err(Error::provider(format!(
                "API stream error {status}: {body}"
            )));
        }

        let (tx, rx) = tokio::sync::mpsc::channel::<Result<StreamEvent>>(64);
        let byte_stream = http_resp.bytes_stream();

        tokio::spawn(parse_sse(byte_stream, tx));

        Ok(Box::pin(ReceiverStream::new(rx)))
    }

    fn model_info(&self) -> &ModelInfo {
        &self.model_info
    }
}

/// Parse SSE byte stream and send `StreamEvent`s.
async fn parse_sse(
    mut byte_stream: impl tokio_stream::Stream<Item = reqwest::Result<bytes::Bytes>> + Send + Unpin,
    tx: tokio::sync::mpsc::Sender<Result<StreamEvent>>,
) {
    use tokio_stream::StreamExt;

    let mut buffer = String::new();
    // Track partial tool calls: index → (id, name, accumulated_args)
    let mut tool_calls: std::collections::HashMap<u32, (String, String, String)> =
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

        // Process complete lines
        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].trim().to_string();
            buffer.drain(..=pos);

            if line.is_empty() || !line.starts_with("data: ") {
                continue;
            }

            let data = &line["data: ".len()..];

            if data == "[DONE]" {
                let _ = tx.send(Ok(StreamEvent::Done)).await;
                return;
            }

            let chunk_val: StreamChunk = match serde_json::from_str(data) {
                Ok(v) => v,
                Err(_) => continue,
            };

            for choice in chunk_val.choices {
                // Text delta
                if let Some(text) = choice.delta.content.filter(|t| !t.is_empty()) {
                    if tx.send(Ok(StreamEvent::TextDelta(text))).await.is_err() {
                        return;
                    }
                }

                // Tool call deltas
                if let Some(tc_deltas) = choice.delta.tool_calls {
                    for tc_delta in tc_deltas {
                        let entry = tool_calls
                            .entry(tc_delta.index)
                            .or_insert_with(|| (String::new(), String::new(), String::new()));

                        if let Some(id) = &tc_delta.id {
                            entry.0.clone_from(id);
                        }

                        if let Some(ref func) = tc_delta.function {
                            if let Some(ref name) = func.name {
                                entry.1.clone_from(name);
                                let id = entry.0.clone();
                                let sent = tx
                                    .send(Ok(StreamEvent::ToolCallStart {
                                        id,
                                        name: name.clone(),
                                    }))
                                    .await;
                                if sent.is_err() {
                                    return;
                                }
                            }

                            if let Some(ref args_delta) = func.arguments {
                                entry.2.push_str(args_delta);
                                let id = entry.0.clone();
                                let sent = tx
                                    .send(Ok(StreamEvent::ToolCallDelta {
                                        id,
                                        arguments_delta: args_delta.clone(),
                                    }))
                                    .await;
                                if sent.is_err() {
                                    return;
                                }
                            }
                        }
                    }
                }

                let finish = choice.finish_reason.as_deref();
                if finish == Some("stop") || finish == Some("tool_calls") {
                    let _ = tx.send(Ok(StreamEvent::Done)).await;
                    return;
                }
            }
        }
    }

    // Stream ended without [DONE]
    let _ = tx.send(Ok(StreamEvent::Done)).await;
}

/// Infer `ModelInfo` from model name heuristics.
fn infer_model_info(model: &str, base_url: &str) -> ModelInfo {
    let m = model.to_lowercase();

    let tier = if m.contains("gpt-4o") && !m.contains("mini")
        || m.contains("claude-3-5-sonnet")
        || m.contains("claude-3-opus")
        || m.contains("70b")
        || m.contains("mistral-large")
        || m.contains("gemini-1.5-pro")
    {
        ModelTier::Large
    } else if m.contains("gpt-4o-mini")
        || m.contains("haiku")
        || m.contains("phi")
        || m.contains("gemini-flash")
        || m.contains("7b")
        || m.contains("8b")
    {
        ModelTier::Small
    } else {
        ModelTier::Medium
    };

    let context_window = if m.contains("gpt-4o") {
        128_000
    } else if m.contains("claude") {
        200_000
    } else if m.contains("gemini-1.5") {
        1_000_000
    } else if base_url.contains("localhost") {
        8_192
    } else {
        32_768
    };

    ModelInfo::new(
        model,
        tier,
        context_window,
        true,
        m.contains("vision")
            || (m.contains("gpt-4o") && !m.contains("mini"))
            || m.contains("claude-3"),
        m.contains("gpt-4o") || m.contains("claude-3-5-sonnet"),
    )
}
