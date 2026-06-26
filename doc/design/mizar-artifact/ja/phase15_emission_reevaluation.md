# mizar-artifact Phase 15 Emission Reevaluation

> 正本は英語です。英語版:
> [../en/phase15_emission_reevaluation.md](../en/phase15_emission_reevaluation.md)。

## 範囲

task 17 は、task 23 が artifact 側の formula/substitution proof-witness schema を完了
した後で再評価された。この再評価は、`mizar-artifact` が実 producer projection から
full `VerifiedArtifact` emission を接続できるかを判断する。

分類結果:

- `external_dependency_gap`: phase 15 emission には、まだ real producer-owned
  projection output と proof witness publication が不足している。
- `deferred`: task 17 implementation は deferred のまま。
- `spec_gap`: crate-owned artifact schema では見つからない。
- `test_gap`: crate-owned behavior については開かない。

## 所見

task 23 は、将来の emission が必要とする artifact 側 witness projection を提供した。

- `ProofWitnessRef` schema version `2.0`。
- 現在の唯一の trusted evidence kind としての
  `formula_substitution_kernel_evidence`。
- target binding、formula evidence、substitution evidence、provenance、optional な
  formula context、accepted result の hash。
- legacy certificate、resolution trace、backend log、backend method、SMT proof object、
  instantiated formula、SAT problem payload に対する通常 trusted reader の拒否。

task 17 に残る blocker は `mizar-artifact` の外側にある。

- この checkout には `mizar-proof` workspace crate が存在しない。
- artifact store と manifest に入力する producer-owned な witness staging/publication
  output が存在しない。
- full `VerifiedArtifact` emission 用の real producer output が存在しない。
- checked kernel evidence、選択済み proof status、witness file、manifest entry を接続する
  producer-owned publication hook が存在しない。

現時点で task 17 を実装するには、placeholder proof authority、捏造された producer
projection、または fake witness publication が必要になる。それらは artifact boundary と
evidence-pipeline correction により禁止されている。

## 処置

task 17 は `external_dependency_gap` として deferred のままとする。source stub、
placeholder crate、fake witness schema、fake producer projection、artifact publication shim は
追加しない。

次に task 17 を実施できるのは、少なくとも次が存在してからである。

- real producer-owned `VerifiedArtifact` projection input。
- proof/witness publication owner。想定は `mizar-proof`。
- stable witness staging と manifest publication output。
- `mizar-artifact` 内の proof authority を変更せず、real emission を実行する integration test。

それらの依存が存在するまでは、`mizar-artifact` は stable schema、canonical writer/reader、
hash validation、store primitive、manifest transaction だけを所有する。

## Verification

この task は documentation-only である。必要な verification は次の通り。

```text
git diff --check
git diff --cached --check
```

Rust source を変更しないため、この task では Rust verification は不要である。
