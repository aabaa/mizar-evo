# Bilingual Documentation Synchronization Audit

> Canonical language: English. Japanese companion:
> [../ja/bilingual_documentation_synchronization.md](../ja/bilingual_documentation_synchronization.md).

Status: task R-028 audit complete; task R-029 and close-out scopes re-run
complete; 2026-07-02 roadmap synchronization overlay complete; task R-024
implementation overlay complete; planned R-032A/R-032B / Checker Task 258B5C lower
prerequisite pair recheck complete.

## Scope

This audit compares each English canonical design document under
`doc/design/mizar-resolve/en/` with its Japanese companion under
`doc/design/mizar-resolve/ja/`. It checks paired filenames, API lists, task
status, gap/deferred classifications, behavior promises, boundary statements,
terminology, and links relevant to the `mizar-resolve` task stream.

The audit covers the completed non-deferred resolver work through close-out, the
original R-024 `external_dependency_gap` deferral record, the 2026-07-02
roadmap synchronization update that marked the artifact-side blocker resolved,
and the R-024 resolver-side implementation overlay. It does not replace the source/spec correspondence audit in
[source_spec_correspondence.md](./source_spec_correspondence.md), and it does
not change `doc/spec`, `.miz` sources, or expectation sidecars.

## Result

- Every English design file currently has a same-named Japanese companion, and
  this audit adds the same paired file in both language directories.
- No remaining English/Japanese mismatch was found in public resolver API
  families, public enum forward-compatibility decisions, task completion
  states, deferred/external dependency records, or milestone handoff wording.
- Task status is synchronized as: R-001 to R-029 complete, including R-024
  after the resolved artifact-side `external_dependency_gap`.
- Existing follow-up classifications remain synchronized: R-G001
  `spec_gap`, R-G002 `test_gap`, R-G003 resolved by R-024, R-G004
  `boundary_violation` risk, R-G005 resolved `design_drift`, R-G006
  `external_dependency_gap`, and R-G007 `test_gap` as the current concrete
  refinement of R-G002.
- The R-032A/R-032B pair synchronizes the Medium normal-source proof-label
  `source_drift`, stale R-023 attribution `design_drift`, R-G007 B5C
  `test_gap`, and Low deferred R-G001 public-code `spec_gap`.
- No new `spec_gap`, `test_gap`, `design_drift`, `source_drift`,
  `source_undocumented_behavior`, `test_expectation_drift`,
  `boundary_violation`, or `repo_metadata_conflict` was introduced by this
  audit.

## Pair Checklist

| English canonical document | Japanese companion | Synchronization result |
|---|---|---|
| [00.crate_plan.md](./00.crate_plan.md) | [../ja/00.crate_plan.md](../ja/00.crate_plan.md) | Responsibility, inventory, gap table, completed extensions, and planned four-step B5C/R-032A/R-032B contract are synchronized. |
| [declarations.md](./declarations.md) | [../ja/declarations.md](../ja/declarations.md) | Declaration shell kinds, excluded/transparent nodes, visibility, recovery, identity/provenance, and public enum policy are synchronized. |
| [env.md](./env.md) | [../ja/env.md](../ja/env.md) | `SymbolEnv` index families, contribution tracking, invalidation notes, determinism, and public enum policy are synchronized. |
| [imports.md](./imports.md) | [../ja/imports.md](../ja/imports.md) | Import inputs/outputs, two-pass contract, path resolution, alias/export/cycle/unresolved policy, determinism, boundary notes, and public enum policy are synchronized. |
| [labels.md](./labels.md) | [../ja/labels.md](../ja/labels.md) | Existing label policy and planned R-032B API/subtree/origin/error contract are synchronized. |
| [module_summary_reuse.md](./module_summary_reuse.md) | [../ja/module_summary_reuse.md](../ja/module_summary_reuse.md) | R-024 summary reuse scope, known-field identity validation, fallback policy, source-backed agreement, determinism, and public enum policy are synchronized. |
| [names.md](./names.md) | [../ja/names.md](../ja/names.md) | Name-use sites, scope model, namespace-before-symbol lookup, visibility/shadowing, unresolved/ambiguous records, dot-chain finalization, diagnostics, and public enum policy are synchronized. |
| [recovery.md](./recovery.md) | [../ja/recovery.md](../ja/recovery.md) | Recovered syntax stage disposition, boundary rules, and test intent are synchronized. |
| [resolved_ast.md](./resolved_ast.md) | [../ja/resolved_ast.md](../ja/resolved_ast.md) | Top-level `ResolvedAst` shape, stable identity, node/name/label/import tables, recovered shells, provenance, determinism, and public enum policy are synchronized. |
| [source_spec_correspondence.md](./source_spec_correspondence.md) | [../ja/source_spec_correspondence.md](../ja/source_spec_correspondence.md) | Existing audit and planned R-032A/R-032B repair correspondence are synchronized. |
| [symbols.md](./symbols.md) | [../ja/symbols.md](../ja/symbols.md) | Symbol-bearing shells, collection order, identities/origins, signatures, duplicates/overloads, visibility/export/summary policy, dependency relations, recovery/diagnostics, determinism, and public enum policy are synchronized. |
| [todo.md](./todo.md) | [../ja/todo.md](../ja/todo.md) | Ordered task states and split R-032A/R-032B ownership/dependencies are synchronized. |
| [bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md) | [../ja/bilingual_documentation_synchronization.md](../ja/bilingual_documentation_synchronization.md) | This R-028 audit, R-029 scope re-run, close-out re-run, and roadmap synchronization overlay are recorded in both languages with the same scope, result, pair checklist, and handoff notes. |
| [module_boundary_refactor.md](./module_boundary_refactor.md) | [../ja/module_boundary_refactor.md](../ja/module_boundary_refactor.md) | R-029 source-layout audit, private helper/test split list, re-run audit notes, verification requirements, and bounded R-032 ownership recheck are synchronized. |
| [crate_exit_report.md](./crate_exit_report.md) | [../ja/crate_exit_report.md](../ja/crate_exit_report.md) | Close-out status, quality score, hard gates, deferred items, human-review surface, verification, task commits, next-task handoff, and planned R-032 extension are synchronized. |

## R-031 Pair Recheck

R-031 rechecks the paired plan, TODO, symbols design, source correspondence,
and close-out extension. Both languages name the same ordinary-functor-only
syntactic key, appended `SameSignatureDefinitionConflict` diagnostic and
definition variants, exact `same_signature_definition_conflict` SymbolEnv
snapshot spelling, exact declaration-symbol detail key, mixed-group priority,
candidate/range/order behavior, sidecar/trace transition, coverage impact, and
forbidden semantic/public-code/phase boundaries. No bilingual drift remains in
the R-031 extension.

## R-032A / R-032B Pair Recheck

The paired documents freeze the same four-step order and classifications. They
name identical R-032A arena API/error variants/derives, R-032B collector
API/error variants/derives, `u32` overflow policy, exact file ownership,
collector lifetime/storage/module rule, theorem-root and module-global ordinal
walk, completion rules, exact length-framed `proof-step-v1` grammar, B5C
origin paths, supported subtree and exclusions, cross-theorem direction,
own-proof boundary, mutation matrix, downstream private key, and forbidden
changes. Both include R-032A resolution-state/reference-key mismatch variants.
Both also reproduce the same exhaustive default-deny direct Surface edge table
semantics and the positive-per-edge, negative mutation, mixed-list, and
representative all-other test obligations. The upper hierarchy is identically
`Root` -> `CompilationUnit` -> `ItemList` -> direct theorem, with matching
missing/additional/wrong/relocated/wrapped tests.
R-032A arena origin `[surface_id]` and R-032B richer table origins are
intentionally distinct in both languages. Neither language retains the
rejected callback/unmapped contract.

## Handoff

Post-close-out resolver updates should treat this audit as the baseline
bilingual sync state. The next resolver-owned dependencies are R-032A then
R-032B, each followed by fresh inventory, before the active B5C consumer. Add
future design files in both language directories in the same change. Behavior
cleanup, public API changes, or new diagnostics remain outside the completed
resolver milestone and require separate spec/test authority.
