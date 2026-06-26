# Bilingual Sync Audit: mizar-kernel

> 正本は英語です。英語版:
> [../en/bilingual_sync_audit.md](../en/bilingual_sync_audit.md)。

## Scope And Authority

Task 21 は `doc/design/mizar-kernel/en/` と `doc/design/mizar-kernel/ja/` の
paired document すべてを audit した。Task 22 は module-boundary audit document
と整理済み task-ledger handoff section を含めるため、この audit を更新した。
Closeout task は paired crate exit report と Task 22 hash backfill を含めるため、
もう一度更新する。Tasks 23-24 は formula/SAT correction specs と SAT dependency
audit を含めるため、この audit を更新する。英語は canonical のままである。日本語
companion は英語文書に同期する。ただし、commit hash や task status の欠落のように
task-local bookkeeping omission が明らかに paired であり、同じ rationale で両言語を
修正できる場合は例外とする。Japanese-only semantic drift を、別の classified finding
なしに英語へ昇格してはならない。

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
| `00.crate_plan.md` | EN -> JA and JA -> EN | 8 / 8 | 64 / 64 | post-closeout task rows 追加後に synchronized. |
| `bilingual_sync_audit.md` | EN -> JA and JA -> EN | 6 / 6 | 34 / 34 | task-24 audit rows 追加後に synchronized. |
| `certificate_parser.md` | EN -> JA and JA -> EN | 15 / 15 | 29 / 29 | Synchronized. |
| `checker.md` | EN -> JA and JA -> EN | 15 / 15 | 15 / 15 | Synchronized. |
| `clause.md` | EN -> JA and JA -> EN | 12 / 12 | 5 / 5 | Synchronized. |
| `crate_exit_report.md` | EN -> JA and JA -> EN | 12 / 12 | 71 / 71 | Closeout で追加し synchronized. |
| `formula_evidence.md` | EN -> JA and JA -> EN | 9 / 9 | 0 / 0 | Task 23 で追加し synchronized. |
| `module_boundary_audit.md` | EN -> JA and JA -> EN | 6 / 6 | 13 / 13 | Synchronized. |
| `public_enum_policy.md` | EN -> JA and JA -> EN | 5 / 5 | 0 / 0 | Synchronized. |
| `rejection.md` | EN -> JA and JA -> EN | 14 / 14 | 32 / 32 | Synchronized. |
| `resolution_trace.md` | EN -> JA and JA -> EN | 12 / 12 | 15 / 15 | Synchronized. |
| `sat_checker.md` | EN -> JA and JA -> EN | 6 / 6 | 0 / 0 | Task 23 で追加し task 24 で更新。 |
| `sat_dependency_audit.md` | EN -> JA and JA -> EN | 13 / 13 | 32 / 32 | Task 24 で追加し synchronized. |
| `sat_encoding.md` | EN -> JA and JA -> EN | 7 / 7 | 0 / 0 | Task 23 で追加し synchronized. |
| `source_spec_audit.md` | EN -> JA and JA -> EN | 15 / 15 | 23 / 23 | task-24 addendum 後に synchronized. |
| `substitution_checker.md` | EN -> JA and JA -> EN | 15 / 15 | 17 / 17 | Synchronized. |
| `task_ledger.md` | EN -> JA and JA -> EN | 2 / 2 | 27 / 27 | Task 23 hash backfill 後に synchronized. |
| `todo.md` | EN -> JA and JA -> EN | 13 / 13 | 11 / 11 | Task 24 dependency decision 後に synchronized. |

Count check は完全な translation proof ではない。下の semantic check を補助する
drift screen である。

## Semantic Sync Checks

| Area | Result |
|---|---|
| Canonical/companion headers | すべての English file は Japanese companion を指し、すべての Japanese file は English canonical file を指す。 |
| Task status and sequencing | Tasks 0-23 は complete として一貫する。Task 24 は ready to commit で self-hash pending であり、paired dependency audit を記録し、task 27 の source integration までは docs-only のままである。 |
| Task 21 bookkeeping | `73a919c16b48da82038fd7267e86e1a844cb4c6f` は完了済み Task 21 commit であり、両 ledger に backfill される。 |
| Task 22 bookkeeping | `814e47bb9aaaff75ebfe4cc1be10d2eb4618498b` は完了済み Task 22 commit であり、両 ledger に backfill される。 |
| Closeout report inventory | `crate_exit_report.md` は paired であり、同じ hard gates、task commits、residual gaps、quality score、verification plan、next-crate handoff を英語/日本語で記録する。 |
| Closeout handoff | 両 ledger は closeout to next-crate handoff だけを保持する。 |
| Public enum inventory | `public_enum_policy.md` は英語/日本語で同じ `public-enum-inventory` block を使う。正確な inventory validation の source は `crates/mizar-kernel/tests/lint_policy.rs` の executable guard のままである。 |
| Source/spec audit inventory | `source_spec_audit.md` は英語/日本語で module、public item、test-traceability、gap、verification sections が対応している。 |
| Module-boundary audit inventory | `module_boundary_audit.md` は paired であり、同じ move-only test-module split、drift classification、verification plan を英語/日本語で記録する。 |
| Trust Statement wording | 各 module は task-20 trusted-kernel prohibition wording を持つ paired `## Trust Statement` sections を維持している。 |
| Gap/deferred classification | 残る external integration は `external_dependency_gap` または `deferred` として分類されたままであり、この task は placeholder integration を追加しない。 |
| Repository metadata conflicts | Closeout で `repo_metadata_conflict` は観測されない。 |

## Remaining Gaps

Closeout は external producer / consumer gap を閉じない。以下は module spec と
`source_spec_audit.md` に残る:

- source-derived formula/substitution evidence and service envelopes;
- `mizar-atp` による formula/substitution candidate evidence production;
- legacy migration/audit material としての ATP proof translation と
  MiniSAT-compatible backend trace extraction。trusted acceptance target では決してない;
- `mizar-checker` cluster/reduction payload production;
- current checked input を超える derived-fact payload schema work;
- service-envelope normalization、cancellation token plumbing、external worker
  scheduling;
- downstream `mizar-proof`, `mizar-cache`, `mizar-artifact` consumers;
- public kernel enum に対する downstream wildcard-arm checks.

## Verification Plan

Closeout の refreshed bilingual audit verification:

- `doc/design/mizar-kernel/{en,ja}` に対する deterministic file-pair and
  companion-link checks;
- `cargo fmt --check`;
- `cargo clippy --all-targets --all-features -- -D warnings`;
- `cargo test`;
- `git diff --check`;
- explicit path staging 後の `git diff --cached --check`。
