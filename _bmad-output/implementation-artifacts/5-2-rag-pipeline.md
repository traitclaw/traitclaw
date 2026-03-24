# Story 5.2: RAG Pipeline

Status: ready-for-dev

## Story

As a developer,
I want a RAG pipeline with Retriever trait,
So that agents can ground responses in external knowledge.

## Acceptance Criteria

1. **Given** `traitclaw-rag` crate with feature `"rag"` **When** I implement a custom `Retriever` **Then** retrieved documents are injected into agent context
2. **And** `GroundingStrategy` controls how documents are used
3. **And** built-in `KeywordRetriever` provides BM25 search

## Tasks / Subtasks

- [ ] Task 1: Create `traitclaw-rag` crate
- [ ] Task 2: Define `Retriever` trait
- [ ] Task 3: Define `GroundingStrategy` trait
- [ ] Task 4: Implement `KeywordRetriever` with BM25
- [ ] Task 5: Integrate retriever into Agent runtime
- [ ] Task 6: Write tests

## Dev Notes

### Architecture Requirements
- Retriever trait: `async fn retrieve(&self, query: &str, limit: usize) -> Result<Vec<Document>>`
- GroundingStrategy: controls document injection (prepend, context window, etc.)
- BM25 search for keyword retriever (no vector DB dependency)

### References
- [Source: _bmad-output/architecture.md#6 Optional - rag]
- [Source: _bmad-output/epics.md#Story 5.2]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log
