# MVM R4 — 最小限の検証付き実行器

> 英語正本: [mvm_executor.md](../en/mvm_executor.md)。
> [R4 設計契約](../../task_contracts/ja/MVM-R4-EXECUTOR-DESIGN.md)。
> 設計草案のみ。実行器・証明受理・公開形式の実装完了ではない。

## 境界と最小対象

[仕様 §20.9](../../../spec/ja/20.algorithm_and_verification.md#209-execution-model-and-computability)
では値渡し、局所状態、正常復帰・実行失敗・制限超過を区別する。
R4 の最小候補は `idalgo(a)` の直接 `return a` だが、既存テストの
VC は Open であり、型付き ground 呼出しも未確定である。

| 担当 | 入力・出力 | 制限 |
|---|---|---|
| R2 | Checker/Core/VC/Proof の元呼出し・型・本体・受理済み証拠 | 未証明結果を MVM が補ってはならない |
| R3 | Artifact/Build/Driver の検証済み実行内容と依存閉包 | 欠損・旧版・改竄・非公開 import で拒否、ソース代替禁止 |
| R4 | 値渡しで実際に実行し、正常復帰時のみ内部報告 | `fail`・実行不能・制限超過は証明を与えない |
| R6–R7 | 計算の決定性受理、契約変換・replay・Proof 状態 | R4 では `computed`／`kernel_verified` を発行しない |

正常復帰の報告は元の本体・入力・文脈 C（該当時）・実行結果・
依存 Artifact・VM 版と結び付ける。報告の形式と API は未決定。
E5 の内部入力と、R6/R7 による証明受理は異なる。

## 資源制限と未解決事項

`steps`・`timeout`・`nest` はそれぞれ 0 が無制限。
実際の VM ステップの課金、呼出し深さ、時間計測、
制限・キャンセルの優先順位は実装前に凍結する。
既存の Pre-ATP discharge 用予算を VM gas と同一視しない。

R4 実装には、R2 の真正な型付き呼出しと本体の検証結果、
R3 の実行形式・検証付きローダー、実行の肯定／拒否テストが必要。
R1〜R3 の独立レビューが終わるまで本設計だけで実装を解禁しない。
