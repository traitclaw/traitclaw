//! Built-in [`ContextManager`] implementations for common compression strategies.
//!
//! - [`RuleBasedCompressor`]: importance-scored message pruning
//! - [`LlmCompressor`]: LLM-powered summarization of old messages
//! - [`TieredCompressor`]: chained keep-recent → rule-compress → LLM-summarize

use std::sync::Arc;

use async_trait::async_trait;

use crate::traits::context_manager::ContextManager;
use crate::traits::provider::Provider;
use crate::types::agent_state::AgentState;
use crate::types::completion::{CompletionRequest, ResponseContent};
use crate::types::message::{Message, MessageRole};

/// Estimate token count for a message list (4 chars ≈ 1 token).
fn estimate_tokens(messages: &[Message]) -> usize {
    messages.iter().map(|m| m.content.len() / 4 + 1).sum()
}

// ---------------------------------------------------------------------------
// RuleBasedCompressor
// ---------------------------------------------------------------------------

/// Scores messages by importance and removes lowest-scored first.
///
/// Scoring table (configurable):
/// - System messages: **never removed** (score ∞)
/// - Last `recent_count` messages: 0.9
/// - Tool-result messages: 0.7
/// - Older user/assistant messages: 0.3
///
/// # Example
///
/// ```rust
/// use traitclaw_core::context_managers::RuleBasedCompressor;
///
/// let compressor = RuleBasedCompressor::new(0.85, 3);
/// ```
pub struct RuleBasedCompressor {
    /// Fraction of `context_window` at which pruning begins.
    threshold: f64,
    /// Number of recent non-system messages to protect (score 0.9).
    recent_count: usize,
}

impl RuleBasedCompressor {
    /// Create a compressor with custom threshold and recent-message protection.
    ///
    /// - `threshold`: 0.0–1.0, fraction of `context_window` triggering compression.
    /// - `recent_count`: number of most-recent messages to protect from removal.
    #[must_use]
    pub fn new(threshold: f64, recent_count: usize) -> Self {
        Self {
            threshold: threshold.clamp(0.0, 1.0),
            recent_count,
        }
    }

    /// Score a message by importance. Higher = more important.
    fn score_message(msg: &Message, is_recent: bool) -> f64 {
        if msg.role == MessageRole::System {
            return f64::INFINITY; // never remove
        }
        if is_recent {
            return 0.9;
        }
        if msg.tool_call_id.is_some() || msg.role == MessageRole::Tool {
            return 0.7;
        }
        0.3
    }
}

impl Default for RuleBasedCompressor {
    fn default() -> Self {
        Self::new(0.85, 3)
    }
}

#[async_trait]
impl ContextManager for RuleBasedCompressor {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    async fn prepare(
        &self,
        messages: &mut Vec<Message>,
        context_window: usize,
        state: &mut AgentState,
    ) {
        let max_tokens = (context_window as f64 * self.threshold) as usize;

        if estimate_tokens(messages) <= max_tokens {
            return;
        }

        // Score all messages, remembering their indices and per-message token costs
        let total_non_system = messages
            .iter()
            .filter(|m| m.role != MessageRole::System)
            .count();
        let recent_start = total_non_system.saturating_sub(self.recent_count);

        let mut scored: Vec<(usize, f64, usize)> = Vec::new(); // (idx, score, tokens)
        let mut non_system_idx = 0usize;
        for (i, msg) in messages.iter().enumerate() {
            if msg.role == MessageRole::System {
                continue;
            }
            let is_recent = non_system_idx >= recent_start;
            let tokens = msg.content.len() / 4 + 1;
            scored.push((i, Self::score_message(msg, is_recent), tokens));
            non_system_idx += 1;
        }

        // Sort by score ascending (lowest importance first)
        scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        // Project removals: track projected token total without mutating messages
        let mut projected_tokens = estimate_tokens(messages);
        let mut remove_set: Vec<usize> = Vec::new();
        for &(idx, score, tokens) in &scored {
            if projected_tokens <= max_tokens {
                break;
            }
            if score.is_infinite() {
                continue; // never remove system
            }
            remove_set.push(idx);
            projected_tokens = projected_tokens.saturating_sub(tokens);
        }

        if !remove_set.is_empty() {
            // Remove in reverse index order to preserve earlier indices
            remove_set.sort_unstable();
            for &idx in remove_set.iter().rev() {
                messages.remove(idx);
            }
            state.last_output_truncated = true;
        }
    }
}

// ---------------------------------------------------------------------------
// LlmCompressor
// ---------------------------------------------------------------------------

/// Summarizes old messages using an LLM provider.
///
/// Makes exactly **one** LLM call per compression event. If the call fails,
/// falls back to rule-based pruning (remove oldest non-system messages).
///
/// # Example
///
/// ```rust,no_run
/// use traitclaw_core::context_managers::LlmCompressor;
/// # fn example(provider: std::sync::Arc<dyn traitclaw_core::Provider>) {
/// let compressor = LlmCompressor::new(provider);
/// # }
/// ```
pub struct LlmCompressor {
    /// Provider for summarization calls.
    provider: Arc<dyn Provider>,
    /// Prompt template for summarization.
    summary_prompt: String,
    /// Fraction of `context_window` at which compression triggers.
    threshold: f64,
    /// Number of recent messages to keep verbatim.
    keep_recent: usize,
}

impl LlmCompressor {
    /// Default summarization prompt.
    const DEFAULT_PROMPT: &str = "Summarize the following conversation messages \
        into a concise paragraph. Preserve key facts, decisions, and context. \
        Omit greetings and filler.";

    /// Create a compressor with a given provider.
    #[must_use]
    pub fn new(provider: Arc<dyn Provider>) -> Self {
        Self {
            provider,
            summary_prompt: Self::DEFAULT_PROMPT.to_string(),
            threshold: 0.80,
            keep_recent: 4,
        }
    }

    /// Set a custom summarization prompt template.
    #[must_use]
    pub fn with_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.summary_prompt = prompt.into();
        self
    }

    /// Set the compression threshold (0.0–1.0).
    #[must_use]
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Set the number of recent messages to keep verbatim.
    #[must_use]
    pub fn with_keep_recent(mut self, count: usize) -> Self {
        self.keep_recent = count;
        self
    }
}

#[async_trait]
impl ContextManager for LlmCompressor {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    async fn prepare(
        &self,
        messages: &mut Vec<Message>,
        context_window: usize,
        state: &mut AgentState,
    ) {
        let max_tokens = (context_window as f64 * self.threshold) as usize;

        if estimate_tokens(messages) <= max_tokens {
            return;
        }

        // Partition: system messages | old messages (to summarize) | recent messages (keep)
        let system_msgs: Vec<Message> = messages
            .iter()
            .filter(|m| m.role == MessageRole::System)
            .cloned()
            .collect();

        let non_system: Vec<Message> = messages
            .iter()
            .filter(|m| m.role != MessageRole::System)
            .cloned()
            .collect();

        if non_system.len() <= self.keep_recent {
            return; // not enough messages to summarize
        }

        let split_at = non_system.len() - self.keep_recent;
        let old_messages = &non_system[..split_at];
        let recent_messages = &non_system[split_at..];

        // Build text of old messages for summarization
        let old_text: String = old_messages
            .iter()
            .map(|m| format!("{:?}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        // Single LLM call for summarization
        let req = CompletionRequest {
            model: self.provider.model_info().name.clone(),
            messages: vec![
                Message {
                    role: MessageRole::System,
                    content: self.summary_prompt.clone(),
                    tool_call_id: None,
                },
                Message {
                    role: MessageRole::User,
                    content: old_text,
                    tool_call_id: None,
                },
            ],
            tools: vec![],
            max_tokens: Some(500),
            temperature: Some(0.3),
            response_format: None,
            stream: false,
        };

        let summary_text = match self.provider.complete(req).await {
            Ok(response) => match response.content {
                ResponseContent::Text(text) => text,
                ResponseContent::ToolCalls(_) => {
                    tracing::warn!("LlmCompressor: provider returned tool calls instead of text");
                    Self::fallback_summary(old_messages)
                }
            },
            Err(e) => {
                tracing::warn!("LlmCompressor: summarization failed ({e}), using fallback");
                Self::fallback_summary(old_messages)
            }
        };

        // Rebuild messages: system + summary + recent
        // NOTE: Summary uses Assistant role so it appears naturally in the
        // conversation flow. If a provider rejects consecutive assistant
        // messages, consider switching to System role with a marker prefix.
        let summary_msg = Message {
            role: MessageRole::Assistant,
            content: format!("[Context Summary] {summary_text}"),
            tool_call_id: None,
        };

        messages.clear();
        messages.extend(system_msgs);
        messages.push(summary_msg);
        messages.extend(recent_messages.iter().cloned());

        state.last_output_truncated = true;
    }
}

impl LlmCompressor {
    /// Fallback when LLM call fails: truncate old messages to a brief note.
    fn fallback_summary(old_messages: &[Message]) -> String {
        format!(
            "{} earlier messages were removed to save context space.",
            old_messages.len()
        )
    }
}

// ---------------------------------------------------------------------------
// TieredCompressor
// ---------------------------------------------------------------------------

/// Chains compression tiers: keep recent → rule-compress mid → LLM-summarize old.
///
/// Without an LLM provider, uses only rule-based compression for the older messages.
///
/// # Example
///
/// ```rust
/// use traitclaw_core::context_managers::TieredCompressor;
///
/// let compressor = TieredCompressor::new(5); // keep last 5 messages
/// ```
pub struct TieredCompressor {
    /// Number of recent messages to keep verbatim.
    recent_count: usize,
    /// Rule-based compressor for the middle tier.
    rule_compressor: RuleBasedCompressor,
    /// Optional LLM compressor for the oldest tier.
    llm_compressor: Option<LlmCompressor>,
}

impl TieredCompressor {
    /// Create a tiered compressor (rule-only mode).
    #[must_use]
    pub fn new(recent_count: usize) -> Self {
        Self {
            recent_count,
            rule_compressor: RuleBasedCompressor::new(0.85, recent_count),
            llm_compressor: None,
        }
    }

    /// Enable the LLM summarization tier.
    #[must_use]
    pub fn with_llm(mut self, provider: Arc<dyn Provider>) -> Self {
        self.llm_compressor =
            Some(LlmCompressor::new(provider).with_keep_recent(self.recent_count));
        self
    }
}

#[async_trait]
impl ContextManager for TieredCompressor {
    async fn prepare(
        &self,
        messages: &mut Vec<Message>,
        context_window: usize,
        state: &mut AgentState,
    ) {
        // Tier 1: Try LLM summarization of oldest messages (if available)
        if let Some(llm) = &self.llm_compressor {
            llm.prepare(messages, context_window, state).await;
        }

        // Tier 2: Rule-based compression for remaining overflow
        self.rule_compressor
            .prepare(messages, context_window, state)
            .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::completion::{CompletionResponse, ResponseContent, Usage};
    use crate::types::model_info::{ModelInfo, ModelTier};
    use crate::types::stream::CompletionStream;

    fn msg(role: MessageRole, content: &str) -> Message {
        Message {
            role,
            content: content.to_string(),
            tool_call_id: None,
        }
    }

    fn tool_msg(content: &str) -> Message {
        Message {
            role: MessageRole::Tool,
            content: content.to_string(),
            tool_call_id: Some("call_1".to_string()),
        }
    }

    fn default_state() -> AgentState {
        AgentState::new(ModelTier::Medium, 128_000)
    }

    // ── RuleBasedCompressor ─────────────────────────────────────────────

    #[tokio::test]
    async fn test_rule_compressor_no_pruning_under_threshold() {
        let comp = RuleBasedCompressor::default();
        let mut msgs = vec![
            msg(MessageRole::System, "system"),
            msg(MessageRole::User, "hello"),
        ];
        let mut state = default_state();

        comp.prepare(&mut msgs, 100_000, &mut state).await;
        assert_eq!(msgs.len(), 2);
        assert!(!state.last_output_truncated);
    }

    #[tokio::test]
    async fn test_rule_compressor_removes_lowest_scored() {
        let comp = RuleBasedCompressor::new(0.85, 1);
        let mut msgs = vec![
            msg(MessageRole::System, "system"),
            msg(MessageRole::User, &"old1 ".repeat(500)), // old, score 0.3
            msg(MessageRole::Assistant, &"old2 ".repeat(500)), // old, score 0.3
            tool_msg(&"tool ".repeat(500)),               // tool, score 0.7
            msg(MessageRole::User, &"recent ".repeat(500)), // recent, score 0.9
        ];
        let mut state = default_state();

        // context_window small enough to trigger pruning
        comp.prepare(&mut msgs, 800, &mut state).await;

        // System must survive
        assert_eq!(msgs[0].role, MessageRole::System);
        // Old messages should be removed first
        assert!(msgs.len() < 5, "should have removed some messages");
        assert!(state.last_output_truncated);
    }

    #[tokio::test]
    async fn test_rule_compressor_never_removes_system() {
        let comp = RuleBasedCompressor::new(0.5, 0);
        let mut msgs = vec![
            msg(MessageRole::System, &"sys ".repeat(1000)),
            msg(MessageRole::User, "tiny"),
        ];
        let mut state = default_state();

        comp.prepare(&mut msgs, 100, &mut state).await;

        // System message must survive even if it alone exceeds the budget
        assert!(msgs.iter().any(|m| m.role == MessageRole::System));
    }

    #[tokio::test]
    async fn test_rule_compressor_updates_state() {
        let comp = RuleBasedCompressor::new(0.5, 0);
        let mut msgs = vec![
            msg(MessageRole::System, "sys"),
            msg(MessageRole::User, &"x".repeat(4000)),
            msg(MessageRole::Assistant, &"y".repeat(4000)),
        ];
        let mut state = default_state();

        comp.prepare(&mut msgs, 1000, &mut state).await;
        assert!(state.last_output_truncated);
    }

    // ── LlmCompressor ───────────────────────────────────────────────────

    struct MockSummarizer {
        info: ModelInfo,
    }

    impl MockSummarizer {
        fn new() -> Self {
            Self {
                info: ModelInfo::new(
                    "mock-summarizer",
                    ModelTier::Small,
                    4096,
                    false,
                    false,
                    false,
                ),
            }
        }
    }

    #[async_trait]
    impl Provider for MockSummarizer {
        async fn complete(&self, _req: CompletionRequest) -> crate::Result<CompletionResponse> {
            Ok(CompletionResponse {
                content: ResponseContent::Text(
                    "User asked about Rust. Assistant explained traits.".to_string(),
                ),
                usage: Usage {
                    prompt_tokens: 50,
                    completion_tokens: 20,
                    total_tokens: 70,
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

    struct FailingSummarizer {
        info: ModelInfo,
    }

    impl FailingSummarizer {
        fn new() -> Self {
            Self {
                info: ModelInfo::new("failing", ModelTier::Small, 4096, false, false, false),
            }
        }
    }

    #[async_trait]
    impl Provider for FailingSummarizer {
        async fn complete(&self, _req: CompletionRequest) -> crate::Result<CompletionResponse> {
            Err(crate::Error::Provider {
                message: "network error".into(),
                status_code: None,
            })
        }

        async fn stream(&self, _req: CompletionRequest) -> crate::Result<CompletionStream> {
            unimplemented!()
        }

        fn model_info(&self) -> &ModelInfo {
            &self.info
        }
    }

    #[tokio::test]
    async fn test_llm_compressor_summarizes_old_messages() {
        let provider: Arc<dyn Provider> = Arc::new(MockSummarizer::new());
        let comp = LlmCompressor::new(provider).with_keep_recent(2);

        let mut msgs = vec![
            msg(MessageRole::System, "You are helpful"),
            msg(MessageRole::User, &"old question ".repeat(500)),
            msg(MessageRole::Assistant, &"old answer ".repeat(500)),
            msg(MessageRole::User, "recent question"),
            msg(MessageRole::Assistant, "recent answer"),
        ];
        let mut state = default_state();

        comp.prepare(&mut msgs, 800, &mut state).await;

        // System message preserved
        assert_eq!(msgs[0].role, MessageRole::System);
        // Summary inserted
        assert!(
            msgs[1].content.contains("[Context Summary]"),
            "should have summary: {}",
            msgs[1].content
        );
        // Recent messages kept
        assert_eq!(msgs.len(), 4); // system + summary + 2 recent
        assert!(state.last_output_truncated);
    }

    #[tokio::test]
    async fn test_llm_compressor_no_compression_under_threshold() {
        let provider: Arc<dyn Provider> = Arc::new(MockSummarizer::new());
        let comp = LlmCompressor::new(provider).with_keep_recent(2);

        let mut msgs = vec![
            msg(MessageRole::System, "sys"),
            msg(MessageRole::User, "hi"),
            msg(MessageRole::Assistant, "hello"),
        ];
        let mut state = default_state();

        comp.prepare(&mut msgs, 100_000, &mut state).await;
        assert_eq!(msgs.len(), 3);
        assert!(!state.last_output_truncated);
    }

    #[tokio::test]
    async fn test_llm_compressor_custom_prompt() {
        let provider: Arc<dyn Provider> = Arc::new(MockSummarizer::new());
        let comp = LlmCompressor::new(provider)
            .with_prompt("Custom prompt")
            .with_keep_recent(1);

        let mut msgs = vec![
            msg(MessageRole::System, "sys"),
            msg(MessageRole::User, &"old ".repeat(2000)),
            msg(MessageRole::User, "recent"),
        ];
        let mut state = default_state();

        comp.prepare(&mut msgs, 500, &mut state).await;
        assert!(msgs[1].content.contains("[Context Summary]"));
    }

    #[tokio::test]
    async fn test_llm_compressor_fallback_on_failure() {
        let provider: Arc<dyn Provider> = Arc::new(FailingSummarizer::new());
        let comp = LlmCompressor::new(provider).with_keep_recent(1);

        let mut msgs = vec![
            msg(MessageRole::System, "sys"),
            msg(MessageRole::User, &"old ".repeat(2000)),
            msg(MessageRole::Assistant, &"old ".repeat(2000)),
            msg(MessageRole::User, "recent"),
        ];
        let mut state = default_state();

        comp.prepare(&mut msgs, 500, &mut state).await;

        // Should still compress with fallback
        assert!(msgs[1].content.contains("[Context Summary]"));
        assert!(msgs[1].content.contains("removed to save context"));
        assert!(state.last_output_truncated);
    }

    // ── TieredCompressor ────────────────────────────────────────────────

    #[tokio::test]
    async fn test_tiered_compressor_rule_only() {
        let comp = TieredCompressor::new(2);

        let mut msgs = vec![
            msg(MessageRole::System, "sys"),
            msg(MessageRole::User, &"old1 ".repeat(500)),
            msg(MessageRole::Assistant, &"old2 ".repeat(500)),
            msg(MessageRole::User, &"recent1 ".repeat(500)),
            msg(MessageRole::Assistant, &"recent2 ".repeat(500)),
        ];
        let mut state = default_state();

        comp.prepare(&mut msgs, 1500, &mut state).await;

        assert_eq!(msgs[0].role, MessageRole::System);
        assert!(msgs.len() < 5);
        assert!(state.last_output_truncated);
    }

    #[tokio::test]
    async fn test_tiered_compressor_with_llm() {
        let provider: Arc<dyn Provider> = Arc::new(MockSummarizer::new());
        let comp = TieredCompressor::new(2).with_llm(provider);

        let mut msgs = vec![
            msg(MessageRole::System, "sys"),
            msg(MessageRole::User, &"old ".repeat(1000)),
            msg(MessageRole::Assistant, &"old ".repeat(1000)),
            msg(MessageRole::User, "recent1"),
            msg(MessageRole::Assistant, "recent2"),
        ];
        let mut state = default_state();

        comp.prepare(&mut msgs, 800, &mut state).await;

        assert_eq!(msgs[0].role, MessageRole::System);
        assert!(
            msgs.iter().any(|m| m.content.contains("[Context Summary]")),
            "should have LLM summary"
        );
        assert!(state.last_output_truncated);
    }

    // ── 50-message stress test ──────────────────────────────────────────

    #[tokio::test]
    async fn test_rule_compressor_50_messages_within_budget() {
        let comp = RuleBasedCompressor::new(0.85, 5);

        let mut msgs = vec![msg(MessageRole::System, "You are a helpful assistant")];
        for i in 0..50 {
            msgs.push(msg(
                if i % 2 == 0 {
                    MessageRole::User
                } else {
                    MessageRole::Assistant
                },
                &format!("Message number {i}: {}", "content ".repeat(100)),
            ));
        }
        let mut state = default_state();

        // Small window to force heavy compression
        let window = 2000;
        comp.prepare(&mut msgs, window, &mut state).await;

        // Verify within budget
        let tokens: usize = msgs.iter().map(|m| m.content.len() / 4 + 1).sum();
        let max = (window as f64 * 0.85) as usize;
        assert!(tokens <= max, "should be within budget: {tokens} <= {max}");
        // System message must survive
        assert_eq!(msgs[0].role, MessageRole::System);
        assert!(state.last_output_truncated);
    }
}
