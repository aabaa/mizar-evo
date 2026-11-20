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
| R6-C | VC 58 と後続 VC | VC58 の同一パッケージ Open goal 生成を含め、R3 exports が前提。必要時は型ガード・`requires`・`ensures` に基づく一意性 VC を生成 |
| R6-D | Proof | 閉包全体について真正な受理済み証明または正規の免除を確認 |

Open VC、ソース名、未検証の Artifact、実行結果、E1 の一意性、
replay やステータスフラグだけで R6 は通過させない。
不確定な閉包や依存関係は未受理のままとし、E0330 の報告は
実際の不成立事例と区別する。R4/R5 の実行結果は証明の受理ではない。

## 未解決事項

以下の問いはすべて未決のままである。

- **R6-Q1 — VC（閉包生成）、入力は R2/R3。blocked target は R6-A の imported 部分と R6-C：**
  推移的 helper と可視性を含む真正な呼出し閉包と検証済み輸入実行データを、どう認証するか。
  VC58 の同一パッケージ Open goal 生成も R3 が前提であり、ローカル準備は輸入閉包を捏造できない。
- **R6-Q2 — Proof。blocked target は R6-D：**
  各受理済み VC または正規の構文的免除を同一の要求・契約・閉包・policy・version にどう結び付け、変更時に再検証するか。
- **R6-Q3 — Proof（E0330 の投影）。blocked target は R6-D：**
  真正な一意性 VC が証明できない場合と producer／閉包入力が未供給の場合を、診断上どう区別するか。
  public E0330 の採用は [別の診断権限](../../todo.md#algorithm-definitional-semantics-specification-amendment--) に従う。
- **N8 — VC。blocked target は R6-C と R6-D：**
  §20.9.2 の直接・推移的に評価される全アルゴリズムの計算閉包を、静的に確立するか、動的に確立するか。

輸入実行 Artifact に関わる R6-Q1/Q2 は、未決の
[R3-N2 の信頼根拠の問い](../../mizar-artifact/ja/mvm_runtime_export.md)、
[Proof の imported promotion 境界](../../mizar-proof/ja/policy.md#imported-promotion-authority-boundary)、
[Artifact の imported promotion 境界](../../mizar-artifact/ja/manifest.md#imported-promotion-authority-boundary)を参照する。
この参照は輸入実行 Artifact に認証済み公開を必須とするかを決めず、
計算固有の Artifact／対応関係検査を維持する。

R6-A〜D は設計上の分割であり、実装契約ではない。
R2/R3 の真正な入力、VC 58 の生成と Proof 受理が成立するまで
R6 の実装・計算証明の受理を完了扱いにしない。
