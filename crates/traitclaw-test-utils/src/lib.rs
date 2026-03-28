#![deny(missing_docs)]
#![allow(clippy::redundant_closure)]

//! # `TraitClaw` Test Utilities
//!
//! Shared test utilities for the `TraitClaw` AI Agent Framework.
//!
//! This crate provides reusable mock implementations and helpers
//! for testing agents without hitting real LLM APIs:
//!
//! - [`MockProvider`](provider::MockProvider) — Deterministic LLM provider returning pre-defined responses
//! - [`MockMemory`](memory::MockMemory) — In-memory session-based memory backend
//! - [`EchoTool`](tools::EchoTool) — Tool that echoes its input for tool-calling tests
//! - [`FailTool`](tools::FailTool) — Tool that always returns an error
//! - [`make_runtime`](runtime::make_runtime) — One-call `AgentRuntime` setup for strategy tests
//!
//! # Quick Start
//!
//! ```rust
//! use traitclaw_test_utils::provider::MockProvider;
//! use traitclaw_test_utils::runtime::make_runtime;
//!
//! let runtime = make_runtime(MockProvider::text("hello"), vec![]);
//! // Use runtime with any AgentStrategy for deterministic testing
//! ```

pub mod memory;
pub mod provider;
pub mod runtime;
pub mod tools;
