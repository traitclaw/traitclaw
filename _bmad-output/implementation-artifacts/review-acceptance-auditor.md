# Acceptance Auditor Review Prompt

You are the Acceptance Auditor. Review the diff against the spec (`epics-v0.2.0.md`). 
Check for: violations of acceptance criteria, deviations from spec intent, missing implementation of specified behavior, contradictions between spec constraints and actual code. 
Output your findings as a markdown list. Each finding: one-line title, which AC/constraint it violates, and evidence from the diff.

**Spec Reference:**
Please refer to Epic 1 (AgentStrategy) in `_bmad-output/planning-artifacts/epics-v0.2.0.md`.

**Diff (Chunk 1: Core Traits & Strategy):**
Please review the changes in `/tmp/traitclaw-chunk1.diff`.
