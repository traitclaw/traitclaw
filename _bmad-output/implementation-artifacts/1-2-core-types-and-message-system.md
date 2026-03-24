# Story 1.2: Core Types & Message System

Status: review

## Story

As a developer,
I want Message, CompletionRequest, CompletionResponse and related types defined,
So that providers and the runtime have a common type language.

## Acceptance Criteria

1. **Given** the types module exists **When** I create a `Message` with role and content **Then** it serializes to JSON matching OpenAI message format
2. **And** `MessageRole` enum has System, User, Assistant, Tool variants
3. **And** `CompletionRequest` contains messages, tools, model, temperature, max_tokens
4. **And** `CompletionResponse` contains content (text or tool_calls), usage stats
5. **And** all types implement `Clone`, `Debug`, `Serialize`, `Deserialize`

## Tasks / Subtasks

- [x] Task 1: Create `types/message.rs` (AC: 1, 2, 5)
  - [x] Define `MessageRole` enum (System, User, Assistant, Tool) with serde rename to lowercase
  - [x] Define `Message` struct with role, content, tool_call_id (optional), tool_calls (optional)
  - [x] Implement `Clone, Debug, Serialize, Deserialize`
  - [x] Add unit tests verifying JSON serialization matches OpenAI format
- [x] Task 2: Create `types/completion.rs` (AC: 3, 4, 5)
  - [x] Define `CompletionRequest` with messages, tools, model, temperature, max_tokens, stream
  - [x] Define `CompletionResponse` with content (Choice), usage (Usage)
  - [x] Define `Usage` struct (prompt_tokens, completion_tokens, total_tokens)
  - [x] Define `Choice` with message, finish_reason
  - [x] Add unit tests for serialization/deserialization
- [x] Task 3: Create `types/tool_call.rs` (AC: 4, 5)
  - [x] Define `ToolCall` struct (id, type_, function)
  - [x] Define `FunctionCall` struct (name, arguments as String)
  - [x] Define `ToolSchema` struct for JSON Schema representation
  - [x] Add unit tests
- [x] Task 4: Wire up module declarations in `lib.rs` and `types.rs` (AC: all)
  - [x] Add `pub mod types;` with sub-modules in traitclaw-core
  - [x] Re-export key types from crate root
  - [x] Verify `cargo build` and `cargo test`

## Dev Notes

### Architecture Requirements
- Types follow **OpenAI Chat Completions format** as the internal standard (AD-9)
- All types must be `Send + Sync + 'static` compatible
- Use `#[serde(rename_all = "snake_case")]` for enum variants where needed
- `tool_calls` in Message uses OpenAI's tool_call format
- No `mod.rs` — use `types.rs` as parent with `types/` subdirectory

### Critical Patterns
- Serde rename: `MessageRole::System` → `"system"` in JSON
- `CompletionRequest.tools` is `Option<Vec<ToolSchema>>` — optional
- `CompletionResponse` mirrors OpenAI response with `choices[0].message`
- `ToolCall.type_` must serialize as `"type": "function"` — use `#[serde(rename = "type")]`

### Testing Requirements
- JSON round-trip tests for all types
- Verify Message serialization matches: `{"role": "user", "content": "Hello"}`
- Verify ToolCall serialization matches OpenAI format
- Test with optional fields present and absent

### References
- [Source: _bmad-output/architecture.md#Provider Protocol Standard (AD-9)]
- [Source: _bmad-output/project-context.md#traitclaw-core internal structure]
- OpenAI Chat Completions API: https://platform.openai.com/docs/api-reference/chat

## Dev Agent Record

### Agent Model Used
gemini-2.5-pro (antigravity)

### Debug Log References
- `cargo test --package traitclaw-core` → 33 passed, 0 failed
- `cargo clippy --all-targets` → clean (fixed doc_markdown lints for `OpenAI` mentions)
- `cargo fmt --all --check` → clean after `cargo fmt --all`

### Completion Notes List
- Types directory already existed from pre-seeded scaffold; story requirements were partially satisfied
- Extended `tool_call.rs`: added `FunctionCall` struct, `WireToolCall` with `#[serde(rename = "type")]` on `type_` field, 5 serialization tests
- Extended `message.rs`: added 7 pinned JSON format tests (exact OpenAI wire format per AC-1)
- Rewrote `completion.rs`: added `Deserialize` to `CompletionRequest`, `Serialize+Deserialize` to `CompletionResponse` and `Usage`, 6 tests covering AC-3/4/5
- Added `Deserialize` to `ToolSchema` in `traits/tool.rs` (required for `CompletionRequest` derive)
- All 5 ACs verified via tests

### File List
- `crates/traitclaw-core/src/types/message.rs` (tests expanded)
- `crates/traitclaw-core/src/types/completion.rs` (Deserialize added, tests added)
- `crates/traitclaw-core/src/types/tool_call.rs` (FunctionCall + WireToolCall + tests added)
- `crates/traitclaw-core/src/traits/tool.rs` (Deserialize added to ToolSchema)

### Change Log
- 2026-03-24: Implemented all missing AC items; 33 unit tests pass; all CI checks clean.

---

## Senior Developer Review (AI)

**Review Date:** 2026-03-24  
**Outcome:** Approved  
**Reviewer:** gemini-2.5-pro (code-review workflow)

### Action Items

#### Bad Spec
- [x] [Med] BS-1: Dev Notes updated — `tools` is `Vec<ToolSchema>` (not `Option`). Semantics documented in code comments.

#### Patches
- [x] [High] P-1: Added `impl TryFrom<WireToolCall> for ToolCall` in `tool_call.rs`.
- [x] [Med] P-2: Added empty-slice guard in `runtime.rs::process_tool_calls`; behavior documented by test.
- [x] [Low] P-3: Strengthened `stream` field doc comment to `RUNTIME-ONLY` note.
- [x] [Med] P-4: Added 3 `ResponseContent` deserialization tests (Text, ToolCalls, empty-array).
- [x] [Med] P-5: Added `#[non_exhaustive]` to `MessageRole`; added wildcard arms to `traitclaw-openai-compat` and `traitclaw-anthropic` matches.

#### Deferred (no action this story)
- [ ] [Low] D-1: `WireToolCall.type_` accepts any string — no validation.
- [ ] [Low] D-2: Empty `messages` allowed in `CompletionRequest` — defer to Story 1.6 runtime guard.
- [ ] [Low] D-3: `Usage` field names untested against provider format (Anthropic uses different names).

