# mizar-test Bilingual Documentation Sync Audit

> Canonical language: English. Japanese companion:
> [../ja/bilingual_sync_audit.md](../ja/bilingual_sync_audit.md).

## Scope

Task 13 established the paired-file sync baseline for every English canonical
document under `doc/design/mizar-test/en/` and its Japanese companion under
`doc/design/mizar-test/ja/`.

Task 15 re-runs that audit after the task-14 architecture-22 matrix metadata
work. The follow-up covers task status through task 15, architecture-22
scenario ids and equivalence classes, planned/active gating, traceability
records, consumer-runner pacing, determinism coverage, and module-spec
contracts.

Task 21 re-runs the relevant paired-document check for the kernel F7
corrected-path soundness vocabulary. The follow-up covers `fail_soundness.md`,
`layout.md`, `expectation_schema.md`, `todo.md`, `00.crate_plan.md`, and the
kernel `soundness_argument.md` references updated in the same change.

Task 248 adds and reviews the paired `module_boundary_audit.md` documents and
synchronizes the runner-refactor task decomposition, preservation matrix,
source inventory, and backlog state in both languages.

This audit does not change language behavior, corpus expectations, existing
`.miz` fixtures, or expectation meaning.

## Method

The task 13 baseline checked:

- the EN/JA file sets are paired one-to-one;
- each companion keeps the same task status and completion notes;
- each module spec keeps the same major section structure and content intent;
- public enum policy tables list the same enum inventories and decisions;
- consumer-runner `prepared/implemented` and `paced/open` ledger entries match.

The task 15 follow-up checked:

- the task-14 updates in `expectation_schema.md`, `harness.md`,
  `traceability.md`, `todo.md`, and `00.crate_plan.md` have matching Japanese
  companion content;
- the task-14 scenario ids and equivalence classes match the properties listed
  in architecture 20 and architecture 22;
- all task-14 rows remain `planned`, and `active` remains gated off because no
  prepared consumer runner was confirmed for clean/incremental/parallel or
  cache-race execution;
- the metadata-only `architecture22_matrix_001` anchor and
  `spec.en.architecture_22.regression_matrix.metadata` trace row remain
  documentation-only coverage and do not fabricate execution;
- the source/spec audit ledger in `00.crate_plan.md` records the remaining
  architecture-22 matrix work as a follow-up `test_gap` with a consumer-paced
  external dependency, not as a `spec_gap` or `repo_metadata_conflict`;
- task 10 remains consumer-paced with no newly prepared runner increment.

The task 21 follow-up checked:

- the corrected `soundness.certificate.*` keys, failure categories, rejection
  reasons, domains, and phases match between `fail_soundness.md` companions;
- certificate layout and expectation examples use the implemented
  `tests/certificates/` corpus and corrected SAT-refutation vocabulary;
- `todo.md` and `00.crate_plan.md` both mark task 21 complete and record that
  `doc/design/spec_coverage_audit.md` remains unchanged;
- the paired kernel `soundness_argument.md` files mark F7 resolved.

The task 248 follow-up checked:

- current runner/test counts and ownership facts match between companions;
- the target private source layout, dependency direction, Tasks 249-264, and
  move-only prohibitions match;
- the declaration-symbol integration-test owner remains `tests/metadata.rs`
  and no empty private test owner is invented;
- `doc/design/spec_coverage_audit.md` remains unchanged because no coverage,
  traceability, owner, or deferred-status mapping changes.

## Pair Status

| Document | Synchronized content |
|---|---|
| `00.crate_plan.md` | Task plan, source inventory, task 8-15 and task-21 completion status, architecture-22 matrix audit result, source-derived bridge boundary, and corrected soundness vocabulary audit result. |
| `README.md` | Module index and crate boundary summary. |
| `expectation_schema.md` | Sidecar schema, origin metadata, corrected certificate soundness fields, and public enum policy. |
| `fail_soundness.md` | Required soundness cases, corrected kernel rejection vocabulary, failure contract, and validation rules. |
| `harness.md` | Public API, runner pacing ledger, determinism requirements, architecture-22 matrix reporting, and tests. |
| `module_boundary_audit.md` | Task-248 runner ownership inventory, dependency map, target layout, preservation matrix, and ordered move tasks. |
| `layout.md` | Directory layout, naming rules, corrected certificate rejection reasons, expected-result files, and public enum cross-reference. |
| `minimal_crate.md` | Minimal metadata-only crate scope, CLI behavior, and verification expectations. |
| `miz_corpus.md` | Corpus classes, size/review policy, provenance, stress, fuzz, and property rules. |
| `snapshot.md` | Snapshot records, baseline/update policy, determinism helpers, and public enum policy. |
| `staged_model.md` | Stage ids, admission rules, prerequisite policy, and public enum policy. |
| `todo.md` | Ordered tasks, task 8-15 and task-21 completion notes, and remaining architecture-22 follow-up boundary. |
| `traceability.md` | Manifest schema, coverage/status rules, prerequisite validation, architecture-22 matrix summary, and public enum policy. |
| `bilingual_sync_audit.md` | Task-13 baseline, task-15 follow-up audit, and Japanese companion. |

## Result

No bilingual documentation drift remains for the current task scope. The
Japanese companions are synchronized with the English canonical documents for
the mizar-test design surface through Task 248's runner module-boundary gate.

The task-15 source/spec follow-up found no blocking `spec_gap`, accepted
`repo_metadata_conflict`, language behavior change, or need to change existing
expectation semantics. Remaining architecture-22 matrix execution work is
recorded as consumer-paced follow-up work because the task-14 support is
metadata/reporting-only.

Remaining open work is intentionally outside task 15:

- the source-derived semantic bridge requested after task 15;
- active architecture-22 matrix execution in consumer crates, after prepared
  source-derived clean/incremental/parallel/cache-race runner increments exist;
- existing non-bilingual source/design watches recorded in `00.crate_plan.md`.

## Verification

Task 15 used documentation-focused checks:

- paired EN/JA file-set listing for `doc/design/mizar-test`;
- heading-structure review for every paired document;
- task-status review confirming task 14 and task 15 are complete;
- architecture-20/22 scenario coverage review for the 18 task-14 registry rows;
- source/spec audit ledger review in `00.crate_plan.md`;
- `git diff --check`.

Task 21 additionally uses `cargo test -p mizar-test` and `git diff --check`
before commit. Crate-local Rust checks do not prove whole-document bilingual
semantic equivalence. The remaining risk is the normal manual-review risk of
this audit: wording may still diverge at sentence level even when file sets,
major sections, task status, recorded inventories, and corrected-vocabulary
metadata match.

Task 248 additionally compares the paired module-boundary headings, tables,
task ids, preservation invariants, and backlog status before the full required
workspace verification.

## Core Task 31 / Task-10 Pair Recheck

The later Core Task-31 consumer increment rechecks the paired
`00.crate_plan.md`, `expectation_schema.md`, `harness.md`,
`module_boundary_audit.md`, `snapshot.md`, `todo.md`, and `traceability.md`.
Both languages describe the same singular Task-180 CoreIr baseline, public
snapshot-failure result/reporting, 17-path/19,803-line ownership state,
403/368 and 236/224 coverage counts, exact-only credit, and broad deferred
boundary. No bilingual `design_drift` remains in this scope.
