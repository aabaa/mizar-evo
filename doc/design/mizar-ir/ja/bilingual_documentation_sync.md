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
| File coverage | 両言語 directory は paired crate plan、TODO、module spec、dispatch input spec、source/spec audit、bilingual audit、architecture-22 audit、module-boundary gate、crate exit report を含む。 | task 20 scope で synchronized。 |
| Heading parity | paired task-20 document は matching section を保つ。既存 bilingual pair は dispatch input と closeout 更新後も揃っている。 | Synchronized。 |
| Task status parity | TODO task state は揃っている: task 1-20 は complete。 | task 20 scope で synchronized。 |
| Gap classification parity | crate plan の `IR-G-001` から `IR-G-012`、source/spec の `IR-G-004` から `IR-G-012` は両言語に存在し、class と disposition が対応している。 | task 20 scope で synchronized。 |
| Ownership boundaries | 両言語 set は、`mizar-ir` が internal IR storage、snapshot handle、phase output reference、publication、cache adaptation、projection、snapshot replacement を所有する一方、proof acceptance、trusted status、kernel acceptance、`CacheKey`、dependency fingerprint、proof-reuse validation は所有しないと述べている。 | Synchronized。 |
| External dependency gaps | 両言語 set は、remaining producer dispatch、diagnostics rendering、producer projection payload、artifact publication token、cache-compatibility wiring、LSP conversion、system equivalence を placeholder API ではなく deferred external gap として分類している。 | Synchronized。 |
| Cache and snapshot freshness | 両言語 set は、handle reconstruction 前の fail-closed cache miss behavior を要求し、obsolete/open output を current artifact として publish することを禁止している。 | Synchronized。 |
| Projection boundary | 両言語 set は、raw `SurfaceAst`、`TypedAst`、`CoreIr`、`ControlFlowIr`、`VcIr`、`AtpProblem`、kernel-internal state、storage handle、inline proof-witness payload の公開を禁止している。 | Synchronized。 |

## Classified Drift

mizar-ir document set に current な bilingual 固有の `spec_gap`、`source_drift`、
`test_expectation_drift`、`boundary_violation`、`repo_metadata_conflict` は見つからなかった。
既存の crate-plan gap row は `design_drift` を含む protocol class を保持しており、
それらの class は両言語で同期している。deferred integration は既存の crate plan と
module specification に `external_dependency_gap` risk tag として記録済みである。
task 20 は `dispatch_input.md` を追加し、古い driver 不在という表現を、missing
driver crate ではなく現在の downstream integration gap として再分類する。

## Audit Result

Task 20 は bilingual sync invariant を保つ。新しい dispatch input boundary、closeout
status、verification record、gap classification は両言語 directory に反映されている。
