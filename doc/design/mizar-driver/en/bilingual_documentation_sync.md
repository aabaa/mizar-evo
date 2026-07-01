# Bilingual documentation sync audit

> Canonical language: English. Japanese companion:
> [../ja/bilingual_documentation_sync.md](../ja/bilingual_documentation_sync.md).

Status: completed by task D-019; refreshed by task D-020 for the
architecture-22 follow-up audit pair and by task D-021 for the module-boundary
refactor gate pair.

## Scope

This audit compares the English canonical design documents under
`doc/design/mizar-driver/en/` with their Japanese companions under
`doc/design/mizar-driver/ja/`.

The audit checks:

- every English driver design document has a Japanese companion with the same
  file name;
- every Japanese driver design document has a matching English canonical file;
- section structure, task records, gap classifications, and ownership
  boundaries are synchronized at the level required for downstream engineering
  work;
- newly added audit documents are paired in both languages.

Localized headings and wording are allowed to differ when the technical content
is equivalent. This audit does not change source behavior or language
semantics.

## Result

- No unresolved blocking or high EN/JA documentation drift was found.
- The English and Japanese file sets are paired one-to-one for the current
  driver design corpus, including this audit document.
- Task records through D-021, the known `DRIVER-G-*` classifications,
  `external_dependency_gap`, `deferred`, and report-only
  `repo_metadata_conflict` records are present in both languages.
- The D-018 source/spec correspondence audit remains synchronized and continues
  to report no unresolved blocking, high, or medium source/spec drift.
- The D-020 architecture-22 follow-up audit is now paired in both languages and
  reports no unresolved blocking/high drift for implemented driver seams.
- The D-021 module-boundary refactor gate is paired in both languages and
  records a private-helper-only source split with unchanged public APIs.

## Pair Coverage

| English canonical file | Japanese companion | Sync result |
|---|---|---|
| `00.crate_plan.md` | `00.crate_plan.md` | Paired. Responsibility, preflight, gap table, task decomposition through D-021, exit criteria, and known deferred/external gaps are aligned. |
| `todo.md` | `todo.md` | Paired. Module ownership, prerequisites, ordered tasks, D-018 through D-021 completion, source-path table, verification notes, and non-owner boundaries are aligned. |
| `request.md` | `request.md` | Paired. Request/session data model, currentness lanes, snapshot capture, publication suppression, supersession, error handling, tests, and public enum policy are aligned. |
| `registry.md` | `registry.md` | Paired. Phase service table, readiness gaps, registration rules, cache-key purity, salsa boundary, scheduler/cache seams, diagnostics/artifact/LSP boundaries, tests, and public enum policy are aligned. |
| `driver.md` | `driver.md` | Paired. Driver front-door ownership, public API, submit flow, scheduler boundary, cancellation, artifact/diagnostics boundaries, tests, and public enum policy are aligned. |
| `events.md` | `events.md` | Paired. Protocol-agnostic event shape, freshness/suppression, deterministic ordering, diagnostics/artifact events, consumer rules, tests, and public enum policy are aligned. |
| `cli.md` | `cli.md` | Paired. Batch command surface, request mapping, progress/diagnostics rendering, exit codes, owner-gap handling, tests, and public enum policy are aligned. |
| `frontend_adapter.md` | `frontend_adapter.md` | Paired. D-006 `SourceFrontend` readiness inventory and `external_dependency_gap` decision are aligned. |
| `source_spec_correspondence.md` | `source_spec_correspondence.md` | Paired. D-018 public API, public method surface, promised behavior, gap records, and docs-only verification path are aligned. |
| `bilingual_documentation_sync.md` | `bilingual_documentation_sync.md` | Paired by this task. |
| `architecture_22_follow_up_audit.md` | `architecture_22_follow_up_audit.md` | Paired by D-020. Architecture-22 query-boundary, stale-output, diagnostics, artifact-publication, and determinism classifications are aligned. |
| `module_boundary_refactor_gate.md` | `module_boundary_refactor_gate.md` | Paired by D-021. Private helper split, source-path table updates, owner-boundary preservation, and verification requirements are aligned. |

## Drift And Follow-up Records

No new blocking/high bilingual documentation drift was found.

Existing classified records remain unchanged:

- `DRIVER-G-001` and `DRIVER-G-009` remain report-only
  `repo_metadata_conflict` items for artifact metadata; this task does not
  repair `mizar-artifact` metadata.
- `DRIVER-G-010` through `DRIVER-G-014` remain the current owner-seam
  `external_dependency_gap` or `deferred` records for frontend, scheduler
  dispatch, watch/LSP bridge, semantic/proof/artifact adapters, and document
  extraction.
- Full clean/incremental/parallel equivalence with real producer/cache/artifact
  and proof seams remains deferred until the corresponding owner seams exist.

## Verification

This audit's D-019 and D-020 updates were documentation-only. Its D-021 refresh
is part of a Rust source split, so it follows the D-021 verification plan.
Required local documentation checks are:

- `git diff --check`
- `git diff --cached --check` after staging the task-related paths

For the D-021 Rust source change, also run the crate-local Rust checks named in
`module_boundary_refactor_gate.md`; final crate closeout runs the full
repository hard gates.
