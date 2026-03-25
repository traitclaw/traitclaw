//! Shared test utilities for `traitclaw-core` tests.
//!
//! Provides reusable mock implementations of [`Provider`], [`Tool`],
//! and [`Guard`] for deterministic, isolated testing.
//!
//! # Mock Provider
//!
//! [`SequenceProvider`] returns responses in order using an [`AtomicUsize`]
//! counter, making tests fully deterministic with zero randomness.
//!
//! # Mock Tools
//!
//! - [`EchoTool`] — echoes its text input back as output
//! - [`DangerousTool`] — named "dangerous_operation" for hook interception tests
//!
//! # Mock Guards
//!
//! - [`DenyGuard`] — unconditionally denies all actions

use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::traits::guard::{Guard, GuardResult, GuardSeverity};
use crate::traits::provider::Provider;
use crate::types::action::Action;
use crate::types::completion::{CompletionRequest, CompletionResponse, ResponseContent, Usage};
use crate::types::model_info::{ModelInfo, ModelTier};
use crate::types::stream::CompletionStream;

// ---- Mock Provider ----

/// A deterministic mock provider that returns responses in sequence.
///
/// Uses [`AtomicUsize`] for lock-free call counting. The first call
/// returns `responses[0]`, the second `responses[1]`, etc. If calls
/// exceed the response count, the last response is repeated.
pub(crate) struct SequenceProvider {
    pub(crate) info: ModelInfo,
    pub(crate) responses: Vec<CompletionResponse>,
    pub(crate) call_idx: AtomicUsize,
}

impl SequenceProvider {
    /// Create a provider that returns a single text response.
    pub(crate) fn text(text: &str) -> Self {
        Self {
            info: ModelInfo::new("test-model", ModelTier::Small, 4096, false, false, false),
            responses: vec![CompletionResponse {
                content: ResponseContent::Text(text.into()),
                usage: Usage {
                    prompt_tokens: 10,
                    completion_tokens: 5,
                    total_tokens: 15,
                },
            }],
            call_idx: AtomicUsize::new(0),
        }
    }

    /// Create a provider that returns responses in sequence.
    pub(crate) fn with_responses(responses: Vec<CompletionResponse>) -> Self {
        Self {
            info: ModelInfo::new("test-model", ModelTier::Small, 4096, true, false, false),
            responses,
            call_idx: AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl Provider for SequenceProvider {
    async fn complete(&self, _req: CompletionRequest) -> crate::Result<CompletionResponse> {
        let idx = self.call_idx.fetch_add(1, Ordering::SeqCst);
        Ok(self.responses[idx.min(self.responses.len() - 1)].clone())
    }
    async fn stream(&self, _req: CompletionRequest) -> crate::Result<CompletionStream> {
        unimplemented!()
    }
    fn model_info(&self) -> &ModelInfo {
        &self.info
    }
}

// ---- Mock Tool (echo) ----

#[derive(Deserialize, schemars::JsonSchema)]
pub(crate) struct EchoInput {
    pub(crate) text: String,
}

#[derive(Serialize)]
pub(crate) struct EchoOutput {
    pub(crate) echo: String,
}

/// A simple echo tool that returns its input text as output.
pub(crate) struct EchoTool;

#[async_trait]
impl crate::traits::tool::Tool for EchoTool {
    type Input = EchoInput;
    type Output = EchoOutput;
    fn name(&self) -> &'static str {
        "echo"
    }
    fn description(&self) -> &'static str {
        "Echoes input"
    }
    async fn execute(&self, input: Self::Input) -> crate::Result<Self::Output> {
        Ok(EchoOutput {
            echo: input.text.clone(),
        })
    }
}

// ---- Mock Tool (dangerous) ----

#[derive(Deserialize, schemars::JsonSchema)]
pub(crate) struct DangerousInput {
    #[allow(dead_code)]
    pub(crate) payload: String,
}

#[derive(Serialize)]
pub(crate) struct DangerousOutput {
    pub(crate) result: String,
}

/// A mock dangerous tool for testing hook-based interception.
pub(crate) struct DangerousTool;

#[async_trait]
impl crate::traits::tool::Tool for DangerousTool {
    type Input = DangerousInput;
    type Output = DangerousOutput;
    fn name(&self) -> &'static str {
        "dangerous_operation"
    }
    fn description(&self) -> &'static str {
        "A dangerous tool"
    }
    async fn execute(&self, _input: Self::Input) -> crate::Result<Self::Output> {
        Ok(DangerousOutput {
            result: "SHOULD NOT RUN".into(),
        })
    }
}

// ---- Mock Guard (deny all) ----

/// A guard that unconditionally denies all actions.
pub(crate) struct DenyGuard;

impl Guard for DenyGuard {
    fn name(&self) -> &'static str {
        "deny-all"
    }
    fn check(&self, _action: &Action) -> GuardResult {
        GuardResult::Deny {
            reason: "blocked by test guard".into(),
            severity: GuardSeverity::High,
        }
    }
}
