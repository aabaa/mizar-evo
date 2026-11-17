# mizar-artifact Phase 15 Emission Reevaluation

> 正本は英語です。英語版:
> [../en/phase15_emission_reevaluation.md](../en/phase15_emission_reevaluation.md)。

## 範囲

Task 17 の一般 `VerifiedArtifact` emission は deferred のままとする。
実 producer projection と witness publication が一切存在しないという従来の記録は、
bounded genuine source-theorem reflexivity 経路については古い (`design_drift`)。
言語、proof acceptance、coverage credit は変更しない。

## 実装済みの bounded 経路

[Driver genuine consumer](../../mizar-driver/ja/frontend_adapter.md#genuine-reflexivity-fileartifact-consumer)
は実 captured source/Core/Preparation/VC と sealed Kernel/status を Proof の genuine FILE producer に接続する。
対応する Disk singleton reflexivity では [explicit built-in enclosing profile](verified_artifact.md#explicit-built-in-enclosing-profile)
により DischargedBuiltin と trusted used_axioms を含む `VerifiedArtifact` を投影する。
既存の [manifest transaction](manifest.md#manifest-transaction-protocol) が atomic manifest commit で main を公開し、
対応する main/FILE readback と committed reachability により
[Proof witness publication](../../mizar-proof/en/witness_store.md#publication-references) が可能となる。
この bounded case に fake authority や捏造 projection は不要である。

Artifact は schema/codec、canonical hash、store/manifest commit、reader reachability を所有する。
Proof/Kernel は evidence、policy、status、witness authority を維持する。
Driver は authentic producer association を運び live publication guard を検証する。
保持した ManifestCommit/DATA は新たな authority を与えない。

## 一般 emission に残る gap

- 対応 singleton 経路を越える一般 resolver/checker/Core/VC/proof projection と、その end-to-end publication integration は未提供。
- Builtin2.0 は statements が空で captured-source provenance がないため、coherent `SnapshotPublished` の既存 captured-source guard を満たさない。
  OpenBuffer temporary root の cleanup 後も、保持 DATA/commit から file/publication reachability を復元できない。
- generic/state built-in、完全な dependency slice、cache/proof reuse、verified-session completion は deferred。
  incomplete slice/mandatory MISS、selected-witness-hash の不在、既存 retained-token 制限を維持する。

## 処置

Task 17 は一般 emission の `external_dependency_gap`/`deferred` として open のまま。
次の一般 emission unit は owner-produced projection、対応 proof/witness staging、committed publication/readback integration と、
対応範囲の実 end-to-end test を必要とする。通常 KernelVerified guard を維持し、placeholder、built-in status の再分類、proof authority の捏造は行わない。
bounded 経路は一般 emission を完了しない。G6 correspondence correction は別 unit とする。
