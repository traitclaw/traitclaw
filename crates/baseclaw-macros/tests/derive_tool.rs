//! Integration tests for the `#[derive(Tool)]` macro.

use baseclaw_macros::Tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Tool, Deserialize, JsonSchema, Serialize)]
#[tool(description = "Search the web for relevant information")]
struct WebSearch {
    /// The search query to look up
    query: String,
    /// Maximum number of results to return
    max_results: Option<u32>,
}

#[derive(Tool, Deserialize, JsonSchema, Serialize)]
#[tool(name = "calculator", description = "Perform arithmetic calculations")]
struct Calculator {
    /// The arithmetic expression to evaluate
    expression: String,
}

#[test]
fn test_derive_tool_generates_name() {
    // Default name = snake_case of struct name
    assert_eq!(WebSearch::tool_name(), "web_search");
}

#[test]
fn test_derive_tool_name_override() {
    // Explicit name override via #[tool(name = "...")]
    assert_eq!(Calculator::tool_name(), "calculator");
}

#[test]
fn test_derive_tool_description() {
    assert_eq!(
        WebSearch::tool_description(),
        "Search the web for relevant information"
    );
}

#[test]
fn test_derive_tool_schema_structure() {
    let schema = WebSearch::tool_schema();
    assert_eq!(schema.name, "web_search");
    assert_eq!(schema.description, "Search the web for relevant information");

    // Schema should be a JSON object (from schemars)
    assert!(schema.parameters.is_object());

    // The schema should have properties
    let obj = schema.parameters.as_object().unwrap();
    // schemars wraps in definitions or properties at root
    assert!(
        obj.contains_key("properties") || obj.contains_key("$schema"),
        "Expected schema object, got: {obj:?}"
    );
}

#[test]
fn test_derive_tool_schema_for_calculator() {
    let schema = Calculator::tool_schema();
    assert_eq!(schema.name, "calculator");
    assert_eq!(schema.description, "Perform arithmetic calculations");
}
