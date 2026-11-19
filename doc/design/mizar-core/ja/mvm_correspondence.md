# MVM R2 — 検証済み本体と呼出しの対応

> 英語正本: [mvm_correspondence.md](../en/mvm_correspondence.md)。
> 設計案のみ。実行・証明受理・公開形式・coverage は完成していない。

## 仕様と通常入力

[§20.1.1](../../../spec/ja/20.algorithm_and_verification.md)
では通常の引数は項であり、式・ラベル・`thesis` は
`expr of boolean` 引数に限られる。[§20.9](../../../spec/ja/20.algorithm_and_verification.md)
は値渡し・局所状態・検証済み実行 Artifact を要求する。

**Q1 は未承認の提案：** 通常の ground 入力に架空の
`expr`・Binder・UID・再構成文脈 C を生成しない。
ただし既存 AST-TYPE/AST-LOWER/ALG-MVM-EVALUATION の依存関係は
変更しない。通常値専用の経路を許可するには契約担当の審査が必要。

## 真正な対応関係

| 担当 | 内容 | 消費側 |
|---|---|---|
| Resolver / Checker（定義側） | 元宣言・モジュール・版・アルゴリズム ID、可視性・型 | Core と R3 |
| Core / VC / Kernel / Proof（定義側） | 本体・契約・CFG と、実際の証明義務受理を同一ソースに対応付ける | R3 の再利用可能な実行 Artifact |
| Resolver / Checker（呼出し側） | 独立に型付けされた真正な ground 呼出し・引数順序 | R4（**R3 の公開には不要**） |
| Artifact / Build（R3） | 検証済み定義に対応する実行内容・ローダー | R4 で呼出しと結合 |

名前やハッシュ、未証明ステータスだけで受理しない。欠落、
重複、順序違い、古い版、回復ノード、異なる所有者を拒否する。
Core は証明を行わず元ソースを再解析しない。

**Q2：** E1 は戻り値型・一意性・居住性、E2 は受理済み部分正当性、
`terminating` の昇格にはさらに E4 正常復帰保証を要求する。
局所 `return` や Open VC は不足。部分アルゴリズムの具体的実行に
全域 E4 は不要な場合があるが、検証済み本体と Artifact は必要。
E5 は将来の MVM 正常実行からのみ生成される。

## 最小ケースと未完了事項

[`idalgo(a)`](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz)
の[期待結果](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.expect.toml)
は Open VC 生成まで。`idalgo(0)` の呼出し型付けは未検証。
型・契約・本体・順序・CFG・文脈・極性・証拠・依存版の
不一致と ghost 流出を、将来の試験で拒否する。

**R2 実装はブロック中。** Q1 の承認と検証済み定義に必要な
E1/E2/E4 producer の確認後、定義側の実装契約を確定する。
R3 は具体的な呼出しなしで公開可能だが、R4 には別途真正な呼出しが必要。
実行形式とローダー、コード抽出用 RuntimeIr、同一パッケージ内の
公開順序は R3 の別設計とする。新しい crate・API・Kernel 規則は作らない。
