//! Shared test utilities for traitclaw-team tests.

use async_trait::async_trait;
use traitclaw_core::traits::provider::Provider;
use traitclaw_core::types::completion::{
    CompletionRequest, CompletionResponse, ResponseContent, Usage,
};
use traitclaw_core::types::message::MessageRole;
use traitclaw_core::types::model_info::{ModelInfo, ModelTier};
use traitclaw_core::types::stream::CompletionStream;

/// A provider that echoes back the last user message with a prefix.
pub struct EchoProvider {
    info: ModelInfo,
    prefix: String,
}

impl EchoProvider {
    /// Create a new echo provider with the given prefix.
    pub fn new(prefix: &str) -> Self {
        Self {
            info: ModelInfo::new("echo", ModelTier::Small, 4_096, false, false, false),
            prefix: prefix.to_string(),
        }
    }
}

#[async_trait]
impl Provider for EchoProvider {
    async fn complete(&self, req: CompletionRequest) -> traitclaw_core::Result<CompletionResponse> {
        let last_msg = req
            .messages
            .iter()
            .rev()
            .find(|m| m.role == MessageRole::User)
            .map(|m| m.content.clone())
            .unwrap_or_default();
        Ok(CompletionResponse {
            content: ResponseContent::Text(format!("[{}] {}", self.prefix, last_msg)),
            usage: Usage {
                prompt_tokens: 1,
                completion_tokens: 1,
                total_tokens: 2,
            },
        })
    }
    async fn stream(&self, _req: CompletionRequest) -> traitclaw_core::Result<CompletionStream> {
        unimplemented!()
    }
    fn model_info(&self) -> &ModelInfo {
        &self.info
    }
}
