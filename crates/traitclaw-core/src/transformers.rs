//! Built-in [`OutputTransformer`] implementations for common use cases.
//!
//! These transformers can be used directly or composed for more complex processing.
//!
//! # Available Transformers
//!
//! - [`BudgetAwareTruncator`] — truncate by char count based on context utilization
//! - [`JsonExtractor`] — extract JSON from verbose output
//! - [`TransformerChain`] — chain multiple transformers
//! - [`ProgressiveTransformer`] — summarize large outputs; full output on demand

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;

use crate::traits::output_transformer::OutputTransformer;
use crate::traits::provider::Provider;
use crate::types::agent_state::AgentState;
use crate::types::completion::{CompletionRequest, ResponseContent};
use crate::types::message::Message;

// ===========================================================================
// BudgetAwareTruncator
// ===========================================================================

/// Truncates output to a maximum character count, respecting context utilization.
///
/// When context utilization exceeds the `aggressive_threshold`, the limit is
/// halved to preserve context budget.
///
/// # Example
///
/// ```rust
/// use traitclaw_core::transformers::BudgetAwareTruncator;
///
/// let t = BudgetAwareTruncator::new(1000, 0.8);
/// ```
pub struct BudgetAwareTruncator {
    max_chars: usize,
    aggressive_threshold: f32,
}

impl BudgetAwareTruncator {
    /// Create a new truncator.
    ///
    /// - `max_chars`: Maximum output length in characters.
    /// - `aggressive_threshold`: Context utilization (0.0–1.0) above which
    ///   truncation becomes more aggressive (halved limit).
    #[must_use]
    pub fn new(max_chars: usize, aggressive_threshold: f32) -> Self {
        Self {
            max_chars,
            aggressive_threshold: aggressive_threshold.clamp(0.0, 1.0),
        }
    }
}

impl Default for BudgetAwareTruncator {
    fn default() -> Self {
        Self::new(10_000, 0.8)
    }
}

#[async_trait]
impl OutputTransformer for BudgetAwareTruncator {
    async fn transform(&self, output: String, _tool_name: &str, state: &AgentState) -> String {
        let limit = if state.context_utilization() > self.aggressive_threshold {
            self.max_chars / 2
        } else {
            self.max_chars
        };

        if output.len() <= limit {
            return output;
        }

        // Truncate at char boundary
        let truncated: String = output.chars().take(limit).collect();
        format!(
            "{truncated}\n\n[output truncated from {} to {limit} chars]",
            output.len()
        )
    }
}

// ===========================================================================
// JsonExtractor
// ===========================================================================

/// Extracts JSON from tool output, discarding surrounding text.
///
/// Useful for tools that embed JSON in verbose output.
pub struct JsonExtractor;

#[async_trait]
impl OutputTransformer for JsonExtractor {
    async fn transform(&self, output: String, _tool_name: &str, _state: &AgentState) -> String {
        // Try to find JSON object or array in the output
        if let Some(start) = output.find('{') {
            if let Some(end) = output.rfind('}') {
                if end >= start {
                    return output[start..=end].to_string();
                }
            }
        }
        if let Some(start) = output.find('[') {
            if let Some(end) = output.rfind(']') {
                if end >= start {
                    return output[start..=end].to_string();
                }
            }
        }
        // No JSON found, return as-is
        output
    }
}

// ===========================================================================
// TransformerChain
// ===========================================================================

/// Pipes output through multiple transformers in order.
pub struct TransformerChain {
    transformers: Vec<Box<dyn OutputTransformer>>,
}

impl TransformerChain {
    /// Create a chain from a list of transformers.
    #[must_use]
    pub fn new(transformers: Vec<Box<dyn OutputTransformer>>) -> Self {
        Self { transformers }
    }
}

#[async_trait]
impl OutputTransformer for TransformerChain {
    async fn transform(&self, mut output: String, tool_name: &str, state: &AgentState) -> String {
        for t in &self.transformers {
            output = t.transform(output, tool_name, state).await;
        }
        output
    }
}

// ===========================================================================
// ProgressiveTransformer
// ===========================================================================

/// Default summarization prompt template.
const DEFAULT_SUMMARY_PROMPT: &str =
    "Summarize the following tool output concisely, preserving all key data points and values. \
     Be brief but complete:\n\n{output}";

/// A two-phase output transformer that returns an **LLM-generated summary** first,
/// with the **full output** cached and available on demand via the
/// `__get_full_output` virtual tool.
///
/// # Workflow
///
/// 1. Output arrives from a tool call.
/// 2. If `output.len() <= max_summary_length` → returned unchanged (no LLM call).
/// 3. If larger → LLM is called to summarize → summary returned + note appended.
/// 4. Full output cached internally keyed by `tool_name`.
/// 5. Agent can call `__get_full_output` → [`FullOutputRetriever`] serves it.
///
/// # Example
///
/// ```rust,no_run
/// use traitclaw_core::transformers::ProgressiveTransformer;
/// use std::sync::Arc;
///
/// // let transformer = ProgressiveTransformer::new(provider.clone(), 500)
/// //     .with_summary_prompt("Give a one-sentence summary: {output}");
/// ```
pub struct ProgressiveTransformer {
    provider: Arc<dyn Provider>,
    max_summary_length: usize,
    summary_prompt: String,
    /// Cache: tool_name → full output
    cache: Arc<RwLock<HashMap<String, String>>>,
}

impl ProgressiveTransformer {
    /// Create a new progressive transformer.
    ///
    /// - `provider`: LLM used to generate summaries.
    /// - `max_summary_length`: Outputs shorter than this are passed through unchanged.
    #[must_use]
    pub fn new(provider: Arc<dyn Provider>, max_summary_length: usize) -> Self {
        Self {
            provider,
            max_summary_length,
            summary_prompt: DEFAULT_SUMMARY_PROMPT.to_string(),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Override the summarization prompt template.
    ///
    /// Use `{output}` as the placeholder for the tool output.
    #[must_use]
    pub fn with_summary_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.summary_prompt = prompt.into();
        self
    }

    /// Build a [`FullOutputRetriever`] that reads from this transformer's cache.
    ///
    /// Register this tool with the agent so the LLM can call `__get_full_output`.
    #[must_use]
    pub fn retriever_tool(&self) -> FullOutputRetriever {
        FullOutputRetriever {
            cache: Arc::clone(&self.cache),
        }
    }

    /// Store full output in cache keyed by tool name.
    fn cache_output(&self, tool_name: &str, output: &str) {
        let mut cache = self
            .cache
            .write()
            .expect("ProgressiveTransformer cache lock poisoned");
        cache.insert(tool_name.to_string(), output.to_string());
    }

    /// Build the prompt with the output injected.
    fn build_prompt(&self, output: &str) -> String {
        self.summary_prompt.replace("{output}", output)
    }
}

#[async_trait]
impl OutputTransformer for ProgressiveTransformer {
    async fn transform(&self, output: String, tool_name: &str, _state: &AgentState) -> String {
        // AC #7: short output → pass through unchanged
        if output.len() <= self.max_summary_length {
            return output;
        }

        // Cache the full output for later retrieval
        self.cache_output(tool_name, &output);

        // AC #2: call LLM to summarize
        let prompt = self.build_prompt(&output);
        let request = CompletionRequest {
            model: self.provider.model_info().name.clone(),
            messages: vec![Message::user(prompt)],
            tools: vec![],
            max_tokens: Some(500),
            temperature: Some(0.3),
            response_format: None,
            stream: false,
        };

        match self.provider.complete(request).await {
            Ok(response) => {
                // AC #2: return summary + footer note
                let summary = match response.content {
                    ResponseContent::Text(t) => t,
                    ResponseContent::ToolCalls(_) => {
                        // Unexpected tool calls from summarizer — fallback
                        let truncated: String =
                            output.chars().take(self.max_summary_length).collect();
                        return format!(
                            "{truncated}\n\n\
                             [output truncated from {} chars — summarizer returned tool calls]",
                            output.len()
                        );
                    }
                };
                format!(
                    "{summary}\n\n\
                     [Full output ({} chars) cached. \
                     Call __get_full_output with {{\"tool_name\": \"{tool_name}\"}} to retrieve it.]",
                    output.len()
                )
            }
            Err(e) => {
                // AC #6: fallback to truncation on LLM failure
                tracing::warn!(
                    "ProgressiveTransformer: LLM summarization failed for '{tool_name}': {e}. \
                     Falling back to truncation."
                );
                let truncated: String = output.chars().take(self.max_summary_length).collect();
                format!(
                    "{truncated}\n\n\
                     [output truncated from {} chars — LLM summarization failed]",
                    output.len()
                )
            }
        }
    }
}

// ===========================================================================
// FullOutputRetriever — virtual tool
// ===========================================================================

/// Virtual tool that retrieves the full output cached by [`ProgressiveTransformer`].
///
/// The LLM calls this tool as `__get_full_output` with `{"tool_name": "..."}`.
///
/// Obtain via [`ProgressiveTransformer::retriever_tool()`].
pub struct FullOutputRetriever {
    cache: Arc<RwLock<HashMap<String, String>>>,
}

impl FullOutputRetriever {
    /// Retrieve cached full output for a tool name.
    ///
    /// Returns the cached output or an error message if not found.
    #[must_use]
    pub fn retrieve(&self, tool_name: &str) -> String {
        let cache = self
            .cache
            .read()
            .expect("FullOutputRetriever cache lock poisoned");
        match cache.get(tool_name) {
            Some(output) => output.clone(),
            None => format!(
                "[No cached output found for tool '{tool_name}'. \
                 The output may have expired or the tool name is incorrect.]"
            ),
        }
    }

    /// Check if a full output exists in cache for the given tool name.
    #[must_use]
    pub fn has_cached(&self, tool_name: &str) -> bool {
        let cache = self
            .cache
            .read()
            .expect("FullOutputRetriever cache lock poisoned");
        cache.contains_key(tool_name)
    }
}
