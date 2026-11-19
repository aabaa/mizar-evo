# MVM R2 — 検証済みアルゴリズム本体と呼出しの対応

> 英語正本: [mvm_correspondence.md](../en/mvm_correspondence.md)。
> 契約: [MVM-R2-CORRESPONDENCE-DESIGN](../../task_contracts/ja/MVM-R2-CORRESPONDENCE-DESIGN.md)。
> 設計案のみ。MVM 実行、本体の検証済み認定、Artifact 実行形式の承認ではない。

## 言語仕様と境界

[§20.1.1](../../../spec/ja/20.algorithm_and_verification.md#2011-algorithm-definition-block)
では、引数を期待型に従って解析する。通常の型は普通の項を受け取り、
`expr of boolean` に対してのみ式・命題ラベル・`thesis` を許す。
[§20.9.1](../../../spec/ja/20.algorithm_and_verification.md#2091-the-mizar-virtual-machine-mvm)
では、実行は値渡し、局所可変状態、検証済み実行 Artifact に限定される。
E1〜E5 の証明義務・実行報告は独立した責務である。

**Q1 は提案段階。** 通常の ground 値から、反映構文の UID・Binder・
再構成文脈 C を作り出してはならない。ただし、既存の
[AST-LOWER](../../task_contracts/ja/AST-LOWER.md) と
[ALG-MVM-EVALUATION](../../task_contracts/ja/ALG-MVM-EVALUATION.md) の
依存関係を自動的に解除するものではない。通常値だけの実行経路を
別途認めるかは仕様・契約担当の審査が必要である。

## 対応関係に必要な根拠

| 担当 | 必要な真正の内容 | 消費側 |
|---|---|---|
| Resolver / Checker | 元ソース・モジュール・スナップショット、正規のアルゴリズム ID、引数順序・型・可視性、型付き呼出し | Core |
| Core | 同一 owner の本体、契約、CoreIr/CFG、文の順序、ソース対応 | VC / R3 |
| VC / Kernel / Proof | 同一証明義務・文脈・極性・ポリシー・依存関係に結び付いた、実際に受理された証拠 | R3 |
| Artifact / Build | 実行内容と検証済み本体・契約・バージョン・依存 Artifact を照合する R3 の公開・ローダー | MVM R4 |

ソースハッシュ、名前、Core ノード、公開ステータスだけでは検証済みとは認定しない。
変更された本体・引数・契約・CFG・依存関係・証拠は再確認する。

**Q2 の必須条件。** 値を返すアルゴリズムの E1 は型・一意性・
居住性、E2 は部分正当性の受理を必要とする。`terminating` の
昇格にはさらに E4 の正常復帰保証が必要である。未証明の VC や
単独の局所的な return 検査だけでは不足する。部分的アルゴリズムは
全域 E4 がなくても具体的入力で後に実行できる場合があるが、
必要な検証済み本体・Artifact は省略できない。E5 は R4 の実行結果であり
R2 で生成しない。

## 最小ケースと次の条件

既存の [`idalgo(a)`](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz)
は `object` を取り、`ensures result = a` と `return a` を持つ。
[期待結果](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.expect.toml)
は未証明 VC の生成までであり、実行可能性を主張しない。
`idalgo(0)` は引数解析・型付けについて検証が必要な提案である。

R2 の次の実装契約は、Q1 の承認、真正な呼出し入力、E1/E2/E4 の
該当証明の producer を確認した後に確定する。正しい入力と対応する
本体の組に対し、引数型・文の順序・ソース・モジュール・CFG・
文脈・極性・証拠・依存バージョンの不一致をすべて拒否する。
既存の Core 42〜53 は未完了部分を保持する。

R3 の実行内容のシリアライズと検証付きローダー、
コード抽出用 `RuntimeIr` との関係、同一パッケージ内の公開順序は
別途設計する。R2 の実装と MVM 実行は引き続きブロック中である。
