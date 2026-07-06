# mizar-proof Bilingual Documentation Sync Audit

> Canonical language: English. Japanese companion:
> [../ja/bilingual_sync_audit.md](../ja/bilingual_sync_audit.md).

## Scope

Task 16 compares every English canonical document under
`doc/design/mizar-proof/en/` with its Japanese companion under
`doc/design/mizar-proof/ja/`.

Files audited:

| English canonical | Japanese companion | Result |
|---|---|---|
| `00.crate_plan.md` | `00.crate_plan.md` | synchronized; task-21 alignment refresh required |
| `policy.md` | `policy.md` | synchronized |
| `selection.md` | `selection.md` | synchronized |
| `status.md` | `status.md` | synchronized |
| `witness_store.md` | `witness_store.md` | synchronized |
| `source_spec_audit.md` | `source_spec_audit.md` | synchronized |
| `todo.md` | `todo.md` | synchronized; task-21 status update required |
| `task_ledger.md` | `task_ledger.md` | synchronized; task-20 hash backfill and task-24 handoff required |
| `bilingual_sync_audit.md` | `bilingual_sync_audit.md` | created by this task |
| `crate_exit_report.md` | `crate_exit_report.md` | synchronized; task-21 post-closeout alignment refresh required |

No file is missing its companion. No Japanese placeholder remains.

## Method

The audit checked:

- paired file names and canonical-language links;
- heading structure and section order;
- task status, task ledger, current handoff, and recommended reasoning text;
- trust-boundary wording: `mizar-proof` is not a proof acceptor, trusted
  acceptance comes only from `mizar-kernel`, and external/backend/cache
  material is never promoted to trusted status or trusted `used_axioms`;
- deferred and `external_dependency_gap` wording for cache reuse,
  `DischargedBuiltin` artifact witnesses, artifact publication tokens, copied
  kernel metadata, payload canonicality validators, and ATP early-stop
  integration;
- public enum policy statements and no-exhaustive-exception statements.

Japanese documents intentionally retain stable English identifiers, enum names,
status strings, gap ids, and command names. That terminology is not drift.

Task 21 refreshed the paired documentation after the kernel soundness-audit
alignment work. The refresh covered `00.crate_plan.md`, `policy.md`,
`selection.md`, `status.md`, `source_spec_audit.md`, `todo.md`,
`task_ledger.md`, `bilingual_sync_audit.md`, and `crate_exit_report.md`.
English and Japanese documents agree that source-backed corrected terminal
kernel rejections are not policy-open fallback material, that legacy
unsupported-certificate real-payload coverage is an `external_dependency_gap`,
and that accepted goal polarity is exported only for trusted accepted
proof-obligation selections.

## Findings

No blocking bilingual drift was found. The only required edits are task-local
metadata updates:

| ID | Classification | Finding | Resolution |
|---|---|---|---|
| `PROOF16-SYNC-001` | `design_drift` | `00.crate_plan.md` listed `bilingual_sync_audit.md` as planned before task 16. | Update paired crate plans to mark `source_spec_audit.md` and `bilingual_sync_audit.md` completed and leave later audits planned. |
| `PROOF16-SYNC-002` | `design_drift` | `todo.md` still marked task 16 incomplete before this audit. | Update paired TODOs with task-16 completion and audit summary. |
| `PROOF16-SYNC-003` | `design_drift` | `task_ledger.md` cannot contain task 15's self-hash until task 16 edits the ledger. | Backfill task 15, add task 16, and move the handoff to task 17. |

These are expected task-finalization updates, not source/spec behavior drift.

## Trust Boundary Check

The paired documents agree on these canonical constraints:

- `mizar-proof` owns proof policy, deterministic winner selection, status
  projection, witness staging/publication references, early-stop policy hooks,
  and proof-reuse validation metadata.
- `mizar-proof` does not run ATP backends, SAT solving, kernel acceptance,
  proof search, premise selection, substitution invention, cache lookup, or
  artifact manifest commit.
- Trusted acceptance and trusted `used_axioms` come only from accepted
  proof-obligation `mizar-kernel::checker::KernelCheckResult` values.
- Externally attested evidence, backend diagnostics, backend proof payloads,
  backend-reported axiom lists, cache records, policy assumptions, open
  obligations, consistency checks, and witness metadata remain non-trusted
  unless an accepted proof-obligation kernel result independently supports the
  trusted class.
- `require_kernel_certificates` prevents externally attested evidence and
  policy assumptions from becoming winners.
- Arrival order, completion time, runtime duration, worker/process ids,
  temporary paths, and cache timing are not proof identity.

## Remaining Gaps

Task 16 introduces no new implementation gap. It preserves the existing
`deferred` and `external_dependency_gap` records from the module specs and
source/spec audit:

| Gap area | Classification | Current owner / future task |
|---|---|---|
| cache-facing proof-reuse export contract | `deferred` | task 17 / future cache consumer |
| `DischargedBuiltin` artifact witness schema support | `external_dependency_gap` | `mizar-artifact` |
| committed witness publication proof token | `external_dependency_gap` | artifact publication boundary |
| copied kernel acceptance metadata for witness drafts | `external_dependency_gap` | kernel/artifact boundary |
| byte-level witness payload canonicality validators | `deferred` | concrete payload producers |
| live ATP early-stop adoption/cancellation wiring | `external_dependency_gap` | `mizar-atp` |
| downstream cache consumption of accepted goal polarity | `external_dependency_gap` | `mizar-cache` task 24 |

No `repo_metadata_conflict` was observed during the task-16 bilingual sync
audit. The later ATP closeout metadata conflict was recorded by tasks 18-20
and resolved by focused correction commit `36d1a9c`.

The task-21 refresh observed no new bilingual drift and no
`repo_metadata_conflict`; it updates existing paired docs and Rust tests only.

## Conclusion

The English canonical documentation and Japanese companions are synchronized
for the current `mizar-proof` state, including the task-21 kernel-audit
alignment refresh.
