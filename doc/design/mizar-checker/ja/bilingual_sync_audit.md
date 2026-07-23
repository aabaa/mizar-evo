# 二言語ドキュメント同期監査: mizar-checker

> 正本は英語です。英語版:
> [../en/bilingual_sync_audit.md](../en/bilingual_sync_audit.md)。

task 33 は checker design documentation の English canonical document と
Japanese companion を監査する。checker source behavior、public API、`.miz`
fixture、expectation は変更しない。

## 同期の定義

task 33 で pair が同期済みであるとは、以下をすべて満たすことをいう:

- English file と Japanese file が同じ filename で両方存在する。
- English file は Japanese companion を指し、Japanese file は English canonical
  file を指す。
- top-level document intent、task status、module table、task row、MC-G id、
  public enum policy row、source/spec inventory row、cross-link は、それらの構造が
  存在する場合に揃っている。
- localization-only wording、翻訳済み heading、Japanese/English が混在する technical
  term は、同じ意図を保つ限り許容する。
- sync debt は `none` と記録する。将来 `none` 以外の値を使う場合、task 33 を
  完了状態に保つには具体的な理由と owning follow-up task が必要である。

結果: この task 後、checker design directory に既知の bilingual sync debt は残らない。

## Pair Inventory

| Pair | EN companion | JA companion | Comparison basis | Sync debt |
|---|---|---|---|---|
| `00.crate_plan.md` | `../ja/00.crate_plan.md` | `../en/00.crate_plan.md` | crate status、responsibility、authority refs、test coverage、design/source inventory、MC-G tables、task decomposition、forbidden behavior、exit criteria | none |
| `binding_env.md` | `../ja/binding_env.md` | `../en/binding_env.md` | purpose/boundary、context and binding tables、lookup/reserve/closure behavior、diagnostics、public enum policy、task classification | none |
| `bilingual_sync_audit.md` | `../ja/bilingual_sync_audit.md` | `../en/bilingual_sync_audit.md` | pair inventory、synchronization definition、task classification、completion decision | none |
| `cluster_trace.md` | `../ja/cluster_trace.md` | `../en/cluster_trace.md` | authority/scope、trace model、cluster/reduction steps、determinism、bounds/failures、public enum policy、deferred inputs | none |
| `crate_exit_report.md` | `../ja/crate_exit_report.md` | `../en/crate_exit_report.md` | result、scope、task commit、hard gate、score breakdown、deferred item、verification、handoff | none |
| `module_boundary_audit.md` | `../ja/module_boundary_audit.md` | `../en/module_boundary_audit.md` | split gate、source layout inventory、task classification、completion decision | none |
| `overload_resolution.md` | `../ja/overload_resolution.md` | `../en/overload_resolution.md` | phase-8 boundary、site/candidate collection、template expansion、viability、specificity、selection/views、diagnostics、public enum policy、deferred gaps | none |
| `payload_family_decomposition.md` | `../ja/payload_family_decomposition.md` | `../en/payload_family_decomposition.md` | Task-247 authority/baseline、Tasks 248-264/269-279 scope/dependency/gate/consumer、Task-10 runner increment、literal Task-49 24-fixture reconciliation mapping、disagreement class、exit criteria | none |
| `registration_resolution.md` | `../ja/registration_resolution.md` | `../en/registration_resolution.md` | registration model、pending/activated database、validation、existential gates、cluster/reduction handoff、diagnostics、public enum policy、gap table | none |
| `resolved_typed_ast.md` | `../ja/resolved_typed_ast.md` | `../en/resolved_typed_ast.md` | responsibility、inputs、data shape、metadata/summaries、overload/coercion/cluster tables、failure/recovery、public enum policy、deferred gaps | none |
| `semantic_spec_audit.md` | `../ja/semantic_spec_audit.md` | `../en/semantic_spec_audit.md` | audit scope、severity legend、findings index/details、adversarial corpus table、traceability requirement ids、TODO impact | none |
| `source_spec_audit.md` | `../ja/source_spec_audit.md` | `../en/source_spec_audit.md` | public surface inventory、behavior/test correspondence、MC-G reconciliation、task classification | none |
| `source_context.md` | `../ja/source_context.md` | `../en/source_context.md` | Task-248 authority/boundary、projection model、validation/recovery/atomicity、determinism、coverage、public enum policy | none |
| `source_type.md` | `../ja/source_type.md` | `../en/source_type.md` | Task-249 authority/boundary、flat application/expression/argument model、environment/arena/graph/provenance validation、ownership、consumer、exclusion、public enum policy | none |
| `todo.md` | `../ja/todo.md` | `../en/todo.md` | module implementation table、prerequisites、resolved decisions、ordered task list、task statuses、verification、notes | none |
| `typed_ast.md` | `../ja/typed_ast.md` | `../en/typed_ast.md` | purpose/boundary、top-level shape、arena/context/type/fact/coercion/obligation/diagnostic tables、public enum policy、task classification | none |
| `type_checker.md` | `../ja/type_checker.md` | `../en/type_checker.md` | phase-6 boundary、normalization、declaration checking、inference、coercions/obligations、fact queries、diagnostics、determinism、public enum policy、task classification | none |

## Task 33 Classification

| Class | Evidence | Action |
|---|---|---|
| `spec_gap` | この audit は language specification behavior を変更しない。 | spec edit なし。 |
| `test_gap` | task は documentation sync である。実行可能 coverage は file pairing と audit row を検査する lint-policy guard。 | `.miz` fixture は追加しない。 |
| `design_drift` | Pair inventory、companion link、task status row、MC-G row、public enum policy row、source/spec audit row は現在の checker docs で同期済み。 | audit を記録し、future drift を guard する。 |
| `source_drift` | Source behavior は変更しない。 | lint-policy test 以外の source/API edit はない。 |
| `source_undocumented_behavior` | 該当なし。source/spec public-surface audit は task 32 が所有する。 | source correspondence record として task 32 audit を維持する。 |
| `external_dependency_gap` | 新規なし。既存 checker external gap は crate plan と source/spec audit に記録済み。 | 新規 deferral なし。 |
| `deferred` | task 33 では bilingual sync debt を defer しない。 | future sync debt を受け入れるには理由と owner を明記する。 |

## Completion Decision

task 33 は、この English audit と Japanese companion、crate plan / todo update、
lint-policy bilingual sync guard が同じ commit に含まれた時点で完了する。task 33
単体では crate completion を主張しない。task 34 と closeout task はすでに
module-boundary refactor gate と crate exit report を記録している。

Task 247は新しいsource-payload decomposition authorityについてpaired-file
inventoryを再実行した。英日graph row、blocked gate、Task-10 consumer increment、
literal Task-49 24-fixture reconciliation mapping、no-credit boundaryは同期され、
新規sync debtはない。
既存exact-pair guardが新filename pairを発見するためsource/lint-policy変更は不要。

Core Task 32はpaired payload-family decomposition noteを再確認する。両言語は
algorithm producer/loweringをchecker task IDの捏造なしにjoint Core Tasks 42-47へ
割り当て、Gates A1/S1を保持する。
