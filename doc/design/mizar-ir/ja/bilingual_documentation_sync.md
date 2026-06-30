# mizar-ir Bilingual Documentation Sync

> 正本は英語です。英語版:
> [../en/bilingual_documentation_sync.md](../en/bilingual_documentation_sync.md)。

## Scope

この task-17 audit は、`doc/design/mizar-ir/en/` 以下の各 English canonical
document を `doc/design/mizar-ir/ja/` 以下の対応する Japanese companion と比較する。
filename parity、heading parity、task status parity、gap classification parity、
および `mizar-ir` が proof authority、`mizar-cache` identity、downstream
driver/diagnostic integration を所有しないための crate boundary statement を確認する。

この audit では unresolved blocking/high bilingual drift は見つからなかった。Japanese
companion なしで残された English canonical content もない。

## Audit Checks

| Check | Evidence | Result |
|---|---|---|
| File coverage | 両言語 directory は `00.crate_plan.md`、`cache_adapter.md`、`identity.md`、`projection.md`、`publisher.md`、`source_spec_correspondence.md`、`storage.md`、`todo.md` を含む。この audit は `bilingual_documentation_sync.md` を両方に追加する。 | Synchronized。 |
| Heading parity | 既存 file ごとの heading count は一致する: crate plan 27、cache adapter 13、identity 10、projection 15、publisher 11、source/spec audit 6、storage 12、TODO 11。この audit pair も対応する section を持つ。 | Synchronized。 |
| Task status parity | TODO task state は揃っている: task 1-17 は complete、task 18-19 は open のまま。 | Synchronized。 |
| Gap classification parity | crate plan の `IR-G-001` から `IR-G-009`、source/spec の `IR-G-004` から `IR-G-009` は両言語に存在し、class と disposition が対応している。 | Synchronized。 |
| Ownership boundaries | 両言語 set は、`mizar-ir` が internal IR storage、snapshot handle、phase output reference、publication、cache adaptation、projection、snapshot replacement を所有する一方、proof acceptance、trusted status、kernel acceptance、`CacheKey`、dependency fingerprint、proof-reuse validation は所有しないと述べている。 | Synchronized。 |
| External dependency gaps | 両言語 set は、real `mizar-driver`、`mizar-diagnostics`、producer projection payload、artifact publication token、system equivalence を placeholder API ではなく deferred external gap として分類している。 | Synchronized。 |
| Cache and snapshot freshness | 両言語 set は、handle reconstruction 前の fail-closed cache miss behavior を要求し、obsolete/open output を current artifact として publish することを禁止している。 | Synchronized。 |
| Projection boundary | 両言語 set は、raw `SurfaceAst`、`TypedAst`、`CoreIr`、`ControlFlowIr`、`VcIr`、`AtpProblem`、kernel-internal state、storage handle、inline proof-witness payload の公開を禁止している。 | Synchronized。 |

## Classified Drift

mizar-ir document set に current な bilingual 固有の `spec_gap`、`source_drift`、
`test_expectation_drift`、`boundary_violation`、`repo_metadata_conflict` は見つからなかった。
既存の crate-plan gap row は `design_drift` を含む protocol class を保持しており、
それらの class は両言語で同期している。deferred integration は既存の crate plan と
module specification に `external_dependency_gap` risk tag として記録済みである。

## Audit Result

Task 17 は、この audit record、crate-plan inventory entry、TODO status update 以外の
synchronized content change なしで close する。Task 18 は、この bilingual scope を再利用しつつ、
architecture-22 の publisher、cache-adapter、snapshot-replacement contract に特化して確認する。
