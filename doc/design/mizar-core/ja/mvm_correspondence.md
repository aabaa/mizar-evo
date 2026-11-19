# MVM R2 — 検証済み本体と呼出しの対応

> 英語正本: [mvm_correspondence.md](../en/mvm_correspondence.md)。
> 設計案のみ。実行・証明受理・公開形式・coverage は完成していない。

## Q1：通常値と反射の依存境界

[§20.1.1](../../../spec/ja/20.algorithm_and_verification.md)
では通常の引数は項であり、式・ラベル・`thesis` は
`expr of boolean` 引数に限られる。[§20.9](../../../spec/ja/20.algorithm_and_verification.md)
は値渡し・局所状態・検証済み実行 Artifact を要求する。

**決定：既存の依存ゲートを維持する。** 項という構文だけでは
通常値の経路とは判定できない。期待型が `expr of T` なら、
[§20.5.3](../../../spec/ja/20.algorithm_and_verification.md) の R1 が項を反射し、
R2 が既存の構文値を埋め込む。通常引数の宣言でも本体や推移的な
呼出し先の反射処理は排除されない。通常値にも真正な宣言・型の識別と
ソース由来情報が必要であり、反射固有の UID・環境・再構成文脈 `C` を
捏造したり、必要な場合に省略したりしない。

**担当者への提案であり、迂回の承認ではない：** 将来の最小ケースを
真正な非反射の通常値と、反射処理を含まない検査済み本体・依存閉包に
限定する。反射・非 ground・template・パッケージ横断呼出しは提案の対象外。
AST-TYPE/AST-LOWER/ALG-MVM-EVALUATION の前提は、担当者がより狭い
契約を明示的に審査・確定するまで維持する。本設計は経路を許可しない。

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

## Q2：定義側の証拠と呼出し側の入力

E1 は戻り値型が主張される全入力での型・一意性・居住性、E2 は
事前条件と正確な継続文脈における受理済み部分正当性を要求する。
`terminating` の昇格にはさらに E4 正常復帰保証を要求する。
局所 `return` や Open VC は不足。部分アルゴリズムの具体的実行に
全域 E4 は不要な場合があるが、検証済み本体と Artifact は必要。
E5 は将来の MVM 正常実行からのみ生成される。

| 定義側の義務 | 現在の真正な producer と限界 | R3 に不足する受理 |
|---|---|---|
| E1 | Checker の [`check_source_algorithm_types`](../../../../crates/mizar-checker/src/type_checker.rs) は単一引数・`object` 戻り値の限定形を検査し、Core の [`lower_source_algorithms`](../../../../crates/mizar-core/src/elaborator.rs) が真正な検査結果を消費する。VC は Open な契約・居住性義務を生成する。 | 局所の戻り値型付けと Open 目標は完全な E1 ではない。型が主張される全入力で型・一意性・居住性を正当化する。 |
| E2 | VC の [`generate_source_algorithm_postconditions`](../../../../crates/mizar-vc/src/generator.rs) が実際の戻り値目標・文脈を生成し、Proof の [`check_source_algorithm_return_reflexivity`](../../../../crates/mizar-proof/src/status.rs) が選択された一つの VC を Kernel・方針で検査する。 | 単一の戻り値証明は、同一本体・契約・CFG・方針に対応する全義務の受理ではない。[投影試験](../../../../crates/mizar-test/src/runner/proof_verification/term_proof_tests/source_algorithm_return.rs) でも依存 slice は Open・不完全で、Artifact 公開の不足が残る。 |
| 適用される E4 | [最小ケースの snapshot](../../../../tests/snapshots/vc/pass_proof_verification_algorithm_ensures_return_001.vc_ir.snap) の停止性 seed は具体的 VC のない deferred 状態。残る正常復帰前提は ALGDEF-VC53-NORMAL-RETURN が所有する。 | `idalgo` を含む `terminating` には明示・暗黙の失敗、子呼出し、反復範囲を含めた正常復帰の受理が必要。宣言・局所 `return`・等式・継続の仮定では不足。 |

AST-CONTRACT の反射 E1/E2/E4 ガードと形成依存も維持する。
限定 producer や計画契約は完全な定義の受理証拠ではない。
R3 は不足する受理済み義務と不変の定義への真正な対応付けを必要とする。
独立に R4 は元の呼出し・対象の解決済み識別、シグネチャに対して型検査された
順序付き実引数、完全な対応値とソース・文脈由来情報を必要とする。
呼出し入力で定義の証拠不足を補ったり、R3 の公開を制限したりしない。

## 最小ケースと未完了事項

[`idalgo(a)`](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz)
の[期待結果](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.expect.toml)
は Open VC 生成まで。`idalgo(0)` の呼出し型付けは未検証。
将来の試験では真正に対応する定義の証拠を R3 の正例とし、
それと結合する独立の型付き呼出しを R4 の正例とする。

| 境界 | 必須の拒否対照 |
|---|---|
| 定義の対応 | 本体・契約・シグネチャ・CFG・出口の変更、文の順序違い、異なる所有者。 |
| 証拠の受理 | 義務の欠落・deferred・Open・汚染、証明の偽造、目標・文脈・極性・方針の相違。 |
| E1 と E4 | 型・一意性・居住性の不足、terminating 定義の正常復帰義務の省略（部分子呼出し・範囲・失敗経路を含む）。 |
| 現在性 | ソース・モジュール・版・依存証拠の版の相違。同じ名前やハッシュだけでは対応を回復できない。 |
| 呼出しの結合 | 未解決・異なる対象、型違い、真正な型付き元呼出しに対する実引数の欠落・重複・順序違い、古い呼出し文脈。R4 で拒否し、R3 に呼出しを要求しない。 |
| 実行適格性 | 未対応の反射入力・本体・依存、回復ノード、ghost 流出。省略した文脈・`C` や E5 結果を捏造しない。 |

既存試験と sidecar は保存する。これらは将来の段階別対照であり、
有効化された実行試験ではない。

**R2 実装はブロック中。** 通常値だけに依存を狭める場合は担当者の別契約決定が
必要であり、完全に受理された E1/E2 と適用される E4 producer は未完了。
R3 は定義の証拠が揃えば具体的な呼出しなしで公開できるが、
R4 には別途真正な呼出しが必要。既存 Core 42–53 と VC53 の依存も維持する。
実行形式とローダー、コード抽出用 RuntimeIr、同一パッケージ内の
公開順序は R3 の別設計とする。新しい crate・API・Kernel 規則は作らない。
