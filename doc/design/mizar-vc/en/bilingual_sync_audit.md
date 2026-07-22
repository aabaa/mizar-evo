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
`repo_metadata_conflict` was observed. Task 22's self-hash is backfilled by the
closeout task; the closeout self-hash is recorded in the final user handoff
because a commit cannot embed its own hash.

## Pair Inventory

| Document | Synchronized content checked | Result |
|---|---|---|
| `00.crate_plan.md` | Responsibility, out-of-scope boundaries, authority order, known gaps/drift, task decomposition, hard gates, and verification expectations. | Synchronized. |
| `vc_ir.md` | Snapshot-local `VcId`, seed accounting, generated formula ownership, local context, premises, statuses, anchors, rendering, planned tests, and public enum policy. | Synchronized. |
| `generator.md` | Explicit-payload generation scope, unavailable registration/algorithm payload boundaries, local context, controlled unfolding, normalization handoff, task slices, planned tests, and public enum policy. | Synchronized. |
| `discharge.md` | Deterministic pre-ATP scope, supported classes, limits, evidence/explanation model, status interaction, no-erase ATP boundary, planned tests, and public enum policy. | Synchronized. |
| `dependency_slice.md` | Conservative slice inputs/outputs, dependency entry classes, unknown coverage, reusable fingerprint contract, task-26 kernel-evidence identity integration, task-28 context-identity hash integration, planned tests, and public enum policy. | Synchronized by Task 28 updates. |
| `kernel_evidence_handoff.md` | Producer-side formula/substitution evidence handoff mapping, prohibited backend/legacy material, gap classification, task-25 builder public enum policy, resolved task-26 reuse-identity gap, task-27 explicit polarity contract, task-28 context-identity payload, and post-task-28 kernel handoff draft. | Synchronized by Tasks 24-28. |
| `source_spec_audit.md` | Public module exports, public surface inventory, cross-module evidence, explicit goal-polarity correspondence, context-identity correspondence, and classified external/deferred follow-ups. | Synchronized by task 28 updates. |
| `bilingual_sync_audit.md` | Audit scope, method, pair inventory, classification, and Task 19/21/22/closeout/24/25/26/27/28 sync edits. | Synchronized by this paired audit document. |
| `architecture_22_audit.md` | Task 20 architecture-22 identity correspondence, deterministic-discharge branch evidence, task-28 kernel-handoff context-identity reuse behavior, remaining external/deferred gaps, and no-drift classification. | Synchronized by Task 28 updates. |
| `module_boundary_audit.md` | Task 22 source-layout line counts, module-boundary review, no-required-split decision, and optional maintenance deferrals. | Synchronized by Task 22. |
| `crate_exit_report.md` | Final status, quality score, hard gates, task commits through task 26, verification, review outcomes, remaining deferred/external items, and next-crate handoff draft. | Synchronized by prior closeout/task-27 bookkeeping; task 28 remains post-closeout follow-up. |
| `task_ledger.md` | Task status, commit hashes available through task 27, complete task-28 review/verification evidence, deferred notes, and post-task-28 handoff draft. | Synchronized by task 28 updates; task 28 self-hash is backfilled after its commit exists. |
| `todo.md` | Ordered task list, completed tasks through task 28, closeout status, recommended verification, and notes. | Synchronized by task 28 updates. |

## Classification

Task 19 recorded no new `spec_gap`, `test_gap`, `design_drift`,
`source_drift`, `source_undocumented_behavior`, `test_expectation_drift`,
`boundary_violation`, or `repo_metadata_conflict`. The Task 22 update preserves
that classification while adding the module-boundary pair to the inventory.
Closeout preserves the same no-drift classification while adding the paired
exit reports.

Existing classified records remain:

- `external_dependency_gap`: active `proof_verification` runner support and
  source-to-core / source-to-VC extraction seams are absent from `mizar-test`;
  Task 15 records the deferred corpus obligation.
- `external_dependency_gap` / `deferred`: `mizar-kernel` now owns the
  checker-side formula/substitution evidence acceptance path, and `mizar-vc`
  now owns the explicit-payload producer-side handoff builder and reuse
  identity integration, but ATP candidate production, proof/cache consumers,
  and artifact witness consumers remain incomplete. ATP translation, proof
  policy, cache lookup/reuse, and artifact persistence remain downstream.
- `external_dependency_gap`: upstream explicit/stable payloads remain
  incomplete for registration/redefinition/reduction details,
  call-precondition, branch, match, range-loop, collection-loop, term-only
  termination, partial termination, Pick non-emptiness, ghost-erasure, complete
  trace families, source-derived core formula payloads, definition payloads,
  quantified binder payloads, and source-derived obligation payload families.
- `deferred`: proof-witness hashes, ATP/kernel/proof/cache validation,
  artifact consumers, and source-derived runner integration remain downstream
  before architecture-22 reuse can be accepted outside deterministic-discharge
  and current kernel-evidence handoff identity candidate keys.
- `deferred`: optional private helper/test splits inside large `vc_ir`,
  `generator`, and `dependency_slice` implementation files remain future
  move-only maintenance tasks if pursued. Final quality review and crate-exit
  status are recorded in [crate_exit_report.md](./crate_exit_report.md).

## Task 19 Sync Edits

This task adds the paired bilingual sync audit documents, backfills Task 18's
commit hash in the paired ledgers, records the Task 19 review/verification
outcome in the paired ledgers, and marks Task 19 complete in the paired todos.

No other paired content needed synchronization.

## Task 21 Sync Edits

Task 21 adds the paired architecture-22 audit documents and rechecks the
Task 20 identity contract across the English canonical documents and Japanese
companions. It backfills the Task 20 commit hash in the paired ledgers, records
the Task 21 review/verification outcome, marks Task 21 complete in the paired
todos, and records that the remaining architecture-22 gaps are classified
external/deferred rather than untracked drift.

## Task 22 Sync Edits

Task 22 adds the paired module-boundary audit documents and rechecks source
layout against the English canonical module specs, Japanese companions, and
internal crate-layout guidance. It backfills the Task 21 commit hash in the
paired ledgers, records the Task 22 review/verification outcome, marks Task 22
complete in the paired todos, and records that optional private helper/test
splits are future move-only maintenance work rather than crate-exit blockers.

## Closeout Sync Edits

Closeout adds the paired crate exit reports, backfills the Task 22 commit hash
in the paired ledgers, records final quality review score 94/100, records the
passing broad workspace verification, and adds closeout status to the paired
todos. It keeps the English canonical report and Japanese companion
semantically synchronized.

## Task 24 Sync Edits

Task 24 adds the paired kernel evidence handoff specification, adds it to the
pair inventory, updates the stale closeout-era kernel gap classification after
`mizar-kernel` tasks 23-29, marks Task 24 complete in the paired todos, and
records the task-25 handoff prompt in the paired ledgers. It keeps the English
canonical document and Japanese companion semantically synchronized and changes
no Rust source.

## Task 25 Sync Edits

Task 25 adds the paired source/spec, todo, plan, exit-report, ledger, and
kernel evidence handoff updates for the new Rust builder. It records the
`kernel_evidence_handoff` public enum policy in both languages, marks Task 25
complete in the paired todos, backfills the Task 24 hash in the paired ledgers,
and records the task-26 handoff prompt. The English canonical document and
Japanese companion stay semantically synchronized.

## Task 26 Sync Edits

Task 26 updates the paired dependency-slice, kernel-evidence handoff, todo,
plan, exit-report, ledger, and source/spec audit records for reuse identity
integration. Both languages record that the current canonical kernel evidence
handoff hash participates in dependency-slice fingerprints and proof-reuse
candidate keys, that legacy reuse without a current handoff fails closed, and
that downstream proof/cache/artifact consumers remain
`external_dependency_gap` / `deferred`.

## Task 27 Sync Edits

Task 27 updates the paired kernel-evidence handoff, source/spec audit, todo,
plan, ledger, bilingual sync audit, and mizar-kernel soundness argument records
for explicit producer-side goal polarity. Both languages record that current
proof-obligation handoffs require `AssertFalseForRefutation`, reject
`AssertTrueForConsistency` before canonical package assembly. The trusted
checker-side B4/F1 acceptance binding is implemented by `mizar-kernel` task 30.

## Task 28 Sync Edits

Task 28 updates the paired kernel-evidence handoff, dependency-slice,
architecture-22 audit, source/spec audit, todo, plan, ledger, bilingual sync
audit, and paired mizar-kernel F2 task records for producer-side context
identity. Both languages record that `context_identity_hash()` covers
local-hypothesis, cited-premise, and generated-VC-fact source bindings,
participates in dependency-slice and proof-reuse identity, excludes imported
facts, and is checked by trusted membership verification in `mizar-kernel` task
31.

## Core Task 32 Ownership Sync

Core Task 32 rechecks the paired plan, TODO, and source/spec audit. Both
languages record VC Task 30 as dependency-authorized for docs-only
decomposition while preserving Core 33-53, Gates A1/S1, concrete substitution,
and zero VC-generation/coverage authority.
