//! Dynamic tool registry — pluggable tool management for v0.3.0.
//!
//! [`ToolRegistry`] provides a unified interface for managing tools at runtime.
//! It supports both read-only registries (like [`SimpleRegistry`]) and
//! dynamic registries that allow runtime tool activation/deactivation.
//!
//! # Design
//!
//! All methods take `&self` to enable shared ownership via `Arc<dyn ToolRegistry>`.
//! Mutable operations (register, unregister, set_enabled) use interior mutability
//! (e.g., `RwLock`) in implementations that support them.
//!
//! # Example
//!
//! ```rust
//! use traitclaw_core::traits::tool_registry::{ToolRegistry, SimpleRegistry};
//! use traitclaw_core::traits::tool::ErasedTool;
//! use std::sync::Arc;
//!
//! // Wrap existing tools in a SimpleRegistry
//! let tools: Vec<Arc<dyn ErasedTool>> = vec![];
//! let registry = SimpleRegistry::new(tools);
//! assert_eq!(registry.get_tools().len(), 0);
//! ```

use std::sync::Arc;

use crate::traits::tool::ErasedTool;

/// Trait for pluggable tool management.
///
/// Provides read access to the current tool set and optional write operations
/// for dynamic tool management. Write methods return `bool` to indicate success.
///
/// All methods take `&self` — implementations requiring mutation should use
/// interior mutability (e.g., `RwLock`).
pub trait ToolRegistry: Send + Sync {
    /// Get all currently enabled tools.
    fn get_tools(&self) -> Vec<Arc<dyn ErasedTool>>;

    /// Find a tool by name.
    fn find_tool(&self, name: &str) -> Option<Arc<dyn ErasedTool>> {
        self.get_tools()
            .into_iter()
            .find(|t| t.schema().name == name)
    }

    /// Register a new tool. Returns `true` if the tool was added.
    ///
    /// Default implementation returns `false` (read-only registry).
    fn register(&self, _tool: Arc<dyn ErasedTool>) -> bool {
        false
    }

    /// Unregister a tool by name. Returns `true` if the tool was removed.
    ///
    /// Default implementation returns `false` (read-only registry).
    fn unregister(&self, _name: &str) -> bool {
        false
    }

    /// Enable or disable a tool by name. Returns `true` if the state changed.
    ///
    /// Default implementation returns `false` (read-only registry).
    fn set_enabled(&self, _name: &str, _enabled: bool) -> bool {
        false
    }

    /// Check if a tool is currently enabled.
    fn is_enabled(&self, name: &str) -> bool {
        self.find_tool(name).is_some()
    }

    /// Get the number of currently enabled tools.
    fn len(&self) -> usize {
        self.get_tools().len()
    }

    /// Check if the registry has no enabled tools.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// ---------------------------------------------------------------------------
// SimpleRegistry — immutable wrapper around Vec<Arc<dyn ErasedTool>>
// ---------------------------------------------------------------------------

/// A simple, immutable tool registry that wraps `Vec<Arc<dyn ErasedTool>>`.
///
/// This is the default registry used when no custom registry is configured.
/// It preserves the v0.2.0 behavior where tools are fixed at agent construction time.
///
/// For dynamic tool management, use `DynamicRegistry` (Story 3.3).
pub struct SimpleRegistry {
    tools: Vec<Arc<dyn ErasedTool>>,
}

impl SimpleRegistry {
    /// Create a registry from an existing tool list.
    #[must_use]
    pub fn new(tools: Vec<Arc<dyn ErasedTool>>) -> Self {
        Self { tools }
    }
}

impl ToolRegistry for SimpleRegistry {
    fn get_tools(&self) -> Vec<Arc<dyn ErasedTool>> {
        self.tools.clone()
    }

    fn find_tool(&self, name: &str) -> Option<Arc<dyn ErasedTool>> {
        self.tools.iter().find(|t| t.schema().name == name).cloned()
    }

    fn len(&self) -> usize {
        self.tools.len()
    }

    fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Object safety ───────────────────────────────────────────────────
    #[test]
    fn test_tool_registry_is_object_safe() {
        let registry = SimpleRegistry::new(vec![]);
        let _: Arc<dyn ToolRegistry> = Arc::new(registry);
    }

    // ── SimpleRegistry basic operations ─────────────────────────────────
    #[test]
    fn test_simple_registry_empty() {
        let registry = SimpleRegistry::new(vec![]);
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
        assert_eq!(registry.get_tools().len(), 0);
        assert!(registry.find_tool("nonexistent").is_none());
    }

    // ── Write operations rejected on SimpleRegistry ─────────────────────
    #[test]
    fn test_simple_registry_is_read_only() {
        let registry = SimpleRegistry::new(vec![]);
        assert!(!registry.register(Arc::new(DummyTool)));
        assert!(!registry.unregister("dummy"));
        assert!(!registry.set_enabled("dummy", false));
    }

    // ── SimpleRegistry with tools ──────────────────────────────────────
    #[test]
    fn test_simple_registry_with_tools() {
        let tool: Arc<dyn ErasedTool> = Arc::new(DummyTool);
        let registry = SimpleRegistry::new(vec![tool]);
        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
        assert!(registry.find_tool("dummy_tool").is_some());
        assert!(registry.find_tool("nonexistent").is_none());
        assert!(registry.is_enabled("dummy_tool"));
        assert!(!registry.is_enabled("nonexistent"));
    }

    // ── Dummy tool for tests ────────────────────────────────────────────
    struct DummyTool;

    #[async_trait::async_trait]
    impl ErasedTool for DummyTool {
        fn name(&self) -> &str {
            "dummy_tool"
        }

        fn description(&self) -> &str {
            "A test tool"
        }

        fn schema(&self) -> crate::traits::tool::ToolSchema {
            crate::traits::tool::ToolSchema {
                name: "dummy_tool".to_string(),
                description: "A test tool".to_string(),
                parameters: serde_json::json!({"type": "object", "properties": {}}),
            }
        }

        async fn execute_json(
            &self,
            _input: serde_json::Value,
        ) -> crate::Result<serde_json::Value> {
            Ok(serde_json::json!("ok"))
        }
    }
}
