//! Agent runtime loop — the execution engine.
//!
//! The runtime loop logic has been extracted into [`DefaultStrategy`](crate::default_strategy::DefaultStrategy)
//! as part of the v0.2.0 AgentStrategy refactoring. This module now only
//! contains end-to-end agent tests, split into focused submodules.

#[cfg(test)]
mod tests_basic;

#[cfg(test)]
mod tests_guards;

#[cfg(test)]
mod tests_hooks;
