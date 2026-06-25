# Bilingual Sync Audit: mizar-kernel

> 正本は英語です。英語版:
> [../en/bilingual_sync_audit.md](../en/bilingual_sync_audit.md)。

## Scope And Authority

Task 21 は `doc/design/mizar-kernel/en/` と `doc/design/mizar-kernel/ja/` の
paired document すべてを audit した。Task 22 は module-boundary audit document
と整理済み task-ledger handoff section を含めるため、この audit を更新する。
英語は canonical のままである。日本語 companion は英語文書に同期する。ただし、
commit hash や task status の欠落のように task-local bookkeeping omission が
明らかに paired であり、同じ rationale で両言語を修正できる場合は例外とする。
Japanese-only semantic drift を、別の classified finding なしに英語へ昇格して
はならない。

これは documentation audit である。Rust behavior、public API、certificate
semantics、rejection semantics、`doc/spec`、`.miz` fixture、expectation、
SAT/ATP/proof search、premise selection、overload resolution、cluster search、
implicit coercion insertion、fallback inference、global mutable state、downstream
ATP/proof/cache/artifact integration は変更しない。

## Pair Inventory

現在のすべての English design document は同名の Japanese companion を持ち、
すべての Japanese design document は同名の English canonical file を持つ。

| File | Companion links | Heading count | Table row count | Sync result |
|---|---|---:|---:|---|
| `00.crate_plan.md` | EN -> JA and JA -> EN | 8 / 8 | 56 / 56 | Synchronized. |
| `bilingual_sync_audit.md` | EN -> JA and JA -> EN | 6 / 6 | 27 / 27 | task-22 audit rows 追加後に synchronized. |
| `certificate_parser.md` | EN -> JA and JA -> EN | 15 / 15 | 29 / 29 | Synchronized. |
| `checker.md` | EN -> JA and JA -> EN | 15 / 15 | 15 / 15 | Synchronized. |
| `clause.md` | EN -> JA and JA -> EN | 12 / 12 | 5 / 5 | Synchronized. |
| `module_boundary_audit.md` | EN -> JA and JA -> EN | 6 / 6 | 13 / 13 | Synchronized. |
| `public_enum_policy.md` | EN -> JA and JA -> EN | 5 / 5 | 0 / 0 | Synchronized. |
| `rejection.md` | EN -> JA and JA -> EN | 14 / 14 | 32 / 32 | Synchronized. |
| `resolution_trace.md` | EN -> JA and JA -> EN | 12 / 12 | 15 / 15 | Synchronized. |
| `source_spec_audit.md` | EN -> JA and JA -> EN | 14 / 14 | 22 / 22 | Synchronized. |
| `substitution_checker.md` | EN -> JA and JA -> EN | 15 / 15 | 17 / 17 | Synchronized. |
| `task_ledger.md` | EN -> JA and JA -> EN | 2 / 2 | 26 / 26 | Task 21 hash backfill と stale handoff cleanup 後に synchronized. |
| `todo.md` | EN -> JA and JA -> EN | 12 / 12 | 8 / 8 | Task 21 completion と Task 22 progress status update 後に synchronized. |

Count check は完全な translation proof ではない。下の semantic check を補助する
drift screen である。

## Semantic Sync Checks

| Area | Result |
|---|---|
| Canonical/companion headers | すべての English file は Japanese companion を指し、すべての Japanese file は English canonical file を指す。 |
| Task status and sequencing | Tasks 0-21 は complete として一貫し、Task 22 は self-hash pending の current ready-to-commit module-boundary gate、closeout は pending のままである。 |
| Task 21 bookkeeping | `73a919c16b48da82038fd7267e86e1a844cb4c6f` は完了済み Task 21 commit であり、両 ledger に backfill される。 |
| Task 22 handoff | stale historical handoff を削除し、両 ledger は Task 22 to closeout handoff だけを保持する。 |
| Public enum inventory | `public_enum_policy.md` は英語/日本語で同じ `public-enum-inventory` block を使う。正確な inventory validation の source は `crates/mizar-kernel/tests/lint_policy.rs` の executable guard のままである。 |
| Source/spec audit inventory | `source_spec_audit.md` は英語/日本語で module、public item、test-traceability、gap、verification sections が対応している。 |
| Module-boundary audit inventory | `module_boundary_audit.md` は paired であり、同じ move-only test-module split、drift classification、verification plan を英語/日本語で記録する。 |
| Trust Statement wording | 各 module は task-20 trusted-kernel prohibition wording を持つ paired `## Trust Statement` sections を維持している。 |
| Gap/deferred classification | 残る external integration は `external_dependency_gap` または `deferred` として分類されたままであり、この task は placeholder integration を追加しない。 |
| Repository metadata conflicts | Task 22 で `repo_metadata_conflict` は観測されない。 |

## Remaining Gaps

Task 22 は external producer / consumer gap を閉じない。以下は module spec と
`source_spec_audit.md` に残る:

- source-derived certificate and service envelopes;
- ATP proof translation and MiniSAT-compatible backend trace extraction;
- `mizar-checker` cluster/reduction payload production;
- current checked input を超える derived-fact payload schema work;
- service-envelope normalization、cancellation token plumbing、external worker
  scheduling;
- downstream `mizar-proof`, `mizar-cache`, `mizar-artifact` consumers;
- public kernel enum に対する downstream wildcard-arm checks.

## Verification Plan

Task 22 の refreshed bilingual audit verification:

- `doc/design/mizar-kernel/{en,ja}` に対する deterministic file-pair and
  companion-link checks;
- documentation / trusted-boundary guard を再利用する
  `cargo test -p mizar-kernel --test lint_policy`;
- `git diff --check`;
- explicit path staging 後の `git diff --cached --check`。

Task 22 は Rust source layout と executable source-layout guard も変更するため、
full task verification には `cargo fmt --check`、`cargo test -p mizar-kernel`、
`cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings` も含む。
