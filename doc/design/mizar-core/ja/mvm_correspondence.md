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

**担当者決定 2026-11-19 23:28 JST：** 非反射の通常値だけを使う
最小ケースの依存縮小は承認されなかった。現行の
AST-TYPE/AST-LOWER/ALG-MVM-EVALUATION ゲートを維持する。

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
| E1 | Checker の [`check_source_algorithm_types`](../../../../crates/mizar-checker/src/type_checker.rs) と Core の [`lower_source_algorithms`](../../../../crates/mizar-core/src/elaborator.rs) が限定形を認証する。Proof は検査済みの元の型・一意性 schema を対応付け、別途 native 方針の下で元の居住性を採用する。 | schema 対応や単独義務の採用は完全な E1・定義受理ではない。 |
| E2 | VC の [`generate_source_algorithm_postconditions`](../../../../crates/mizar-vc/src/generator.rs) が実際の戻り値目標・文脈を生成し、Proof の [`check_source_algorithm_return_reflexivity`](../../../../crates/mizar-proof/src/status.rs) が選択された一つの VC を Kernel・方針で検査する。 | 単一の戻り値証明は、同一本体・契約・CFG・方針に対応する全義務の受理ではない。[投影試験](../../../../crates/mizar-test/src/runner/proof_verification/term_proof_tests/source_algorithm_return.rs) でも依存 slice は Open・不完全で、Artifact 公開の不足が残る。 |
| 適用される E4 | Proof は native 方針の下で限定形の元の E4 存在命題を採用できる。[最小ケースの snapshot](../../../../tests/snapshots/vc/pass_proof_verification_algorithm_ensures_return_001.vc_ir.snap) の停止性 seed は具体的 VC のない Deferred 状態。 | 局所 E4 採用は seed と独立の E1/E2 義務を変更しない。一般の正常復帰は失敗・子呼出し・反復範囲も含む。 |

AST-CONTRACT の反射 E1/E2/E4 ガードと形成依存も維持する。
限定 producer や計画契約は完全な定義の受理証拠ではない。
R3 は不足する受理済み義務と不変の定義への真正な対応付けを必要とする。
独立に R4 は元の呼出し・対象の解決済み識別、シグネチャに対して型検査された
順序付き実引数、完全な対応値とソース・文脈由来情報を必要とする。
呼出し入力で定義の証拠不足を補ったり、R3 の公開を制限したりしない。

## 限定された定義受理の境界

[既存 VC profile](../../mizar-vc/ja/generator.md#bounded-direct-return-e1e4-producer-design)
は、不変の `object` 引数一つ、`object` 戻り値、等式 ensures、
その引数だけを返す唯一の return、認証された `terminating` 意図に限る。
呼出し・ループ・Pick・再帰・group・反射型・不透明型・回復ノード・ghost 流出は
対象外であり、通常値だけを理由に依存ゲートを省略しない。

限定集合は五つの異なる元の義務からなる。E1 の戻り値型・一意性・全型付き入力の
居住性、E2 の代入後の事後条件、独立した E4 正常復帰の存在である。
居住性には型付き入力ガードを残し、復帰を仮定したり `requires` 内に制限したりしない。
E4 は操作的復帰関係を保つ。E2 は正確な事前条件・継続文脈とともに対応本体全体を
覆う必要があり、等式 ensures や選択した return の証明だけでは不足する。
E3 は E1 の不足前提を補わず、E5・記号導入・昇格は対象外とする。

**対応と原子性。** ソース、Checker 結果、symbols、元宣言・module・signature、
不変 Core 本体・契約、CFG・出口、五義務の元の目標・文脈を所有 schema の下で
再生成・認証する。E2 return identity は自身の generation/VC schema を保ち、
四役割 direct-return identity に付け替えない。各役割の raw slice 全体と元 identity を
同一定義・snapshot に結び付ける。正規ハッシュは snapshot identity を含まないため、
snapshot の一致と同一の不変 Core arena を別途要求する。捕捉ソース・module・依存・
snapshot の現在性は呼出し側の責任であり、受理境界で確立する。
供給された snapshot ID や同名・同ハッシュだけでは不足する。

同一呼出しで得た真正な下位所有者の結果を用い、現在の有効方針の下で実際の rule、
checker/evidence schema、origin、polarity を再評価する。型・一意性の schema 値だけは
方針受理済み証拠ではない。既存の[居住性](../../mizar-proof/ja/status.md#native-e1-inhabitation-adoption)・
[E4](../../mizar-proof/ja/status.md#native-original-e4-adoption)・
[E2](../../mizar-proof/ja/status.md#source-algorithm-return-reflexivity) 経路の限界を保つ。
切り離した token、過去の方針 fingerprint、捏造 generic receipt は代替にならない。
五役割と対応が同じ現在方針で全て成功して初めて限定採用集合を返す。
拒否・エラー時には部分集合・記号・昇格・export を出さず、raw Open/Deferred graph を変更しない。

| 限定形の拒否条件 | 必須結果 |
|---|---|
| 役割の欠落・重複・交換、元の目標・文脈・所有者・schema 違い、Open だけまたは汚染された証拠 | 集合全体を拒否。raw Open 義務には独立した真正な採用が必要であり、状態の付け替えは不可。 |
| E1 の省略、本体全体を覆わない E2 単独ステップ、宣言・等式・継続・停止だけによる E4 代替 | 拒否。一つの役割で他を補わない。 |
| ソース・check・symbols・Core・CFG・契約の変更、異なる snapshot、古い依存、方針無効・非互換 | 真正な上流検査後に全体を拒否し、producer エラーを保持する。 |
| 対応外 profile、必要な provenance・方針入力の未解決 | 拒否。ガード・完全 slice・適格性を捏造しない。 |

**R3 の入力であり export ではない。** 将来の消費側へ、対応する定義、五つの元 identity と
現在方針での採用判断、raw slice 全体、snapshot/provenance の対応、明示した未解決依存を渡す。
unknown/incomplete slice は再利用不可のまま。独立した限定採用で Open4 や目標のない
Deferred termination seed を消さない。G10 はここで登録せず保留し、P13 を制約する。
この凍結は G10 を証明せず、必要な残余条件がある間は定義全体・R3 の準備完了を主張しない。
R3-Q2/R5-Q2 は G/RESERVED/unassigned のままとする。

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

**R2 実装はブロック中。** 維持された依存ゲートは未充足であり、
完全に受理された E1/E2 と適用される E4 producer は未完了。
R3 は定義の証拠が揃えば具体的な呼出しなしで公開できるが、
R4 には別途真正な呼出しが必要。既存 Core 42–53 と VC53 の依存も維持する。
実行形式とローダー、コード抽出用 RuntimeIr、同一パッケージ内の
公開順序は R3 の別設計とする。新しい crate・API・Kernel 規則は作らない。
