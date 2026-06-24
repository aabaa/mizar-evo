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
観測されなかった。Task 18 の自己 hash は設計上まだ ledger で pending だったため、
この task で backfill する。

## ペア inventory

| Document | 確認した同期内容 | 結果 |
|---|---|---|
| `00.crate_plan.md` | Responsibility、out-of-scope boundary、authority order、known gaps/drift、task decomposition、hard gates、verification expectations。 | 同期済み。 |
| `vc_ir.md` | snapshot-local `VcId`、seed accounting、generated formula ownership、local context、premise、status、anchor、rendering、planned tests、public enum policy。 | 同期済み。 |
| `generator.md` | explicit-payload generation scope、利用不能な registration / algorithm payload boundary、local context、controlled unfolding、normalization handoff、task slice、planned tests、public enum policy。 | 同期済み。 |
| `discharge.md` | deterministic pre-ATP scope、supported classes、limit、evidence/explanation model、status interaction、no-erase ATP boundary、planned tests、public enum policy。 | 同期済み。 |
| `dependency_slice.md` | conservative slice inputs/outputs、dependency entry classes、unknown coverage、reusable fingerprint contract、planned tests、public enum policy。 | 同期済み。 |
| `source_spec_audit.md` | public module exports、public surface inventory、cross-module evidence、classified external/deferred follow-ups。 | 同期済み。 |
| `bilingual_sync_audit.md` | audit scope、method、pair inventory、classification、Task 19 sync edits。 | この paired Task 19 document により同期済み。 |
| `architecture_22_audit.md` | Task 20 architecture-22 identity correspondence、deterministic-discharge branch evidence、remaining external/deferred gaps、no-drift classification。 | Task 21 により同期済み。 |
| `task_ledger.md` | Task status、Task 20 までに利用可能な commit hash、review outcome、verification summary、deferred notes、handoff prompt。 | Task 21 で Task 20 hash を backfill し Task 21 audit checkpoint を記録した後に同期済み。 |
| `todo.md` | ordered task list、完了 task、残り Task 22 と closeout の scope、recommended verification、notes。 | Task 21 で architecture-22 follow-up audit を完了にした後に同期済み。 |

## 分類

Task 19 は新しい `spec_gap`、`test_gap`、`design_drift`、`source_drift`、
`source_undocumented_behavior`、`test_expectation_drift`、`boundary_violation`、
`repo_metadata_conflict` を記録しない。

既存の分類済み record は残る:

- `external_dependency_gap`: active `proof_verification` runner support と
  source-to-core / source-to-VC extraction seam は `mizar-test` に存在しない。
  Task 15 が deferred corpus obligation を記録済み。
- `external_dependency_gap`: `mizar-atp`、`mizar-kernel`、`mizar-proof`、
  `mizar-cache` は active workspace consumer ではない。そのため ATP
  translation、certificate acceptance、proof policy、cache lookup/reuse、
  artifact persistence は downstream に残る。
- `external_dependency_gap`: registration/redefinition/reduction details、
  call-precondition、branch、match、range-loop、collection-loop、term-only
  termination、partial termination、Pick non-emptiness、ghost-erasure、
  complete trace family、source-derived core formula payload、definition payload、
  quantified binder payload、source-derived obligation payload family について、
  upstream explicit/stable payload はまだ不完全である。
- `deferred`: proof-witness hash、ATP/kernel/proof/cache validation、
  artifact consumer、source-derived runner integration は、architecture-22 reuse を
  deterministic-discharge candidate key の外で受理する前に downstream に残る。
- `deferred`: Task 22 は module-boundary refactor gate、closeout は final quality
  review と crate-exit reporting を所有する。

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
