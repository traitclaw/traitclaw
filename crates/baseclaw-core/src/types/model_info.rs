//! Model information types.
//!
//! Describes the capabilities of a specific LLM used by the agent.
//! The `tier` field is consumed by the Steering system to auto-configure
//! Guard, Hint, and Tracker density.

use serde::{Deserialize, Serialize};

/// Model capability tier used by the steering system to auto-configure
/// Guard, Hint, and Tracker behavior.
///
/// Tiers control how aggressively the Steering layer applies constraints
/// to compensate for model capability differences.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ModelTier {
    /// Small models (e.g., Haiku, Phi, Gemma).
    ///
    /// Need aggressive steering: hints every 3 iterations, serial concurrency.
    Small,
    /// Medium models (e.g., Sonnet, GPT-4o-mini).
    ///
    /// Moderate steering: hints every 6 iterations, limited concurrency.
    Medium,
    /// Large models (e.g., Opus, GPT-4o, Gemini Ultra).
    ///
    /// Light steering: hints every 12 iterations, full concurrency.
    Large,
}

/// Information about the LLM model being used by a provider.
///
/// Returned by [`Provider::model_info`] and used by the runtime and
/// steering system to adapt behavior to the specific model.
///
/// This struct is `#[non_exhaustive]` — new capability fields may be added
/// in future releases without breaking changes.
///
/// [`Provider::model_info`]: crate::traits::provider::Provider::model_info
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ModelInfo {
    /// Model name (e.g., `"gpt-4o"`, `"claude-sonnet-3-5"`).
    pub name: String,
    /// Model capability tier for steering auto-configuration.
    pub tier: ModelTier,
    /// Maximum context window size in tokens.
    pub context_window: usize,
    /// Whether the model supports tool/function calling.
    pub supports_tools: bool,
    /// Whether the model supports image/vision input.
    pub supports_vision: bool,
    /// Whether the model supports structured JSON output mode.
    pub supports_structured: bool,
}

impl ModelInfo {
    /// Create a new `ModelInfo`.
    ///
    /// Use this constructor when building `ModelInfo` outside of the
    /// `baseclaw-core` crate, since the struct is `#[non_exhaustive]`.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        tier: ModelTier,
        context_window: usize,
        supports_tools: bool,
        supports_vision: bool,
        supports_structured: bool,
    ) -> Self {
        Self {
            name: name.into(),
            tier,
            context_window,
            supports_tools,
            supports_vision,
            supports_structured,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_tier_variants_ac4() {
        // AC-4: ModelTier has Small, Medium, Large variants
        let tiers = [ModelTier::Small, ModelTier::Medium, ModelTier::Large];
        assert_eq!(tiers[0], ModelTier::Small);
        assert_eq!(tiers[1], ModelTier::Medium);
        assert_eq!(tiers[2], ModelTier::Large);
    }

    #[test]
    fn test_model_tier_copy() {
        // ModelTier is Copy — verify it can be used by value freely
        let t = ModelTier::Large;
        let t2 = t; // copy
        let t3 = t; // still accessible
        assert_eq!(t2, t3); // Use both to satisfy lints
    }

    #[test]
    fn test_model_info_construction_ac3() {
        // AC-3: ModelInfo has name, tier, context_window, capabilities
        let info = ModelInfo {
            name: "gpt-4o".to_string(),
            tier: ModelTier::Large,
            context_window: 128_000,
            supports_tools: true,
            supports_vision: true,
            supports_structured: true,
        };
        assert_eq!(info.name, "gpt-4o");
        assert_eq!(info.tier, ModelTier::Large);
        assert_eq!(info.context_window, 128_000);
        assert!(info.supports_tools);
        assert!(info.supports_vision);
        assert!(info.supports_structured);
    }

    #[test]
    fn test_model_tier_serde_roundtrip() {
        let tier = ModelTier::Medium;
        let json = serde_json::to_string(&tier).unwrap();
        let decoded: ModelTier = serde_json::from_str(&json).unwrap();
        assert_eq!(tier, decoded);
    }
}
