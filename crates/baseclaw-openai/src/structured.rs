//! Native OpenAI structured output support.
//!
//! Wraps any `Provider` and injects a JSON Schema instruction so the model
//! always returns valid JSON matching your Rust type.
//!
//! # Example
//!
//! ```rust,no_run
//! use baseclaw_openai::structured::StructuredOutputProvider;
//! use baseclaw_openai::openai;
//! use schemars::JsonSchema;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize, JsonSchema)]
//! struct WeatherReport {
//!     city: String,
//!     temperature_celsius: f32,
//!     condition: String,
//! }
//!
//! # async fn example() -> baseclaw_core::Result<()> {
//! let provider = StructuredOutputProvider::<WeatherReport>::new(
//!     openai("gpt-4o-mini"),
//!     "weather_report",
//! );
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;
use baseclaw_core::traits::provider::{ModelInfo, Provider};
use baseclaw_core::types::completion::{CompletionRequest, CompletionResponse};
use baseclaw_core::types::message::{Message, MessageRole};
use baseclaw_core::types::stream::StreamEvent;
use baseclaw_core::{Error, Result};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use tokio_stream::Stream;

/// A provider wrapper that enforces structured JSON output.
///
/// Wraps any `Provider` and injects a system message describing the required
/// JSON schema. The response text can then be deserialized into `T` via
/// [`StructuredOutputProvider::parse`].
pub struct StructuredOutputProvider<T>
where
    T: JsonSchema + DeserializeOwned + Send + Sync + 'static,
{
    inner: Box<dyn Provider>,
    schema: serde_json::Value,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> StructuredOutputProvider<T>
where
    T: JsonSchema + DeserializeOwned + Send + Sync + 'static,
{
    /// Create a new `StructuredOutputProvider`.
    ///
    /// - `inner` — the underlying provider (e.g. from `openai("gpt-4o-mini")`)
    /// - `schema_name` — a short identifier used in the schema instruction (snake_case)
    #[must_use]
    pub fn new(inner: impl Provider + 'static, schema_name: impl Into<String>) -> Self {
        let schema_name = schema_name.into();
        let root_schema = schemars::schema_for!(T);
        let mut schema_value =
            serde_json::to_value(&root_schema).unwrap_or(serde_json::Value::Null);

        // Annotate with the schema name for clarity in the instruction
        if let Some(obj) = schema_value.as_object_mut() {
            obj.insert("title".to_string(), serde_json::Value::String(schema_name));
        }

        Self {
            inner: Box::new(inner),
            schema: schema_value,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Deserialize the text response into `T`.
    ///
    /// # Errors
    ///
    /// Returns an error if the response content is not text or JSON fails to parse.
    pub fn parse(response: &CompletionResponse) -> Result<T> {
        use baseclaw_core::types::completion::ResponseContent;
        match &response.content {
            ResponseContent::Text(text) => serde_json::from_str(text)
                .map_err(|e| Error::provider(format!("Structured output parse error: {e}"))),
            ResponseContent::ToolCalls(_) => Err(Error::provider(
                "Expected structured text response, got tool calls",
            )),
        }
    }

    fn inject_schema_instruction(&self, mut request: CompletionRequest) -> CompletionRequest {
        let schema_str = serde_json::to_string_pretty(&self.schema).unwrap_or_default();
        let instruction = format!(
            "You MUST respond with valid JSON matching this exact schema. \
             Do not add any text, explanation, or markdown. \
             Only output the raw JSON object.\n\nSchema:\n{schema_str}"
        );

        let has_system = request
            .messages
            .iter()
            .any(|m| matches!(m.role, MessageRole::System));

        if has_system {
            for msg in &mut request.messages {
                if matches!(msg.role, MessageRole::System) {
                    msg.content.push_str("\n\n");
                    msg.content.push_str(&instruction);
                    break;
                }
            }
        } else {
            request.messages.insert(
                0,
                Message {
                    role: MessageRole::System,
                    content: instruction,
                    tool_call_id: None,
                },
            );
        }

        request
    }
}

#[async_trait]
impl<T> Provider for StructuredOutputProvider<T>
where
    T: JsonSchema + DeserializeOwned + Send + Sync + 'static,
{
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let enriched = self.inject_schema_instruction(request);
        self.inner.complete(enriched).await
    }

    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<std::pin::Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>> {
        let enriched = self.inject_schema_instruction(request);
        self.inner.stream(enriched).await
    }

    fn model_info(&self) -> &ModelInfo {
        self.inner.model_info()
    }
}
