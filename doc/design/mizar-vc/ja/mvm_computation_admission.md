# MVM R6 — 計算閉包全体の決定性と受理

> 英語正本: [mvm_computation_admission.md](../en/mvm_computation_admission.md)。
> 設計タスク: `MVM-R6-ADMISSION-DESIGN`。実装・受理の完了ではない。

## R6 の対象

[§20.9.2](../../../spec/ja/20.algorithm_and_verification.md)
に従い、証明に利用する計算の直接・推移呼出し全体を調べる。
値を返すアルゴリズムが **非 ghost Pick** または
**順序未指定の集合 `for ... in`** を含む場合に限り、
原則として `ensures` が結果を一意に定める証明義務を要する。
`result = t`、`result = true iff φ` で右辺に `result`
がない規定の形式は免除する。免除条件は実際の解決済み契約で判断する。
値を返さない手続き、順序付き列だけのループ、ghost-only Pick
だけでは結果一意性の義務を発生させない。

## 分割と権限

| 段階 | 担当 | 条件 |
|---|---|---|
| R6-A | Resolver/Artifact の入力を VC が消費 | 同一パッケージと検証済み公開 Artifact の呼出し閉包。private helper も対象 |
| R6-B | Checker/Core → VC | Pick の ghost 判別、集合／列、値の有無、`ensures` と免除条件 |
| R6-C | VC 58 と後続 VC | 必要時は型ガード・`requires`・`ensures` に基づく一意性 VC を生成 |
| R6-D | Proof | 閉包全体について真正な受理済み証明または正規の免除を確認 |

Open VC、ソース名、未検証の Artifact、実行結果、E1 の一意性、
replay やステータスフラグだけで R6 は通過させない。
不確定な閉包や依存関係は未受理のままとし、E0330 の報告は
実際の不成立事例と区別する。R4/R5 の実行結果は証明の受理ではない。

## 未解決事項

R6-Q1：真正な呼出し閉包と輸入側の検証済み実行データ。
R6-Q2：VC・Proof が同一契約・閉包へ結び付ける真正な証拠。
R6-Q3：証明が拒否された場合と入力未供給の場合の診断区別。

R6-A〜D は設計上の分割であり、実装契約ではない。
R2/R3 の真正な入力、VC 58 の生成と Proof 受理が成立するまで
R6 の実装・計算証明の受理を完了扱いにしない。
