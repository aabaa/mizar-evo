# テンプレート論理エンコーディング健全性監査(仕様18章、§18.10)

> 正本は英語版:
> [../en/template_encoding_audit.md](../en/template_encoding_audit.md)。

## 目的

本書は、[doc/spec/en/18.templates.md](../../../spec/en/18.templates.md) に
規定されたテンプレート機構の一階論理エンコーディング(特に §18.10)に対する
論理レベルの健全性監査である。テンプレートは第二階的パターン(scheme、
型パラメータ、述語/functor パラメータ)を使用箇所での一階公理生成に
還元する。監査の中心的問いは次のとおり:

> 規定されたエンコード規則をすべて通過するが、意図された集合論的意味論で
> 偽となる一階公理を生成するインスタンス化が存在するか?

各エンコード規則について (a) 健全性が成り立つ理由を再構成し、(b) 論証が
崩れる箇所・規則が曖昧な箇所には `.miz` 断片による具体的反例候補を示す。
タスク範囲に従い監査した相互作用:

- 3章の型消去・widening
- 5章の構造体エンコード(Critical 所見の根本原因として到達)
- 7章のパラメータ化 mode に対する `sethood` 義務
- 13章の choice(`the T`)、Fraenkel 内包、`qua` エンコード
- §18.9 scheme application と §18.2.7 パラメータ推論の決定性
- 20章 algorithm templates(§18.8.4)

本書は監査成果物である。元の監査変更で実施した仕様修正は各所見の **処置** に
記載し、実装 follow-up は現在の `mizar-core` と roadmap TODO で追跡する。
task 27 は explicit-payload elaborator progress を記録している。

## 方法と健全性の基準線

テンプレート系の大域的健全性論証は次の形をとる(§18.10.1 から再構成):

1. **記号的検証。** テンプレート本体は宣言時に一度だけ、各 schema
   パラメータの未解釈記号(`is_T/1`、`P/n`、`F/n`)、宣言されたガード、
   `such that` 制約を仮説として含む文脈 Γ_schema で検証される。
2. **代入としてのインスタンス化。** 使用箇所は未解釈記号に実引数を代入し、
   代入後の公理・定理インスタンスを生成する。
3. **代入補題。** 等号付き古典一階論理で Γ_schema ⊢ φ が成り立ち、σ が
   各未解釈記号を定義可能な解釈に写し σ(Γ_schema) が成立するなら、σ(φ)
   も成立する。

ステップ 3 は標準的メタ定理なので、エンコーディングが健全であるのは、
Γ_schema に現れるすべての仮定がすべてのインスタンス化で実際に保証(証明・
検査・前件化)され、かつ代入が真に capture-avoiding な一階代入である
**場合に限る**。以下の各所見は、仕様が (i) インスタンス化側で保証されない
仮定を Γ_schema に暗黙に含めている箇所、(ii) 代入そのものが未定義な箇所、
(iii) 数学的に異なる 2 つのインスタンス化が生成 FOL 上で区別不能になる
箇所のいずれかである。

深刻度: **Critical** = 現行本文の自然な読みで偽の公理が導出可能。
**Major** = 規則が未定義で、実装のもっともらしい読みの少なくとも一つが
不健全。**Medium** = 合成時に健全性へ影響する規則の欠落。**Minor** = 例や
非規範的本文の欠陥。

## 規則ごとの健全性再構成

### §18.10.1 axiom schema としてのテンプレート

**健全である理由(意図どおりなら):** 一度だけ検証 + 代入補題。本体内の
すべての証明義務(`func` の existence/uniqueness、mode existence、
sethood、registration の正当性条件)はパラメータのガードに相対的に
放される。**ただし条件付き:** F1–F5 参照 — Γ_schema のいくつかの仮定が
現行本文ではインスタンス化まで追跡されない。

### §18.10.2 型パラメータのエンコード

**健全である理由(素の `let T be type`):** `is_T/1` は未解釈で何も仮定
されない。任意の型式 σ(is_T) := λx. is_σT(x) は定義可能述語であり代入
補題が適用される。ただし Γ_schema が `T` の非空性・sethood を仮定しない
こと(F2、F5)、素の型パラメータに値レベル実引数を許さないこと(F3)が
前提。

**書かれたままでは不健全(`let T be type extends M`):** 表はこの
パラメータを schema 述語 `is_T/1` + ガード `∀x. is_T(x) → is_M(x)` と
するが、§18.2.2 は同じパラメータを**オブジェクトレベルのインスタンス
ビュー**(`PermProduct[R qua AddMagma]`)でインスタンス化する。単項述語
記号への項の代入は定義されておらず、本体での `T` の構造体的使用
(`T.binop`、再写像された記法 `*`、`Element of T`、`FinSequence of T`)に
対する FOL 規則が表に存在しない。F3 参照。ビュー同一性の問題は F1。

### §18.10.3 述語パラメータのエンコード

**健全である理由:** `P/n` は未解釈。インスタンスは `defpred` の閉包を
capture-avoiding 展開(§15.11.3)で代入し、一階定義可能なので各
インスタンスは通常の FOL 論理式であり、本体が `P` 上で行う内包は ZF の
分出・置換公理で覆われる。インライン λ 引数の禁止(§18.9)により実引数は
解決可能かつ定義可能に保たれる。残る欠落: 実引数のシグネチャ適合
(F4/F6 の随伴規則)と、テンプレート本体内で外側パラメータを実引数として
scheme を適用する場合(F6)。

### §18.10.4 functor パラメータのエンコード

**健全である理由(意図された読み):** `F/n` は未解釈で、型ガード
`∀x. is_T(x) → is_S(F(x))` は記号的検証では**仮定**され、インスタンス化
時に実引数の宣言シグネチャから**再確立**される。
**書かれたままでは曖昧:** 表はガードを `F` に付随する公理と呼び、
§18.10.1 はインスタンス化が「公理を生成する」と述べる。この読みでは
インスタンス化されたガードは証明されず主張される — 結果型が `T` 全域で
`S` に収まらない実引数に対して偽の公理となる。F4 参照。

### §18.10.5 制約のエンコード

**健全である理由:** `such that φ` は Γ_schema で仮定され、各使用箇所で
証明され、生成されるすべての公理で前件として保持される。呼び出し側の
証明が誤っているインスタンスは不健全ではなく証明不能になる。定理仮説は
通常の前件としてエンコードされる。反例は見つからなかった。本体内で放た
れた正当性義務(mode existence、functor の existence/uniqueness、
sethood)のエクスポートされるインスタンスが制約前件を継承する必要がある
点は、本文の「すべての生成公理で前件として働く」で足りることを確認した。

### 所見なしの相互作用再構成

- **choice の安定性(§13.5):** `the T` は**テンプレートインスタンス化後
  の**所有コア項目ごとに `choice_T(params)` へ低下するため、異なる
  インスタンス化は異なる choice 項を得て(params はインスタンス化された
  パラメータを含む)、同一インスタンス内の再出現は同一項を共有する。
  インスタンスごとの義務を伴う Hilbert ε として健全 — ただし非空性義務
  自体の扱いは F2 に依存する。
- **テンプレートブロック内の registration:** conditional cluster は
  ガード付き全称定理であり、インスタンスは代入で従う。cluster 無矛盾性・
  inherit 非循環性(§13.8.7)は公理ではなく検査であり、テンプレート
  インスタンスがそこから矛盾を持ち込むことはできない。
- **algorithm templates(§18.8.4、20章):** VC スキーマ(§20.13.3)は
  `F`、`P` を未解釈のまま記号的に放たれる。未解釈記号で証明可能な VC は
  すべてのインスタンスで証明可能(代入補題)なので記号的検証は保守的。
  停止測度は functor ガード経由で `Nat` 値であることを要し、`F` に依存
  する測度はガードから証明可能な場合のみ受理される — 保守的ゆえ健全。
  promotion 公理(§20.13.2)は検証済み契約のインスタンスごとの代入。
  計算不能な実引数に対する実行時検証失敗(§20.9.3)は実行側のみで、
  検証側の健全性を弱めない。
- **型消去(§3.7.3):** テンプレートインスタンスは既にガード形式で生成
  されるため、ATP 向け消去は F1 を超えるテンプレート固有の危険を追加
  しない。

## 所見

### F1(Critical)— 構造体ビューの崩壊: フィールド改名を伴うダイヤモンド継承はフラット化エンコードの下で矛盾し、テンプレートの `qua` インスタンス化がその上に載っている

**関与規則:** §5.8.3、§5.8.5、§13.8.7(「FOL エンコーディングで項 `x`
自体は不変」)、§18.2.2(記法再写像、`qua` 実引数)、§18.10.2。

**欠陥。** §5.8.3 は `inherit D extends B where field d from b` を包摂 +
**大域セレクタ等式**でエンコードする:
`∀x. is_D(x) → is_B(x)` および `∀x. is_D(x) → d(x) = b(x)`。18章自身が
主要例に使うダイヤモンド(Ring は AddGroup 経由と MulMonoid 経由で Magma
に到達し、一方の経路で `field add from binop`、他方で
`field mul from binop`)では、任意の Ring `R` について:

```
add(R) = binop(R)        (is_AddGroup(R) より)
mul(R) = binop(R)        (is_MulMonoid(R) より)
⟹ add(R) = mul(R)
```

`1+1 ≠ 1*1` となる具体的な環では、テンプレート不要でカーネル理論内に ⊥
が導出される。独立に、§5.8.5 の外延性は `is_S` 上で述べられており、包摂
によりより豊かな子孫にも適用される: `carrier` と `zero` が一致し `one`
だけ異なる 2 つの `ZeroOneStr` 値は `ZeroStr` の外延性で等しいと強制され
る — 再び ⊥。

**テンプレートレベルの反例(本監査が探索対象とした「エンコードを通過する
が不健全」なインスタンス)。** §13.8.7 が `qua` を消去する(「x 不変」)
ため、同一項の異なるビュー上の属性アトムは**同一の FOL アトム**になる:
`R qua AddMagma is commutative` と `R qua MulMagma is commutative` は
ともに `is_commutative(R)` へ低下する。`R` を行列環(`+` では可換、`*`
では非可換)とすると:

```mizar
Comm: R qua AddMagma is commutative by mml.algebra.matrix.add_comm;
:: `let T be type extends commutative Magma` の bound 検査は
:: アトム is_commutative(R) — 加法ビューで証明されたもの — で放たれる:
thus Product[R qua MulMagma](s) = Product[R qua MulMagma](s * p)
  by PermProduct[R qua MulMagma], Comm;   :: 拒否されなければならない
```

生成インスタンスは**行列積**の置換不変性を主張する — 偽である。この
インスタンス化は現行本文のすべての規則を通過する: bound の `inherit*`
義務は成立し、`commutative` 属性義務はエンコーディングがビューごとに
区別できないアトムで放たれてしまう。

**規則を局所修正できない理由。** フィールド改名により「一つの項、複数の
ビュー」は共有大域セレクタでは表現不能になる: ビューはどのセレクタが
`binop` を実現するかで異なるため、ビューはメタ注釈ではなく**項**でなけれ
ばならない。

**要求されるエンコーディング(仕様パッチで採用)。** レダクト(ビュー)項:

- フィールド写像が恒等で、祖先への経路が一意な `inherit` 辺は包摂
  エンコードを維持する。
- 改名を伴う辺、および複数経路で到達可能な祖先は、明示的レダクト関数
  `view_{D→B}` でエンコードする:
  `∀x. is_D(x) → is_B(view_{D→B}(x))` と、写像された各フィールドについて
  `∀x. is_D(x) → b(view_{D→B}(x)) = d(x)`。この種の辺では
  `is_D → is_B` 包摂を**放出しない**。
- この経路に沿う `x qua B` は(合成された)レダクト項へ低下し、属性
  アトムは `is_commutative(view_add(R))` と
  `is_commutative(view_mul(R))` に分離される。
- 外延性は `is_S` 全域ではなく**厳密**インスタンス(アグリゲート型の値)
  に制限する。
- 有界型パラメータのインスタンス化 `PermProduct[R qua AddMagma]` は
  パラメータをレダクト項でインスタンス化する。記法再写像(§18.2.2)は
  規約ではなく定理(`binop(view(R)) = add(R)`)になる。

**処置。** 本変更で仕様修正: §5.8.3、§5.8.5(エンコード)、§13.8.7
(qua 低下)、§3.7.2(サブタイプ注記)、§18.10.2(レダクト経由の
インスタンス化)。reject-first テスト追加:
`tests/miz/fail/templates/fail_template_qua_view_attribute_leak_001.miz`。
エラボレータ/カーネルへの影響は後述「mizar-core への影響」に記録。

### F2(Critical)— 型パラメータの非空性と空な型式実引数

**関与規則:** §18.10.2、§18.2.2(実引数分類)、§7.8(「系のすべての型は
非空」)、§17.3.4(existential gating 表)、§13.5(`the T`)。

**欠陥。** 系は使用中のすべての型が非空という大域不変量を維持する(mode
existence は必須で欠落はハードエラー、§7.8。属性付き型は existential
registration でゲート、§17.3.4)。テンプレート本体内でこの不変量こそが
`the T`、`T` 上の `consider`/`take`、パラメータの住人を使う mode
existence 証明を正当化する。しかし:

1. §18.10.2 は Γ_schema が型パラメータの `∃x. is_T(x)` を含むか否かを
   述べず、
2. §17.3.4 のゲート表は「テンプレート実引数として使われる型式」を
   **含まず**、§18.2.2 は任意の `type_expression` を実引数として受理する。

実装が §7.8 の不変量を記号的に尊重する場合(自然な読み — さもなくば
`the T` はいかなるテンプレートでも検証されない)、次はすべての明文規則を
通過する:

```mizar
definition
  let x be set;
  attr HollowDef: x is hollow means not x = x;   :: 充足不能、登録されない
end;

definition
  let T be type;
  theorem Inhab[T]: ex y being object st y is T
  proof
    take the T;      :: 型は非空という不変量から放たれる
    thus thesis;
  end;
end;

theorem Boom: ex y being object st y is hollow set
  by Inhab[hollow set];   :: 拒否されなければならない — ∃y. is_set(y) ∧ y ≠ y を生成
```

`hollow set` は変数宣言・mode 基底・functor 戻り型のいずれにも現れない
ため §17.3.4 のどの規則も発火せず、インスタンスは偽の公理になる。

**健全な設計(採用)。** 不変量を維持しゲートを閉じる: Γ_schema は各型
パラメータについて `∃x. is_T(x)` を仮定して**よい**。**その代わり**
インスタンス化はすべての型実引数について非空性証拠の提示を要求される —
§17.3.4 が他所で既に要求するものと同じ証拠(§7.8 の mode existence、
属性連鎖の existential registration、`type extends M` の bound mode
existence)。これは古典 Mizar の非空型レジームと一致し、テンプレート
本体で `the T` を使用可能に保つ。

**処置。** 本変更で仕様修正: §17.3.4(ゲート行の追加)、§18.10.2(規範的
非空性段落)、§18.2.2(実引数側の一文)。reject-first テスト追加:
`tests/miz/fail/templates/fail_template_type_actual_missing_existential_001.miz`。

### F3(Major)— `type extends M` が schema 述語エンコードとオブジェクトレベルエンコードを混同し、本体の構造体演算に FOL 規則がない

**関与規則:** §18.10.2(表)、§18.2.2。

**欠陥。** 上記の再構成のとおり、ガード付き `is_T/1` エンコードは
インスタンスビュー実引数(`R qua AddMagma`)を吸収できず、本体の
`T.binop`、再写像記法、`Element of T`、`FinSequence of T` に意味を与え
ない。表を字義どおりに実装すると、章自身の例を拒否するか、不健全な橋渡し
を即興することになる(例: いずれかのビューの carrier 述語を代入する —
まさに F1 のリーク)。

**要求される規則(採用)。** 構造体で有界なパラメータ
`let T be type extends M` は**オブジェクトレベルの schema 定数** `t`:

- Γ_schema ガード: `is_M(t)`(および bound の属性アトムを `t` 上に)、
  `M` のフィールドは `field(t)` として利用可能;
- 本体で型位置に現れる `T` は `Element of t`(`t` の carrier 経由で
  エンコード)を意味し、`T.f` と再写像記法は `f(t)` を意味する;
- 実引数 = mode/struct 名 `N`: インスタンスは**全称閉包**
  `∀t. is_N(t) ∧ bound-attributes(t) → φ(t)`(`N` のインスタンスが `M`
  に到達すること、すなわち `inherit*(N, M)` の検査後);
- 実引数 = `v qua N…` ビュー: インスタンスは F1 のレダクト項による
  `φ(viewpath(v))`。§13.8.7 の妥当性義務と bound の属性義務は
  **ビュー項上で**課される;
- 素の `let T be type` パラメータは schema 述語変数のままとし、
  `type_expression` 実引数のみを受理する — `qua_arg` 実引数は構造体で
  有界なパラメータに対してのみ妥当。

**処置。** 本変更で仕様修正: §18.10.2 の表と注記。F1 テスト(ビュー義務)
と F2 テスト(実引数分類)が併せて覆うため、独立テストなし。

### F4(Major)— functor パラメータのガード: 公理か義務か、および `defpred`/`deffunc` 実引数のシグネチャ適合規則の欠落

**関与規則:** §18.10.4、§18.9、§15.11.3。

**欠陥。** インスタンス化されたガード `∀x. is_T(x) → is_S(F(x))` が公理
として放出される(表の読み)なら、次が通過する:

```mizar
deffunc shrink(x be Nat) -> Integer equals x - 5;
:: schema は func(Nat) -> Nat を期待
thus ... by IterBound[shrink], ...;
:: インスタンス化された「ガード公理」: ∀x. is_Nat(x) → is_Nat(x - 5)   — x = 0 で偽
```

同様に、`defpred` 実引数のパラメータ型が schema の宣言 `pred(T₁,…,Tₙ)`
ドメインから widening で得られることを要求する規則がない。より狭い実引数
(`pred(Nat)` に対する `defpred P(p be Prime)`)はインスタンスに検査済み
ドメイン外での展開適用をさせる(§15.11.3 の適用ごと検査は、エラボレータ
が代入後に全出現を再検査する場合にのみ発火するが、§18.10 はそれを要求
していない)。

**要求される規則(採用)。** インスタンス化時、functor パラメータ
`func(T₁,…,Tₙ) -> S` の実引数 `f`、述語パラメータ `pred(T₁,…,Tₙ)` の
実引数 `p` それぞれについて: schema の各ドメイン型 `Tᵢ` は実引数の宣言
パラメータ型へ **widening 可能**(反変)であり、実引数の宣言結果型は `S`
へ **widening 可能**(共変)でなければならない。これによりガードは放たれ
た証明義務であり、仮定される公理には決してならない。`F` のより強い性質を
必要とするテンプレートはそれを仮説として述べる(§18.10.4 の例が既にそう
しているとおり)。

**処置。** 本変更で仕様修正: §18.10.4、§18.9 注記。reject-first テスト
追加:
`tests/miz/fail/templates/fail_template_func_actual_result_widening_001.miz`。

### F5(Major)— 型パラメータの sethood が未規定; テンプレート本体の Fraenkel 内包が Russell に到達しうる

**関与規則:** §18.10.2、§13.4.2、§7.8.1。

**欠陥。** 型パラメータに `sethood` 証拠を与える(または否定する)規則が
ない。§13.4.2 は内包の生成子に証明済み sethood を要求するが、実装が
「T は型パラメータである」を十分条件として扱う(例えば非空性不変量からの
類推で)なら:

```mizar
definition
  let T be type;
  func para[T] -> set equals { x where x is T : not x in x };
end;
set r = para[set];    :: Russell: r in r iff not r in r
```

`set` 自体に sethood はない(全集合のクラスは真クラス)ため、記号的検査
は失敗しなければならない — 仕様はどの証拠に対してかを述べていないだけで
ある。

**要求される規則(採用)。** 素の型パラメータは sethood 証拠を**持たな
い**。有界パラメータ `type extends M` は bound `M` が証明済み `sethood`
を持つ場合に限り sethood 証拠を継承する(健全: `is_T ⊆ is_M ⊆ S`)。
追加の sethood は `such that` 制約として要求でき、他の制約同様に使用箇所
で放たれる。パラメータ上の内包は、これらのいずれかの根拠がない限り記号的
に拒否される。

**処置。** audit 変更で仕様修正: §18.10.2 sethood 段落。reject-first テスト
追加:
`tests/miz/fail/templates/fail_template_fraenkel_over_type_param_001.miz`。
task 30 は explicit-payload core 側を実装する: Step 2 は bound 継承、
constraint 供給、bare-missing sethood row を保存し、Step 3 は
template-parameter Fraenkel comprehension が generated origin を emit する前に
accepted bound/constraint row を cross-reference することを要求する。
source-derived sethood extraction と active corpus execution は checker/runner
bridge 到着まで gated のままである。

### F6(Medium)— テンプレート本体内で外側テンプレートのパラメータを実引数として scheme を適用する場合

**関与規則:** §18.10.3 の例(NatInduction の本体が外側の `P` をスコープに
持ったまま `mml.number.natural.Nat_induction` を引用)、§18.9。

**欠陥。** 章自身の例が**未解釈の** schema パラメータを別の scheme の
(暗黙の)実引数として渡しているが、これを定義する規則がない: scheme
実引数は `defpred`/`deffunc` 識別子としてのみ規定されている。健全な意味論
は代入合成 — 内側 scheme を外側 schema の未解釈記号でインスタンス化し、
記号的義務は Γ_schema で検査、最終インスタンスは代入を合成する。明文規則
なしでは、実装が章の例を拒否するか、悪ければパラメータを新規スコーレム化
する(内外の `P` の束縛を壊す)可能性がある。

**処置。** 本変更で仕様修正: §18.10.3 に規範的段落を 1 つ追加。テスト
なし(pass 側の挙動であり、その reject コーパスは本タスクの範囲外)。

### F7(Medium)— §18.2.7 の「一意に推論される」が widening 束の上で未定義

**欠陥。** 省略された `[T]` の引数型からの推論は、候補を引数の宣言型で
比較するのか任意の widening で比較するのかを述べていない(例:
`s : FinSequence of Prime` に対する `Product(s)` は `T := Prime`、
`Integer`、… を許す)。整形式なインスタンスはどれも(F1–F5 対処後は)
*健全*なので健全性の穴ではないが、生成される公理は選択ごとに異なり、
検証結果が実装定義になる。`qua` ビューは決して推論されない(§18.2.7)
ことが、重要な点として、F1 のビュー選択を推論から締め出している。

**処置。** task 26 で解決済み。spec 18 §18.2.7 は、省略された func/pred
template 型パラメータを mode-unfolded declared argument type だけから推論する
と明記した。widening 祖先の探索、cluster expansion、`qua` view 推論は行わない。
相異なる declared-type candidate が残る場合、それらの closure が同値でも
ambiguous template instantiation とし、inactive seed
`fail_template_inference_declared_type_ambiguity_001` と
`fail_template_inference_requires_explicit_qua_view_001` が reject-first intent
を記録する。

### F8(Minor)— §18.8.4 の例が schema functor を一階引数として渡している; 部分 algorithm の functor 実引数が未規定

**欠陥。** `sigma[F]` の契約は `ensures result = Sigma(F, lo, hi)` と
読め、メタレベル記号 `F` を一階 functor の項引数として渡している —
functor パラメータは論議領域の要素ではないという §18.2.4 自身の規則に
違反する。テンプレートインスタンス化 `Sigma[F](lo, hi)` でなければなら
ない。別途、FOL 関数記号を表すのは `deffunc`、テンプレート functor、
**promotion 済み `terminating` algorithm** のみである。部分(未 promotion)
algorithm は FOL 記号を持たず `func(...)` 実引数として拒否されなければ
ならない — §18.8.4 は逆方向(計算不能だが妥当なインスタンス化)しか
扱っていない。

**処置。** 本変更で仕様修正: §18.8.4 の例と、妥当な functor 実引数に
関する一文。

## タスクのチェックリストに対する相互作用まとめ

| 相互作用 | 判定 |
|---|---|
| 3章 消去/widening | F1 のレダクトエンコード採用を条件に健全。§3.7.2 に注記追加。消去自体はテンプレート固有の危険を追加しない。 |
| 7章 sethood / パラメータ化 mode からの真クラス | F5 の証拠規則を採用する**場合に限り**健全。テンプレートで宣言され記号的に証明された `sethood` は代入によりインスタンスへ移る。 |
| 13章 `the T` / 非空性 | F2 のゲートを採用する**場合に限り**健全。インスタンス化ごとの choice 項同一性は健全と確認。 |
| §18.9 scheme application / §18.2.7 推論 | F4(シグネチャ規則)と F6 を除き健全。F7 は決定性の問題であり健全性ではない。 |
| 20章 algorithm templates | 記号的 VC 放出は保守的ゆえ健全。インスタンスごとの promotion は代入により健全。F8 は例の欠陥。 |

## mizar-core(エラボレータ)への影響

所見から次の実装義務が従う。現在の roadmap は、それぞれの owning task と
task 27 の進捗を記録している。

1. **レダクト/ビュー低下(F1、F3)。** task 27 は explicit-payload の core 側を
   実装した。checker-owned `QuaPathKey` と順序付き reduct functor により、
   改名/複数経路 inherit 辺上の `qua` と bounded-type-parameter view actual は
   view term へ lower され、attribute atom と field selection はその view term を
   対象にできる。この task は reduct term 上の明示的 exact-instance guard formula を
   保持する。source-derived extensionality emission と real payload extraction は
   checker/runner bridge まで gated のままである。
2. **テンプレート実引数の非空性ゲート(F2)。** task 28 は explicit-payload の
   core 側を実装済みである。schema 文脈は checker が提供する parameter ごとの
   `∃x. is_T(x)` assumption を得て、template `type_expression` actual は checker
   existential-gate status、registration/base/fact evidence、diagnostic を保存する。
   reject された actual は diagnostic/backref のままであり、actual-side existential
   axiom にはならない。source-derived actual extraction と active runner execution は
   checker/runner bridge に gated のままである。
3. **scheme 実引数のシグネチャ適合検査(F4/F6)。** task 29 は
   explicit-payload の core 側を実装済みである。checker-owned row は
   `defpred`/`deffunc`、template functor、enclosing-parameter、promoted
   terminating-algorithm actual の directional widening evidence を保持する。
   accepted functor row は traceability として `Skipped` guard obligation seed を
   emit し、guard axiom や active VC は決して assert しない。source-derived
   closure expansion と active runner extraction は checker/runner bridge に
   gated のままである。
4. **型パラメータの sethood 証拠の配管(F5)。** task 30 は explicit-payload
   core 側を実装する: テンプレート本体での Fraenkel ゲートは accepted な
   bound 継承または constraint 供給の sethood record に key され、bare type
   parameter は missing-sethood error path へ lower する。source-derived
   sethood extraction と active runner execution は checker/runner bridge に
   gated のままである。
5. **部分 algorithm の functor 実引数の拒否(F8)。** task 29 は partial、
   void、unsupported algorithm actual を accepted evidence のない diagnostic-only
   explicit payload row として記録する。F7 の推論決定性は task 26 で解決済みで、
   source-derived active execution は payload 到着まで gated。

カーネル側(7月3日のカーネル監査の観点から): レダクトエンコードは構造体
widening に言及する証明書の形を変える。原子的属性述語に関する
soundness-argument 文書の仮定は、task 27 が explicit-payload core encoding を
land したため mizar-kernel task 35 で再訪すべきである。

## テスト成果物(reject-first)

すべて `tests/miz/fail/templates/` 配下、既存コーパス規約に従う inactive
`advanced_semantics` シード。既存テスト・期待値の変更なし:

| テスト | 所見 | 拒否内容 |
|---|---|---|
| `fail_template_qua_view_attribute_leak_001` | F1 | ある構造体ビューの属性証拠が別ビューの bound を放ってはならない |
| `fail_template_type_actual_missing_existential_001` | F2 | existential registration のない属性付き型実引数 |
| `fail_template_fraenkel_over_type_param_001` | F5 | 素の型パラメータ上を動く内包生成子 |
| `fail_template_func_actual_result_widening_001` | F4 | 結果型が schema コドメインへ widening できない `deffunc` 実引数 |
| `fail_template_inference_declared_type_ambiguity_001` | F7 | 省略 template パラメータ推論は、残る相異なる宣言型候補を拒否しなければならない |
| `fail_template_inference_requires_explicit_qua_view_001` | F7 | 省略 template パラメータ推論は継承された `qua` view を推論してはならない |
