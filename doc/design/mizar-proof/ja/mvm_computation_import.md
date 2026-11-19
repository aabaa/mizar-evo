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

R7-Q1 は真正な E5・契約・R6 受理、Q2 は T1〜T8 の証明根拠、
Q3 は Artifact/Cache の記録・互換性である。
R7-A〜E は設計分割に留まり、実装契約や既存テストの成功扱いにはしない。
