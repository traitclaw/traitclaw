//! Built-in [`ToolRegistry`] implementations.
//!
//! [`DynamicRegistry`] supports runtime tool activation/deactivation
//! using interior mutability (`RwLock`).

use std::sync::{Arc, RwLock};

use crate::traits::tool::ErasedTool;
use crate::traits::tool_registry::ToolRegistry;

/// Entry in the dynamic registry, tracking enabled/disabled state.
struct ToolEntry {
    tool: Arc<dyn ErasedTool>,
    enabled: bool,
}

/// A mutable tool registry supporting runtime add/remove/toggle.
///
/// Uses `RwLock` for interior mutability, making it safe for concurrent access.
///
/// # Example
///
/// ```rust
/// use traitclaw_core::registries::DynamicRegistry;
/// use traitclaw_core::traits::tool_registry::ToolRegistry;
///
/// let registry = DynamicRegistry::new();
/// assert!(registry.is_empty());
/// ```
pub struct DynamicRegistry {
    tools: RwLock<Vec<ToolEntry>>,
}

impl DynamicRegistry {
    /// Create an empty dynamic registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tools: RwLock::new(Vec::new()),
        }
    }

    /// Create a registry pre-loaded with tools (all enabled).
    #[must_use]
    pub fn with_tools(tools: Vec<Arc<dyn ErasedTool>>) -> Self {
        let entries = tools
            .into_iter()
            .map(|tool| ToolEntry {
                tool,
                enabled: true,
            })
            .collect();
        Self {
            tools: RwLock::new(entries),
        }
    }
}

impl Default for DynamicRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry for DynamicRegistry {
    fn get_tools(&self) -> Vec<Arc<dyn ErasedTool>> {
        let tools = self.tools.read().expect("DynamicRegistry lock poisoned");
        tools
            .iter()
            .filter(|e| e.enabled)
            .map(|e| Arc::clone(&e.tool))
            .collect()
    }

    fn find_tool(&self, name: &str) -> Option<Arc<dyn ErasedTool>> {
        let tools = self.tools.read().expect("DynamicRegistry lock poisoned");
        tools
            .iter()
            .find(|e| e.enabled && e.tool.name() == name)
            .map(|e| Arc::clone(&e.tool))
    }

    fn register(&self, tool: Arc<dyn ErasedTool>) -> bool {
        let mut tools = self.tools.write().expect("DynamicRegistry lock poisoned");
        let name = tool.name().to_string();
        // Don't allow duplicate names
        if tools.iter().any(|e| e.tool.name() == name) {
            return false;
        }
        tools.push(ToolEntry {
            tool,
            enabled: true,
        });
        true
    }

    fn unregister(&self, name: &str) -> bool {
        let mut tools = self.tools.write().expect("DynamicRegistry lock poisoned");
        let len_before = tools.len();
        tools.retain(|e| e.tool.name() != name);
        tools.len() < len_before
    }

    fn set_enabled(&self, name: &str, enabled: bool) -> bool {
        let mut tools = self.tools.write().expect("DynamicRegistry lock poisoned");
        if let Some(entry) = tools.iter_mut().find(|e| e.tool.name() == name) {
            if entry.enabled != enabled {
                entry.enabled = enabled;
                return true;
            }
        }
        false
    }

    fn is_enabled(&self, name: &str) -> bool {
        let tools = self.tools.read().expect("DynamicRegistry lock poisoned");
        tools.iter().any(|e| e.tool.name() == name && e.enabled)
    }

    fn len(&self) -> usize {
        let tools = self.tools.read().expect("DynamicRegistry lock poisoned");
        tools.iter().filter(|e| e.enabled).count()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct FakeTool {
        tool_name: String,
    }

    impl FakeTool {
        fn new(name: &str) -> Self {
            Self {
                tool_name: name.to_string(),
            }
        }
    }

    #[async_trait]
    impl ErasedTool for FakeTool {
        fn name(&self) -> &str {
            &self.tool_name
        }
        fn description(&self) -> &str {
            "fake"
        }
        fn schema(&self) -> crate::traits::tool::ToolSchema {
            crate::traits::tool::ToolSchema {
                name: self.tool_name.clone(),
                description: "fake".to_string(),
                parameters: serde_json::json!({}),
            }
        }
        async fn execute_json(
            &self,
            _input: serde_json::Value,
        ) -> crate::Result<serde_json::Value> {
            Ok(serde_json::json!("ok"))
        }
    }

    #[test]
    fn test_dynamic_registry_empty() {
        let reg = DynamicRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
    }

    #[test]
    fn test_dynamic_registry_register_and_find() {
        let reg = DynamicRegistry::new();
        assert!(reg.register(Arc::new(FakeTool::new("search"))));
        assert_eq!(reg.len(), 1);
        assert!(reg.find_tool("search").is_some());
        assert!(reg.find_tool("calc").is_none());
    }

    #[test]
    fn test_dynamic_registry_no_duplicates() {
        let reg = DynamicRegistry::new();
        assert!(reg.register(Arc::new(FakeTool::new("search"))));
        assert!(!reg.register(Arc::new(FakeTool::new("search"))));
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn test_dynamic_registry_unregister() {
        let reg = DynamicRegistry::new();
        reg.register(Arc::new(FakeTool::new("search")));
        assert!(reg.unregister("search"));
        assert!(reg.is_empty());
        assert!(!reg.unregister("search")); // already gone
    }

    #[test]
    fn test_dynamic_registry_set_enabled() {
        let reg = DynamicRegistry::new();
        reg.register(Arc::new(FakeTool::new("search")));
        assert!(reg.is_enabled("search"));

        // Disable
        assert!(reg.set_enabled("search", false));
        assert!(!reg.is_enabled("search"));
        assert_eq!(reg.len(), 0); // disabled tools not counted
        assert!(reg.find_tool("search").is_none()); // not findable

        // Re-enable
        assert!(reg.set_enabled("search", true));
        assert!(reg.is_enabled("search"));
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn test_dynamic_registry_set_enabled_no_change() {
        let reg = DynamicRegistry::new();
        reg.register(Arc::new(FakeTool::new("search")));
        // Already enabled → no change
        assert!(!reg.set_enabled("search", true));
    }

    #[test]
    fn test_dynamic_registry_with_tools() {
        let tools: Vec<Arc<dyn ErasedTool>> =
            vec![Arc::new(FakeTool::new("a")), Arc::new(FakeTool::new("b"))];
        let reg = DynamicRegistry::with_tools(tools);
        assert_eq!(reg.len(), 2);
        assert!(reg.is_enabled("a"));
        assert!(reg.is_enabled("b"));
    }

    #[test]
    fn test_dynamic_registry_get_tools_only_enabled() {
        let reg = DynamicRegistry::new();
        reg.register(Arc::new(FakeTool::new("a")));
        reg.register(Arc::new(FakeTool::new("b")));
        reg.set_enabled("a", false);

        let tools = reg.get_tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name(), "b");
    }
}
