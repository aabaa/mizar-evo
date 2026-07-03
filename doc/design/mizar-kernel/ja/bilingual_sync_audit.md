# Bilingual Sync Audit: mizar-kernel

> 正本は英語です。英語版:
> [../en/bilingual_sync_audit.md](../en/bilingual_sync_audit.md)。

## Scope And Authority

Task 21 は `doc/design/mizar-kernel/en/` と `doc/design/mizar-kernel/ja/` の
paired document すべてを audit した。Task 22 は module-boundary audit document
と整理済み task-ledger handoff section を含めるため、この audit を更新した。
Closeout task は paired crate exit report と Task 22 hash backfill を含めるため、
もう一度更新する。Tasks 23-24 は formula/SAT correction specs と SAT dependency
audit を含めるため、この audit を更新する。Tasks 25-26 は formula-evidence parser
と source-backed SAT encoding implementation を含めるため、この audit を更新する。
Task 27 は trusted SAT checker wrapper、exact dependency/lockfile policy、
read-only encoded SAT problem boundary、public enum/source-spec inventory を含めるため、
この audit を更新する。Task 28 は SAT-backed kernel evidence service path と checker
API inventory のために refresh する。Task 29 は explicit legacy audit gate、
post-correction closeout report、mizar-vc handoff のために refresh する。英語は canonical のままである。日本語
companion は英語文書に同期する。ただし、commit hash や task status の欠落のように
task-local bookkeeping omission が明らかに paired であり、同じ rationale で両言語を
修正できる場合は例外とする。Japanese-only semantic drift を、別の classified finding
なしに英語へ昇格してはならない。

これは、legacy certificate checking を explicit migration/audit policy の背後に gate する
task-29 source change のための documentation audit である。`doc/spec`、`.miz`
fixture、expectation、SAT/ATP/proof search、premise selection、overload resolution、
cluster search、implicit coercion insertion、fallback inference、global mutable state、
downstream ATP/proof/cache/artifact integration は変更しない。

## Pair Inventory

現在のすべての English design document は同名の Japanese companion を持ち、
すべての Japanese design document は同名の English canonical file を持つ。

| File | Companion links | Heading count | Table row count | Sync result |
|---|---|---:|---:|---|
| `00.crate_plan.md` | EN -> JA and JA -> EN | 8 / 8 | 64 / 64 | post-closeout task rows 追加後に synchronized. |
| `bilingual_sync_audit.md` | EN -> JA and JA -> EN | 6 / 6 | 35 / 35 | task-29 migration-audit bookkeeping 後に synchronized. |
| `certificate_parser.md` | EN -> JA and JA -> EN | 15 / 15 | 29 / 29 | Synchronized. |
| `checker.md` | EN -> JA and JA -> EN | 15 / 15 | 15 / 15 | task-29 legacy audit policy gate 向けに refreshed. |
| `clause.md` | EN -> JA and JA -> EN | 12 / 12 | 5 / 5 | Synchronized. |
| `crate_exit_report.md` | EN -> JA and JA -> EN | 12 / 12 | 78 / 78 | task-29 post-correction closeout と mizar-vc handoff 向けに refreshed. |
| `formula_evidence.md` | EN -> JA and JA -> EN | 13 / 13 | 0 / 0 | task-29 parsed evidence read-only boundary 向けに refreshed. |
| `module_boundary_audit.md` | EN -> JA and JA -> EN | 6 / 6 | 13 / 13 | Synchronized. |
| `public_enum_policy.md` | EN -> JA and JA -> EN | 5 / 5 | 0 / 0 | task-27 SAT checker result enum addition 後に synchronized. |
| `rejection.md` | EN -> JA and JA -> EN | 14 / 14 | 32 / 32 | task-27 SAT wrapper failure mapping 向けに refreshed. |
| `resolution_trace.md` | EN -> JA and JA -> EN | 12 / 12 | 15 / 15 | Synchronized. |
| `sat_checker.md` | EN -> JA and JA -> EN | 6 / 6 | 0 / 0 | task-27 wrapper API と unsupported step-budget policy 向けに refreshed. |
| `sat_dependency_audit.md` | EN -> JA and JA -> EN | 13 / 13 | 32 / 32 | task-27 exact dependency と no-callback branch 向けに refreshed. |
| `sat_encoding.md` | EN -> JA and JA -> EN | 8 / 8 | 0 / 0 | task-27 read-only encoded problem boundary 向けに refreshed. |
| `soundness_argument.md` | EN -> JA and JA -> EN | 20 / 20 | 19 / 19 | 実装前健全性監査で追加; 作成時点で synchronized. |
| `source_spec_audit.md` | EN -> JA and JA -> EN | 18 / 18 | 29 / 29 | task-29 legacy audit gate 後に synchronized. |
| `substitution_checker.md` | EN -> JA and JA -> EN | 15 / 15 | 17 / 17 | Synchronized. |
| `task_ledger.md` | EN -> JA and JA -> EN | 2 / 2 | 33 / 33 | Task 28 hash backfill と Task 29 completion row 後に synchronized. |
| `todo.md` | EN -> JA and JA -> EN | 13 / 13 | 11 / 11 | Task 29 completion status update 後に synchronized. |

Count check は完全な translation proof ではない。下の semantic check を補助する
drift screen である。

## Semantic Sync Checks

| Area | Result |
|---|---|
| Canonical/companion headers | すべての English file は Japanese companion を指し、すべての Japanese file は English canonical file を指す。 |
| Task status and sequencing | Tasks 0-29 は complete として一貫する。Task 28 commit `43674a221dd5f43259c480846db7428f85ac9386` は backfill 済みであり、task 29 は complete で、その commit は後続 backfill 待ちである。 |
| Task 21 bookkeeping | `73a919c16b48da82038fd7267e86e1a844cb4c6f` は完了済み Task 21 commit であり、両 ledger に backfill される。 |
| Task 22 bookkeeping | `814e47bb9aaaff75ebfe4cc1be10d2eb4618498b` は完了済み Task 22 commit であり、両 ledger に backfill される。 |
| Closeout report inventory | `crate_exit_report.md` は paired であり、同じ hard gates、task commits、residual gaps、quality score、verification plan、next-crate handoff を英語/日本語で記録する。 |
| Closeout handoff | 両 ledger は closeout to next-crate handoff だけを保持する。 |
| Public enum inventory | `public_enum_policy.md` は英語/日本語で同じ `public-enum-inventory` block を使い、`sat_checker::SatCheckResult` を含む。正確な inventory validation の source は `crates/mizar-kernel/tests/lint_policy.rs` の executable guard のままである。 |
| Source/spec audit inventory | `source_spec_audit.md` は英語/日本語で module、public item、formula-evidence/SAT-checker/checker-service traceability、gap、verification sections が対応している。 |
| Module-boundary audit inventory | `module_boundary_audit.md` は paired であり、同じ move-only test-module split、drift classification、verification plan を英語/日本語で記録する。 |
| Trust Statement wording | 各 module は task-20 trusted-kernel prohibition wording を持つ paired `## Trust Statement` sections を維持している。 |
| Gap/deferred classification | 残る external integration は `external_dependency_gap` または `deferred` として分類されたままであり、この task は placeholder integration を追加しない。 |
| Repository metadata conflicts | Closeout で `repo_metadata_conflict` は観測されない。 |

## Remaining Gaps

Task 29 は `check_kernel_certificate` を explicit audit policy の背後に gate し、legacy
normal-policy acceptance gap を閉じる。External producer / consumer gap は閉じない。
以下は module spec と `source_spec_audit.md` に残る:

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

Task 29 の refreshed bilingual audit verification:

- `doc/design/mizar-kernel/{en,ja}` に対する deterministic file-pair and
  companion-link checks;
- `cargo fmt --check`;
- `cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings`;
- `cargo test -p mizar-kernel`;
- `mizar-core`、`mizar-vc`、`mizar-artifact`、`mizar-checker` の boundary tests;
- 実用上可能なら broad `cargo clippy --all-targets --all-features -- -D warnings` と
  `cargo test`;
- `git diff --check`;
- explicit path staging 後の `git diff --cached --check`。
