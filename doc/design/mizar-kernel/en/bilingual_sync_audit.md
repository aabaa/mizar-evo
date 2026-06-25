# Bilingual Sync Audit: mizar-kernel

> Canonical language: English. Japanese companion:
> [../ja/bilingual_sync_audit.md](../ja/bilingual_sync_audit.md).

## Scope And Authority

Task 21 audited every paired document under `doc/design/mizar-kernel/en/` and
`doc/design/mizar-kernel/ja/`. Task 22 refreshes that audit for the
module-boundary audit document and the cleaned task-ledger handoff section.
English remains canonical. Japanese companions are synchronized to the English
document unless a task-local bookkeeping omission, such as a missing commit
hash or task status, is obviously paired and can be fixed in both languages
with the same rationale. Japanese-only semantic drift must not be promoted into
English without a separate classified finding.

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
| `00.crate_plan.md` | EN -> JA and JA -> EN | 8 / 8 | 56 / 56 | Synchronized. |
| `bilingual_sync_audit.md` | EN -> JA and JA -> EN | 6 / 6 | 27 / 27 | Synchronized after adding the task-22 audit rows. |
| `certificate_parser.md` | EN -> JA and JA -> EN | 15 / 15 | 29 / 29 | Synchronized. |
| `checker.md` | EN -> JA and JA -> EN | 15 / 15 | 15 / 15 | Synchronized. |
| `clause.md` | EN -> JA and JA -> EN | 12 / 12 | 5 / 5 | Synchronized. |
| `module_boundary_audit.md` | EN -> JA and JA -> EN | 6 / 6 | 13 / 13 | Synchronized. |
| `public_enum_policy.md` | EN -> JA and JA -> EN | 5 / 5 | 0 / 0 | Synchronized. |
| `rejection.md` | EN -> JA and JA -> EN | 14 / 14 | 32 / 32 | Synchronized. |
| `resolution_trace.md` | EN -> JA and JA -> EN | 12 / 12 | 15 / 15 | Synchronized. |
| `source_spec_audit.md` | EN -> JA and JA -> EN | 14 / 14 | 22 / 22 | Synchronized. |
| `substitution_checker.md` | EN -> JA and JA -> EN | 15 / 15 | 17 / 17 | Synchronized. |
| `task_ledger.md` | EN -> JA and JA -> EN | 2 / 2 | 26 / 26 | Synchronized after Task 21 hash backfill and stale handoff cleanup. |
| `todo.md` | EN -> JA and JA -> EN | 12 / 12 | 8 / 8 | Synchronized after Task 21 completion and Task 22 progress status updates. |

The count checks are not used as a full translation proof. They are a drift
screen that supports the semantic checks below.

## Semantic Sync Checks

| Area | Result |
|---|---|
| Canonical/companion headers | Every English file points at its Japanese companion, and every Japanese file points back to the English canonical file. |
| Task status and sequencing | Tasks 0-21 are complete consistently; Task 22 is the current ready-to-commit module-boundary gate with self-hash pending; closeout remains pending. |
| Task 21 bookkeeping | `73a919c16b48da82038fd7267e86e1a844cb4c6f` is the completed Task 21 commit and is backfilled in both ledgers. |
| Task 22 handoff | Stale historical handoffs are removed, and both ledgers keep only the Task 22 to closeout handoff. |
| Public enum inventory | `public_enum_policy.md` uses the same `public-enum-inventory` block in English and Japanese. The executable guard in `crates/mizar-kernel/tests/lint_policy.rs` remains the source of exact inventory validation. |
| Source/spec audit inventory | `source_spec_audit.md` has matched module, public item, test-traceability, gap, and verification sections in English and Japanese. |
| Module-boundary audit inventory | `module_boundary_audit.md` is paired and records the same move-only test-module split, drift classification, and verification plan in English and Japanese. |
| Trust Statement wording | Each module keeps paired `## Trust Statement` sections with the task-20 trusted-kernel prohibition wording. |
| Gap/deferred classification | Remaining external integrations stay classified as `external_dependency_gap` or `deferred`; no placeholder integration is added by this task. |
| Repository metadata conflicts | No `repo_metadata_conflict` is observed in Task 22. |

## Remaining Gaps

Task 22 does not close external producer or consumer gaps. The following remain
documented in the module specs and `source_spec_audit.md`:

- source-derived certificate and service envelopes;
- ATP proof translation and MiniSAT-compatible backend trace extraction;
- `mizar-checker` cluster/reduction payload production;
- derived-fact payload schema work beyond current checked inputs;
- service-envelope normalization, cancellation token plumbing, and external
  worker scheduling;
- downstream `mizar-proof`, `mizar-cache`, and `mizar-artifact` consumers;
- downstream wildcard-arm checks for public kernel enums.

## Verification Plan

Task 22 verification for this refreshed bilingual audit includes:

- deterministic file-pair and companion-link checks over
  `doc/design/mizar-kernel/{en,ja}`;
- `cargo test -p mizar-kernel --test lint_policy`, reusing the documentation
  and trusted-boundary guards;
- `git diff --check`;
- `git diff --cached --check` after explicit path staging.

Because Task 22 changes Rust source layout and the executable source-layout
guard, its full task verification also includes `cargo fmt --check`,
`cargo test -p mizar-kernel`, and
`cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings`.
