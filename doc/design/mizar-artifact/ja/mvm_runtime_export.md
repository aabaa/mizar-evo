# MVM R3 — 実行 Artifact と検証付きローダー

> 英語正本: [mvm_runtime_export.md](../en/mvm_runtime_export.md)。
> 設計契約。
> 現段階は設計案であり、実行形式や実装の承認ではない。

## 責務と順序

[仕様 §20.9.1](../../../spec/ja/20.algorithm_and_verification.md)
に従い、公開可能な計算アルゴリズムを元の本体・型・契約・検証証拠・
推移的実行依存・VM 版と結び付ける。公開宣言やハッシュだけでは
実行可能と認定しない。

1. R2（Core / Checker / VC / Proof）：元の**定義・本体**と受理済み義務。
2. R3（Artifact）：個々の呼出しに依存せず実行内容を正準公開。
3. R3（Build / Driver）：manifest と依存 Artifact の検証付き読込。
4. R4（MVM）：別途型付けされた真正な呼出しと結合し、正常復帰時だけ E5 を生成.
5. R6–R7（VC / Proof）：計算の受理と証明への変換。
6. R7 以降（Artifact）：計算結果の来歴を公開。実行内容の初回公開を
   この結果に依存させず、循環を防ぐ。

公開アルゴリズムから内部の private 補助関数を使えるが、
他パッケージから private 名を呼び出せない。依存 Artifact が
欠損・不正・旧版でもソースコードへフォールバックしない。
非 computable な public アルゴリズムは契約で利用でき、契約だけを使う
importer は runtime Artifact を読まなくてよい。ただし computable な
public アルゴリズムの公開には検証済み runtime Artifact が必須であり、
欠落はエラーであって契約のみの公開モードではない（§20.9.1）。
読み込みでは本体・契約・証拠・VM・版・全依存関係・公開範囲を
検証する。失敗は未検証の値や証明を生成しない。

R6-C が消費する R3 export 内容として、private helper を含む閉包内の
各 callee の型ガード・`requires`・完全な `ensures`・値返却／non-ghost
Pick／集合対列ループの分類を登録する。VC58 の同一パッケージ Open
目標生成にも必要となる。exporter が一意性を証明して運ぶか importer が
義務を生成するかは、この登録では決めない（下記 R3-N1）。

## 未決定事項

すべて未決であり、実行形式は未承認。R3-Q1/Q3 の担当は
2026-11-20 11:00 JST の owner 判断による。

- R3-Q1：Artifact 担当。R3 実装 freeze と R4 の形式選択をブロックする。
  実行形式・版と Core/CFG との厳密な意味対応、およびコード抽出用
  [RuntimeIr](../../mizar-doc/ja/todo.md) と共有するか別形式にするか。
  [R1-Q3](../../mizar-vc/ja/mvm_readiness.md) から移管された同じ問いであり、
  将来の Artifact 所有 freeze まで、どちらの形式も選ばない。
- R3-Q2：R3 freeze 時の owner／仕様判断として RESERVED／未決。
  このスライスでは担当を割り当てない。R3 export と R4 の同一モジュール
  パイロットをブロックする。同一パッケージ内の検証時評価を、完全に検証・
  認証された仮置き項目で行うか、公開済み検証済み依存を待つか。
  同一ファイル内の計算例（§20.9.2／§20.13.4）とファイル末尾の
  Artifact emit（§23.4）が制約となる。定義ファイル内の private entry の
  直接評価も未決。自己依存の証明・検証・公開循環や未検証 token は認めない。
- R3-Q3：Artifact 担当、Build／Proof の合意が必要。R3 export をブロックする。
  対応を示す producer 証拠・完全な依存 closure・reader 検証と、形式／manifest
  の統合・版管理をどう構成するか。R3-N2 として、imported 実行 Artifact の
  検証系譜（exporter 側 E1／E2／適用される E4 の受理）の信頼根拠も未決登録する。
  [Proof trust-transfer 規則（英語正本）](../../mizar-proof/en/policy.md#trust-transfer-rule) と
  [Artifact transport 境界](./manifest.md#imported-promotion-authority-boundary)
  （[producer transport 設計](./manifest.md#producer-transport-design)）
  を参照するが、その認証済み公開を runtime Artifact に必須とするかは決めない。
  計算には別途 Artifact／対応検証が必要である。
- R3-N1：R3 に登録。R6-C 用の R3 export 入力をブロックする。
  契約が importer に見えない private helper も含め、閉包の一意性を
  exporter が証明して運ぶか、importer が義務を生成するか。
- R3-N3：R3 に登録。computable-public export 適格性をブロックする。
  computable（§20.9.1）と判定する producer は何か。producer とその担当は未決。
  N1〜N3 の登録は同じ owner 判断によるもので、実装の担当は確定しない。

既存の [`idalgo` テスト](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz)
は未証明の VC 生成までで、実行可能性の根拠ではない。
設計レビューの出口は、境界の独立レビューと未決事項・処置・ブロック対象の記録。
R2 のレビューは完了済みだが、その実装は引き続きブロック中。
真正な R2 証拠と形式・対応の判断が揃うまで、AST-ARTIFACT 配下の
Artifact 所有実装 freeze、R3 export と MVM 実行はブロック中とする。
