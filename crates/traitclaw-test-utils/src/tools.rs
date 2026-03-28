//! Mock tools for testing tool-calling scenarios.
//!
//! - [`EchoTool`] — echoes its text input back as output
//! - [`FailTool`] — always returns an error on execution
//!
//! Both tools are designed for use with [`AgentRuntime`](traitclaw_core::traits::strategy::AgentRuntime)
//! to test tool-calling flows deterministically.
//!
//! # Example
//!
//! ```rust
//! use traitclaw_test_utils::tools::{EchoTool, FailTool};
//! use traitclaw_core::traits::tool::Tool;
//!
//! assert_eq!(EchoTool.name(), "echo");
//! assert_eq!(FailTool.name(), "fail");
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use traitclaw_core::traits::tool::Tool;
use traitclaw_core::{Error, Result};

// ── EchoTool ────────────────────────────────────────────────────────────

/// Input for [`EchoTool`]: a single text field.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EchoInput {
    /// The text to echo back.
    pub text: String,
}

/// Output from [`EchoTool`]: the echoed text.
#[derive(Debug, Serialize)]
pub struct EchoOutput {
    /// The echoed text (identical to input).
    pub echo: String,
}

/// A simple tool that echoes its text input.
///
/// Useful for testing that the agent runtime correctly routes
/// tool calls and processes tool output.
///
/// # Example
///
/// ```rust
/// use traitclaw_test_utils::tools::EchoTool;
/// use traitclaw_core::traits::tool::Tool;
///
/// # tokio_test::block_on(async {
/// let result = EchoTool.execute(traitclaw_test_utils::tools::EchoInput {
///     text: "hello".into(),
/// }).await.unwrap();
/// assert_eq!(result.echo, "hello");
/// # });
/// ```
pub struct EchoTool;

#[async_trait]
impl Tool for EchoTool {
    type Input = EchoInput;
    type Output = EchoOutput;

    fn name(&self) -> &'static str {
        "echo"
    }

    fn description(&self) -> &'static str {
        "Echoes input text back as output"
    }

    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        Ok(EchoOutput {
            echo: input.text.clone(),
        })
    }
}

// ── FailTool ────────────────────────────────────────────────────────────

/// Input for [`FailTool`]: an optional message.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FailInput {
    /// Optional custom failure message (not used — tool always fails).
    #[allow(dead_code)]
    pub message: Option<String>,
}

/// A tool that always fails with an error.
///
/// Useful for testing error handling in agent loops, tool budgets,
/// and error recovery strategies.
///
/// # Example
///
/// ```rust
/// use traitclaw_test_utils::tools::FailTool;
/// use traitclaw_core::traits::tool::Tool;
///
/// # tokio_test::block_on(async {
/// let result = FailTool.execute(traitclaw_test_utils::tools::FailInput {
///     message: None,
/// }).await;
/// assert!(result.is_err());
/// # });
/// ```
pub struct FailTool;

#[async_trait]
impl Tool for FailTool {
    type Input = FailInput;
    type Output = serde_json::Value;

    fn name(&self) -> &'static str {
        "fail"
    }

    fn description(&self) -> &'static str {
        "Always fails with an error"
    }

    async fn execute(&self, _input: Self::Input) -> Result<Self::Output> {
        Err(Error::Runtime("tool failure".into()))
    }
}

// ── DangerousTool ──────────────────────────────────────────────────────

/// Input for [`DangerousTool`].
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DangerousInput {
    /// Payload (unused — tool exists for hook interception tests).
    #[allow(dead_code)]
    pub payload: String,
}

/// Output from [`DangerousTool`].
#[derive(Debug, Serialize)]
pub struct DangerousOutput {
    /// Result text.
    pub result: String,
}

/// A mock "dangerous" tool for testing hook-based interception.
///
/// Named `"dangerous_operation"` so that hook tests can identify and
/// intercept it by name before execution.
///
/// # Example
///
/// ```rust
/// use traitclaw_test_utils::tools::DangerousTool;
/// use traitclaw_core::traits::tool::Tool;
///
/// assert_eq!(DangerousTool.name(), "dangerous_operation");
/// ```
pub struct DangerousTool;

#[async_trait]
impl Tool for DangerousTool {
    type Input = DangerousInput;
    type Output = DangerousOutput;

    fn name(&self) -> &'static str {
        "dangerous_operation"
    }

    fn description(&self) -> &'static str {
        "A dangerous tool for hook interception tests"
    }

    async fn execute(&self, _input: Self::Input) -> Result<Self::Output> {
        Ok(DangerousOutput {
            result: "SHOULD NOT RUN".into(),
        })
    }
}

// ── DenyGuard ──────────────────────────────────────────────────────────

/// A guard that unconditionally denies all actions.
///
/// Useful for testing guard interception and abort flows.
///
/// # Example
///
/// ```rust
/// use traitclaw_test_utils::tools::DenyGuard;
/// use traitclaw_core::traits::guard::Guard;
///
/// let guard = DenyGuard;
/// assert_eq!(guard.name(), "deny-all");
/// ```
pub struct DenyGuard;

impl traitclaw_core::traits::guard::Guard for DenyGuard {
    fn name(&self) -> &'static str {
        "deny-all"
    }

    fn check(
        &self,
        _action: &traitclaw_core::types::action::Action,
    ) -> traitclaw_core::traits::guard::GuardResult {
        traitclaw_core::traits::guard::GuardResult::Deny {
            reason: "blocked by test guard".into(),
            severity: traitclaw_core::traits::guard::GuardSeverity::High,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_echo_tool_returns_input() {
        let result = EchoTool
            .execute(EchoInput {
                text: "hello world".into(),
            })
            .await
            .unwrap();
        assert_eq!(result.echo, "hello world");
    }

    #[tokio::test]
    async fn test_echo_tool_handles_empty_string() {
        let result = EchoTool
            .execute(EchoInput {
                text: String::new(),
            })
            .await
            .unwrap();
        assert_eq!(result.echo, "");
    }

    #[tokio::test]
    async fn test_fail_tool_returns_error() {
        let result = FailTool.execute(FailInput { message: None }).await;
        assert!(result.is_err());
        let err_str = result.unwrap_err().to_string();
        assert!(err_str.contains("tool failure"), "got: {err_str}");
    }

    #[test]
    fn test_echo_tool_name() {
        assert_eq!(EchoTool.name(), "echo");
        assert_eq!(EchoTool.description(), "Echoes input text back as output");
    }

    #[test]
    fn test_fail_tool_name() {
        assert_eq!(FailTool.name(), "fail");
        assert_eq!(FailTool.description(), "Always fails with an error");
    }

    #[test]
    fn test_dangerous_tool_name() {
        assert_eq!(DangerousTool.name(), "dangerous_operation");
    }

    #[test]
    fn test_deny_guard_name() {
        use traitclaw_core::traits::guard::Guard;
        assert_eq!(DenyGuard.name(), "deny-all");
    }

    #[test]
    fn test_tools_are_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<EchoTool>();
        assert_send_sync::<FailTool>();
        assert_send_sync::<DangerousTool>();
        assert_send_sync::<DenyGuard>();
    }
}
