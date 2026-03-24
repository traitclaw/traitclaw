# Story 4.1: OpenAI-Compatible Provider

Status: ready-for-dev

## Story

As a developer,
I want an OpenAI-compatible provider that works with any OpenAI API endpoint,
So that I can use OpenAI, Azure OpenAI, local vLLM, etc.

## Acceptance Criteria

1. **Given** `traitclaw-openai-compat` crate **When** I configure with `openai_compat("http://localhost:8080/v1", "api-key")` **Then** it sends requests to the specified endpoint
2. **And** supports `/chat/completions` endpoint
3. **And** supports tool calling format
4. **And** supports streaming via SSE
5. **And** reuses reqwest Client for connection pooling

## Tasks / Subtasks

- [ ] Task 1: Create `traitclaw-openai-compat` crate (AC: all)
  - [ ] Cargo.toml with deps: reqwest, serde, serde_json, tokio-stream, traitclaw-core
  - [ ] Module structure: lib.rs, provider.rs, types.rs, stream.rs
- [ ] Task 2: Implement `OpenAiCompatProvider` (AC: 1, 2, 3, 5)
  - [ ] Constructor: `new(base_url, api_key)` with reqwest::Client shared
  - [ ] Implement `Provider::complete()` — POST to /chat/completions
  - [ ] Implement `Provider::model_info()` — configurable ModelInfo
  - [ ] Map CompletionRequest to OpenAI request format
  - [ ] Map OpenAI response to CompletionResponse
- [ ] Task 3: Implement SSE streaming (AC: 4)
  - [ ] Implement `Provider::stream()` — POST with stream=true
  - [ ] Parse SSE events (data: {...}) from response body
  - [ ] Convert to CompletionStream
  - [ ] Handle `[DONE]` sentinel
- [ ] Task 4: Add convenience function (AC: 1)
  - [ ] `pub fn openai_compat(base_url, api_key) -> OpenAiCompatProvider`
- [ ] Task 5: Write tests (AC: all)
  - [ ] Test request serialization matches OpenAI format
  - [ ] Test response deserialization
  - [ ] Test SSE parsing
  - [ ] Mock HTTP for integration tests

## Dev Notes

### Architecture Requirements
- No third-party LLM SDK (AD-9) — direct HTTP via reqwest
- Reuse reqwest::Client (stored in struct) for connection pooling
- Wire format = OpenAI Chat Completions exactly
- reqwest 0.12+ with json and stream features

### Critical Patterns
- SSE format: `data: {"choices":[...]}\n\n`
- `data: [DONE]` signals end of stream
- Auth header: `Authorization: Bearer {api_key}`
- Content-Type: `application/json`

### References
- [Source: _bmad-output/architecture.md#9 Provider Protocol Standard (AD-9)]
- [Source: _bmad-output/project-context.md#Provider crate structure]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log
