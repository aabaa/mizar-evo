# MVM R3 — 実行 Artifact と検証付きローダー

> 英語正本: [mvm_runtime_export.md](../en/mvm_runtime_export.md)。
> [設計契約](../../task_contracts/ja/MVM-R3-ARTIFACT-LOADER-DESIGN.md)。
> 現段階は設計案であり、実行形式や実装の承認ではない。

## 責務と順序

[仕様 §20.9.1](../../../spec/ja/20.algorithm_and_verification.md#2091-the-mizar-virtual-machine-mvm)
に従い、公開可能な計算アルゴリズムを元の本体・型・契約・検証証拠・
推移的実行依存・VM 版と結び付ける。公開宣言やハッシュだけでは
実行可能と認定しない。

1. R2（Core / Checker / VC / Proof）：元の本体・呼出しと受理済み義務。
2. R3（Artifact）：実行内容を検証済み本体と対応付けて正準公開。
3. R3（Build / Driver）：manifest と依存 Artifact の検証付き読込。
4. R4（MVM）：真正な呼出しを実行し、正常復帰時だけ E5 を生成。
5. R6–R7（VC / Proof）：計算の受理と証明への変換。
6. R7 以降（Artifact）：計算結果の来歴を公開。実行内容の初回公開を
   この結果に依存させず、循環を防ぐ。

公開アルゴリズムから内部の private 補助関数を使えるが、
他パッケージから private 名を呼び出せない。依存 Artifact が
欠損・不正・旧版でもソースコードへフォールバックしない。
読み込みでは本体・契約・証拠・VM・版・全依存関係・公開範囲を
検証する。失敗は未検証の値や証明を生成しない。

## 未決定事項

- R3-Q1：実行形式・版と Core/CFG からの意味保存対応。
- R3-Q2：同一パッケージ内での検証・仮置き・公開・実行順序。
- R3-Q3：証拠・依存 closure・読み込み時の検証責務の厳密な構成。

既存の [`idalgo` テスト](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz)
は未証明の VC 生成までで、実行可能性の根拠ではない。
R2 の検証済み producer と上記の判断を確定するまで、
R3 実装と MVM 実行はブロック中とする。
