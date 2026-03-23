//! Action types for the Guard system.

use std::path::PathBuf;

use serde_json::Value;

/// An action that the agent wants to perform.
///
/// Every action goes through the Guard system before execution.
#[derive(Debug, Clone)]
pub enum Action {
    /// Call a tool with arguments.
    ToolCall {
        /// Name of the tool.
        name: String,
        /// Arguments as JSON.
        arguments: Value,
    },
    /// Execute a shell command.
    ShellCommand {
        /// The command to execute.
        command: String,
    },
    /// Write content to a file.
    FileWrite {
        /// Target file path.
        path: PathBuf,
        /// Content to write.
        content: String,
    },
    /// Make an HTTP request.
    HttpRequest {
        /// Target URL.
        url: String,
        /// HTTP method.
        method: String,
    },
    /// Delegate a task to another agent.
    AgentDelegation {
        /// Name of the target agent.
        to: String,
        /// Task description.
        task: String,
    },
    /// Raw text output to the user.
    RawOutput {
        /// The output content.
        content: String,
    },
}
