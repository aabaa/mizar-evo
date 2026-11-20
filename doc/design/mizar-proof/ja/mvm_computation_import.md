# MVM R7 — 認証された計算結果の変換と replay

> 英語正本: [mvm_computation_import.md](../en/mvm_computation_import.md)。
> 設計契約 `MVM-R7-CERTIFIED-IMPORT-DESIGN`。実装・受理の完了ではない。

## 段階と担当

| 段階 | 内容 | 境界 |
|---|---|---|
| R7-A | 実際の MVM 正常復帰、元入力・文脈 C、R6 の閉包受理、契約と独立した `requires`・`Adm`・`InputOK` | 失敗・未証明入力を採用しない |
| R7-B | 全 `ensures` と値の等式を候補として構築し、T1〜T8 の一方向有限変換 | 型付き UID、ガード、Binder 証拠を省略しない |
| R7-C | 候補全体の内部構文残留検査 | 部分的な `ensures` の公開を禁止 |
| R7-D | `computed` と全呼出し replay 成功後の `kernel_verified` の分離 | replay 失敗時に非 replay に戻さない |
| R7-E | Artifact に計算証拠の来歴を公開し Cache の一致を確認 | 実行内容・依存版・有効 replay mode を必ず照合 |

## 意味論と未解決事項

[§20.13.7](../../../spec/ja/20.algorithm_and_verification.md)
の T1〜T8 を使用し、内部の `returns` と公開可能な通常の式を
分離する。T5/T6 では更新値を UID 置換の前に変換し、
T7 は正確な構文サイズ・構造比較を行う。型・注釈・集合・列の
内部に AST/UID/C や変換前の演算が残れば候補全体を拒否する。
変換失敗後も既存の内部正常復帰は保持してよい。

`replay` 省略時は package manifest の `computation_replay` が
既定 `false`、明示指定が優先する。replay 成功には
実行された直接・推移呼出しすべての定義式証拠と
minimum-kernel SAT 検査が必要であり、後から kernel 検査を
行っただけでは unreplayed `computed` 依存を消せない。
厳格リリースでは `computed` 依存を拒否する。

**未解決（すべて未決）:**

- **R7-Q1 — Proof 担当、R7-A をブロック:** R4/R5、R6、AST-CONTRACT、
  formation の producer は、真正に受理された E5・契約・R6 と形成入力をどう供給するか。
- **R7-Q2 — Proof 担当、R7-B と R7-C をブロック:** 一方向 converter／Binder の
  証明 producer と、[§22.4.8](../../../spec/ja/22.error_handling_and_diagnostics.md)
  に対応する有限な診断段階の対応付けは何か。
- **R7-Q3 — Proof が記録内容を担当、R7-D と R7-E をブロック:**
  runtime／実行内容／Proof の計算記録 schema、replay／定義式証拠、policy identity を
  Artifact（manifest）と Cache（再利用）にどう供給するか。
  [R3-Q1/Q3](../../mizar-artifact/ja/mvm_runtime_export.md)および
  [manifest transport 設計](../../mizar-artifact/ja/manifest.md#producer-transport-design)との関係も未決である。

輸入実行 Artifact に関わる R7-Q1/Q3 は、未決の
[R3-N2 の信頼根拠の問い](../../mizar-artifact/ja/mvm_runtime_export.md)、
[Proof の imported promotion 境界](./policy.md#imported-promotion-authority-boundary)、
[Artifact の imported promotion 境界](../../mizar-artifact/ja/manifest.md#imported-promotion-authority-boundary)を参照する。
この参照は輸入実行 Artifact に認証済み公開を必須とするかを決めず、
計算固有の Artifact／対応関係検査を維持する。

R7-A〜E は設計分割に留まり、実装契約や既存テストの成功扱いにはしない。
