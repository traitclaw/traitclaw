//! Tool trait for agent capabilities.
//!
//! The [`Tool`] trait defines a capability that an agent can use.
//! Tools have typed inputs and outputs with automatic JSON Schema generation.

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

use crate::Result;

/// JSON Schema representation for a tool's parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── Test fixtures ──────────────────────────────────────────────

    #[derive(Deserialize, JsonSchema)]
    struct AddInput {
        a: i64,
        b: i64,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct AddOutput {
        sum: i64,
    }

    struct AddTool;

    #[async_trait]
    #[allow(clippy::unnecessary_literal_bound)]
    impl Tool for AddTool {
        type Input = AddInput;
        type Output = AddOutput;

        fn name(&self) -> &str {
            "add"
        }
        fn description(&self) -> &str {
            "Add two numbers"
        }

        async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
            Ok(AddOutput {
                sum: input.a + input.b,
            })
        }
    }

    // ── AC 1+2: Manual Tool implementation with typed I/O ──────

    #[tokio::test]
    async fn test_tool_execute_typed() {
        let tool = AddTool;
        let result = tool.execute(AddInput { a: 3, b: 4 }).await.unwrap();
        assert_eq!(result.sum, 7);
    }

    #[test]
    fn test_tool_name_and_description() {
        let tool = AddTool;
        assert_eq!(Tool::name(&tool), "add");
        assert_eq!(Tool::description(&tool), "Add two numbers");
    }

    // ── AC 1: Schema generation from schemars ──────────────────

    #[test]
    fn test_schema_generation() {
        let tool = AddTool;
        let schema = Tool::schema(&tool);

        assert_eq!(schema.name, "add");
        assert_eq!(schema.description, "Add two numbers");

        // Schema must contain properties for a and b
        let params = &schema.parameters;
        let props = params
            .get("properties")
            .expect("schema should have properties");
        assert!(props.get("a").is_some(), "schema missing 'a' property");
        assert!(props.get("b").is_some(), "schema missing 'b' property");
    }

    #[test]
    fn test_tool_schema_serializes_to_json() {
        let tool = AddTool;
        let schema = Tool::schema(&tool);

        // ToolSchema must serialize (for OpenAI tools parameter format)
        let json = serde_json::to_value(&schema).unwrap();
        assert_eq!(json["name"], "add");
        assert_eq!(json["description"], "Add two numbers");
        assert!(json["parameters"].is_object());
    }

    // ── AC 3: ErasedTool enables heterogeneous storage ─────────

    #[test]
    fn test_erased_tool_in_vec() {
        let tools: Vec<std::sync::Arc<dyn ErasedTool>> = vec![std::sync::Arc::new(AddTool)];

        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name(), "add");
        assert_eq!(tools[0].description(), "Add two numbers");
    }

    // ── AC 4: ErasedTool JSON round-trip ───────────────────────

    #[tokio::test]
    async fn test_erased_tool_json_round_trip() {
        let tool: std::sync::Arc<dyn ErasedTool> = std::sync::Arc::new(AddTool);

        let input = serde_json::json!({"a": 10, "b": 20});
        let output = tool.execute_json(input).await.unwrap();

        let result: AddOutput = serde_json::from_value(output).unwrap();
        assert_eq!(result.sum, 30);
    }

    #[tokio::test]
    async fn test_erased_tool_invalid_input_returns_error() {
        let tool: std::sync::Arc<dyn ErasedTool> = std::sync::Arc::new(AddTool);

        let bad_input = serde_json::json!({"x": "not a number"});
        let result = tool.execute_json(bad_input).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("add"),
            "error should mention tool name"
        );
        assert!(
            err.to_string().contains("Invalid input"),
            "error should say invalid input"
        );
    }

    #[test]
    fn test_erased_tool_schema_matches_tool_schema() {
        let tool = AddTool;
        let direct_schema = Tool::schema(&tool);
        let erased_schema = ErasedTool::schema(&tool);

        assert_eq!(direct_schema.name, erased_schema.name);
        assert_eq!(direct_schema.description, erased_schema.description);
        assert_eq!(direct_schema.parameters, erased_schema.parameters);
    }
}
