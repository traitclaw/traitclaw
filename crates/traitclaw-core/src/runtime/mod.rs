//! Agent runtime loop — the execution engine.
//!
//! The runtime loop logic has been extracted into [`DefaultStrategy`](crate::default_strategy::DefaultStrategy)
//! as part of the v0.2.0 AgentStrategy refactoring.
//!
//! Tests have been migrated to integration tests under `tests/runtime_*.rs`
//! to enable use of shared test utilities from `traitclaw-test-utils`.
