# Story 1.4: Implement `make_runtime()` Helper

Status: review

## Story

As a framework contributor,
I want a `make_runtime(provider, tools)` helper that constructs a ready-to-use `AgentRuntime`,
so that test setup is a single function call instead of multi-line builder boilerplate.

## Acceptance Criteria

1. ✅ `make_runtime(provider, tools)` returns a fully configured `AgentRuntime`
2. ✅ Default components: MockMemory, NoopTracker, NoopContextManager, NoopOutputTransformer, etc.
3. ✅ `make_runtime_with_config(provider, tools, config)` overload accepts custom `AgentConfig`
4. ✅ No guards, hints, or hooks by default (empty vecs)
5. ✅ Doc comments with `/// # Example` showing typical usage
6. ✅ `cargo test -p traitclaw-test-utils` passes with all tests

## Dev Agent Record

### Agent Model Used
Antigravity (Google DeepMind)

### Completion Notes List
- make_runtime: creates AgentRuntime with all noop defaults + MockMemory
- make_runtime_with_config: same but accepts custom AgentConfig
- Noop types (Tracker, ContextManager, OutputTransformer) are pub(crate) 
- 4 unit tests + 3 doc-tests
- #[allow(deprecated)] for ContextStrategy/OutputProcessor compat fields
- Crate-wide lib.rs doc example now compiles (changed from `ignore` to real test)

### File List
- MODIFIED: `crates/traitclaw-test-utils/src/runtime.rs`
- MODIFIED: `crates/traitclaw-test-utils/src/lib.rs`
