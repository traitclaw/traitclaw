//! Tool trait for agent capabilities.
//!
//! The [`Tool`] trait defines a capability that an agent can use.
//! Tools have typed inputs and outputs with automatic JSON Schema generation.

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

use crate::Result;

/// JSON Schema representation for a tool's parameters.
#[derive(Debug, Clone, Serialize)]
pub struct ToolSchema {
    /// Tool name.
    pub name: String,
    /// Tool description.
    pub description: String,
    /// JSON Schema for the tool's input parameters.
    pub parameters: Value,
}

/// Trait for defining agent tools with typed inputs and outputs.
///
/// Tools are the primary way agents interact with the outside world.
/// Each tool has a typed `Input` (auto-generates JSON Schema) and `Output`.
///
/// # Example
///
/// ```rust,no_run
/// use async_trait::async_trait;
/// use baseclaw_core::prelude::*;
/// use schemars::JsonSchema;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Deserialize, JsonSchema)]
/// struct SearchInput {
///     query: String,
/// }
///
/// #[derive(Serialize)]
/// struct SearchOutput {
///     results: Vec<String>,
/// }
///
/// struct WebSearch;
///
/// #[async_trait]
/// impl Tool for WebSearch {
///     type Input = SearchInput;
///     type Output = SearchOutput;
///
///     fn name(&self) -> &str { "web_search" }
///     fn description(&self) -> &str { "Search the web" }
///
///     async fn execute(&self, input: Self::Input) -> baseclaw_core::Result<Self::Output> {
///         Ok(SearchOutput { results: vec![format!("Result for: {}", input.query)] })
///     }
/// }
/// ```
#[async_trait]
pub trait Tool: Send + Sync + 'static {
    /// Input type — must be deserializable from JSON and have a JSON Schema.
    type Input: DeserializeOwned + JsonSchema + Send;
    /// Output type — must be serializable to JSON.
    type Output: Serialize + Send;

    /// The unique name of this tool.
    fn name(&self) -> &str;

    /// A description of what this tool does (sent to the LLM).
    fn description(&self) -> &str;

    /// Generate the JSON Schema for this tool's parameters.
    fn schema(&self) -> ToolSchema {
        let schema = schemars::schema_for!(Self::Input);
        ToolSchema {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: serde_json::to_value(schema).unwrap_or_default(),
        }
    }

    /// Execute this tool with the given input.
    async fn execute(&self, input: Self::Input) -> Result<Self::Output>;
}

/// Type-erased tool wrapper for dynamic dispatch.
///
/// Allows storing heterogeneous tools in `Vec<Arc<dyn ErasedTool>>`.
#[async_trait]
pub trait ErasedTool: Send + Sync + 'static {
    /// The unique name of this tool.
    fn name(&self) -> &str;

    /// A description of what this tool does.
    fn description(&self) -> &str;

    /// Get the JSON Schema for this tool.
    fn schema(&self) -> ToolSchema;

    /// Execute this tool with JSON input, returning JSON output.
    async fn execute_json(&self, input: Value) -> Result<Value>;
}

#[async_trait]
impl<T: Tool> ErasedTool for T {
    fn name(&self) -> &str {
        Tool::name(self)
    }

    fn description(&self) -> &str {
        Tool::description(self)
    }

    fn schema(&self) -> ToolSchema {
        Tool::schema(self)
    }

    async fn execute_json(&self, input: Value) -> Result<Value> {
        let typed_input: T::Input = serde_json::from_value(input).map_err(|e| {
            crate::Error::tool_execution(self.name(), format!("Invalid input: {e}"))
        })?;

        let output = self.execute(typed_input).await?;

        serde_json::to_value(output).map_err(|e| {
            crate::Error::tool_execution(self.name(), format!("Failed to serialize output: {e}"))
        })
    }
}
