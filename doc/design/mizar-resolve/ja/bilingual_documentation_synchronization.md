# 二言語ドキュメント同期監査

> 正本は英語です。英語版:
> [../en/bilingual_documentation_synchronization.md](../en/bilingual_documentation_synchronization.md)。

状態: task R-028 audit complete; task R-029 scope re-run complete。

## 範囲

この監査は、`doc/design/mizar-resolve/en/` の各英語正本 design document と
`doc/design/mizar-resolve/ja/` の日本語 companion を比較する。確認対象は paired
filename、API list、task status、gap / deferred classification、behavior promise、
boundary statement、terminology、`mizar-resolve` task stream に関係する link である。

監査範囲は R-029 までの完了済み non-deferred resolver work と、R-024
`external_dependency_gap` deferral record である。この監査は
[source_spec_correspondence.md](./source_spec_correspondence.md) の source/spec
correspondence audit を置き換えない。また、`doc/spec`、`.miz` source、expectation
sidecar は変更しない。

## 結果

- 現在の英語 design file はすべて同名の日本語 companion を持ち、この監査も両言語
  directory に同じ paired file として追加した。
- public resolver API family、public enum forward-compatibility decision、task
  completion state、deferred / external dependency record、milestone handoff wording に
  残る英日不一致は見つからなかった。
- task status は、R-001〜R-023 完了、R-024 は R-G003
  `external_dependency_gap` として明示的に deferred、R-025〜R-029 完了として同期している。
- 既存 follow-up classification は同期している: R-G001 `spec_gap`、R-G002
  `test_gap`、R-G003 `external_dependency_gap` / deferred、R-G004
  `boundary_violation` risk、R-G005 resolved `design_drift`、R-G006
  `external_dependency_gap`、そして R-G002 の現在の具体的な精緻化である
  R-G007 `test_gap`。
- この監査により新しい `spec_gap`、`test_gap`、`design_drift`、`source_drift`、
  `source_undocumented_behavior`、`test_expectation_drift`、`boundary_violation`、
  `repo_metadata_conflict` は導入されていない。

## pair checklist

| 英語正本 document | 日本語 companion | 同期結果 |
|---|---|---|
| [00.crate_plan.md](../en/00.crate_plan.md) | [./00.crate_plan.md](./00.crate_plan.md) | responsibility、spec/test inventory、design/source inventory、gap table、R-024 deferral、R-027 audit result、R-028 audit result、R-029 refactor result、close-out handoff が同期している。 |
| [declarations.md](../en/declarations.md) | [./declarations.md](./declarations.md) | declaration shell kind、excluded / transparent node、visibility、recovery、identity / provenance、public enum policy が同期している。 |
| [env.md](../en/env.md) | [./env.md](./env.md) | `SymbolEnv` index family、contribution tracking、invalidation note、determinism、public enum policy が同期している。 |
| [imports.md](../en/imports.md) | [./imports.md](./imports.md) | import input/output、two-pass contract、path resolution、alias / export / cycle / unresolved policy、determinism、boundary note、public enum policy が同期している。 |
| [labels.md](../en/labels.md) | [./labels.md](./labels.md) | label scope family、proof-block scope、forward-reference policy、citation lookup、origin path、diagnostics / recovery、determinism、public enum policy が同期している。 |
| [names.md](../en/names.md) | [./names.md](./names.md) | name-use site、scope model、namespace-before-symbol lookup、visibility / shadowing、unresolved / ambiguous record、dot-chain finalization、diagnostics、public enum policy が同期している。 |
| [recovery.md](../en/recovery.md) | [./recovery.md](./recovery.md) | recovered syntax stage disposition、boundary rule、test intent が同期している。 |
| [resolved_ast.md](../en/resolved_ast.md) | [./resolved_ast.md](./resolved_ast.md) | top-level `ResolvedAst` shape、stable identity、node / name / label / import table、recovered shell、provenance、determinism、public enum policy が同期している。 |
| [source_spec_correspondence.md](../en/source_spec_correspondence.md) | [./source_spec_correspondence.md](./source_spec_correspondence.md) | R-027 の public API、behavior-boundary、task-requirement、follow-up record が、R-G002 と R-G007 の関係を含めて同期している。R-029 の moved-source scope re-run も同期している。 |
| [symbols.md](../en/symbols.md) | [./symbols.md](./symbols.md) | symbol-bearing shell、collection order、identity / origin、signature、duplicate / overload、visibility / export / summary policy、dependency relation、recovery / diagnostics、determinism、public enum policy が同期している。 |
| [todo.md](../en/todo.md) | [./todo.md](./todo.md) | ordered task state、deferral note、recommended verification、close-out handoff wording が同期している。 |
| [bilingual_documentation_synchronization.md](../en/bilingual_documentation_synchronization.md) | [./bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md) | この R-028 audit と R-029 scope re-run は、同じ scope、result、pair checklist、handoff note を両言語で記録している。 |
| [module_boundary_refactor.md](../en/module_boundary_refactor.md) | [./module_boundary_refactor.md](./module_boundary_refactor.md) | R-029 source-layout audit、private helper / test split list、re-run audit note、verification requirement が同期している。 |

## handoff

crate-wide close-out はこの監査を二言語同期状態の baseline として扱う。close-out report で
新しい design file を追加する場合は、両言語 companion を同時に追加する。挙動 cleanup、
public API change、新しい diagnostics は完了済み refactor gate の範囲外であり、独立した
spec/test authority を要求する。
