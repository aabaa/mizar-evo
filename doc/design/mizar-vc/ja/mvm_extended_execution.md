# MVM R5 — 制御構造・失敗・資源制限

> 英語正本: [mvm_extended_execution.md](../en/mvm_extended_execution.md)。
> 設計タスク: `MVM-R5-EXTENDED-EXECUTION-DESIGN`。
> 設計のみ。R5 実装と証明受理を承認しない。

## R5 の分解

| 項目 | 要件 | 実装の前提 |
|---|---|---|
| R5-A | 局所 `var`/`const`、代入、`if` と正常復帰 | R4、真正な Core/CFG、証拠、実行 Artifact |
| R5-B | `while`、`for to/downto`、列の `for in`、`break`/`continue` | Core のループ・終了状態、R4 |
| R5-C | 呼出し・再帰・検証済み依存 Artifact の連鎖 | Resolver/Core の真正な call、R3 のローダー |
| R5-D | 順序未指定の集合反復、実行時 Pick、構文 `match` | 実行可能な要素選択と構文認証（証明への利用は R6 の決定性受理が追加条件） |
| R5-E | `fail`、計算不可能な演算、ghost 隔離 | Core/Checker の失敗配置と効果検証 |
| R5-F | `steps`・`timeout`・`nest`、中断と制限超過 | R4 の実行ステップ計数規則と版管理 |

## 実行と証明の境界

R4/R5 は実際に正常復帰した場合にのみ E5 内部候補を渡す。
実行失敗・非計算可能・不足 Artifact・時間／ステップ／深さ制限で
証明事実、正常復帰、数学的な非停止を推論しない。
`requires`・`ensures`・`invariant`・`assert` の検証を
MVM の値の返却で代用しない。部分アルゴリズムの具体的実行は
全域 E4 保証とは異なる。

集合 `for in` は順序未指定、列の場合は順序を保存する。
非 ghost Pick と集合反復のある計算を証明に使うには、
実行した閉包全体について R6 による一意性の証明・免除判定が必要。
`match` は数学的な等式ではなく構文構造を比較する。
ghost-only Pick と管理用 `processed` は実行状態に流さない。

[仕様 §20.9](../../../spec/ja/20.algorithm_and_verification.md) の
`steps`、`timeout`、`nest` は 0 を無制限とする。
具体的な課金単位・時間計測・中断優先順位は未確定であり、
Pre-ATP の既存制限値を流用しない。

## 未決定事項

- R5-Q1：VM ステップ課金・時間測定・再帰深さ・中断順序。
- R5-Q2：集合列挙・Pick の実行可能な具体値の提供方法。
- R5-Q3：Core/CFG の対応、call frame、match/Binder の認証範囲。

R5-A〜F は設計上の分割であり、実装契約として登録していない。
R2／R3 の前提を満たす R4 が実行可能となり、各 producer と
テストの独立レビューが完了してから個別実装に進む。
