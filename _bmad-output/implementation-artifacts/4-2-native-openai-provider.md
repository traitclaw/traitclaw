# Story 4.2: Native OpenAI Provider

Status: ready-for-dev

## Story

As a developer,
I want a native OpenAI provider with full API support,
So that I get the best experience with OpenAI models.

## Acceptance Criteria

1. **Given** `traitclaw-openai` crate with feature `"openai"` **When** I use `openai("gpt-4o")` **Then** it connects to `api.openai.com` with proper authentication
2. **And** supports all GPT-4o/4o-mini models
3. **And** auto-sets `ModelTier` based on model name (gpt-4o=Large, gpt-4o-mini=Medium)
4. **And** supports structured output via `response_format: json_schema`

## Tasks / Subtasks

- [ ] Task 1: Create `traitclaw-openai` crate (AC: all)
  - [ ] Extends traitclaw-openai-compat with native features
  - [ ] Cargo.toml with deps
- [ ] Task 2: Implement `OpenAiProvider` (AC: 1, 2, 3)
  - [ ] Wraps or extends OpenAiCompatProvider
  - [ ] Default base_url: `https://api.openai.com/v1`
  - [ ] API key from constructor or OPENAI_API_KEY env var
  - [ ] Auto-detect ModelTier from model name
- [ ] Task 3: Add structured output support (AC: 4)
  - [ ] Support `response_format: { type: "json_schema", json_schema }` in request
  - [ ] Set `supports_structured: true` in ModelInfo
- [ ] Task 4: Add `openai()` convenience function (AC: 1)
  - [ ] `pub fn openai(model: &str) -> OpenAiProvider`
  - [ ] Read API key from env
- [ ] Task 5: Write tests (AC: all)
  - [ ] Test model tier auto-detection
  - [ ] Test request/response format
  - [ ] Test structured output format

## Dev Notes

### Architecture Requirements
- traitclaw-openai can extend/wrap traitclaw-openai-compat
- Model tier mapping: gpt-4o/gpt-4-turbo=Large, gpt-4o-mini/gpt-3.5-turbo=Medium
- API key: prefer constructor arg, fallback to OPENAI_API_KEY env

### References
- [Source: _bmad-output/architecture.md#Provider Protocol Standard]
- [Source: _bmad-output/epics.md#Story 4.2]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log
