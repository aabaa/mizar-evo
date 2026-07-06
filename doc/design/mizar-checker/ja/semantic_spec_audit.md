# 意味論仕様監査: mizar-checker スコープ

> 正本は英語版です。英語正本:
> [../en/semantic_spec_audit.md](../en/semantic_spec_audit.md).

本監査は、`mizar-checker` クレート(パイプラインフェーズ 6-8)が実装すべき
言語仕様章 — 型システム、構造体、attribute、mode、型推論、
cluster/registration、オーバーロード解決 — を対象に、矛盾・未規定ケース・
非決定性の余地・停止性が示せない箇所を洗い出す。あわせて architecture 04/05
および checker TODO との整合を確認する。

本変更のスコープ:

- **仕様本文は編集しない。** 仕様修正が必要な所見はいずれも複数の妥当な解決
  策を持ち、その選択は表層言語または checker の挙動を変えるため、本書に解決
  案を記録し、後続の仕様タスクに委ねる。
- 仕様がすでに一義的に定めている挙動については、敵対的拒否コーパスを
  `tests/miz/fail/` 配下に test-first で固定した
  ([敵対的コーパス](#敵対的コーパス)参照)。

監査対象: `doc/spec/en/` 03, 05, 06, 07, 08, 13, 14, 17, 18, 19;
`doc/design/architecture/en/` 04, 05; `doc/design/mizar-checker/en/todo.md`。

## 重大度凡例

| 重大度 | 意味 |
|---|---|
| critical | 仕様準拠の実装が不健全になり得る(偽が証明可能になる)。 |
| high | 決定なしには checker を決定的に実装できない。または仕様と設計が矛盾。 |
| medium | 未規定の挙動により実装・診断が分岐する。 |
| low | 修正が一意に定まる字句・編集上の欠陥。 |

分類は AGENTS.md の分類法(`spec_gap`、`design_drift` など)に従う。

## 所見一覧

| Id | 重大度 | 領域 | 概要 |
|---|---|---|---|
| SSA-001 | critical | 5.5/5.8 | task 35 で解決済み: constructor-supplied property 値と field-only extensionality が論理を崩壊させていた |
| SSA-002 | high | 5.3/5.4 | task 36 で解決済み: member identity は root declaration と inheritance path/view を追跡 |
| SSA-003 | high | 19.6.1 | テンプレート推論 Case 2-3 が 19.4.3 の ⊑ 基準選択と矛盾 |
| SSA-004 | high | 17.5/17.9.3 | functorial cluster の `for T` 句に FOL エンコードの意味がない |
| SSA-005 | high | 7.4.1 | 重なり合う mode 上の property 実装に coherence 条件がない |
| SSA-006 | high | 17.1 vs arch 04 | registration 活性化時期: 仕様は項目順、設計は verifier 受理まで保留 |
| SSA-007 | medium | 17.10/3.3 | cluster の停止性が adjective 文法の制限に暗黙に依存 |
| SSA-008 | medium | 17.7.3 | 矛盾検出の場所が不整合(ATP か閉包か) |
| SSA-009 | medium | 17.6.4 | reduce の決定性の主張が `such` 条件の文脈依存と矛盾 |
| SSA-010 | medium | 19.4.3/19.4.4 | 同等特異度の別ルートが「唯一の最良」にも「比較不能」にも該当しない |
| SSA-011 | medium | 5.4 vs 19.2.2 | task 36 で解決済み: implicit upcast path uniqueness は syntactic |
| SSA-012 | medium | 5.3 | task 36 で解決済み: inheritance acyclicity と `structures.inherit.cycle` を明文化 |
| SSA-013 | medium | 7.8.1 | 依存(パラメータ付き)mode の `sethood` 義務の形が未提示 |
| SSA-014 | medium | 7.8/17.3.4 | 無属性基底型と組み込み型の存在要件が未記載 |
| SSA-015 | medium | 8.2 | 正当化省略時の `reconsider` の解消経路が未定義 |
| SSA-016 | low | 19.2.3 | 反対称性の主張は閉包同値類上でのみ成立 |
| SSA-017 | low | 6.7/19.4.1 | `coherence with` 省略かつ候補複数時の診断が未規定 |
| SSA-018 | low | 19.6.4 | 貪欲 `of`/`over` 構文解析がスコープ内アリティ集合に依存 |
| SSA-019 | low | 19.6.1 | 編集上: 導入文が 3 回重複 |
| SSA-020 | medium | 3.3/6.2 | 引数リスト形式の attribute `attr(args)` が使用可能だが宣言不能 |

## 所見詳細

### SSA-001 (critical, 解決済み `spec_gap`) — コンストラクタの property 引数対フィールド限定外延性

**該当箇所:** 05.structures.md §5.5.1, §5.8.4, §5.8.5; 07.modes.md §7.4.1。

task 35 より前は、§5.5.1 がデフォルトコンストラクタに **property** 値の供給を
許し(`OneStr(carrier: A, one: b)`)、§5.8.4 はその射影公理
(`one(Agg_OneStr(A, b)) = b`)を生成していた。§5.8.5 の外延性は
**フィールドのみ**を対象としていた。両者は不整合だった: property 型の任意の
`b1, b2` について `Agg_OneStr(A, b1)` と `Agg_OneStr(A, b2)` は全フィールドで
一致するため外延性が両者の同一性を強制し、射影公理から `b1 = b2` が導かれる
— すべてのキャリアが高々 1 要素に崩壊し、偽の命題が証明可能になる。また
property のコンストラクタ引数は §7.4.1 の `means`/`equals` 実装と、値の
供給源として調停なく競合していた。

**影響:** 記載どおりに 3 系統の公理を生成する実装は不健全。コンストラクタ
呼び出しが生成すべき義務の確定が必要で、elaboration/VC 側の実装をブロック
する。

**解決案(いずれかを選択):**

1. コンストラクタは**フィールドのみ**受け取り、property 値は常に property
   実装から得る。§5.5.1 の例と §5.8.4 のアグリゲータを変更。
2. property 引数を維持しつつ、(a) 外延性タプルに property を含め、(b) 実装
   のある property には供給値が定義条件を満たす証明義務を課す。(a) は
   §7.8.2 の一意性の説明および「同一性はフィールドのみで決まる」と衝突する
   点に注意。

推奨は解決案 1(many-sorted set としての読みを保てる)。

**Disposition:** task 35 は解決案 1 を採用した。spec 05 はデフォルト
constructor を field-only とし、property projection axiom を削除した。
spec 07 は property implementation が property 値の唯一の供給源であると
明記した。inactive reject-first seed
`fail_structure_constructor_property_arg_001` は、`advanced_semantics`
runner と source-to-checker payload gap が閉じるまで、拒否される
constructor-property 形を固定する。

### SSA-002 (high, `spec_gap`) — リネーム下のダイアモンドメンバー同一性

**該当箇所:** 05.structures.md §5.3.1(リネーム), §5.4。

§5.4 はダイアモンド整合性の自動確認を「同名・同型」のメンバーに関して定める
一方で「`from` チェーンをたどって確認する」とも述べる。§5.3.1 はリネームを
許すため、1 つの祖父メンバーが親ごとに別名で現れたり、無関係なメンバーが
同名で現れたりし得る。名前基準と起源基準の同一性はまさにその場合に食い違う
が、どちらが規範かは未規定。親同士の型が比較不能な場合に子のメンバー型が
満たすべき条件も示されていない。

**解決:** task 36 は spec 05 に root-plus-path 規則を記録した。継承 member
identity は `from` 写像で到達する root declaration と、そこへ到達する
inheritance path/view を追跡する。root 座標は、カバーされる祖先宣言を識別し、
path/view 座標は reduct term を保持して、名前変更または multi-path view 間の
evidence leakage を防ぐ。子 member は diamond join で複数の parent member を
実現できるが、その型はカバーするすべての parent member 型に対し `⊑` で
なければならない。非同一型義務はその `inherit` 宣言の既存 `coherence`
block が discharge する。異なる root からの同名・同型 join は有効なままで、
異なる名前変更パスから到達した同じ root も distinct child view/selector と
して公開されたままでよい。

**コーパス:** `fail_structure_diamond_member_type_conflict_001`、
`fail_structure_inherit_uncovered_member_001`、task 36 の
`fail_structure_inherit_duplicate_member_coverage_001` が拒否ケースを固定する。
task 36 は、renamed-view exposure が positive behavior であるため
renamed-view reject seed を追加しない。`fail_template_qua_view_attribute_leak_001`
がそれらの view 間で evidence が漏れることへの negative guard のままである。

### SSA-003 (high, `spec_gap`) — テンプレート推論の例が選択規則と矛盾

**該当箇所:** 19.overload_resolution.md §19.6.1 Case 2-3 対 §19.4.3;
architecture 05 「narrow tie-breakers」。

Phase A のインスタンス化後、成功した各テンプレートは `T` を**同一の**引数
正確型に束縛するため、テンプレート由来候補のパラメータベクトルはすべて同一
になる。Case 2 は「overload 2(制約 B)がより厳しい → 勝つ」と主張するが、
制約の厳しさは §19.4.3 の `⊑` 比較の一部ではなく、architecture 05 が許す
タイブレークは非テンプレート優先のみ。Case 3(`f(c)`)は非テンプレート
`f(x: B)` が `C` でインスタンス化されたテンプレートに勝つと主張するが、
`C ⊏ B` によりテンプレート由来候補の方が*厳密に特異*であり、§19.4.3 は
テンプレートを選択する — 逆の結果になる。

**解決案(いずれかを選択):**

1. 明示規則を追加: 同一シンボルのテンプレート由来候補間は**宣言制約**
   (mode 階層)で先に比較し、非テンプレート候補は両者が viable なら常に
   (同点時に限らず)勝つ。§19.4.3 との相互作用を再計算する。
2. インスタンス化後シグネチャ上の純粋な `⊑` 選択を維持し、Case 2/3 の期待
   結果を修正する(Case 2 は曖昧、Case 3 後半はテンプレート選択)。

いずれの場合も architecture 05 のタイブレーク一覧と checker の
`overload_resolution.md` を一致させる必要がある。

### SSA-004 (high, `spec_gap`) — functorial cluster の `for T` にエンコードがない

**該当箇所:** 17.clusters_and_registrations.md §17.5, §17.9.3。

構文は `cluster F(args) -> adjectives for T` だが、§17.9.3 の FOL エンコード
はすべて `T` を落としている(`cluster n ! -> positive for Nat` ⟹
`∀n. is_Nat(n) → is_positive(factorial(n))` — `for Nat` は何も寄与しない)。
候補となる意味は観測可能に異なる: (a) 帰結制約 `is_T(F(args))` を公理に追加;
(b) 適用可能性ガード — 結果がすでに `T` と分かる場所でのみ発火; (c) 説明のみ。
(a) と (b) ではトリガー索引と閉包結果が異なる。

**解決案:** (b) を規定する — 結果型の radix が `T` またはその部分型のとき
に適用(条件 cluster の §17.7.2 と対称)— とし、加えて coherence 義務に
`is_T(F(args))` 前提を生成する。§17.9.3 の表を更新する。

### SSA-005 (high, `spec_gap`) — 重なり合う property 実装に coherence がない

**該当箇所:** 07.modes.md §7.4.1, §7.8.2。

異なる mode でパラメータ化された 2 つの `property S.p means/equals` ブロック
(例: `let M be UnitalMagma` と `let M be Group`)は同一の値に同時適用され
得る。それぞれ自分の mode に相対的な existence/uniqueness を持つが、2 つの
定義条件を関係づけるものがなく、共有インスタンス上で食い違えば uniqueness
公理から矛盾が導かれる。`redefine` は同じ問題を必須 coherence 義務で解決
した(§19.5)が、property 実装には対応物がない。

**解決案:** 同一 struct property の定義域が重なる任意の 2 実装に coherence
義務を要求するか、`inherit` 連結な mode 族ごとに実装を高々 1 つに制限する。

### SSA-006 (high, `design_drift`) — registration 活性化時期

**該当箇所:** 17.clusters_and_registrations.md §17.1 対 architecture 04
「Registration Databases Separate Pending and Activated Registrations」;
todo.md task 19。

仕様は項目順の活性化を約束する: registration は「それ自身の正当性条件が受理
された後」、同一モジュールの後続項目から使用可能。architecture 04(および
実装済みの task-19 暫定方針)は、ローカル registration を同一未検証パス内で
後から使ってはならず、構成済み verifier 方針が義務を受理して初めて活性化
するとする — その受理はまだ存在しないフェーズで起きる。仕様の下では
`fail_mode_existential_after_declaration_001` が利用者の見る唯一の順序エラー
だが、暫定方針の下では*先行する*ローカル registration ですら同一パス内の
mode 宣言を正当化しないため、本来合法なモジュールが拒否される。

**解決案:** §17.1 を言語契約として維持しつつ、正当性条件の受理が非同期で
あり得ることを §17.1 に明記する: 実装はモジュールを保留状態に置いてよいが、
完了した検証パスなら受理する使用箇所を*拒否*してはならない。暫定方針が
保守近似であり `mizar-vc`/`mizar-proof` 到来時に解除されることを
`registration_resolution.md` に記録する。

### SSA-007 (medium, `spec_gap`) — cluster 閉包の停止性が adjective 文法に依存

**該当箇所:** 17.10 `adjective`, 19.2.1, 3.3 `attribute_ref`。

§19.2.1 は「属性集合が有限だから」不動点が存在すると論じる。これが真なのは
cluster 文法が adjective を `[non] [param_prefix] attribute_name`
(`parameter ::= identifier | numeral`)に制限しているからに過ぎない —
帰結が新しいパラメータ項(`(n+1)-dim`)を作れず、functorial cluster は既存
項に事実を付すだけで項を生成しない。この制限が停止性の根拠であることは
明文化されていない。§3.3 の引数リスト形式(`attribute_name(args)`)が
cluster 帰結に許されれば事実空間は項で索引付けられ、素朴な不動点は発散し
得る — その場合 architecture 04 の「saturation limits」が暗黙に意味論化
してしまう。

**解決案:** §17.7.1 に、停止性は adjective 文法(let 束縛パラメータ上の
有限属性語彙)から従うこと、adjective を項引数へ拡張する場合は新たな停止性
論証を要することを明記する。architecture 04 の飽和上限は意味論装置ではなく
防御的診断に留める。

### SSA-008 (medium, `spec_gap`) — 矛盾する導出属性はどこで検出されるか

**該当箇所:** 17.7(ATP 呼び出しなし)対 17.7.3(「ATP 解決中に検出」);
architecture 04 の診断表(解決中の「contradictory derived attributes」)。

§17.7 は cluster 解決を ATP 非依存のグラフ走査と定めるのに、§17.7.3 は矛盾
を「ATP 解決中に」検出されるものとして記述する。checker には確定的な答えが
必要である: 閉包時検出(同一対象に `A` と `non A` が両方入る)は決定可能で
あり、これを規定のトリガーとすべき。ATP 時の不整合は別種の後段障害である。

**解決案:** 閉包時検出を致命的 `cluster` 診断として規定し(§17.7.3 の重大度
と一致)、§17.7.3 は残余の ATP 可視な不整合を別途扱うよう文言修正する。
**コーパス:** `fail_cluster_contradictory_consequent_001` が単一 registration
の静的ケースを固定。

### SSA-009 (medium, `spec_gap`) — reduce の決定性対 `such` 側条件

**該当箇所:** 17.6.4 「Deterministic normalization」と「Matching」行。

正規化は「項とスコープ内規則集合の決定的関数」と宣言されるが、Matching 行は
規則の適用可能性を `such` 条件が「記録済みローカル事実または引用事実として
既に利用可能」かどうか — すなわちローカル証明文脈 — に依存させる。同じ項と
規則でもローカル事実が異なる 2 箇所は異なる正規形になり、宣言された関数
シグネチャは誤りである。加えて「マッチング制約全体にわたる特異度」は、
パターン包摂と §19.2.3 の型特異度が食い違う場合の積順序を定義していない。

**解決案:** 決定性を(項、スコープ内規則、**解消済み側条件集合**)の関数と
して言い直す。結合特異度は、パターン包摂を先に、次に位置ごとのガード比較、
残る混在ケースはすべて比較不能 → FQN タイブレークと定義する。

### SSA-010 (medium, `spec_gap`) — 同等特異度の別ルート

**該当箇所:** 19.2.3 注記, 19.4.3, 19.4.4, 19.1 制限。

パラメータ型の閉包が同一(生の綴りが異なる)2 ルートは「同等に特異」で
ある。両方向に比較可能なため唯一の最良ルートは存在しないが、§19.4.4 は
曖昧性を**比較不能**なルートに対してのみ定義する。関連: 引数シグネチャも
戻り型も同一の 2 つの通常定義は、規定済みの定義衝突(戻り型が異なる場合の
規則)にも解決可能なオーバーロードにも該当しない。

**解決案:** §19.4.4 を「唯一の極大ルートが存在しない」に拡張し(同点を
包含)、§19.1 の衝突規則を戻り型に関わらず同一シグネチャ宣言に拡張する。
**コーパス:** `fail_resolve_same_signature_return_conflict_001` が規定済みの
戻り型衝突を固定。同点ケースは仕様決定待ち。

### SSA-011 (medium, `spec_gap`) — 唯一の「path」対唯一の「embedding」

**該当箇所:** 5.4 対 19.2.2/19.6.2。

§5.4 は同型メンバーの `from` チェーンが一致するダイアモンドの整合性を自動
確認するが、§19.2.2 は**構文的パスが 2 本以上**あれば — すべてのメンバー
埋め込みが一致しアップキャストが意味論的に一意でも — 暗黙アップキャストを
遮断する。パス同一性が構文的(宣言対)か意味論的(メンバー埋め込み)かを
仕様が明言すべきである。task 36 は spec 19 に syntactic choice を記録した。
path は resolved `inherit` declaration path が 1 本の場合にのみ一意である。
整合した member join は overload 解決では複数の reduct/view path を畳み込ま
ないため、2 本以上の path がある場合は明示的な `qua` が必要である。

**コーパス:** `fail_overload_inheritance_path_ambiguity_001` が syntactic
behavior を test-first で固定。

### SSA-012 (medium, `spec_gap`) — 継承の非循環性が明文化されていない

**該当箇所:** 5.3; 13.8.7(cycle-freedom 公理を前提)。

`inherit` 閉包は well-founded でなければならず、§13.8.7 の qua エンコードは
「cycle freedom」を前提とするが、第 5 章は `inherit A extends B; inherit B
extends A;` を禁止しておらず診断名もない。task 36 は §5.3 に明示的な非循環
要件を追加し、diagnostic detail key `structures.inherit.cycle` を明記した。
**コーパス:** `fail_structure_inherit_cycle_001`。

### SSA-013 (medium, `spec_gap`) — 依存 mode の `sethood`

**該当箇所:** 7.8.1。

義務の表は非パラメータ形 `∃S. ∀x. (is_T(x) → x ∈ S)` のみを与える。
`Subset of X` 型の mode では意図される義務はおそらく
`∀params. ∃S. ∀x. (is_T(x, params) → x ∈ S)` であり、§13.4.2 の内包ゲートは
*インスタンス化されたパラメータで* sethood を検査しなければならない。
未記載であり、sethood がモジュール公開インターフェースに含まれるかも未定。

### SSA-014 (medium, `spec_gap`) — 無属性基底型と組み込み型の存在要件

**該当箇所:** 7.8 対 17.3.4。

§7.8 は「基底型の existential registration が見つからなければ」mode 宣言を
ハードエラーとする一方、§17.3.4 は**属性付き**型にのみ registration を要求
する。属性なしの `mode M is set` はどちらに従うのか。`object`、`set`、
struct radix は暗黙に inhabited か(struct を非 inhabited のままにしても
FOL 上は無矛盾)。checker の existential gate(todo task 20)には組み込み
inhabitation 表の明文化が必要。

### SSA-015 (medium, `spec_gap`) — 正当化なしの `reconsider`

**該当箇所:** 8.2 EBNF(「完全に省略可」), 8.2.2。

正当化省略時に、narrowing 義務が cluster 閉包のみで解消されるのか、ATP に
送られるのか、エラーなのかを仕様は述べない。**解決案:** 省略は義務が
widening/閉包証拠のみで解消される場合に限り合法とし、それ以外は正当化を
求める診断とする。

### SSA-016 (low, `spec_gap`) — 反対称性の文言

§19.2.3 は `⊑` を「反対称」と呼ぶが、閉包が等しい構文的に異なる 2 型は
`T₁ ⊑ T₂ ⊑ T₁` を満たす。`⊑` は前順序であり、反対称性は閉包同値類上で
成立する。文言修正のみ(SSA-010 と相互作用)。

### SSA-017 (low, `spec_gap`) — 曖昧な `coherence with` 省略

§19.4.1 は `coherence with` なしの `redefine` を「シグネチャを研ぎ澄ます
唯一の先行定義」に割り当てる。複数該当する場合のエラー名・挙動が未規定。
「redefinition 対象曖昧」診断を規定すべき。

### SSA-018 (low, `design_drift`) — 貪欲 `of`/`over` 解析のスコープ依存

§19.6.4 の最長一致規則により `M of A, B` の構文木は可視アリティに依存し、
import 追加が既存テキストを再解析し得る。文書化済みで決定的だが脆弱。
低アリティ解釈も存在する場合の lint を推奨。パーサが resolver のアリティ
情報を必要とする点(レイヤリング)にも注意。

### SSA-019 (low, 編集上) — 文の重複

§19.6.1 で「The following examples use an abstract mode hierarchy ...」が
3 回連続で繰り返される。編集上の整理。

### SSA-020 (medium, `spec_gap`) — `attr(args)` は使用可能だが宣言不能

`attribute_ref`(§3.3, §6.9)は `attribute_name "(" argument_list ")"` を
許すが、§6.2 はハイフン `param_prefix` パラメータの宣言形しか定めず、
cluster の `adjective` 文法(§17.10)は引数リスト形式を完全に除外する。
引数リスト attribute の宣言・registration の扱いを定義するか、
`attribute_ref` から形式を除去するかのいずれかが必要。SSA-007 と相互作用
(cluster への許容は停止性論証を壊す)。

## 敵対的コーパス

監査では 16 件の拒否 fixture を test-first で固定した(sidecar + traceability
エントリ。いずれも `advanced_semantics` ランナーと source-to-checker
ペイロード抽出が存在するまで inactive seed — MC-G020/MC-G021/MC-G023/
MC-G027)。task 35 は同じ inactive-seed 規則の下で SSA-001 の
constructor-property seed を後から追加する。task 36 は同じ規則の下で
duplicate-member-coverage seed を追加しつつ、renamed-view exposure は既存
template view-leak seed に guard される positive behavior として残す。既存テストと
期待値は implementation behavior に合わせて rebaseline していない。

| Fixture | 対象挙動 | 仕様 |
|---|---|---|
| `fail/clusters/fail_cluster_reduce_cycle_orientation_001` | reduce 循環は登録不能(サイズ順序) | 17.6.4 |
| `fail/clusters/fail_cluster_reduce_commutative_orientation_001` | 同サイズ向き付けの拒否 | 17.6.4 |
| `fail/clusters/fail_cluster_reduce_fresh_variable_001` | RHS 新規変数の拒否 | 17.6.4 r1 |
| `fail/clusters/fail_cluster_reduce_duplicating_variable_001` | RHS 出現数増加の拒否 | 17.6.4 r2 |
| `fail/clusters/fail_cluster_contradictory_consequent_001` | 矛盾する帰結 adjective | 17.4, 17.7.3 |
| `fail/modes/fail_mode_missing_existential_001` | existential 証拠なしの属性付き型 | 17.3.4, 7.8 |
| `fail/modes/fail_mode_existential_after_declaration_001` | 活性化は項目順で遡及しない | 17.1, 7.8 |
| `fail/structures/fail_structure_diamond_member_type_conflict_001` | root+path/view identity 下の join member 型不整合 | 5.3.1, 5.4 |
| `fail/structures/fail_structure_inherit_duplicate_member_coverage_001` | parent member coverage の重複 | 5.3.1 |
| `fail/structures/fail_structure_inherit_cycle_001` | 継承循環 | 5.3, 13.8.7 |
| `fail/structures/fail_structure_inherit_uncovered_member_001` | 基底メンバーの未カバー | 5.3.1 |
| `fail/overload/fail_overload_incomparable_roots_001` | 比較不能ルート → 曖昧性 | 19.2.3, 19.4.4 |
| `fail/overload/fail_overload_inheritance_path_ambiguity_001` | 複数パスのアップキャストは `qua` 必須 | 19.2.2, 19.6.2 |
| `fail/resolve/fail_resolve_same_signature_return_conflict_001` | 同一シグネチャ・異なる戻り型 | 19.1 |
| `fail/types/fail_types_qua_narrowing_001` | `qua` narrowing の拒否 | 13.6.4, 8.2.2 |
| `fail/types/fail_types_qua_unrelated_struct_001` | 無関係 struct への `qua` の拒否 | 13.6.1, 13.6.4 |
| `fail/types/fail_types_comprehension_missing_sethood_001` | sethood なしの Fraenkel の拒否 | 13.4.2, 7.8.1 |
| `fail/structures/fail_structure_constructor_property_arg_001` | constructor property 引数の拒否 | 5.5.1, 5.8.4, 7.4.1 |

新規 traceability 要件: `spec.en.05.structures.constructor_fields_only.semantic`、
`spec.en.05.structures.inheritance.semantic`、
`spec.en.07.modes.property_implementation.not_constructor_source.semantic`、
`spec.en.07.modes.existential_gating.semantic`、
`spec.en.13.qua.widening_only.semantic`、
`spec.en.13.sethood.comprehension.semantic`、
`spec.en.17.clusters.pattern_consistency.semantic`、
`spec.en.17.reductions.termination_order.semantic`、
`spec.en.19.overload.ambiguity.semantic`、
`spec.en.19.overload.definition_conflict.declaration`。

## mizar-checker TODO への影響

推奨のみ。todo.md の改訂は後続タスクで行う。

- **解決済み仕様タスク:** SSA-001(constructor/extensionality) は task 35 で
  解決済み。`doc/spec/en/` + `ja/` の同期編集と inactive reject-first corpus
  seed を含む。SSA-002+SSA-011+SSA-012 は task 36 で解決済み。spec 05/19 の
  同期編集、duplicate-member-coverage seed、traceability note 更新を含む。
  renamed-view reject seed は不要だった。
- **残りの仕様タスク(checker 意味論の続行前):** SSA-003(テンプレートタイブレーク)、
  SSA-004(functorial `for` の意味論)、SSA-005(property 実装の coherence)に
  各 1 タスク。いずれも `doc/spec/en/` + `ja/` を同一変更で更新。
- **Task 19/20(registration ゲート、existential ゲート):** SSA-006 の
  活性化契約と SSA-014 の組み込み inhabitation 表の決定後に再訪。暫定的な
  保守方針は `registration_resolution.md` にその旨を記録すべき。
- **Task 16-18(閉包、ループ、reduce):** SSA-007 の文法基準の停止性論証と
  SSA-008 の閉包時矛盾規則を `cluster_trace.md`/`registration_resolution.md`
  に反映。reduce の決定性には SSA-009 の修正済み関数シグネチャが必要。
- **Task 23-26(テンプレート、viability、選択):** 実ペイロード前に
  SSA-003 と SSA-010 の決定が必要。`overload_resolution.md` に選択された
  タイブレークと同点曖昧性の規則を記録すべき。
- **Task 29 コーパス記録:** deferred の advanced_semantics コーパス要件
  2 件に具体的な兄弟 seed ができた。ランナー到来時に、deferred 記録を上記
  8 件の新規要件 id を指すよう(または置き換えるよう)改訂すべき。
