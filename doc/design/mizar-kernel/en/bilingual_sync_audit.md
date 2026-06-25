# Bilingual Sync Audit: mizar-kernel

> Canonical language: English. Japanese companion:
> [../ja/bilingual_sync_audit.md](../ja/bilingual_sync_audit.md).

## Scope And Authority

Task 21 audits every paired document under `doc/design/mizar-kernel/en/` and
`doc/design/mizar-kernel/ja/`. English remains canonical. Japanese companions
are synchronized to the English document unless a task-local bookkeeping
omission, such as a missing commit hash or task status, is obviously paired and
can be fixed in both languages with the same rationale. Japanese-only semantic
drift must not be promoted into English without a separate classified finding.

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
| `bilingual_sync_audit.md` | EN -> JA and JA -> EN | 6 / 6 | 25 / 25 | Synchronized. |
| `certificate_parser.md` | EN -> JA and JA -> EN | 15 / 15 | 29 / 29 | Synchronized. |
| `checker.md` | EN -> JA and JA -> EN | 15 / 15 | 15 / 15 | Synchronized. |
| `clause.md` | EN -> JA and JA -> EN | 12 / 12 | 5 / 5 | Synchronized. |
| `public_enum_policy.md` | EN -> JA and JA -> EN | 5 / 5 | 0 / 0 | Synchronized. |
| `rejection.md` | EN -> JA and JA -> EN | 14 / 14 | 32 / 32 | Synchronized. |
| `resolution_trace.md` | EN -> JA and JA -> EN | 12 / 12 | 15 / 15 | Synchronized. |
| `source_spec_audit.md` | EN -> JA and JA -> EN | 14 / 14 | 22 / 22 | Synchronized. |
| `substitution_checker.md` | EN -> JA and JA -> EN | 15 / 15 | 17 / 17 | Synchronized. |
| `task_ledger.md` | EN -> JA and JA -> EN | 10 / 10 | 26 / 26 | Synchronized after Task 20 hash backfill in this task. |
| `todo.md` | EN -> JA and JA -> EN | 12 / 12 | 8 / 8 | Synchronized after Task 20 completion and Task 21 progress status updates in this task. |

The count checks are not used as a full translation proof. They are a drift
screen that supports the semantic checks below.

## Semantic Sync Checks

| Area | Result |
|---|---|
| Canonical/companion headers | Every English file points at its Japanese companion, and every Japanese file points back to the English canonical file. |
| Task status and sequencing | Tasks 0-20 are complete or being backfilled consistently; Task 21 is the current in-progress documentation audit; Task 22 and closeout remain pending. |
| Task 20 bookkeeping | `fb81213c33d5b2a31eb976a4fa6804bfc0ffe6c5` is the completed Task 20 commit and is backfilled in both ledgers. |
| Task 21 handoff | The previous Task 20 handoff is replaced by a Task 21 to Task 22 handoff in both ledgers. |
| Public enum inventory | `public_enum_policy.md` uses the same `public-enum-inventory` block in English and Japanese. The executable guard in `crates/mizar-kernel/tests/lint_policy.rs` remains the source of exact inventory validation. |
| Source/spec audit inventory | `source_spec_audit.md` has matched module, public item, test-traceability, gap, and verification sections in English and Japanese. |
| Trust Statement wording | Each module keeps paired `## Trust Statement` sections with the task-20 trusted-kernel prohibition wording. |
| Gap/deferred classification | Remaining external integrations stay classified as `external_dependency_gap` or `deferred`; no placeholder integration is added by this task. |
| Repository metadata conflicts | No `repo_metadata_conflict` is observed in Task 21. |

## Remaining Gaps

Task 21 does not close external producer or consumer gaps. The following remain
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

Task 21 verification includes:

- deterministic file-pair and companion-link checks over
  `doc/design/mizar-kernel/{en,ja}`;
- `cargo test -p mizar-kernel --test lint_policy`, reusing the existing
  documentation and trusted-boundary guards;
- `git diff --check`;
- `git diff --cached --check` after explicit path staging.

`cargo fmt`, full `cargo test -p mizar-kernel`, and clippy are not required
unless this task changes Rust source or executable lint behavior; it does not.
