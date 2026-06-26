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
parser and source-backed SAT encoding implementation. English remains
canonical. Japanese companions are
synchronized to the English document unless a task-local bookkeeping omission,
such as a missing commit hash or task status, is obviously paired and can be
fixed in both languages with the same rationale. Japanese-only semantic drift
must not be promoted into English without a separate classified finding.

This is a documentation audit. It does not change Rust behavior, public APIs,
certificate semantics, rejection semantics, `doc/spec`, `.miz` fixtures,
expectations, SAT/ATP/proof search, premise selection, overload resolution,
cluster search, implicit coercion insertion, fallback inference, global mutable
state, or downstream ATP/proof/cache/artifact integration.

## Pair Inventory

All current English design documents have a Japanese companion with the same
file name, and all Japanese design documents have an English canonical file
with the same file name.

| File | Companion links | Heading count | Table row count | Sync result |
|---|---|---:|---:|---|
| `00.crate_plan.md` | EN -> JA and JA -> EN | 8 / 8 | 64 / 64 | Synchronized after post-closeout task rows. |
| `bilingual_sync_audit.md` | EN -> JA and JA -> EN | 6 / 6 | 34 / 34 | Synchronized after task-26 SAT encoding implementation bookkeeping. |
| `certificate_parser.md` | EN -> JA and JA -> EN | 15 / 15 | 29 / 29 | Synchronized. |
| `checker.md` | EN -> JA and JA -> EN | 15 / 15 | 15 / 15 | Synchronized. |
| `clause.md` | EN -> JA and JA -> EN | 12 / 12 | 5 / 5 | Synchronized. |
| `crate_exit_report.md` | EN -> JA and JA -> EN | 12 / 12 | 71 / 71 | Added by closeout and synchronized. |
| `formula_evidence.md` | EN -> JA and JA -> EN | 13 / 13 | 0 / 0 | Refined for task 25 implementation and synchronized. |
| `module_boundary_audit.md` | EN -> JA and JA -> EN | 6 / 6 | 13 / 13 | Synchronized. |
| `public_enum_policy.md` | EN -> JA and JA -> EN | 5 / 5 | 0 / 0 | Synchronized after task-25 formula evidence enum additions. |
| `rejection.md` | EN -> JA and JA -> EN | 14 / 14 | 32 / 32 | Synchronized. |
| `resolution_trace.md` | EN -> JA and JA -> EN | 12 / 12 | 15 / 15 | Synchronized. |
| `sat_checker.md` | EN -> JA and JA -> EN | 6 / 6 | 0 / 0 | Added by task 23 and refreshed by task 24. |
| `sat_dependency_audit.md` | EN -> JA and JA -> EN | 13 / 13 | 32 / 32 | Added by task 24 and synchronized. |
| `sat_encoding.md` | EN -> JA and JA -> EN | 8 / 8 | 0 / 0 | Synchronized after task-26 instantiation-scope specification. |
| `source_spec_audit.md` | EN -> JA and JA -> EN | 17 / 17 | 27 / 27 | Synchronized after task-26 source-backed SAT encoding module. |
| `substitution_checker.md` | EN -> JA and JA -> EN | 15 / 15 | 17 / 17 | Synchronized. |
| `task_ledger.md` | EN -> JA and JA -> EN | 2 / 2 | 30 / 30 | Synchronized after Task 25 hash backfill and Task 26 pending-commit row. |
| `todo.md` | EN -> JA and JA -> EN | 13 / 13 | 11 / 11 | Synchronized after Task 26 completion status update. |

The count checks are not used as a full translation proof. They are a drift
screen that supports the semantic checks below.

## Semantic Sync Checks

| Area | Result |
|---|---|
| Canonical/companion headers | Every English file points at its Japanese companion, and every Japanese file points back to the English canonical file. |
| Task status and sequencing | Tasks 0-25 are complete consistently; task 25 commit `35ef60ffba949254e71d86f9be2570b37e5f4a3c` is backfilled, and task 26 is complete pending its own commit hash with paired SAT encoding implementation specs. |
| Task 21 bookkeeping | `73a919c16b48da82038fd7267e86e1a844cb4c6f` is the completed Task 21 commit and is backfilled in both ledgers. |
| Task 22 bookkeeping | `814e47bb9aaaff75ebfe4cc1be10d2eb4618498b` is the completed Task 22 commit and is backfilled in both ledgers. |
| Closeout report inventory | `crate_exit_report.md` is paired and records the same hard gates, task commits, residual gaps, quality score, verification plan, and next-crate handoff in English and Japanese. |
| Closeout handoff | Both ledgers now keep only the closeout to next-crate handoff. |
| Public enum inventory | `public_enum_policy.md` uses the same `public-enum-inventory` block in English and Japanese. The executable guard in `crates/mizar-kernel/tests/lint_policy.rs` remains the source of exact inventory validation. |
| Source/spec audit inventory | `source_spec_audit.md` has matched module, public item, formula-evidence traceability, gap, and verification sections in English and Japanese. |
| Module-boundary audit inventory | `module_boundary_audit.md` is paired and records the same move-only test-module split, drift classification, and verification plan in English and Japanese. |
| Trust Statement wording | Each module keeps paired `## Trust Statement` sections with the task-20 trusted-kernel prohibition wording. |
| Gap/deferred classification | Remaining external integrations stay classified as `external_dependency_gap` or `deferred`; no placeholder integration is added by this task. |
| Repository metadata conflicts | No `repo_metadata_conflict` is observed in closeout. |

## Remaining Gaps

Task 26 does not close external producer or consumer gaps. The following remain
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

Task 26 verification for this refreshed bilingual audit includes:

- deterministic file-pair and companion-link checks over
  `doc/design/mizar-kernel/{en,ja}`;
- `cargo fmt --check`;
- `cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings`;
- `cargo test -p mizar-kernel`;
- `git diff --check`;
- `git diff --cached --check` after explicit path staging.
