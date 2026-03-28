# Story 3.4: Document Semver, MSRV, and Deprecation Policies

Status: done

## Story

As a **user depending on TraitClaw in production**,
I want clear stability policies,
so that I know when breaking changes can happen and how long deprecated APIs will be supported.

## Acceptance Criteria

1. Semver policy documented: API frozen at 1.0, additions in minor, breaks only in major
2. MSRV policy documented: 1.75, bumped only in minor releases, documented in CHANGELOG
3. Deprecation policy documented: 2 minor versions notice; deprecated in v1.x removed earliest in v2.0
4. Policies are consistent with Architecture Decision AD7

## Tasks / Subtasks

- [ ] Task 1 (AC: #1-4): Add policies to README or `docs/policies.md`
  - [ ] Semver section
  - [ ] MSRV section
  - [ ] Deprecation section
  - [ ] Versioning rules table

## Dev Notes

### Policy Content

**Semver:**
| Change Type | Allowed In |
|-------------|-----------|
| Bug fixes | Patch (1.0.x) |
| New public types/traits/methods | Minor (1.x.0) |
| MSRV bump | Minor (1.x.0) |
| Deprecation notice | Minor (1.x.0) |
| Removal of deprecated items | Major (2.0.0) |
| Trait signature change | Major (2.0.0) |

**MSRV:**
- Current: Rust 1.75
- Bumped only in minor releases
- Always documented in CHANGELOG
- Minimum 6-month lag from Rust stable

**Deprecation:**
- Items deprecated with `#[deprecated(since = "1.x", note = "Use Y instead")]`
- Minimum 2 minor versions before removal
- Removed only in next major version (v2.0)

### Placement Decision

Recommend adding to README under "## Stability" section rather than separate file — keeps everything discoverable in one place.

### References

- [Source: architecture-v1.0.0.md#AD7 — Semver Policy]
- [Source: prd-v1.0.0.md#FR15-FR17]
