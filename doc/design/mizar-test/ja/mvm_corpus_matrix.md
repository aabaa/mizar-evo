# MVM R8 — 段階的検証・統合テストの設計

> 英語正本: [mvm_corpus_matrix.md](../en/mvm_corpus_matrix.md)。
> 設計タスク `MVM-R8-CORPUS-DESIGN`。実行可能なテストやカバレッジの完了ではない。

## テスト分割

| 系列 | 目的 | 必須の前提 |
|---|---|---|
| R8-A | 元ソースの構文、解決済み呼出し、型付き引数 | Parser/Resolver/Checker |
| R8-B | 本体・CFG・契約・VC の完全性 | Core/VC |
| R8-C | E1/E2 と必要時の E4 の真正な証明受理 | VC/Kernel/Proof |
| R8-D | 実行 Artifact と依存関係の検証付き読込 | Artifact/Build/Driver |
| R8-E | 値渡しの実行、分岐・ループ、失敗と制限超過 | R4/R5 の MVM |
| R8-F | 推移的な計算閉包と一意性の義務・免除 | R6 の VC/Proof |
| R8-G | T1〜T8 の変換、候補全体の検査、replay・Cache | R7 の Proof/Artifact |
| R8-H | 元ソースから MVM・証明受理・Artifact 公開までの統合 | R2〜R7 の実処理 |

すべての系列に肯定例、拒否例、改竄例を用意する。
失敗・実行制限超過は E5 や数学的な非停止の証明を生成しない。
計算が成功しても R6 の受理、R7 の T1〜T8 と残留検査なしで
`computed` を発行しない。replay 成功のみ `kernel_verified` の候補となる。
厳格リリースで残留する `computed` 依存を拒否する。

## ステージと未決定事項

既存の `mizar-test` の正式なステージは7種類であり、
`expected_phase` に MVM 実行専用の値はない。
新ステージの追加・後段のエラー判定は別途レビューが必要で、
この設計で構文やテストの値を創作しない。

`idalgo`・`swapalgo`・`loopalgo`・`by computation` の
既存テストは VC 生成までの根拠であり、実行成功例ではない。

**すべて未決定：**

R8-Q1：所有者 Test。R8-E〜H の有効化を阻害。MVM・Proof・公開のランナー／ステージ。
R8-Q2：所有者 Test（入力：Resolver/Checker と R2〜R6）。R8-H と最初の肯定例を阻害。真正な型付き ground 呼出しと、独立した肯定例。
R8-Q3：所有者 Test（[R7-Q3](../../mizar-proof/en/mvm_computation_import.md#trust-controls-and-next-implementation-freeze) に従う）。R8-G、R8-H を阻害。replay・厳格ポリシーと依存 Artifact の一致条件。

すべての肯定例は、実際の producer chain 全体が揃うまで有効化しない。
R8-C は R2、R8-D は R2/R3、R8-E〜H は適用される R2〜R7 の
実 producer と必要なステージランナーを待つ。

R8-A〜H は設計上の分割であり、個別実装契約ではない。
既存の `.miz`、期待値、trace、coverage、実装を変更しない。
