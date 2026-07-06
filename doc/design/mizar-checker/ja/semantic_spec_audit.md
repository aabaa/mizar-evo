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
| SSA-003 | high | 19.6.1 | task 37 で解決済み: 展開後の Phase B ではテンプレート制約をタイブレークに使わない |
| SSA-004 | high | 17.5/17.9.3 | task 38 で解決済み: functorial cluster の `for T` は FOL エンコード上の適用可能性 guard |
| SSA-005 | high | 7.4.1 | task 39 で解決済み: 重なり合う property 実装には coherence が必要 |
| SSA-006 | high | 17.1 vs arch 04 | task 40 で解決済み: item-ordered activation は非同期受理を許し、後続の受理済み use を最終拒否しない |
| SSA-007 | medium | 17.10/3.3 | cluster の停止性が adjective 文法の制限に暗黙に依存 |
| SSA-008 | medium | 17.7.3 | 矛盾検出の場所が不整合(ATP か閉包か) |
| SSA-009 | medium | 17.6.4 | reduce の決定性の主張が `such` 条件の文脈依存と矛盾 |
| SSA-010 | medium | 19.4.3/19.4.4 | task 37 で解決済み: 曖昧性は同値ルートを含む複数極大ルートを扱う |
| SSA-011 | medium | 5.4 vs 19.2.2 | task 36 で解決済み: implicit upcast path uniqueness は syntactic |
| SSA-012 | medium | 5.3 | task 36 で解決済み: inheritance acyclicity と `structures.inherit.cycle` を明文化 |
| SSA-013 | medium | 7.8.1 | 依存(パラメータ付き)mode の `sethood` 義務の形が未提示 |
| SSA-014 | medium | 7.8/17.3.4 | 無属性基底型と組み込み型の存在要件が未記載 |
| SSA-015 | medium | 8.2 | 正当化省略時の `reconsider` の解消経路が未定義 |
| SSA-016 | low | 19.2.3 | task 37 で解決済み: 特異度は閉包同値で商を取る前は前順序 |
| SSA-017 | low | 6.7/19.4.1 | `coherence with` 省略かつ候補複数時の診断が未規定 |
| SSA-018 | low | 19.6.4 | 貪欲 `of`/`over` 構文解析がスコープ内アリティ集合に依存 |
| SSA-019 | low | 19.6.1 | task 37 で解決済み: 重複した導入文を削除 |
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

### SSA-003 (high, 解決済み `spec_gap`) — テンプレート推論の例が選択規則と矛盾していた

**該当箇所:** 19.overload_resolution.md §19.6.1 Case 2-3 対 §19.4.3;
architecture 05 「narrow tie-breakers」。

task 37 は保守的な仕様規則を採用した。Phase A は具体的なテンプレート由来
候補を生成し、Phase B はそれらの具体的な正規化済みパラメータベクトルを
通常の `⊑` 前順序で比較する。宣言されたテンプレート制約の厳しさは Phase B
のタイブレークではない。非テンプレート候補が勝つのは、具体的なベクトルが
テンプレート由来ベクトルと閉包同値で、他の許可済みタイブレークもすべて
同点の場合に限る。

§19.6.1 の例はこの規則に従うようになった。Case 2 では、2 つの
テンプレート由来ルートが同じ具体ベクトルにインスタンス化されるが、別の
通常ルートから来るため曖昧である。Case 3 では、引数の正確型が `C` のとき、
具体パラメータ `C` のテンプレート由来候補が非テンプレート `B` 候補より
`C ⊏ B` により選択される。

architecture 05 と checker の `overload_resolution.md` は同じタイブレーク
一覧を持つ。この決定は `mizar-core` task 26 / F7 が記録した別個の Phase A
規則と整合する。省略テンプレート引数の推論は declared argument type に基づき、
欠落 payload から推測してはならない。

### SSA-004 (high, 解決済み `spec_gap`) — functorial cluster の `for T` にエンコードがない

**該当箇所:** 17.clusters_and_registrations.md §17.5, §17.9.3。

構文は `cluster F(args) -> adjectives for T` だが、§17.9.3 の FOL エンコード
はすべて `T` を落としている(`cluster n ! -> positive for Nat` ⟹
`∀n. is_Nat(n) → is_positive(factorial(n))` — `for Nat` は何も寄与しない)。
候補となる意味は観測可能に異なる: (a) 帰結制約 `is_T(F(args))` を公理に追加;
(b) 適用可能性ガード — 結果がすでに `T` と分かる場所でのみ発火; (c) 説明のみ。
(a) と (b) ではトリガー索引と閉包結果が異なる。

**解決案:** (b) を規定する — 結果の既知の正規化型が完全な `for` 型式
そのもの、またはその subtype である場所で適用される(条件 cluster の §17.7.2
と対称)— とし、加えて coherence 義務に `is_T(F(args))` 前提を生成する。
§17.9.3 の表を更新する。

**処置:** task 38 は applicability-guard 解決を採用し、radix だけでなく完全な
正規化済み `for` 型式に精緻化した。spec 17 は、valid な functorial
registration が発火するのは関手結果が guarded type expression またはその
subtype としてすでに分かっている場合だけであり、coherence/FOL encoding が
結果 guard を前提として含むと述べる。inactive seed
`fail_cluster_functorial_for_guard_001` は、advanced-semantics runner と
checker-ready payload extraction が存在するまで、consequent が利用不能なケースを
固定する。

### SSA-005 (high, resolved `spec_gap`) — 重なり合う property 実装には coherence が必要

**該当箇所:** 07.modes.md §7.4.1, §7.8.2。

異なる mode でパラメータ化された 2 つの `property S.p means/equals` ブロック
(例: `let M be UnitalMagma` と `let M be Group`)は同一の値に同時適用され
得る。それぞれ自分の mode に相対的な existence/uniqueness を持つが、2 つの
定義条件を関係づけるものがなく、共有インスタンス上で食い違えば uniqueness
公理から矛盾が導かれる。`redefine` は同じ問題を必須 coherence 義務で解決
した(§19.5)が、property 実装には対応物がない。

**処置:** task 39 は coherence 義務による解決を選んだ。spec 07 は property
`means`/`equals` implementation の後に `coherence` block を許し、同じ struct
property の先行可視 implementation と正規化済み mode domain が重なる場合に
その block を要求する。この義務は、重なり上で両実装が同じ property 値を
割り当てることを証明する。spec 16 の proof-obligation summary と Appendix A の
grammar mirror も同期した。inactive seed
`fail_mode_property_overlap_missing_coherence_001` は、property payload extraction
と property implementation parser support、advanced-semantics runner が存在するまで、
coherence を欠く狭い重なり実装の拒否を固定する。

### SSA-006 (high, 解決済み `design_drift`) — registration 活性化時期

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

**解決(task 40):** §17.1 は言語契約のままである。正当性条件の受理が非同期で
あり得ることを明記した: 実装は module または依存 use site を pending にしてよいが、
完了した verification pass は、先行 registration の correctness condition が受理された後に
受理されるはずの後続 use を最終的に reject してはならない。Architecture 04 と
`registration_resolution.md` は task-19 の accepted input 不在 policy を、
`mizar-vc`/`mizar-proof`/artifact integration が accepted status を供給した時点で解除すべき
暫定的な保守近似として明記した。既存 inactive seed
`fail_mode_existential_after_declaration_001` が negative non-retroactive slice を固定する。
positive accepted-local activation は MC-G020/MC-G021/MC-G025/MC-G026 により deferred のままである。

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

### SSA-010 (medium, 解決済み `spec_gap`) — 同等特異度の別ルート

**該当箇所:** 19.2.3 注記, 19.4.3, 19.4.4, 19.1 制限。

task 37 以前は、具体的な正規化済みパラメータ型の閉包が同一の 2 ルートは
両方向に比較可能だったため、call site に唯一の最良ルートはない一方で、
§19.4.4 は**比較不能**なルートだけを曖昧性としていた。関連して、引数
シグネチャも戻り型も同一の 2 つの通常定義は、規定済みの定義衝突にも
解決可能な overload にも該当しなかった。

task 37 は、少なくとも 1 つの viable root が存在し、タイブレーク後の
極大ルート集合に 2 つ以上の別 ordinary root が残る場合を曖昧性に拡張する。
これにより比較不能ルートと閉包同値ルートの両方を扱う。また §19.1 の衝突
規則も拡張し、同名かつ同一の引数型シグネチャを持つ 2 つの通常定義は
戻り型に関わらず定義衝突とする。

**コーパス:** `fail_overload_equivalent_roots_ambiguity_001` は同値ルート
曖昧性の inactive advanced-semantics seed である。
`fail_resolve_same_signature_return_conflict_001` は active な異なる戻り型の
宣言衝突 seed のまま残り、`fail_resolve_same_signature_same_return_conflict_001`
は resolver diagnostic が同一戻り型重複を扱うまで inactive declaration-symbol
seed とする。

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

### SSA-016 (low, 解決済み `spec_gap`) — 反対称性の文言

task 37 以前、§19.2.3 は `⊑` を「反対称」と呼んでいたが、閉包が等しい
構文的に異なる 2 つの具体的な正規化済み型式は `T₁ ⊑ T₂ ⊑ T₁` を満たす。
task 37 は、特異度を具体的な正規化済み型式上の前順序として言い直した。
反対称性は閉包同値類で商を取った後にのみ成立する。この文言は SSA-010 の
曖昧性決定の一部である。

### SSA-017 (low, `spec_gap`) — 曖昧な `coherence with` 省略

§19.4.1 は `coherence with` なしの `redefine` を「シグネチャを研ぎ澄ます
唯一の先行定義」に割り当てる。複数該当する場合のエラー名・挙動が未規定。
「redefinition 対象曖昧」診断を規定すべき。

### SSA-018 (low, `design_drift`) — 貪欲 `of`/`over` 解析のスコープ依存

§19.6.4 の最長一致規則により `M of A, B` の構文木は可視アリティに依存し、
import 追加が既存テキストを再解析し得る。文書化済みで決定的だが脆弱。
低アリティ解釈も存在する場合の lint を推奨。パーサが resolver のアリティ
情報を必要とする点(レイヤリング)にも注意。

### SSA-019 (low, 解決済み 編集上) — 文の重複

task 37 はテンプレートタイブレーク例の更新と合わせて、§19.6.1 の重複した
導入文を削除した。

### SSA-020 (medium, `spec_gap`) — `attr(args)` は使用可能だが宣言不能

`attribute_ref`(§3.3, §6.9)は `attribute_name "(" argument_list ")"` を
許すが、§6.2 はハイフン `param_prefix` パラメータの宣言形しか定めず、
cluster の `adjective` 文法(§17.10)は引数リスト形式を完全に除外する。
引数リスト attribute の宣言・registration の扱いを定義するか、
`attribute_ref` から形式を除去するかのいずれかが必要。SSA-007 と相互作用
(cluster への許容は停止性論証を壊す)。

## 敵対的コーパス

元の監査では 16 件の拒否 fixture を test-first で固定した(sidecar +
traceability エントリ)。`advanced_semantics` fixture は advanced runner と
source-to-checker ペイロード抽出が存在するまで inactive seed のままである —
MC-G020/MC-G021/MC-G023/MC-G027。task 35 は同じ inactive advanced-semantics
規則の下で SSA-001 の constructor-property seed を後から追加する。task 36 は
同じ規則の下で duplicate-member-coverage seed を追加しつつ、renamed-view
exposure は既存 template view-leak seed に guard される positive behavior として
残す。task 37 は inactive な ordinary / template-derived equivalent-root
ambiguity seed と、inactive な same-return signature-conflict declaration seed を
追加する。後者は対応する resolver diagnostic を待つ。既存の different-return
signature-conflict declaration seed は active のままである。既存テストと期待値は
implementation behavior に合わせて rebaseline していない。
task 38 は同じ advanced-semantics ルールの下で inactive functorial-`for` guard
seed を追加する。

| Fixture | 対象挙動 | 仕様 |
|---|---|---|
| `fail/clusters/fail_cluster_reduce_cycle_orientation_001` | reduce 循環は登録不能(サイズ順序) | 17.6.4 |
| `fail/clusters/fail_cluster_reduce_commutative_orientation_001` | 同サイズ向き付けの拒否 | 17.6.4 |
| `fail/clusters/fail_cluster_reduce_fresh_variable_001` | RHS 新規変数の拒否 | 17.6.4 r1 |
| `fail/clusters/fail_cluster_reduce_duplicating_variable_001` | RHS 出現数増加の拒否 | 17.6.4 r2 |
| `fail/clusters/fail_cluster_contradictory_consequent_001` | 矛盾する帰結 adjective | 17.4, 17.7.3 |
| `fail/clusters/fail_cluster_functorial_for_guard_001` | `for` guard 外の functorial consequent 利用不可 | 17.5, 17.9.3 |
| `fail/modes/fail_mode_missing_existential_001` | existential 証拠なしの属性付き型 | 17.3.4, 7.8 |
| `fail/modes/fail_mode_existential_after_declaration_001` | 活性化は項目順で遡及しない | 17.1, 7.8 |
| `fail/structures/fail_structure_diamond_member_type_conflict_001` | root+path/view identity 下の join member 型不整合 | 5.3.1, 5.4 |
| `fail/structures/fail_structure_inherit_duplicate_member_coverage_001` | parent member coverage の重複 | 5.3.1 |
| `fail/structures/fail_structure_inherit_cycle_001` | 継承循環 | 5.3, 13.8.7 |
| `fail/structures/fail_structure_inherit_uncovered_member_001` | 基底メンバーの未カバー | 5.3.1 |
| `fail/overload/fail_overload_incomparable_roots_001` | 比較不能ルート → 曖昧性 | 19.2.3, 19.4.4 |
| `fail/overload/fail_overload_equivalent_roots_ambiguity_001` | 同値な別ルート → 曖昧性 | 19.2.3, 19.4.4 |
| `fail/overload/fail_overload_template_equivalent_roots_ambiguity_001` | 同値な template-derived root → 曖昧性 | 19.4.4, 19.6.1 |
| `fail/overload/fail_overload_inheritance_path_ambiguity_001` | 複数パスのアップキャストは `qua` 必須 | 19.2.2, 19.6.2 |
| `fail/resolve/fail_resolve_same_signature_return_conflict_001` | 同一シグネチャ・異なる戻り型 | 19.1 |
| `fail/resolve/fail_resolve_same_signature_same_return_conflict_001` | 同一シグネチャ・同一戻り型 | 19.1 |
| `fail/types/fail_types_qua_narrowing_001` | `qua` narrowing の拒否 | 13.6.4, 8.2.2 |
| `fail/types/fail_types_qua_unrelated_struct_001` | 無関係 struct への `qua` の拒否 | 13.6.1, 13.6.4 |
| `fail/types/fail_types_comprehension_missing_sethood_001` | sethood なしの Fraenkel の拒否 | 13.4.2, 7.8.1 |
| `fail/structures/fail_structure_constructor_property_arg_001` | constructor property 引数の拒否 | 5.5.1, 5.8.4, 7.4.1 |
| `fail/modes/fail_mode_property_overlap_missing_coherence_001` | coherence を欠く重なり property 実装の拒否 | 7.4.1, 7.8.2 |

新規 traceability 要件: `spec.en.05.structures.constructor_fields_only.semantic`、
`spec.en.05.structures.inheritance.semantic`、
`spec.en.07.modes.property_implementation.coherence.semantic`、
`spec.en.07.modes.property_implementation.parser`、
`spec.en.07.modes.property_implementation.not_constructor_source.semantic`、
`spec.en.07.modes.existential_gating.semantic`、
`spec.en.13.qua.widening_only.semantic`、
`spec.en.13.sethood.comprehension.semantic`、
`spec.en.17.clusters.pattern_consistency.semantic`、
`spec.en.17.clusters.functorial_for_guard.semantic`、
`spec.en.17.reductions.termination_order.semantic`、
`spec.en.19.overload.ambiguity.semantic`、
`spec.en.19.overload.definition_conflict.declaration`、
`spec.en.19.overload.definition_conflict.same_return.declaration`。

## mizar-checker TODO への影響

この節は spec-decision task が閉じるたびに `todo.md` と crate plan をどう
同期し続けるかを記録する。

- **解決済み仕様タスク:** SSA-001(constructor/extensionality) は task 35 で
  解決済み。`doc/spec/en/` + `ja/` の同期編集と inactive reject-first corpus
  seed を含む。SSA-002+SSA-011+SSA-012 は task 36 で解決済み。spec 05/19 の
  同期編集、duplicate-member-coverage seed、traceability note 更新を含む。
  renamed-view reject seed は不要だった。SSA-003+SSA-010+SSA-016+SSA-019 は
  task 37 で解決済み。spec 19 の同期編集、overload design の同期、equivalent-root /
  same-signature corpus seed を含む。SSA-004 は task 38 で解決済み。spec 17 の
  同期編集と functorial-`for` guard seed を含む。SSA-005 は task 39 で
  解決済み。spec 07/16/Appendix A の同期編集、overlapping-property coherence
  seed、property implementation syntax の deferred parser traceability row を含む。
  SSA-006 は task 40 で解決済み。spec 17、architecture 04、checker
  registration-resolution の同期編集と、既存 non-retroactive activation seed 用
  traceability row を含む。
- **残りの仕様タスク(checker 意味論の続行前):** SSA-007、SSA-008、
  SSA-009、SSA-013、SSA-014、SSA-015、SSA-017、SSA-020 が tasks 41-44 に残る。
- **Task 19/20(registration ゲート、existential ゲート):** SSA-006 の
  活性化契約は task 40 で、task-19 activation policy を暫定的な保守近似として
  記録した。SSA-014 の組み込み inhabitation 表の決定後に再訪する。
- **Task 16-18(閉包、ループ、reduce):** SSA-007 の文法基準の停止性論証と
  SSA-008 の閉包時矛盾規則を `cluster_trace.md`/`registration_resolution.md`
  に反映。reduce の決定性には SSA-009 の修正済み関数シグネチャが必要。
- **Task 23-26(テンプレート、viability、選択):** task 37 は Phase B の
  タイブレークと同点曖昧性規則を記録する。実ペイロード作業では欠落した
  比較 evidence を推測してはならない。`mizar-core` task 26 / F7 は、別個の
  Phase A 省略テンプレート引数推論決定性規則を記録する。
- **Task 29 コーパス記録:** deferred の advanced_semantics コーパス要件
  2 件に具体的な兄弟 seed ができた。ランナー到来時に、deferred 記録を上記
  の該当する監査由来 requirement id を指すよう(または置き換えるよう)改訂
  すべきである。task 38 が追加した functorial-`for` guard row、task 37 が追加した
  overload ambiguity row と deferred same-return declaration-conflict row も含める。
