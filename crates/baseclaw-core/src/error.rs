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

impl Error {
    /// Create a provider error.
    pub fn provider(message: impl fmt::Display) -> Self {
        Self::Provider {
            message: message.to_string(),
        }
    }

    /// Create a tool execution error.
    pub fn tool_execution(tool_name: impl fmt::Display, message: impl fmt::Display) -> Self {
        Self::ToolExecution {
            tool_name: tool_name.to_string(),
            message: message.to_string(),
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
}
