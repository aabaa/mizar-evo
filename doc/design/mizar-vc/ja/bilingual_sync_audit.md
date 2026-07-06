# Bilingual Documentation Sync Audit: mizar-vc

> 正本言語: 英語。英語正本:
> [../en/bilingual_sync_audit.md](../en/bilingual_sync_audit.md)。

Task 19 は source/spec correspondence audit 後の `mizar-vc` design
documentation pair を監査する。この task は Rust source、`.miz` fixture、
expectation、language specification、traceability metadata、runner support、
downstream ATP/kernel/proof/cache integration を変更しない。

## 範囲と方法

監査対象は `doc/design/mizar-vc/en/` 配下の現在の Markdown document と、
`doc/design/mizar-vc/ja/` 配下の companion の全ペアである。各ペアについて、
次を確認した:

- 両 language directory に同じ filename が存在すること;
- 各 document の先頭に canonical / companion link があること;
- module responsibility、inputs/outputs、behavior rule、status / policy
  semantics、dependency / fingerprint rule、planned tests、public enum
  policy table、audit inventory、task ledger summary、todo task wording、
  follow-up / deferred classification の実質的な意味が同期していること;
- 既知の `external_dependency_gap` と `deferred` record が、黙って解消されたり
  弱められたりせず保持されていること。

日本語 companion は自然な翻訳を使ってよく、Rust identifier、phase name、task
name は英語のままでもよい。同期規則は semantic なものであり、companion は英語
正本に対して normative な意味を省略、弱化、追加してはならない。

結果: 現在の document pair はすべて存在し、意味内容は同期している。意味を変える
bilingual drift、欠けている companion、古い status、`repo_metadata_conflict` は
観測されなかった。Task 22 の自己 hash は closeout task で backfill する。
Closeout の自己 hash は commit 自身に埋め込めないため final user handoff に記録する。

## ペア inventory

| Document | 確認した同期内容 | 結果 |
|---|---|---|
| `00.crate_plan.md` | Responsibility、out-of-scope boundary、authority order、known gaps/drift、task decomposition、hard gates、verification expectations。 | 同期済み。 |
| `vc_ir.md` | snapshot-local `VcId`、seed accounting、generated formula ownership、local context、premise、status、anchor、rendering、planned tests、public enum policy。 | 同期済み。 |
| `generator.md` | explicit-payload generation scope、利用不能な registration / algorithm payload boundary、local context、controlled unfolding、normalization handoff、task slice、planned tests、public enum policy。 | 同期済み。 |
| `discharge.md` | deterministic pre-ATP scope、supported classes、limit、evidence/explanation model、status interaction、no-erase ATP boundary、planned tests、public enum policy。 | 同期済み。 |
| `dependency_slice.md` | conservative slice inputs/outputs、dependency entry classes、unknown coverage、reusable fingerprint contract、task-26 kernel-evidence identity integration、planned tests、public enum policy。 | Task 26 update により同期済み。 |
| `kernel_evidence_handoff.md` | producer-side formula/substitution evidence handoff mapping、禁止される backend/legacy material、gap classification、task-25 builder public enum policy、resolved task-26 reuse-identity gap、task-27 explicit polarity contract、post-task-27 kernel handoff draft。 | Tasks 24-27 により同期済み。 |
| `source_spec_audit.md` | public module exports、public surface inventory、cross-module evidence、explicit goal-polarity correspondence、classified external/deferred follow-ups。 | task 27 update により同期済み。 |
| `bilingual_sync_audit.md` | audit scope、method、pair inventory、classification、Task 19/21/22/closeout/24/25/26/27 sync edits。 | この paired audit document により同期済み。 |
| `architecture_22_audit.md` | Task 20 architecture-22 identity correspondence、deterministic-discharge branch evidence、remaining external/deferred gaps、no-drift classification。 | Task 21 により同期済み。 |
| `module_boundary_audit.md` | Task 22 source-layout line count、module-boundary review、必須 split なしの判断、任意 maintenance deferral。 | Task 22 により同期済み。 |
| `crate_exit_report.md` | final status、quality score、hard gates、task 26 までの task commit、verification、review outcome、remaining deferred/external items、next-crate handoff draft。 | task 27 による task-26 commit hash backfill で同期済み。task 27 は post-closeout follow-up のまま。 |
| `task_ledger.md` | Task status、task 26 までに利用可能な commit hash、complete task-27 review/verification evidence、deferred notes、post-task-27 handoff draft。 | task 27 update により同期済み。task 27 self-hash は commit が存在した後に backfill する。 |
| `todo.md` | ordered task list、task 27 までの完了 task、closeout status、recommended verification、notes。 | task 27 update により同期済み。 |

## 分類

Task 19 は新しい `spec_gap`、`test_gap`、`design_drift`、`source_drift`、
`source_undocumented_behavior`、`test_expectation_drift`、`boundary_violation`、
`repo_metadata_conflict` を記録しなかった。Task 22 update は module-boundary pair を
inventory に追加しつつ、この分類を維持する。closeout は paired exit report を追加しながら
同じ no-drift classification を維持する。

既存の分類済み record は残る:

- `external_dependency_gap`: active `proof_verification` runner support と
  source-to-core / source-to-VC extraction seam は `mizar-test` に存在しない。
  Task 15 が deferred corpus obligation を記録済み。
- `external_dependency_gap` / `deferred`: `mizar-kernel` は現在 checker-side
  formula/substitution evidence acceptance path を所有し、`mizar-vc` は explicit-payload
  producer-side handoff builder と reuse identity integration を所有するが、ATP candidate
  production、proof/cache consumer、artifact witness consumer は incomplete のままである。
  ATP translation、proof policy、cache lookup/reuse、artifact persistence は downstream に残る。
- `external_dependency_gap`: registration/redefinition/reduction details、
  call-precondition、branch、match、range-loop、collection-loop、term-only
  termination、partial termination、Pick non-emptiness、ghost-erasure、
  complete trace family、source-derived core formula payload、definition payload、
  quantified binder payload、source-derived obligation payload family について、
  upstream explicit/stable payload はまだ不完全である。
- `deferred`: proof-witness hash、ATP/kernel/proof/cache validation、artifact consumer、
  source-derived runner integration は、architecture-22 reuse を deterministic-discharge と
  current kernel-evidence handoff identity candidate key の外で受理する前に downstream に残る。
- `deferred`: 大きい `vc_ir`、`generator`、`dependency_slice` implementation file 内の
  optional private helper / test split は、実施する場合には将来の move-only maintenance task
  として残る。final quality review と crate-exit status は
  [crate_exit_report.md](./crate_exit_report.md) に記録済み。

## Task 19 の同期編集

この task は paired bilingual sync audit document を追加し、paired ledger で Task
18 commit hash を backfill し、Task 19 の review / verification outcome を記録し、
paired todo で Task 19 を完了にする。

他の paired content に同期編集は不要だった。

## Task 21 の同期編集

Task 21 は paired architecture-22 audit document を追加し、Task 20 identity contract を
英語正本 document と日本語 companion の間で再確認する。paired ledger で Task 20 commit
hash を backfill し、Task 21 の review / verification outcome を記録し、paired todo で
Task 21 を完了にする。残る architecture-22 gap は未追跡 drift ではなく
external/deferred として分類済みであることも記録する。

## Task 22 の同期編集

Task 22 は paired module-boundary audit document を追加し、source layout を英語正本の
module spec、日本語 companion、internal crate-layout guidance に照らして再確認する。
paired ledger で Task 21 commit hash を backfill し、Task 22 の review / verification
outcome を記録し、paired todo で Task 22 を完了にする。optional private helper / test split
は crate-exit blocker ではなく将来の move-only maintenance work であることも記録する。

## Closeout の同期編集

closeout は paired crate exit report を追加し、paired ledger で Task 22 commit hash を
backfill し、final quality review score 94/100 と passing broad workspace verification を
記録し、paired todo に closeout status を追加する。英語正本 report と日本語 companion は
実質的に同期済みである。

## Task 24 の同期編集

Task 24 は paired kernel evidence handoff specification を追加し、pair inventory に
追加し、`mizar-kernel` task 23-29 後の状態に合わせて closeout 時点の古い kernel gap
classification を更新し、paired todo で Task 24 を完了にし、paired ledger に task-25
handoff prompt を記録する。英語正本 document と日本語 companion は意味的に同期済みで、
Rust source は変更しない。

## Task 25 の同期編集

Task 25 は新しい Rust builder に合わせ、paired source/spec、todo、plan、exit-report、
ledger、kernel evidence handoff document を更新する。両言語で
`kernel_evidence_handoff` public enum policy を記録し、paired todo で Task 25 を完了にし、
paired ledger で Task 24 hash を backfill し、task-26 handoff prompt を記録する。英語正本
document と日本語 companion は意味的に同期済みである。

## Task 26 の同期編集

Task 26 は reuse identity integration に合わせ、paired dependency-slice、
kernel-evidence handoff、todo、plan、exit-report、ledger、source/spec audit record を
更新する。両言語とも、現在の canonical kernel evidence handoff hash が dependency-slice
fingerprint と proof-reuse candidate key に参加すること、current handoff がない legacy reuse は
fail closed すること、downstream proof/cache/artifact consumer は
`external_dependency_gap` / `deferred` のままであることを記録する。

## Task 27 の同期編集

Task 27 は paired kernel-evidence handoff、source/spec audit、todo、plan、ledger、
bilingual sync audit、mizar-kernel soundness argument record を explicit producer-side
goal polarity 向けに更新する。両言語は、現在の proof-obligation handoff が
`AssertFalseForRefutation` を要求し、canonical package assembly 前に
`AssertTrueForConsistency` を拒否し、trusted checker-side B4/F1 acceptance binding を
`mizar-kernel` task 30 に残すことを記録する。
