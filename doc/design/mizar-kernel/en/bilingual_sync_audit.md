# Bilingual Sync Audit: mizar-kernel

> Canonical language: English. Japanese companion:
> [../ja/bilingual_sync_audit.md](../ja/bilingual_sync_audit.md).

## Scope And Authority

Task 21 audited every paired document under `doc/design/mizar-kernel/en/` and
`doc/design/mizar-kernel/ja/`. Task 22 refreshed that audit for the
module-boundary audit document and the cleaned task-ledger handoff section. The
closeout task refreshes it again for the paired crate exit report and Task 22
hash backfill. Tasks 23-24 refresh it for the formula/SAT correction specs and
the SAT dependency audit. Tasks 25-26 refresh it for the formula-evidence
parser and source-backed SAT encoding implementation. Task 27 refreshes it for
the trusted SAT checker wrapper, exact dependency/lockfile policy, the
read-only encoded SAT problem boundary, and public enum/source-spec inventory.
Task 28 refreshes it for the SAT-backed kernel evidence service path and
checker API inventory. Task 29 refreshes it for the explicit legacy audit gate,
post-correction closeout report, and mizar-vc handoff. Task 31 refreshes it for
kernel-side context-identity verification and public context-identity payload
types. Task 34 refreshes it for imported-statement projection hash backfill,
legacy tautology-marker audit semantics, and the mizar-proof task-21 handoff.
English remains canonical. Japanese companions are
synchronized to the English document unless a task-local bookkeeping omission,
such as a missing commit hash or task status, is obviously paired and can be
fixed in both languages with the same rationale. Japanese-only semantic drift
must not be promoted into English without a separate classified finding.

This is a documentation audit for the task-34 source/test/doc change that pins
legacy tautology markers to audit-only replay data. It does not change
`doc/spec`, `.miz` fixtures, expectations, SAT/ATP/proof search, premise
selection, overload resolution, cluster search, implicit coercion insertion,
fallback inference, global mutable state, or downstream ATP/proof/cache/artifact
integration.

## Pair Inventory

All current English design documents have a Japanese companion with the same
file name, and all Japanese design documents have an English canonical file
with the same file name.

| File | Companion links | Heading count | Table row count | Sync result |
|---|---|---:|---:|---|
| `00.crate_plan.md` | EN -> JA and JA -> EN | 8 / 8 | 65 / 65 | Refreshed for task-31 post-correction owned behavior. |
| `bilingual_sync_audit.md` | EN -> JA and JA -> EN | 6 / 6 | 35 / 35 | Synchronized after task-34 tautology-marker bookkeeping. |
| `certificate_parser.md` | EN -> JA and JA -> EN | 15 / 15 | 29 / 29 | Synchronized. |
| `checker.md` | EN -> JA and JA -> EN | 17 / 17 | 15 / 15 | Refreshed for task-34 legacy tautology final-goal rejection. |
| `clause.md` | EN -> JA and JA -> EN | 12 / 12 | 5 / 5 | Synchronized. |
| `crate_exit_report.md` | EN -> JA and JA -> EN | 12 / 12 | 88 / 88 | Refreshed for task-34 post-closeout Step 3 state and mizar-proof handoff. |
| `formula_evidence.md` | EN -> JA and JA -> EN | 13 / 13 | 0 / 0 | Refreshed for task-31 context-identity evidence boundary. |
| `module_boundary_audit.md` | EN -> JA and JA -> EN | 6 / 6 | 13 / 13 | Synchronized. |
| `public_enum_policy.md` | EN -> JA and JA -> EN | 5 / 5 | 0 / 0 | Synchronized after task-31 context-identity enum additions. |
| `rejection.md` | EN -> JA and JA -> EN | 14 / 14 | 32 / 32 | Refreshed for task-27 SAT wrapper failure mapping. |
| `resolution_trace.md` | EN -> JA and JA -> EN | 14 / 14 | 15 / 15 | Refreshed for task-34 legacy tautology-marker semantics. |
| `sat_checker.md` | EN -> JA and JA -> EN | 6 / 6 | 0 / 0 | Refreshed for task-27 wrapper API and unsupported step-budget policy. |
| `sat_dependency_audit.md` | EN -> JA and JA -> EN | 13 / 13 | 32 / 32 | Refreshed for task-27 exact dependency and no-callback branch. |
| `sat_encoding.md` | EN -> JA and JA -> EN | 8 / 8 | 0 / 0 | Refreshed for task-27 read-only encoded problem boundary. |
| `soundness_argument.md` | EN -> JA and JA -> EN | 20 / 20 | 24 / 24 | Refreshed for task-34 F9 closure. |
| `source_spec_audit.md` | EN -> JA and JA -> EN | 18 / 18 | 31 / 31 | Synchronized after task-34 F9 traceability update. |
| `substitution_checker.md` | EN -> JA and JA -> EN | 15 / 15 | 17 / 17 | Synchronized. |
| `task_ledger.md` | EN -> JA and JA -> EN | 2 / 2 | 36 / 36 | Synchronized after task-33 hash backfill and task-34 completion row. |
| `todo.md` | EN -> JA and JA -> EN | 14 / 14 | 22 / 22 | Synchronized after task-34 completion status update. |

The count checks are not used as a full translation proof. They are a drift
screen that supports the semantic checks below.

## Semantic Sync Checks

| Area | Result |
|---|---|
| Canonical/companion headers | Every English file points at its Japanese companion, and every Japanese file points back to the English canonical file. |
| Task status and sequencing | Tasks 0-31 are complete consistently; task 33 commit `0f3d7fa316cffbf7e55722fd255cb3fbf32d9249` is backfilled, task 34 is complete with its commit pending later backfill, and task 32 remains a dependency-triggered deferral. |
| Task 21 bookkeeping | `73a919c16b48da82038fd7267e86e1a844cb4c6f` is the completed Task 21 commit and is backfilled in both ledgers. |
| Task 22 bookkeeping | `814e47bb9aaaff75ebfe4cc1be10d2eb4618498b` is the completed Task 22 commit and is backfilled in both ledgers. |
| Closeout report inventory | `crate_exit_report.md` is paired and records the same hard gates, task commits, residual gaps, quality score, verification plan, and next-crate handoff in English and Japanese. |
| Closeout handoff | Both ledgers now point to `mizar-proof` task 21 as the next requested Step 3 task. |
| Public enum inventory | `public_enum_policy.md` uses the same `public-enum-inventory` block in English and Japanese, including task-31 context-identity source and producer-ref enums. The executable guard in `crates/mizar-kernel/tests/lint_policy.rs` remains the source of exact inventory validation. |
| Source/spec audit inventory | `source_spec_audit.md` has matched module, public item, formula-evidence/SAT-checker/checker-service/context-identity/tautology-marker traceability, gap, and verification sections in English and Japanese. |
| Module-boundary audit inventory | `module_boundary_audit.md` is paired and records the same move-only test-module split, drift classification, and verification plan in English and Japanese. |
| Trust Statement wording | Each module keeps paired `## Trust Statement` sections with the task-20 trusted-kernel prohibition wording. |
| Gap/deferred classification | Remaining external integrations stay classified as `external_dependency_gap` or `deferred`; no placeholder integration is added by this task. |
| Repository metadata conflicts | No `repo_metadata_conflict` is observed in closeout. |

## Remaining Gaps

Task 29 closes the legacy normal-policy acceptance gap by gating
`check_kernel_certificate` behind explicit audit policy. Task 31 closes the
kernel-side F2 context-identity membership gap for non-imported formula
sources. Task 34 closes the F9 legacy tautology-marker semantics gap by pinning
the marker to audit-only replay data and final-goal rejection. External
producer or consumer gaps remain. The following remain
documented in the module specs and `source_spec_audit.md`:

- source-derived formula/substitution evidence and service envelopes;
- formula/substitution candidate evidence production in `mizar-atp`;
- ATP proof translation and MiniSAT-compatible backend trace extraction only
  as legacy migration/audit material, never as trusted acceptance targets;
- `mizar-checker` cluster/reduction payload production;
- derived-fact payload schema work beyond current checked inputs;
- service-envelope normalization, cancellation token plumbing, and external
  worker scheduling;
- downstream `mizar-proof`, `mizar-cache`, and `mizar-artifact` consumers;
- downstream wildcard-arm checks for public kernel enums.

## Verification Plan

Task 34 verification for this refreshed bilingual audit includes:

- deterministic file-pair and companion-link checks over
  `doc/design/mizar-kernel/{en,ja}`;
- focused `cargo test -p mizar-kernel tautology_outcomes_follow_the_active_clause_profile -- --nocapture`;
- focused `cargo test -p mizar-kernel kernel_service_rejects_legacy_tautology_marker_final_goal_in_audit_mode -- --nocapture`;
- `cargo fmt --check`;
- `cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings`;
- `cargo test -p mizar-kernel`;
- broad `cargo clippy --all-targets --all-features -- -D warnings` and
  `cargo test` when practical;
- `git diff --check`;
- `git diff --cached --check` after explicit path staging.
