# mizar-test Bilingual Documentation Sync Audit

> Canonical language: English. English canonical version:
> [../en/bilingual_sync_audit.md](../en/bilingual_sync_audit.md).

## Scope

task 13 は `doc/design/mizar-test/en/` 配下のすべての English canonical document と、
`doc/design/mizar-test/ja/` 配下の Japanese companion について paired-file sync
baseline を確立した。

task 15 は task 14 の architecture-22 matrix metadata work 後にその audit を
再実行する。この follow-up は task 15 までの task status、architecture-22
scenario id と equivalence class、planned/active gating、traceability record、
consumer-runner pacing、determinism coverage、module-spec contracts を対象にする。

task 21 は kernel F7 の訂正後経路 soundness 語彙について、関連する paired
document check を再実行する。この follow-up は同じ変更で更新する
`fail_soundness.md`、`layout.md`、`expectation_schema.md`、`todo.md`、
`00.crate_plan.md`、および kernel `soundness_argument.md` reference を対象にする。

この audit は language behavior、corpus expectations、既存 `.miz` fixtures、
expectation の意味を変更しない。

## Method

task 13 baseline は次を確認した。

- EN/JA file sets が 1 対 1 で対応している。
- 各 companion が同じ task status と completion notes を保持している。
- 各 module spec が同じ major section structure と content intent を保持している。
- public enum policy tables が同じ enum inventory と decision を列挙している。
- consumer-runner の `prepared/implemented` と `paced/open` ledger entries が一致している。

task 15 follow-up は次を確認した。

- `expectation_schema.md`、`harness.md`、`traceability.md`、`todo.md`、
  `00.crate_plan.md` に入った task 14 updates が Japanese companion と
  対応している。
- task 14 の scenario id と equivalence class は architecture 20 と
  architecture 22 が列挙する property と一致している。
- task 14 の row はすべて `planned` のままで、clean/incremental/parallel または
  cache-race execution の prepared consumer runner が確認できていないため、
  `active` は引き続き gate される。
- metadata-only `architecture22_matrix_001` anchor と
  `spec.en.architecture_22.regression_matrix.metadata` trace row は
  documentation-only coverage のままで、execution を捏造していない。
- `00.crate_plan.md` の source/spec audit ledger は、残る architecture-22 matrix
  work を `spec_gap` や `repo_metadata_conflict` ではなく、consumer-paced external
  dependency を伴う follow-up `test_gap` として記録している。
- task 10 は consumer-paced のままで、新しく準備済みの runner increment はない。

task 21 follow-up は次を確認した。

- 訂正後 `soundness.certificate.*` key、failure category、rejection reason、
  domain、phase が `fail_soundness.md` companions 間で一致する。
- certificate layout と expectation example は実装済み `tests/certificates/`
  corpus と訂正後 SAT-refutation vocabulary を使う。
- `todo.md` と `00.crate_plan.md` は task 21 complete を記録し、
  `doc/design/spec_coverage_audit.md` が unchanged であることを記録する。
- paired kernel `soundness_argument.md` は F7 を resolved として記録する。

## Pair Status

| Document | Synchronized content |
|---|---|
| `00.crate_plan.md` | task plan、source inventory、task 8-15 と task-21 completion status、architecture-22 matrix audit result、source-derived bridge boundary、corrected soundness vocabulary audit result。 |
| `README.md` | module index と crate boundary summary。 |
| `expectation_schema.md` | sidecar schema、origin metadata、corrected certificate soundness fields、public enum policy。 |
| `fail_soundness.md` | required soundness cases、corrected kernel rejection vocabulary、failure contract、validation rules。 |
| `harness.md` | public API、runner pacing ledger、determinism requirements、architecture-22 matrix reporting、tests。 |
| `layout.md` | directory layout、naming rules、corrected certificate rejection reasons、expected-result files、public enum cross-reference。 |
| `minimal_crate.md` | minimal metadata-only crate scope、CLI behavior、verification expectations。 |
| `miz_corpus.md` | corpus classes、size/review policy、provenance、stress、fuzz、property rules。 |
| `snapshot.md` | snapshot records、baseline/update policy、determinism helpers、public enum policy。 |
| `staged_model.md` | stage ids、admission rules、prerequisite policy、public enum policy。 |
| `todo.md` | ordered tasks、task 8-15 と task-21 completion notes、remaining architecture-22 follow-up boundary。 |
| `traceability.md` | manifest schema、coverage/status rules、prerequisite validation、architecture-22 matrix summary、public enum policy。 |
| `bilingual_sync_audit.md` | task 13 baseline、task 15 follow-up audit、English canonical companion。 |

## Result

現在の task scope では bilingual documentation drift は残っていない。Japanese
companions は task 21 の corrected-path vocabulary update までの mizar-test design
surface について English canonical documents と同期している。

task 15 の source/spec follow-up では、blocking な `spec_gap`、採用すべき
`repo_metadata_conflict`、language behavior change、または既存 expectation の
意味変更が必要な箇所は見つかっていない。残る architecture-22 matrix execution
work は、task 14 support が metadata/reporting-only であるため consumer-paced
follow-up work として記録する。

残る open work は意図的に task 15 の範囲外である。

- task 15 後に依頼される source-derived semantic bridge
- prepared source-derived clean/incremental/parallel/cache-race runner
  increment が存在した後の consumer crate における active architecture-22 matrix execution
- `00.crate_plan.md` に記録済みの non-bilingual source/design watches

## Verification

task 15 では documentation-focused checks を使った。

- `doc/design/mizar-test` の paired EN/JA file-set listing
- 各 paired document の heading-structure review
- task 14 と task 15 が complete であることの task-status review
- task 14 registry の 18 row に対する architecture 20/22 scenario coverage review
- `00.crate_plan.md` の source/spec audit ledger review
- `git diff --check`

task 21 はさらに commit 前に `cargo test -p mizar-test` と `git diff --check` を
使う。crate-local Rust checks は whole-document bilingual semantic equivalence を
証明しない。残るリスクはこの audit に通常伴う manual-review risk である。file set、
major sections、task status、recorded inventories、corrected-vocabulary metadata が
一致していても、sentence level の wording drift が残る可能性はある。
