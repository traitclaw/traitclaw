//! Error types for `BaseClaw` Core.

use std::fmt;

/// Errors that can occur during agent operation.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error from the LLM provider.
    #[error("Provider error: {message}")]
    Provider {
        /// Error message from the provider.
        message: String,
        /// Optional HTTP status code (for retry classification).
        status_code: Option<u16>,
    },

    /// Error during tool execution.
    #[error("Tool execution failed: {tool_name}: {message}")]
    ToolExecution {
        /// Name of the tool that failed.
        tool_name: String,
        /// Error message describing the failure.
        message: String,
    },

    /// Error from the memory system.
    #[error("Memory error: {0}")]
    Memory(String),

    /// Configuration error (e.g., missing required fields).
    #[error("Configuration error: {0}")]
    Config(String),

    /// Runtime error during agent loop execution.
    #[error("Runtime error: {0}")]
    Runtime(String),

    /// Serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Convenience type alias for `Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// Default status codes considered retryable (transient).
const RETRYABLE_STATUS_CODES: &[u16] = &[429, 500, 502, 503, 504];

impl Error {
    /// Create a provider error.
    #[must_use]
    pub fn provider(message: impl fmt::Display) -> Self {
        Self::Provider {
            message: message.to_string(),
            status_code: None,
        }
    }

    /// Create a provider error with an HTTP status code.
    #[must_use]
    pub fn provider_with_status(message: impl fmt::Display, status_code: u16) -> Self {
        Self::Provider {
            message: message.to_string(),
            status_code: Some(status_code),
        }
    }

    /// Create a tool execution error.
    #[must_use]
    pub fn tool_execution(tool_name: impl fmt::Display, message: impl fmt::Display) -> Self {
        Self::ToolExecution {
            tool_name: tool_name.to_string(),
            message: message.to_string(),
        }
    }

    /// Check whether this error is safe to retry.
    ///
    /// Returns `true` for transient provider errors (429, 500, 502, 503, 504)
    /// and provider errors without a status code (e.g., timeouts).
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Provider { status_code, .. } => match status_code {
                Some(code) => RETRYABLE_STATUS_CODES.contains(code),
                None => true, // timeout / network errors are typically retryable
            },
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_error_display() {
        let err = Error::provider("model not found");
        assert_eq!(err.to_string(), "Provider error: model not found");
    }

    #[test]
    fn test_tool_execution_error_display() {
        let err = Error::tool_execution("web_search", "timeout");
        assert_eq!(
            err.to_string(),
            "Tool execution failed: web_search: timeout"
        );
    }

    #[test]
    fn test_config_error_display() {
        let err = Error::Config("no provider configured".into());
        assert_eq!(
            err.to_string(),
            "Configuration error: no provider configured"
        );
    }

    #[test]
    fn test_runtime_error_display() {
        let err = Error::Runtime("max iterations reached".into());
        assert_eq!(err.to_string(), "Runtime error: max iterations reached");
    }

    #[test]
    fn test_memory_error_display() {
        let err = Error::Memory("session not found".into());
        assert_eq!(err.to_string(), "Memory error: session not found");
    }

    #[test]
    fn test_from_serde_json_error() {
        // AC-4: #[from] conversion for serde_json::Error
        let json_err = serde_json::from_str::<String>("not valid json").unwrap_err();
        let err: Error = json_err.into();
        assert!(matches!(err, Error::Serialization(_)));
        assert!(err.to_string().contains("Serialization error"));
    }

    #[test]
    fn test_from_io_error() {
        // AC-4: #[from] conversion for std::io::Error
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
        assert!(err.to_string().contains("IO error"));
    }

    #[test]
    fn test_result_alias_with_question_mark() {
        // AC-3: Result<T> alias works with ? operator
        fn may_fail(succeed: bool) -> Result<String> {
            if succeed {
                Ok("done".into())
            } else {
                Err(Error::Runtime("failed".into()))
            }
        }

        fn chain() -> Result<String> {
            let val = may_fail(true)?;
            Ok(val)
        }

        assert!(chain().is_ok());
        assert!(may_fail(false).is_err());
    }

    #[test]
    fn test_error_is_std_error() {
        // AC-4: errors implement std::error::Error
        let err = Error::provider("test");
        let _: &dyn std::error::Error = &err;
    }
}
