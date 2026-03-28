//! Built-in [`ToolRegistry`] implementations.
//!
//! - [`DynamicRegistry`] supports runtime tool activation/deactivation.
//! - [`GroupedRegistry`] organizes tools into named groups with group-level
//!   activation/deactivation.
//!
//! Both use `RwLock` for interior mutability, enabling shared `&self` access.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use crate::traits::tool::ErasedTool;
use crate::traits::tool_registry::ToolRegistry;

// ===========================================================================
// DynamicRegistry
// ===========================================================================

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

// ===========================================================================
// GroupedRegistry
// ===========================================================================

/// A tool registry that organizes tools into **named groups** with group-level
/// activation and deactivation.
///
/// Only tools in *active* groups are returned by `get_tools()`, while
/// `find_tool()` searches **all** groups (active or not) to ensure tool
/// execution is always possible even for deactivated groups.
///
/// Uses `RwLock` for interior mutability — group switching is safe from `&self`.
///
/// # Example
///
/// ```rust
/// use traitclaw_core::registries::GroupedRegistry;
/// use traitclaw_core::traits::tool_registry::ToolRegistry;
///
/// let registry = GroupedRegistry::new();
/// assert!(registry.is_empty());
/// ```
///
/// ```rust,no_run
/// use traitclaw_core::registries::GroupedRegistry;
/// use traitclaw_core::traits::tool_registry::ToolRegistry;
///
/// let registry = GroupedRegistry::new()
///     // .group("search", vec![web_search, deep_search])
///     // .group("code", vec![read_file, write_file])
///     .activate("search");
///
/// // Only "search" tools are returned by get_tools()
/// // But find_tool() can still find "code" tools
/// ```
pub struct GroupedRegistry {
    groups: RwLock<HashMap<String, Vec<Arc<dyn ErasedTool>>>>,
    active_groups: RwLock<HashSet<String>>,
}

impl GroupedRegistry {
    /// Create an empty grouped registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            groups: RwLock::new(HashMap::new()),
            active_groups: RwLock::new(HashSet::new()),
        }
    }

    /// Add a named group of tools.
    ///
    /// Tools are provided as `Arc<dyn ErasedTool>`. The group is **not**
    /// activated automatically — call `activate()` to enable it.
    ///
    /// If a group with the same name already exists, it is replaced.
    #[must_use]
    pub fn group(self, name: impl Into<String>, tools: Vec<Arc<dyn ErasedTool>>) -> Self {
        {
            let mut groups = self.groups.write().expect("GroupedRegistry lock poisoned");
            groups.insert(name.into(), tools);
        }
        self
    }

    /// Activate a group, making its tools visible via `get_tools()`.
    ///
    /// Multiple groups can be active simultaneously.
    /// Activating an already-active group is a no-op.
    #[must_use]
    pub fn activate(self, name: impl Into<String>) -> Self {
        {
            let mut active = self
                .active_groups
                .write()
                .expect("GroupedRegistry lock poisoned");
            active.insert(name.into());
        }
        self
    }

    /// Activate a group at runtime (non-builder).
    ///
    /// Returns `true` if the group exists and was activated.
    pub fn activate_group(&self, name: &str) -> bool {
        let groups = self.groups.read().expect("GroupedRegistry lock poisoned");
        if groups.contains_key(name) {
            let mut active = self
                .active_groups
                .write()
                .expect("GroupedRegistry lock poisoned");
            active.insert(name.to_string());
            true
        } else {
            false
        }
    }

    /// Deactivate a group at runtime.
    ///
    /// Returns `true` if the group was previously active.
    pub fn deactivate_group(&self, name: &str) -> bool {
        let mut active = self
            .active_groups
            .write()
            .expect("GroupedRegistry lock poisoned");
        active.remove(name)
    }

    /// Get the names of all registered groups.
    #[must_use]
    pub fn group_names(&self) -> Vec<String> {
        let groups = self.groups.read().expect("GroupedRegistry lock poisoned");
        groups.keys().cloned().collect()
    }

    /// Get the names of currently active groups.
    #[must_use]
    pub fn active_group_names(&self) -> Vec<String> {
        let active = self
            .active_groups
            .read()
            .expect("GroupedRegistry lock poisoned");
        active.iter().cloned().collect()
    }

    /// Check if a specific group is currently active.
    #[must_use]
    pub fn is_group_active(&self, name: &str) -> bool {
        let active = self
            .active_groups
            .read()
            .expect("GroupedRegistry lock poisoned");
        active.contains(name)
    }
}

impl Default for GroupedRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry for GroupedRegistry {
    /// Returns tools from **active groups only**.
    fn get_tools(&self) -> Vec<Arc<dyn ErasedTool>> {
        let groups = self.groups.read().expect("GroupedRegistry lock poisoned");
        let active = self
            .active_groups
            .read()
            .expect("GroupedRegistry lock poisoned");

        let mut tools = Vec::new();
        for group_name in active.iter() {
            if let Some(group_tools) = groups.get(group_name) {
                for tool in group_tools {
                    tools.push(Arc::clone(tool));
                }
            }
        }
        tools
    }

    /// Searches **all groups** (active or not) for a tool by name.
    ///
    /// This allows tool execution even when the tool's group is deactivated.
    fn find_tool(&self, name: &str) -> Option<Arc<dyn ErasedTool>> {
        let groups = self.groups.read().expect("GroupedRegistry lock poisoned");
        for tools in groups.values() {
            if let Some(tool) = tools.iter().find(|t| t.name() == name) {
                return Some(Arc::clone(tool));
            }
        }
        None
    }

    /// Returns the number of tools in **active groups only**.
    fn len(&self) -> usize {
        let groups = self.groups.read().expect("GroupedRegistry lock poisoned");
        let active = self
            .active_groups
            .read()
            .expect("GroupedRegistry lock poisoned");

        let mut count = 0;
        for group_name in active.iter() {
            if let Some(group_tools) = groups.get(group_name) {
                count += group_tools.len();
            }
        }
        count
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// ===========================================================================
// AdaptiveRegistry
// ===========================================================================

/// Per-tier tool limits.
///
/// Defines how many tools are visible for each model tier.
#[derive(Debug, Clone, Copy)]
pub struct TierLimits {
    /// Max tools for small models.
    pub small: usize,
    /// Max tools for medium models.
    pub medium: usize,
    /// Max tools for large models.
    pub large: usize,
}

impl Default for TierLimits {
    fn default() -> Self {
        Self {
            small: 5,
            medium: 15,
            large: usize::MAX,
        }
    }
}

/// A tool registry that **automatically limits** the number of visible
/// tools based on the configured model tier.
///
/// Small models get fewer tools (to fit smaller context windows),
/// while large models get access to all registered tools.
///
/// Tool **priority** is determined by insertion order — tools registered
/// first are selected first when the limit is applied.
///
/// `find_tool()` searches all tools regardless of tier limit, ensuring
/// tool execution always works even for tools beyond the active limit.
///
/// # Example
///
/// ```rust
/// use traitclaw_core::registries::AdaptiveRegistry;
/// use traitclaw_core::traits::tool_registry::ToolRegistry;
/// use traitclaw_core::types::model_info::ModelTier;
///
/// let registry = AdaptiveRegistry::new(vec![], ModelTier::Medium);
/// assert!(registry.is_empty());
/// ```
pub struct AdaptiveRegistry {
    tools: Vec<Arc<dyn ErasedTool>>,
    limits: TierLimits,
    tier: crate::types::model_info::ModelTier,
}

impl AdaptiveRegistry {
    /// Create an adaptive registry with default tier limits.
    ///
    /// Default limits: Small=5, Medium=15, Large=unlimited.
    #[must_use]
    pub fn new(tools: Vec<Arc<dyn ErasedTool>>, tier: crate::types::model_info::ModelTier) -> Self {
        Self {
            tools,
            limits: TierLimits::default(),
            tier,
        }
    }

    /// Override the default tier limits.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use traitclaw_core::registries::AdaptiveRegistry;
    /// use traitclaw_core::types::model_info::ModelTier;
    ///
    /// let registry = AdaptiveRegistry::new(vec![], ModelTier::Small)
    ///     .with_limits(3, 10, 50);
    /// ```
    #[must_use]
    pub fn with_limits(mut self, small: usize, medium: usize, large: usize) -> Self {
        self.limits = TierLimits {
            small,
            medium,
            large,
        };
        self
    }

    /// Get the current tier limits.
    #[must_use]
    pub fn limits(&self) -> TierLimits {
        self.limits
    }

    /// Get the configured model tier.
    #[must_use]
    pub fn tier(&self) -> crate::types::model_info::ModelTier {
        self.tier
    }

    /// Resolve the effective tool limit for the current tier.
    fn effective_limit(&self) -> usize {
        use crate::types::model_info::ModelTier;
        match self.tier {
            ModelTier::Small => self.limits.small,
            ModelTier::Medium => self.limits.medium,
            ModelTier::Large => self.limits.large,
        }
    }
}

impl ToolRegistry for AdaptiveRegistry {
    /// Returns at most `limit` tools based on the configured `ModelTier`.
    ///
    /// Tools are returned in insertion order (first registered = highest priority).
    fn get_tools(&self) -> Vec<Arc<dyn ErasedTool>> {
        let limit = self.effective_limit();
        self.tools
            .iter()
            .take(limit)
            .map(|t| Arc::clone(t))
            .collect()
    }

    /// Searches **all** tools regardless of tier limit.
    ///
    /// This ensures tool execution works even for tools beyond the active limit.
    fn find_tool(&self, name: &str) -> Option<Arc<dyn ErasedTool>> {
        self.tools
            .iter()
            .find(|t| t.name() == name)
            .map(|t| Arc::clone(t))
    }

    /// Returns the count of tools visible for the current tier.
    fn len(&self) -> usize {
        let limit = self.effective_limit();
        self.tools.len().min(limit)
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

    // ── DynamicRegistry tests ───────────────────────────────────────────

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

    // ── GroupedRegistry tests ───────────────────────────────────────────

    #[test]
    fn test_grouped_registry_empty() {
        let reg = GroupedRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
        assert!(reg.get_tools().is_empty());
        assert!(reg.find_tool("anything").is_none());
    }

    #[test]
    fn test_grouped_registry_single_group() {
        let tools: Vec<Arc<dyn ErasedTool>> = vec![
            Arc::new(FakeTool::new("web_search")),
            Arc::new(FakeTool::new("deep_search")),
        ];
        let reg = GroupedRegistry::new()
            .group("search", tools)
            .activate("search");

        assert_eq!(reg.len(), 2);
        assert!(!reg.is_empty());
        let active = reg.get_tools();
        assert_eq!(active.len(), 2);
    }

    #[test]
    fn test_grouped_registry_multiple_groups_activate_switch() {
        // AC #8: activate A → deactivate A → activate B
        let search_tools: Vec<Arc<dyn ErasedTool>> = vec![Arc::new(FakeTool::new("web_search"))];
        let code_tools: Vec<Arc<dyn ErasedTool>> = vec![
            Arc::new(FakeTool::new("read_file")),
            Arc::new(FakeTool::new("write_file")),
        ];

        let reg = GroupedRegistry::new()
            .group("search", search_tools)
            .group("code", code_tools)
            .activate("search");

        // Phase 1: "search" active
        assert_eq!(reg.len(), 1);
        assert_eq!(reg.get_tools()[0].name(), "web_search");

        // Phase 2: deactivate "search"
        assert!(reg.deactivate_group("search"));
        assert!(reg.is_empty());

        // Phase 3: activate "code"
        assert!(reg.activate_group("code"));
        assert_eq!(reg.len(), 2);
        let names: Vec<String> = reg
            .get_tools()
            .iter()
            .map(|t| t.name().to_string())
            .collect();
        assert!(names.contains(&"read_file".to_string()));
        assert!(names.contains(&"write_file".to_string()));
    }

    #[test]
    fn test_grouped_registry_multiple_active_groups() {
        // AC #4: multiple groups active simultaneously
        let search_tools: Vec<Arc<dyn ErasedTool>> = vec![Arc::new(FakeTool::new("web_search"))];
        let code_tools: Vec<Arc<dyn ErasedTool>> = vec![Arc::new(FakeTool::new("read_file"))];

        let reg = GroupedRegistry::new()
            .group("search", search_tools)
            .group("code", code_tools)
            .activate("search")
            .activate("code");

        assert_eq!(reg.len(), 2);
        let names: Vec<String> = reg
            .get_tools()
            .iter()
            .map(|t| t.name().to_string())
            .collect();
        assert!(names.contains(&"web_search".to_string()));
        assert!(names.contains(&"read_file".to_string()));
    }

    #[test]
    fn test_grouped_registry_find_tool_searches_all_groups() {
        // AC #6: find_tool searches ALL groups (active or not)
        let search_tools: Vec<Arc<dyn ErasedTool>> = vec![Arc::new(FakeTool::new("web_search"))];
        let code_tools: Vec<Arc<dyn ErasedTool>> = vec![Arc::new(FakeTool::new("read_file"))];

        let reg = GroupedRegistry::new()
            .group("search", search_tools)
            .group("code", code_tools)
            .activate("search"); // only "search" is active

        // get_tools returns only active group
        assert_eq!(reg.get_tools().len(), 1);

        // find_tool finds tools in ALL groups
        assert!(reg.find_tool("web_search").is_some()); // active group
        assert!(reg.find_tool("read_file").is_some()); // inactive group!
        assert!(reg.find_tool("nonexistent").is_none());
    }

    #[test]
    fn test_grouped_registry_activate_nonexistent_group() {
        let reg = GroupedRegistry::new().group("search", vec![Arc::new(FakeTool::new("a"))]);

        // Activating a group that doesn't exist returns false
        assert!(!reg.activate_group("nonexistent"));
        assert!(reg.activate_group("search"));
    }

    #[test]
    fn test_grouped_registry_deactivate_nonexistent() {
        let reg = GroupedRegistry::new();
        // Deactivating a non-active group returns false
        assert!(!reg.deactivate_group("nonexistent"));
    }

    #[test]
    fn test_grouped_registry_group_names() {
        let reg = GroupedRegistry::new()
            .group("search", vec![])
            .group("code", vec![])
            .activate("search");

        let mut names = reg.group_names();
        names.sort();
        assert_eq!(names, vec!["code", "search"]);

        let active = reg.active_group_names();
        assert_eq!(active.len(), 1);
        assert!(active.contains(&"search".to_string()));
    }

    #[test]
    fn test_grouped_registry_is_group_active() {
        let reg = GroupedRegistry::new()
            .group("search", vec![])
            .group("code", vec![])
            .activate("search");

        assert!(reg.is_group_active("search"));
        assert!(!reg.is_group_active("code"));
    }

    #[test]
    fn test_grouped_registry_concurrent_read() {
        // AC #9: concurrent read access is safe
        use std::thread;

        let reg = Arc::new(
            GroupedRegistry::new()
                .group("a", vec![Arc::new(FakeTool::new("tool_a"))])
                .group("b", vec![Arc::new(FakeTool::new("tool_b"))])
                .activate("a"),
        );

        let mut handles = vec![];
        for _ in 0..10 {
            let reg_clone = Arc::clone(&reg);
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    let tools = reg_clone.get_tools();
                    assert_eq!(tools.len(), 1);
                    assert!(reg_clone.find_tool("tool_a").is_some());
                    assert!(reg_clone.find_tool("tool_b").is_some());
                }
            }));
        }

        for h in handles {
            h.join().expect("thread panicked");
        }
    }

    #[test]
    fn test_grouped_registry_object_safe() {
        let reg = GroupedRegistry::new();
        let _: Arc<dyn ToolRegistry> = Arc::new(reg);
    }

    #[test]
    fn test_grouped_registry_replace_group() {
        // Adding a group with the same name replaces it
        let reg = GroupedRegistry::new()
            .group("search", vec![Arc::new(FakeTool::new("old_tool"))])
            .group("search", vec![Arc::new(FakeTool::new("new_tool"))])
            .activate("search");

        assert_eq!(reg.len(), 1);
        assert!(reg.find_tool("new_tool").is_some());
        assert!(reg.find_tool("old_tool").is_none());
    }

    // ── AdaptiveRegistry tests ──────────────────────────────────────────

    fn make_tools(n: usize) -> Vec<Arc<dyn ErasedTool>> {
        (0..n)
            .map(|i| Arc::new(FakeTool::new(&format!("tool_{i}"))) as Arc<dyn ErasedTool>)
            .collect()
    }

    #[test]
    fn test_adaptive_registry_small_tier_limits() {
        // AC #7: Small tier → 5 tools from 30
        use crate::types::model_info::ModelTier;
        let reg = AdaptiveRegistry::new(make_tools(30), ModelTier::Small);
        assert_eq!(reg.len(), 5);
        assert_eq!(reg.get_tools().len(), 5);
        // First 5 tools by insertion order
        assert_eq!(reg.get_tools()[0].name(), "tool_0");
        assert_eq!(reg.get_tools()[4].name(), "tool_4");
    }

    #[test]
    fn test_adaptive_registry_medium_tier_limits() {
        use crate::types::model_info::ModelTier;
        let reg = AdaptiveRegistry::new(make_tools(30), ModelTier::Medium);
        assert_eq!(reg.len(), 15);
        assert_eq!(reg.get_tools().len(), 15);
    }

    #[test]
    fn test_adaptive_registry_large_tier_all() {
        // AC #8: Large tier → all 30 tools
        use crate::types::model_info::ModelTier;
        let reg = AdaptiveRegistry::new(make_tools(30), ModelTier::Large);
        assert_eq!(reg.len(), 30);
        assert_eq!(reg.get_tools().len(), 30);
    }

    #[test]
    fn test_adaptive_registry_custom_limits() {
        use crate::types::model_info::ModelTier;
        let reg = AdaptiveRegistry::new(make_tools(30), ModelTier::Small).with_limits(3, 10, 50);
        assert_eq!(reg.len(), 3);
        assert_eq!(reg.get_tools().len(), 3);
    }

    #[test]
    fn test_adaptive_registry_find_tool_beyond_limit() {
        // find_tool searches ALL tools, even beyond the tier limit
        use crate::types::model_info::ModelTier;
        let reg = AdaptiveRegistry::new(make_tools(30), ModelTier::Small);
        // tool_29 is beyond the Small limit of 5
        assert!(reg.find_tool("tool_29").is_some());
        assert!(reg.find_tool("tool_0").is_some());
        assert!(reg.find_tool("nonexistent").is_none());
    }

    #[test]
    fn test_adaptive_registry_empty() {
        use crate::types::model_info::ModelTier;
        let reg = AdaptiveRegistry::new(vec![], ModelTier::Large);
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
    }

    #[test]
    fn test_adaptive_registry_object_safe() {
        use crate::types::model_info::ModelTier;
        let reg = AdaptiveRegistry::new(vec![], ModelTier::Medium);
        let _: Arc<dyn ToolRegistry> = Arc::new(reg);
    }
}
