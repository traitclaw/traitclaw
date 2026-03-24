//! Integration tests for the `#[derive(Tool)]` macro.

use traitclaw_core::ErasedTool;
use traitclaw_macros::Tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ── Test fixture: WebSearch tool ──────────────────────────────────

#[derive(Tool, Deserialize, JsonSchema, Serialize)]
#[tool(description = "Search the web for relevant information")]
struct WebSearch {
    /// The search query to look up
    query: String,
    /// Maximum number of results to return
    max_results: Option<u32>,
}

impl WebSearch {
    async fn execute(&self) -> traitclaw_core::Result<serde_json::Value> {
        Ok(serde_json::json!({
            "results": [format!("Result for: {}", self.query)],
            "count": self.max_results.unwrap_or(10),
        }))
    }
}

// ── Test fixture: Calculator with name override ───────────────────

#[derive(Tool, Deserialize, JsonSchema, Serialize)]
#[tool(name = "calculator", description = "Perform arithmetic calculations")]
struct Calculator {
    /// The arithmetic expression to evaluate
    expression: String,
}

impl Calculator {
    async fn execute(&self) -> traitclaw_core::Result<serde_json::Value> {
        Ok(serde_json::json!({"result": self.expression}))
    }
}

// ── AC 1: #[derive(Tool)] auto-generates name, description, schema ──

#[test]
fn test_derive_tool_generates_name() {
    assert_eq!(WebSearch::tool_name(), "web_search");
}

#[test]
fn test_derive_tool_name_override() {
    assert_eq!(Calculator::tool_name(), "calculator");
}

// ── AC 2: description attribute works ─────────────────────────────

#[test]
fn test_derive_tool_description() {
    assert_eq!(
        WebSearch::tool_description(),
        "Search the web for relevant information"
    );
}

// ── AC 5: JSON Schema derived from field types ────────────────────

#[test]
fn test_derive_tool_schema_structure() {
    let schema = WebSearch::tool_schema();
    assert_eq!(schema.name, "web_search");
    assert_eq!(
        schema.description,
        "Search the web for relevant information"
    );
    assert!(schema.parameters.is_object());

    let obj = schema.parameters.as_object().unwrap();
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

// ── AC 1+3: ErasedTool impl is generated (heterogeneous storage) ──

#[test]
fn test_erased_tool_name_and_description() {
    let tool = WebSearch {
        query: String::new(),
        max_results: None,
    };
    assert_eq!(ErasedTool::name(&tool), "web_search");
    assert_eq!(
        ErasedTool::description(&tool),
        "Search the web for relevant information"
    );
}

#[test]
fn test_erased_tool_in_arc_vec() {
    let tools: Vec<std::sync::Arc<dyn ErasedTool>> = vec![
        std::sync::Arc::new(WebSearch {
            query: String::new(),
            max_results: None,
        }),
        std::sync::Arc::new(Calculator {
            expression: String::new(),
        }),
    ];

    assert_eq!(tools.len(), 2);
    assert_eq!(tools[0].name(), "web_search");
    assert_eq!(tools[1].name(), "calculator");
}

// ── AC 4: ErasedTool JSON round-trip via execute_json ─────────────

#[tokio::test]
async fn test_erased_tool_execute_json() {
    let tool = WebSearch {
        query: String::new(),
        max_results: None,
    };

    let input = serde_json::json!({"query": "rust async", "max_results": 5});
    let output = tool.execute_json(input).await.unwrap();

    assert_eq!(output["results"][0], "Result for: rust async");
    assert_eq!(output["count"], 5);
}

#[tokio::test]
async fn test_erased_tool_invalid_input_error() {
    let tool = WebSearch {
        query: String::new(),
        max_results: None,
    };

    let bad_input = serde_json::json!({"wrong_field": 123});
    let result = tool.execute_json(bad_input).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid input"));
}

#[test]
fn test_erased_schema_matches_static_schema() {
    let tool = WebSearch {
        query: String::new(),
        max_results: None,
    };
    let erased = ErasedTool::schema(&tool);
    let static_schema = WebSearch::tool_schema();

    assert_eq!(erased.name, static_schema.name);
    assert_eq!(erased.description, static_schema.description);
    assert_eq!(erased.parameters, static_schema.parameters);
}
