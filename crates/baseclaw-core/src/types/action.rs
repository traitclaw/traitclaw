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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_tool_call() {
        let action = Action::ToolCall {
            name: "search".into(),
            arguments: serde_json::json!({"q": "rust"}),
        };
        let dbg = format!("{action:?}");
        assert!(dbg.contains("ToolCall"));
        assert!(dbg.contains("search"));
    }

    #[test]
    fn test_action_shell_command() {
        let action = Action::ShellCommand {
            command: "ls -la".into(),
        };
        assert!(format!("{action:?}").contains("ShellCommand"));
    }

    #[test]
    fn test_action_file_write() {
        let action = Action::FileWrite {
            path: PathBuf::from("/tmp/test.txt"),
            content: "hello".into(),
        };
        assert!(format!("{action:?}").contains("FileWrite"));
    }

    #[test]
    fn test_action_http_request() {
        let action = Action::HttpRequest {
            url: "https://example.com".into(),
            method: "GET".into(),
        };
        assert!(format!("{action:?}").contains("HttpRequest"));
    }

    #[test]
    fn test_action_agent_delegation() {
        let action = Action::AgentDelegation {
            to: "researcher".into(),
            task: "find data".into(),
        };
        assert!(format!("{action:?}").contains("AgentDelegation"));
    }

    #[test]
    fn test_action_clone() {
        let action = Action::RawOutput {
            content: "hello".into(),
        };
        let cloned = action.clone();
        assert!(format!("{cloned:?}").contains("hello"));
    }
}

