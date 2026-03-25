//! Shared `TeamContext` — thread-safe inter-agent state store.
//!
//! Agents in a pipeline can share state through `TeamContext`, which
//! provides `RwLock`-based concurrent access to a `serde_json::Value` map.
//!
//! # Example
//!
//! ```rust
//! use traitclaw_team::team_context::TeamContext;
//!
//! let ctx = TeamContext::new();
//! ctx.set("topic", serde_json::json!("Rust programming"));
//!
//! let val = ctx.get("topic").unwrap();
//! assert_eq!(val, serde_json::json!("Rust programming"));
//! ```

use std::collections::HashMap;
use std::sync::RwLock;

use serde_json::Value;

/// Thread-safe key-value store for sharing state between agents in a team.
pub struct TeamContext {
    store: RwLock<HashMap<String, Value>>,
}

impl TeamContext {
    /// Create a new empty `TeamContext`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            store: RwLock::new(HashMap::new()),
        }
    }

    /// Set a value for the given key.
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned (a previous writer panicked).
    pub fn set(&self, key: impl Into<String>, value: Value) {
        let mut store = self.store.write().expect("TeamContext lock poisoned");
        store.insert(key.into(), value);
    }

    /// Get the value for the given key, if present.
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<Value> {
        let store = self.store.read().expect("TeamContext lock poisoned");
        store.get(key).cloned()
    }

    /// Remove a key and return its previous value, if any.
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    pub fn remove(&self, key: &str) -> Option<Value> {
        let mut store = self.store.write().expect("TeamContext lock poisoned");
        store.remove(key)
    }

    /// Returns `true` if the context contains the given key.
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    #[must_use]
    pub fn contains_key(&self, key: &str) -> bool {
        let store = self.store.read().expect("TeamContext lock poisoned");
        store.contains_key(key)
    }

    /// Return all keys currently stored.
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    #[must_use]
    pub fn keys(&self) -> Vec<String> {
        let store = self.store.read().expect("TeamContext lock poisoned");
        store.keys().cloned().collect()
    }

    /// Number of entries in the context.
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    #[must_use]
    pub fn len(&self) -> usize {
        let store = self.store.read().expect("TeamContext lock poisoned");
        store.len()
    }

    /// Whether the context is empty.
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for TeamContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn test_set_and_get() {
        // AC #1, #5: set/get values using serde_json::Value
        let ctx = TeamContext::new();
        ctx.set("name", Value::String("Alice".to_string()));

        let val = ctx.get("name").unwrap();
        assert_eq!(val, Value::String("Alice".to_string()));
    }

    #[test]
    fn test_get_missing_key() {
        let ctx = TeamContext::new();
        assert!(ctx.get("missing").is_none());
    }

    #[test]
    fn test_overwrite_key() {
        let ctx = TeamContext::new();
        ctx.set("x", serde_json::json!(1));
        ctx.set("x", serde_json::json!(2));
        assert_eq!(ctx.get("x").unwrap(), serde_json::json!(2));
    }

    #[test]
    fn test_multiple_keys() {
        let ctx = TeamContext::new();
        ctx.set("a", serde_json::json!("value_a"));
        ctx.set("b", serde_json::json!(42));
        ctx.set("c", serde_json::json!(true));

        assert_eq!(ctx.len(), 3);
        assert!(ctx.contains_key("a"));
        assert!(ctx.contains_key("b"));
        assert!(ctx.contains_key("c"));
    }

    #[test]
    fn test_remove_key() {
        let ctx = TeamContext::new();
        ctx.set("k", serde_json::json!("val"));
        let removed = ctx.remove("k");
        assert_eq!(removed, Some(serde_json::json!("val")));
        assert!(ctx.get("k").is_none());
    }

    #[test]
    fn test_concurrent_access() {
        // AC #5: RwLock-based for concurrent access safety
        let ctx = Arc::new(TeamContext::new());

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let ctx = ctx.clone();
                std::thread::spawn(move || {
                    ctx.set(format!("key_{i}"), serde_json::json!(i));
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(ctx.len(), 10);
        for i in 0..10 {
            assert!(ctx.contains_key(&format!("key_{i}")));
        }
    }

    #[test]
    fn test_agent_a_sets_agent_b_reads() {
        // AC #6: simulated: agent A sets key → agent B reads same key → value matches
        let ctx = TeamContext::new();

        // "Agent A" writes
        ctx.set("research_result", serde_json::json!("AI history summary"));

        // "Agent B" reads
        let val = ctx.get("research_result").unwrap();
        assert_eq!(val, serde_json::json!("AI history summary"));
    }

    #[test]
    fn test_keys_listing() {
        let ctx = TeamContext::new();
        ctx.set("x", serde_json::json!(1));
        ctx.set("y", serde_json::json!(2));
        let mut keys = ctx.keys();
        keys.sort();
        assert_eq!(keys, vec!["x", "y"]);
    }

    #[test]
    fn test_is_empty() {
        let ctx = TeamContext::new();
        assert!(ctx.is_empty());
        ctx.set("k", serde_json::json!("v"));
        assert!(!ctx.is_empty());
    }
}
