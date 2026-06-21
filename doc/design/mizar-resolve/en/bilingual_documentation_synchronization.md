# Bilingual Documentation Synchronization Audit

> Canonical language: English. Japanese companion:
> [../ja/bilingual_documentation_synchronization.md](../ja/bilingual_documentation_synchronization.md).

Status: task R-028 audit complete; task R-029 scope re-run complete.

## Scope

This audit compares each English canonical design document under
`doc/design/mizar-resolve/en/` with its Japanese companion under
`doc/design/mizar-resolve/ja/`. It checks paired filenames, API lists, task
status, gap/deferred classifications, behavior promises, boundary statements,
terminology, and links relevant to the `mizar-resolve` task stream.

The audit covers the completed non-deferred resolver work through R-029 and the
R-024 `external_dependency_gap` deferral record. It does not replace the
source/spec correspondence audit in
[source_spec_correspondence.md](./source_spec_correspondence.md), and it does
not change `doc/spec`, `.miz` sources, or expectation sidecars.

## Result

- Every English design file currently has a same-named Japanese companion, and
  this audit adds the same paired file in both language directories.
- No remaining English/Japanese mismatch was found in public resolver API
  families, public enum forward-compatibility decisions, task completion
  states, deferred/external dependency records, or milestone handoff wording.
- Task status is synchronized as: R-001 to R-023 complete, R-024 explicitly
  deferred as R-G003 `external_dependency_gap`, and R-025 to R-029 complete.
- Existing follow-up classifications remain synchronized: R-G001
  `spec_gap`, R-G002 `test_gap`, R-G003 `external_dependency_gap` / deferred,
  R-G004 `boundary_violation` risk, R-G005 resolved `design_drift`, R-G006
  `external_dependency_gap`, and R-G007 `test_gap` as the current concrete
  refinement of R-G002.
- No new `spec_gap`, `test_gap`, `design_drift`, `source_drift`,
  `source_undocumented_behavior`, `test_expectation_drift`,
  `boundary_violation`, or `repo_metadata_conflict` was introduced by this
  audit.

## Pair Checklist

| English canonical document | Japanese companion | Synchronization result |
|---|---|---|
| [00.crate_plan.md](./00.crate_plan.md) | [../ja/00.crate_plan.md](../ja/00.crate_plan.md) | Responsibility, spec/test inventory, design/source inventory, gap table, R-024 deferral, R-027 audit result, R-028 audit result, R-029 refactor result, and close-out handoff are synchronized. |
| [declarations.md](./declarations.md) | [../ja/declarations.md](../ja/declarations.md) | Declaration shell kinds, excluded/transparent nodes, visibility, recovery, identity/provenance, and public enum policy are synchronized. |
| [env.md](./env.md) | [../ja/env.md](../ja/env.md) | `SymbolEnv` index families, contribution tracking, invalidation notes, determinism, and public enum policy are synchronized. |
| [imports.md](./imports.md) | [../ja/imports.md](../ja/imports.md) | Import inputs/outputs, two-pass contract, path resolution, alias/export/cycle/unresolved policy, determinism, boundary notes, and public enum policy are synchronized. |
| [labels.md](./labels.md) | [../ja/labels.md](../ja/labels.md) | Label scope families, proof-block scope, forward-reference policy, citation lookup, origin paths, diagnostics/recovery, determinism, and public enum policy are synchronized. |
| [names.md](./names.md) | [../ja/names.md](../ja/names.md) | Name-use sites, scope model, namespace-before-symbol lookup, visibility/shadowing, unresolved/ambiguous records, dot-chain finalization, diagnostics, and public enum policy are synchronized. |
| [recovery.md](./recovery.md) | [../ja/recovery.md](../ja/recovery.md) | Recovered syntax stage disposition, boundary rules, and test intent are synchronized. |
| [resolved_ast.md](./resolved_ast.md) | [../ja/resolved_ast.md](../ja/resolved_ast.md) | Top-level `ResolvedAst` shape, stable identity, node/name/label/import tables, recovered shells, provenance, determinism, and public enum policy are synchronized. |
| [source_spec_correspondence.md](./source_spec_correspondence.md) | [../ja/source_spec_correspondence.md](../ja/source_spec_correspondence.md) | R-027 public API, behavior-boundary, task-requirement, and follow-up records are synchronized, including the relationship between R-G002 and R-G007; the R-029 moved-source scope re-run is also synchronized. |
| [symbols.md](./symbols.md) | [../ja/symbols.md](../ja/symbols.md) | Symbol-bearing shells, collection order, identities/origins, signatures, duplicates/overloads, visibility/export/summary policy, dependency relations, recovery/diagnostics, determinism, and public enum policy are synchronized. |
| [todo.md](./todo.md) | [../ja/todo.md](../ja/todo.md) | Ordered task states, deferral notes, recommended verification, and close-out handoff wording are synchronized. |
| [bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md) | [../ja/bilingual_documentation_synchronization.md](../ja/bilingual_documentation_synchronization.md) | This R-028 audit and R-029 scope re-run are recorded in both languages with the same scope, result, pair checklist, and handoff notes. |
| [module_boundary_refactor.md](./module_boundary_refactor.md) | [../ja/module_boundary_refactor.md](../ja/module_boundary_refactor.md) | R-029 source-layout audit, private helper/test split list, re-run audit notes, and verification requirements are synchronized. |

## Handoff

Crate-wide close-out should treat this audit as the baseline bilingual sync
state. If the close-out report adds new design files, add both language
companions together. Behavior cleanup, public API changes, or new diagnostics
remain outside the completed refactor gate and require separate spec/test
authority.
