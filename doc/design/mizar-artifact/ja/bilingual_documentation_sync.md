# mizar-artifact 二言語ドキュメント同期監査

> 正本は英語です。英語版:
> [../en/bilingual_documentation_sync.md](../en/bilingual_documentation_sync.md)。

## 範囲

task 21 は task 20 後の bilingual documentation set を監査する。
`doc/design/mizar-artifact/en/` 配下の英語文書が正本であり、
`doc/design/mizar-artifact/ja/` 配下の日本語文書は意味を同期して保つ companion
である。この task は documentation-only であり、schema、source behavior、tests、
diagnostics、public API は変更しない。

分類結果:

- `design_drift`: bilingual documentation set では見つからない。
- `deferred`: この監査では新たに開かない。
- `external_dependency_gap`: [source_spec_correspondence.md](./source_spec_correspondence.md)
  から変わらない。

## ペア棚卸し

すべての英語正本文書は同名の日本語 companion を持ち、すべての日本語 companion
は同名の英語正本文書を持つ。

| 英語正本 | 日本語 companion | 同期結果 |
|---|---|---|
| [../en/00.crate_plan.md](../en/00.crate_plan.md) | [00.crate_plan.md](./00.crate_plan.md) | task 21 までの task result、active gap、verification expectation、exit criteria が同期している。 |
| [../en/todo.md](../en/todo.md) | [todo.md](./todo.md) | task 21 までの task status、task 17 deferral、task 18-20 status note、task 22 handoff scope が同期している。 |
| [../en/store.md](../en/store.md) | [store.md](./store.md) | Store ownership、canonical JSON、schema-version policy、hash separation/exclusion、atomic write、validating read、public-enum policy、implementation staging が同期している。 |
| [../en/module_summary.md](../en/module_summary.md) | [module_summary.md](./module_summary.md) | summary shape、identity/export/label/lexical/reexport/dependency field、interface hash、ordering、reader/writer rule、public-enum policy、implementation status が同期している。 |
| [../en/registration_summary.md](../en/registration_summary.md) | [registration_summary.md](./registration_summary.md) | registration shape、hash domain、trace/dependency reference、registration interface hash、ordering、reader/writer rule、public-enum policy、implementation status が同期している。 |
| [../en/proof_witness.md](../en/proof_witness.md) | [proof_witness.md](./proof_witness.md) | ownership、witness reference shape、hash domain、witness path、resident-set discipline、ordering、reader/writer rule、public-enum policy、implementation boundary が同期している。 |
| [../en/verified_artifact.md](../en/verified_artifact.md) | [verified_artifact.md](./verified_artifact.md) | ownership、top-level shape、export、expression metadata、obligation/witness、diagnostic、provenance、hash domain/participation、ordering、reader/writer rule、public-enum policy、implementation status が同期している。 |
| [../en/manifest.md](../en/manifest.md) | [manifest.md](./manifest.md) | manifest scope、file/version、top-level shape、module/witness/development entry、hash domain、ordering、reader requirement、transaction protocol、recovery、public-enum policy、implementation status が同期している。 |
| [../en/source_spec_correspondence.md](../en/source_spec_correspondence.md) | [source_spec_correspondence.md](./source_spec_correspondence.md) | task 20 scope、classification result、public API trace、promised behavior trace、remaining gaps、verification note が同期している。 |
| [../en/bilingual_documentation_sync.md](../en/bilingual_documentation_sync.md) | [bilingual_documentation_sync.md](./bilingual_documentation_sync.md) | task 21 scope、pair inventory、audit note、verification note が同期している。 |

## 監査メモ

- language directory は file name が対応しており、正本/companion link も相互に
  存在する。
- `doc/design/mizar-artifact/` 配下に未解決の "needs synchronization" marker、
  placeholder、bilingual TODO は残っていない。
- 日本語 companion 内の mixed English technical label は、stable crate/module/API
  terminology と一致する箇所では意図的に維持している。
- 残る upstream gap は documentation drift ではない。crate plan、TODO、
  source/spec audit で `external_dependency_gap` または `deferred` として明示的に
  分類されている。

## Verification

この task は documentation-only である。必要な verification は、audit file と
status update を stage した後の `git diff --check`。
