# Bilingual Documentation Synchronization Audit

> Canonical language: English. Japanese companion:
> [../ja/bilingual_documentation_synchronization.md](../ja/bilingual_documentation_synchronization.md).

Status: task R-028 audit complete; task R-029 and close-out scopes re-run
complete; 2026-07-02 roadmap synchronization overlay complete; task R-024
implementation overlay complete; R-032A implementation synchronization
complete; the R-032B lint-policy correction is committed and its exact
implementation is the current pre-commit task, with active Checker Task
258B5C next.

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
- The implemented R-032A and planned R-032B synchronize the Medium normal-source proof-label
  `source_drift`, stale R-023 attribution `design_drift`, R-G007 B5C
  `test_gap`, and Low deferred R-G001 public-code `spec_gap`.
- No new `spec_gap`, `test_gap`, `design_drift`, `source_drift`,
  `source_undocumented_behavior`, `test_expectation_drift`,
  `boundary_violation`, or `repo_metadata_conflict` was introduced by this
  audit.

## Pair Checklist

| English canonical document | Japanese companion | Synchronization result |
|---|---|---|
| [00.crate_plan.md](./00.crate_plan.md) | [../ja/00.crate_plan.md](../ja/00.crate_plan.md) | Responsibility, inventory, gap table, completed extensions, historical pre-S-026 four-task record, and effective S-026-docs/S-026-implementation/R-032A-lint-docs/R-032A-implementation/R-032B-lint-docs/R-032B-implementation/B5C seven-task order are synchronized. |
| [declarations.md](./declarations.md) | [../ja/declarations.md](../ja/declarations.md) | Declaration shell kinds, excluded/transparent nodes, visibility, recovery, identity/provenance, and public enum policy are synchronized. |
| [env.md](./env.md) | [../ja/env.md](../ja/env.md) | `SymbolEnv` index families, contribution tracking, invalidation notes, determinism, and public enum policy are synchronized. |
| [imports.md](./imports.md) | [../ja/imports.md](../ja/imports.md) | Import inputs/outputs, two-pass contract, path resolution, alias/export/cycle/unresolved policy, determinism, boundary notes, and public enum policy are synchronized. |
| [labels.md](./labels.md) | [../ja/labels.md](../ja/labels.md) | Existing label policy and planned R-032B API/subtree/origin/error contract are synchronized. |
| [module_summary_reuse.md](./module_summary_reuse.md) | [../ja/module_summary_reuse.md](../ja/module_summary_reuse.md) | R-024 summary reuse scope, known-field identity validation, fallback policy, source-backed agreement, determinism, and public enum policy are synchronized. |
| [names.md](./names.md) | [../ja/names.md](../ja/names.md) | Name-use sites, scope model, namespace-before-symbol lookup, visibility/shadowing, unresolved/ambiguous records, dot-chain finalization, diagnostics, and public enum policy are synchronized. |
| [recovery.md](./recovery.md) | [../ja/recovery.md](../ja/recovery.md) | Recovered syntax stage disposition, boundary rules, and test intent are synchronized. |
| [resolved_ast.md](./resolved_ast.md) | [../ja/resolved_ast.md](../ja/resolved_ast.md) | Top-level `ResolvedAst` shape, stable identity, node/name/label/import tables, recovered shells, provenance, determinism, and public enum policy are synchronized. |
| [source_spec_correspondence.md](./source_spec_correspondence.md) | [../ja/source_spec_correspondence.md](../ja/source_spec_correspondence.md) | Existing audit, implemented R-032A, and planned R-032B repair correspondence are synchronized. |
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

The paired documents freeze the same historical pre-S-026 four-task record,
the same effective seven-task order
S-026 docs -> S-026 implementation -> R-032A lint-policy docs correction ->
R-032A implementation -> R-032B lint-policy docs correction ->
R-032B implementation -> active B5C, and the
same classifications. They name identical R-032A arena API/error
variants/derives, R-032B collector
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
bilingual sync state. S-026 documentation and implementation are complete.
The R-032A lint-policy docs correction and R-032A implementation are complete.
The current dependency sequence is completion and dedicated commit of the
R-032B implementation, then the active B5C consumer, with fresh inventory
after each commit. Add future design files in both language
directories in the same change. Behavior cleanup, public API changes, or new
diagnostics remain outside the completed resolver milestone and require
separate spec/test authority.

The S-026/R-032A dependency overlay is synchronized in EN/JA: both languages
historically classified the same boundary defect and deferred R-032A source
until the separate syntax commits. Those commits, the lint-policy correction,
and R-032A implementation are now complete. Resolver ownership, validation
precedence, and exclusions remain synchronized. R-032B source is now present
while its remaining test/review/verification gates are completed.

## R-032A lint-policy scope correction

EN and JA now classify the omitted mandatory R-026 enum-decision owner as the
same High `design_drift`, with no semantic `spec_gap`. Both freeze the later
implementation to exactly `src/resolved_ast.rs`,
`src/resolved_ast/tests.rs`, and `tests/lint_policy.rs`, where the last file
may receive only the `SurfaceResolvedArenaError` owning-spec decision entry.
Both preserve every runtime/API/test contract and forbidden boundary. This
paired correction is documentation-only, precedes implementation as its own
commit, changes no coverage state, and requires fresh inventory afterward.

## R-032A implementation synchronization

EN and JA now record the same implemented `SurfaceResolvedArena` API, exact
three-field ownership, complete dense same-index lowering, fail-closed
validation precedence, public non-exhaustive error surface, helper payloads,
equivalent-input determinism, and sole R-026 decision. Both record the same
three Rust owners and the same prohibited label/runner/artifact/trace/semantic
scope. At that prerequisite record, R-032B remained pending.
`spec_coverage_audit.md` remains a deliberate
no-op because no active mapping, trace status/count, owner, deferral, or
coverage credit changed.

## R-032B lint-policy scope correction (completed prerequisite record)

EN and JA classify the omitted mandatory R-026 enum-decision owner as the same
High `design_drift`, with no semantic `spec_gap`, `test_gap`, or test-intent
change. Both freeze later R-032B implementation to exactly
`crates/mizar-resolve/src/labels.rs`,
`crates/mizar-resolve/src/labels/tests.rs`, and
`crates/mizar-resolve/tests/lint_policy.rs`; the last file may receive only
the sole `ProofLabelSourceCollectionError` owning-spec decision with
`spec_name: "labels.md"`.

The completed docs-only correction spans exactly 31 design files: 16 resolver,
eight checker, six `mizar-test`, and one global ledger. It preserves every
semantic/API/test contract and changes no source, fixture, sidecar,
expectation, trace status/count, or Cargo metadata.
`spec_coverage_audit.md` is a deliberate no-op because coverage ownership and
status do not change. The independent specification, test/scope, and
source/documentation consistency reviews report **NO FINDINGS**, and the
docs-only verification/count/hash gates PASS. Independent final read-only
quality also reports **NO FINDINGS**; all nine hard gates PASS with no cap at
valid `100/100` (`20/20/15/15/10/10/5/5`). At that pre-commit record, only
task-only staging/cached-diff review, commit, and post-commit
invariant/fresh-inventory gates remained pending. They subsequently completed
in correction commit `f1cf0a5d15f2db51176e9e91a4f5a6447a88ad7a` and its
fresh inventory.

## R-032B implementation synchronization

EN and JA now record the same current pre-commit implementation: exact
three-Rust-file ownership; public collector, collection accessors, and
non-exhaustive error table; R-032A validation; AST/arena-only borrowing;
default-deny direct traversal; module-global ordinals; proof scopes;
completion boundaries; simple citations; structural origins; and
`proof-step-v1` identity. Both preserve the private future B5C consumer and
exclude checker handoff, semantic phases, fixtures, expectations, sidecars,
trace state, public diagnostics, and Cargo metadata.

Both languages record the initial High/Medium plus two fresh Medium test gaps,
the Medium third-child implementation defect, and the two Medium unauthorized
`Default` / `From` findings as fixed. They record **NO FINDINGS** for the
preimplementation specification review and the final fresh test-sufficiency,
implementation, and source/documentation rereviews. Focused, crate,
formatting, workspace Clippy/test, diff, CLI, test-list, production, and exact
20-file scope gates PASS. Independent final quality reports **NO FINDINGS**;
all nine hard gates PASS with no score cap at valid `100/100`
(`20/20/15/15/10/10/5/5`). Only task-only restaging/cached-diff review,
commit, and post-commit invariant/fresh inventory remain pending. R-G007
remains open and active B5C remains next.
`spec_coverage_audit.md` is still a deliberate no-op because no active
mapping, trace status/count, owner, deferral, or coverage credit changes.
