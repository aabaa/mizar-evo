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

## R5-Q1 計数・報告の判断資料

**owner の検討用であり、回答・実装 freeze ではない。** 下記 R4 選択肢 (3)
を pilot 前に判断する資料である。R5-F は決定後の規則を拡張する。
推奨は R4 の解禁でも、R3 の表現・load、R5 の witness provider、
R6 の受理、R7 の変換・replay の決定でもない。

### 固定条件と未決の範囲

[仕様 §20.9.2–4](../../../spec/ja/20.algorithm_and_verification.md)
で、0 は無制限、timeout は実時間のミリ秒、nest は関数呼出しスタックの
最大深度である。option は同じ computation reference の全呼出しに適用し、
§20.9.4 の gas は `by computation` invocation 単位である。
子呼出しに新規予算を与えない。entry 前の実引数評価と推移的な呼出しも
実行の仕事に含む。検証・証明検査・変換・最終検査は各段階に残す。
以下は派生 VM 計数・内部報告の設計候補である。言語仕様の意図を変える
必要があれば、pilot の慣例にせず owner の仕様判断へ戻す。

### 選択肢・影響・推奨

| 問い | 選択肢と影響 | owner への推奨 |
|---|---|---|
| step 単位 | 実行命令ごとの課金は単純だが R3 の符号化・lowering に依存し、高価な primitive を隠し得る。版付き論理実行操作と複合 primitive の明示的 cost は符号化の影響を減らすが、cost 表の監査と中断可能性が必要。 | R3 と合意した小さい論理操作の課金表を VM semantics に結び付ける案を推奨。引数・entry・return・primitive の仕事を pilot 前に列挙し、AST 節点・行数・Pre-ATP fuel から推定しない。 |
| 入れ子の共有 | invocation 台帳の共有なら累積 step と deadline が明確。残量と deadline の受渡しでも同じ制約を保てるが、失敗を含む全 exit の精密な清算が必要。子の予算リセットは固定条件外。 | 台帳共有を推奨。課金予約点を凍結し、完了した仕事を返却・二重課金しない。同じ reference の明示／goal 呼出しを一つの gas 上限で扱い、順序と同一呼出しの重複処理を明確にする。 |
| nest | root を 1 とする論理 algorithm frame はスタック深度に直接対応。host frame は実装依存で、tail call の論理 frame 除去は上限内で完了できる実行を変える。root 0 案には仕様の深度との対応が必要。 | root 1、callee push 前の検査、host 最適化によらない論理 frame を推奨。実行中の引数呼出しも数え、完了した兄弟呼出しは累積しない。entry/pop と最大深度の報告を凍結する。 |
| timeout 区間 | 単調時計で引数・本体・callee の実時間を測れば時計調整の影響を避けられる。load・証明・import 全体の計時は非実行仕事を混ぜ、枯渇段階を不明瞭にする。CPU 時間や step 換算は仕様の timeout ではない。 | 子で更新しない VM deadline を推奨し、検証・import 資源は別扱い。開始／終了、中断点、primitive の中断までの最大遅延を凍結する。無制限 primitive の間だけの polling では正の上限を強制できない。 |
| 境界・中断 | 実行前検査なら予算超過の操作を防げるが、事後だけでは超過し得る。固定優先順は内部理由を安定化し、全観測原因の保持は詳細を残すが報告を複雑にする。時間観測は scheduling に依存。 | 課金・push 前検査と、有界な中断点および成功公開前の deadline/cancel 検査を推奨。deadline と等しい時刻、同時原因の優先、cancel の報告は owner が選ぶ。診断 code・優先順は未選択。 |
| 報告構成 | 最小の内部正常復帰記録と別の試行結果なら E5 入力と失敗を分離できる。全命令 trace は調査・replay に役立つが未凍結の形式に結合し、無制限に増え得る。今の永続化は R3/R7 schema の先取りになる。 | 最小内部記録と別の試行結果を推奨。trace・永続化は担当が必要とする時まで留保。以下は概念の確認項目であり、field・type・wire format ではない。 |

命令数 pilot も、単位と高価な primitive を明示的に制限・版管理するなら
候補に残るが、後の論理操作表へ暗黙に読み替えない。有限上限での完了を
変える計数変更は、R3/R7 と整合する VM semantics・消費側の無効化規則が必要。

### 報告の確認項目と freeze への引継ぎ

上記の正常報告の来歴は必須。追加候補は有効制限、計数規則の識別、
消費 step、最大論理深度、timeout 区間、invocation 内の呼出し位置である。
経過時間は運用情報であり再現可能な証明証拠ではない。digest・counter・
子の成功は、要求呼出しの真正で完全な正常復帰の代用にならない。

失敗時は成功結果の印を付けず、段階・操作・該当資源／上限・呼出し経路を残す。
既存の [E0331/E0333](../../../spec/ja/22.error_handling_and_diagnostics.md)
の要求は維持する。実行失敗は message・列挙値・call chain、不完了は実際の
段階・操作・資源・上限を示す。cancel の報告は未決。
先行呼出しの正常復帰後に別の呼出しや import が失敗しても、真正な既存内部
証拠は別に保持する。失敗した操作から新しい E5 や未検査の公開候補を出さない。

pilot 前の owner 回答では、課金表、root/callee 深度、共有制限、時計区間、
検査点、同時原因／cancel 方針、報告構成を R3 の命令・値・VM 版と整合させる。
その後の実装契約で真正な R2/R3 入力と既存 readiness matrix 内の次の検査を結ぶ。

- 必要 step ちょうど／一つ不足、既定・0、引数・入れ子、複数呼出しの
  非リセット・非二重課金。
- root・子の深度境界／超過、兄弟・最適化呼出し。実際の再帰・複雑な制御は
  対応後に R5 が拡張する。
- 実行前／途中の timeout、高価な操作、cancel、同時制限を制御した時計で検査。
  成功値は再現し、経過時間と timeout による完了可否の変動は許容する。
- 失敗／不完了と正常復帰、旧 VM／計数識別、偽計数、成功・失敗の混在。
  後続 import 失敗では既存内部証拠を保持し、公開結果は受理しない。

将来の oracle 要求であり、新しい fixture・coverage credit ではない。
既存の Open request は境界数・timeout 動作を証明せず、架空の receipt は
pilot の前提を満たさない。

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
