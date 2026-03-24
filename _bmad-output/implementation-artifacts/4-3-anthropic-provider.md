# Story 4.3: Anthropic Provider

Status: ready-for-dev

## Story

As a developer,
I want an Anthropic provider for Claude models,
So that I can use Claude with proper prompt caching support.

## Acceptance Criteria

1. **Given** `traitclaw-anthropic` crate with feature `"anthropic"` **When** I use `anthropic("claude-sonnet-4-20250514")` **Then** it sends requests to Anthropic API format (different from OpenAI)
2. **And** supports tool use in Anthropic format
3. **And** supports streaming
4. **And** auto-sets ModelTier (opus=Large, sonnet=Medium, haiku=Small)

## Tasks / Subtasks

- [ ] Task 1: Create `traitclaw-anthropic` crate (AC: all)
  - [ ] Cargo.toml, module structure
- [ ] Task 2: Implement `AnthropicProvider` (AC: 1, 4)
  - [ ] Transform CompletionRequest to Anthropic format
  - [ ] Transform Anthropic response to CompletionResponse
  - [ ] ~20 lines transform: system field, input_tokens → prompt_tokens
  - [ ] Auto-detect ModelTier from model name
- [ ] Task 3: Implement tool use support (AC: 2)
  - [ ] Map tool schemas to Anthropic tool format
  - [ ] Map tool use responses
- [ ] Task 4: Implement streaming (AC: 3)
  - [ ] SSE parsing for Anthropic's streaming format
  - [ ] Event types: message_start, content_block_delta, message_delta, message_stop
- [ ] Task 5: Write tests (AC: all)
  - [ ] Test request/response transformation
  - [ ] Test model tier detection
  - [ ] Test streaming event parsing

## Dev Notes

### Architecture Requirements
- Anthropic API differs from OpenAI: `system` is top-level field, not a message
- Token mapping: `input_tokens` → `prompt_tokens`, `output_tokens` → `completion_tokens`
- Streaming format: SSE with different event types than OpenAI
- Keep transform layer thin (~20 lines)

### Anthropic API Differences
- System prompt: separate `system` field, not in messages array
- Tool results: `tool_result` content block, not separate message role
- Model header: `anthropic-version: 2023-06-01`
- Auth: `x-api-key` header (not Bearer token)

### References
- [Source: _bmad-output/architecture.md#Provider Protocol Standard - Anthropic transform]
- [Source: _bmad-output/epics.md#Story 4.3]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log
