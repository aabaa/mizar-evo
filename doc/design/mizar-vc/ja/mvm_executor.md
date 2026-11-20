# MVM R4 — 最小限の検証付き実行器

> 英語正本: [mvm_executor.md](../en/mvm_executor.md)。
> R4 設計契約 `MVM-R4-EXECUTOR-DESIGN`。
> 設計草案のみ。実行器・証明受理・公開形式の実装完了ではない。

## 境界と最小対象

[仕様 §20.9](../../../spec/ja/20.algorithm_and_verification.md)
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

**R4 の選択肢の処置（owner 決定 2026-11-20 11:00 JST）:**
(1) と (4) は既存の R3 の問いへの参照であり、新しい R4 の問いではない。
- **(1) [R3-Q1](../../mizar-artifact/ja/mvm_runtime_export.md) — Artifact 所有・未決:**
  R3 形式と整合する最小命令／復帰値表現。R3 実装 freeze と R4 形式選択を止める。
- **(2) [R1-Q1](./mvm_readiness.md) — 2026-11-19 23:28 JST 決定済み:**
  AST-TYPE/AST-LOWER ゲートを維持し、ordinary-only narrowing は不承認。
  真正な型付き ground 呼出しは引き続き必要。
- **(3) R5-Q1 — VC が R4 の決定として所有・未決:**
  step 課金・timeout・nest の計数と報告形式。R4 の最初の実行を止め、
  pilot を含め実行前に凍結する。R5 側は参照と R5 固有の拡張だけを残す。
- **(4) [R3-Q2](../../mizar-artifact/ja/mvm_runtime_export.md) — RESERVED・未決・本スライスでは所有者未割当:**
  R3 freeze 時の owner／仕様判断である検証済み staged load の順序。
  R3 export と R4 same-module pilot を止める。
架空の実行 Artifact でこれらを決めない。

**実装解禁条件:** R1〜R3 の独立レビューに加え、R2 の真正な型付き呼出し、
真正に受理済みの本体全体の証拠、R3 の実在するローダーの前提が必要。
AST-LOWER の完全入力と型付き computability facts も必須である。
既存の `ALG-MVM-EVALUATION` 所有の下で最小実行器契約を凍結し、
肯定／拒否の oracle を test-first で用意する。それまでは設計のみであり、
R5〜R7 は別のゲートである。
