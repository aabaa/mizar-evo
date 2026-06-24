# Bilingual Documentation Sync Audit: mizar-vc

> Canonical language: English. Japanese companion:
> [../ja/bilingual_sync_audit.md](../ja/bilingual_sync_audit.md).

Task 19 audits the `mizar-vc` design documentation pairs after the
source/spec correspondence audit. It changes no Rust source, `.miz` fixtures,
expectations, language specifications, traceability metadata, runner support,
or downstream ATP/kernel/proof/cache integration.

## Scope And Method

The audit covers every current Markdown document under
`doc/design/mizar-vc/en/` and its companion under `doc/design/mizar-vc/ja/`.
For each pair, the review checked:

- same filename in both language directories;
- canonical/companion link at the top of each document;
- substantive meaning of module responsibility, inputs/outputs, behavior
  rules, status and policy semantics, dependency and fingerprint rules,
  planned tests, public enum policy tables, audit inventories, task ledger
  summaries, todo task wording, and follow-up/deferred classifications;
- preservation of known `external_dependency_gap` and `deferred` records
  rather than silently resolving or weakening them.

The Japanese companion may use idiomatic translation and may retain Rust
identifiers, phase names, and task names in English. The synchronization rule is
semantic: the companion must not omit, weaken, or add normative meaning relative
to the English canonical document.

Result: all current document pairs exist and are semantically synchronized. No
meaning-changing bilingual drift, missing companion, stale status, or
`repo_metadata_conflict` was observed. Task 18's self-hash was still pending in
the ledger by design and is backfilled by this task.

## Pair Inventory

| Document | Synchronized content checked | Result |
|---|---|---|
| `00.crate_plan.md` | Responsibility, out-of-scope boundaries, authority order, known gaps/drift, task decomposition, hard gates, and verification expectations. | Synchronized. |
| `vc_ir.md` | Snapshot-local `VcId`, seed accounting, generated formula ownership, local context, premises, statuses, anchors, rendering, planned tests, and public enum policy. | Synchronized. |
| `generator.md` | Explicit-payload generation scope, unavailable registration/algorithm payload boundaries, local context, controlled unfolding, normalization handoff, task slices, planned tests, and public enum policy. | Synchronized. |
| `discharge.md` | Deterministic pre-ATP scope, supported classes, limits, evidence/explanation model, status interaction, no-erase ATP boundary, planned tests, and public enum policy. | Synchronized. |
| `dependency_slice.md` | Conservative slice inputs/outputs, dependency entry classes, unknown coverage, reusable fingerprint contract, planned tests, and public enum policy. | Synchronized. |
| `source_spec_audit.md` | Public module exports, public surface inventory, cross-module evidence, and classified external/deferred follow-ups. | Synchronized. |
| `bilingual_sync_audit.md` | Audit scope, method, pair inventory, classification, and Task 19 sync edits. | Synchronized by this paired Task 19 document. |
| `task_ledger.md` | Task status, commit hashes available before Task 19, review outcomes, verification summaries, deferred notes, and handoff prompts. | Synchronized after this task backfills the Task 18 hash and records Task 19. |
| `todo.md` | Ordered task list, completed tasks, remaining Task 20-22 and closeout scope, recommended verification, and notes. | Synchronized after this task marks Task 19 complete. |

## Classification

Task 19 records no new `spec_gap`, `test_gap`, `design_drift`,
`source_drift`, `source_undocumented_behavior`, `test_expectation_drift`,
`boundary_violation`, or `repo_metadata_conflict`.

Existing classified records remain:

- `external_dependency_gap`: active `proof_verification` runner support and
  source-to-core / source-to-VC extraction seams are absent from `mizar-test`;
  Task 15 records the deferred corpus obligation.
- `external_dependency_gap`: `mizar-atp`, `mizar-kernel`, `mizar-proof`, and
  `mizar-cache` are not active workspace consumers, so ATP translation,
  certificate acceptance, proof policy, cache lookup/reuse, and artifact
  persistence remain downstream.
- `external_dependency_gap`: upstream explicit payloads remain incomplete for
  some registration/redefinition/reduction details, call-precondition, branch,
  match, range-loop, collection-loop, term-only termination, partial
  termination, Pick non-emptiness, ghost-erasure, and complete trace families.
- `deferred`: Task 20 owns the remaining architecture-22 cross-edit reuse
  identity work for anchors, canonical VC/context fingerprints, dependency
  fingerprints, compatible verifier policy, and witness/discharge hashes.
- `deferred`: Task 21 owns the architecture-22 follow-up audit, Task 22 owns
  the module-boundary refactor gate, and closeout owns final quality review and
  crate-exit reporting.

## Task 19 Sync Edits

This task adds the paired bilingual sync audit documents, backfills Task 18's
commit hash in the paired ledgers, records the Task 19 review/verification
outcome in the paired ledgers, and marks Task 19 complete in the paired todos.

No other paired content needed synchronization.
