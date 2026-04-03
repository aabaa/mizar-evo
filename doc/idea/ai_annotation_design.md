# AI/IDE 向けアノテーション設計案

## 目的

新Mizarは人間可読性が高く、数理文書として自然に読める一方で、AI や IDE にとっては以下のような「暗黙に解決される情報」が多い。

- 型推論の結果
- overload resolution の候補と最終選択
- `qua` による view 選択
- cluster / registration による属性伝播
- `by` 失敗時に不足している補題や仮定
- loop invariant / `decreasing` / `ensures` に関する未解決 VC

これらは言語の本質的な強みでもあるが、AI にとっては探索空間の増大要因でもある。

本メモでは、**言語意味を極力変えずに** AI・IDE・ATP 連携を改善するためのアノテーション設計を提案する。

設計方針は次の通り。

1. アノテーションは原則として**意味論を変えない**
2. 省略時も既存コードはそのまま動作する
3. 注釈は主に
   - 推論器へのヒント
   - IDE/LLM への可視化
   - 診断メッセージの安定化
   に使う
4. 注釈は source 上に書けるが、build artifact として**解決済み情報を外部出力**できることを重視する

関連:

- Ch.2 §2.9.2 Annotations
- Ch.16 §16.9 Thesis Tracking
- Ch.18 Templates
- Ch.19 Overload Resolution
- Ch.20 Algorithm and Verification
- Ch.22 Source Code Annotation and ATP Integration

## 大きな方針

AI 支援のための情報は次の 3 層に分けるのがよい。

### 1. 手書きヒント

ユーザーが source に直接書く注釈。

例:

- 型ヒント
- overload 解決ヒント
- ATP ヒント
- simp 候補
- loop invariant の補助メタデータ

### 2. 解決済み情報の表示

コンパイラ・検証器が内部で求めた情報を、IDE や LLM に渡すために出力する。

例:

- 式の推論済み型
- functor/predicate の解決先 FQN
- `qua` の挿入結果
- 現在の thesis
- VC 一覧と未証明原因

これは source に埋め込むより、JSON や LSP 経由で機械可読に出すほうが重要。

### 3. 診断安定化ラベル

エラーや未証明義務に、機械が扱いやすい安定 ID を付与する。

例:

- `type.mismatch.narrowing_cast`
- `resolve.ambiguous_overload`
- `vc.loop_invariant.establish`
- `vc.recursion.decrease`
- `proof.by.missing_fact`

AI にとっては、自然言語メッセージよりこちらのほうが使いやすい。

## 提案する注釈カテゴリ

以下のカテゴリを想定する。

1. 型ヒント
2. 解決ヒント
3. 証明ヒント
4. アルゴリズム検証ヒント
5. AI/IDE 表示ヒント

構文は Ch.2 の `@...` 系に揃える。実際の最終構文は Chapter 22 で確定すればよいが、本メモでは可読性のために以下のような形を使う。

```mizar
@annotation_name(arg1, arg2, ...)
```

あるいは対象直前に置く属性型の注釈:

```mizar
@type_hint(...)
let x be ...;
```

## 1. 型ヒント

### 1.1 `@type_hint`

推論器に「この式や束縛について、まずこの型を優先的に試す」と伝える注釈。
意味論は変えず、通常の型推論で同じ結論に到達できるときのみ有効。

```mizar
@type_hint(result: Nat)
algorithm abs_val(n) -> Nat
do
  ...
end;
```

```mizar
@type_hint(x: Element of PolyRing[K, x])
var x;
```

用途:

- pattern variable の推論補助
- overloaded literal の早期確定
- theorem/template instantiation の候補削減

制約:

- 推論結果と矛盾する場合はエラーではなく warning が望ましい
- 強制的な cast ではない
- 実際に型を確定したい場合は既存の `as T` や明示宣言を使う

### 1.2 `@expect_type`

`@type_hint` より強く、「この式はこの型に解決されるはず」という期待を記述する。
こちらは IDE/テスト寄りで、将来の変更で推論結果がずれたとき検知できる。

```mizar
@expect_type(expr: Nat)
const m = n - i;
```

これは実装上は assertion 的に扱える。

用途:

- 仕様書や教材での型説明
- リファクタリング時の回帰検知
- AI への明示的な型境界提示

### 1.3 `@infer_as`

overloaded constant や literal に対して、推論方向を与える局所ヒント。

```mizar
const z = @infer_as(0, Real);
const one = @infer_as(1, Element of R);
```

これは新しい term 構文にしてもよいが、既存の `0 as Real` と役割が近いため、実際には `as` を推奨し、本注釈は IDE 用糖衣として扱うのが無難。

結論:

- 本体仕様に入れるなら `@type_hint`, `@expect_type`
- `@infer_as` はなくてもよい

## 2. 解決ヒント

### 2.1 `@resolve`

曖昧な overload resolution に対する解決ヒント。

```mizar
@resolve(op: algebra.ring.add)
a + b;
```

```mizar
@resolve(functor: mml.number.gcd.Gcd)
Gcd(a, b);
```

用途:

- operator の候補が多い環境での曖昧性削減
- AI が FQN を明示して生成したい場合の受け皿

重要なのは、これを**通常コードの必須機構にしない**こと。
基本は qualification や `qua` で解けるべきであり、`@resolve` は AI/IDE 向けの補助線に留める。

### 2.2 `@view`

`qua` で選ぶ構造 view を注釈で先置きする案。

```mizar
@view(R: AddMagma)
PermProduct[R]
```

ただしこれは既存の `R qua AddMagma` と二重化しやすい。
したがって source 構文としては不要で、**解決済み情報の表示名**として使うほうがよい。

推奨:

- `@view` は source 注釈より IDE 表示向けメタ情報
- source では `qua` を使う

### 2.3 `@template_args`

テンプレート引数を省略せず、AI が安全側に書くための注釈。

```mizar
@template_args(T := Nat, F := succ)
SomeTemplate;
```

ただし Mizar 文法では通常の `SomeTemplate[Nat, succ]` のほうが自然。
よってこれも source 仕様には不要で、IDE の code action 用メタ情報として十分。

## 3. 証明ヒント

### 3.1 `@[hint]`, `@[simp]`, `@[rewrite]`

Chapter 22 にある reasoning annotation を、AI 支援の中心に据える。

- `@[hint]`: `by` 探索で優先候補
- `@[simp]`: 正規化・単純化で常用
- `@[rewrite]`: 方向付き書換え候補

例:

```mizar
@[simp]
theorem add_0_right:
  for n being Nat holds n + 0 = n;

@[rewrite(direction: left_to_right)]
theorem factorial_step:
  factorial(n + 1) = (n + 1) * factorial(n);
```

AI に有効なのは、各定理が「どの用途向けか」を明示できる点である。

### 3.2 `@proof_hint`

個別の `by` や `proof` に対して、探索対象を絞る。

```mizar
@proof_hint(using: [arith, simp, cluster])
thus thesis by ...;
```

```mizar
@proof_hint(
  lemmas: [poly_eval_add, poly_eval_mul],
  max_axioms: 32,
  solver: vampire
)
thus thesis;
```

用途:

- AI が大域ライブラリを投げすぎるのを防ぐ
- `module.*` 展開を避ける
- solver 選択や制限を source に残せる

### 3.3 `@missing_facts`

これは手書き注釈ではなく、検証器が生成する診断メタデータ。

例:

```text
proof.by.missing_fact:
  goal: result = Eval(t, v)
  candidates:
    - poly_eval_add
    - poly_eval_mul
    - ring_distrib_left
```

AI はこれを見て次の `by` 候補を提案できる。

## 4. アルゴリズム検証ヒント

### 4.1 `@vc_hint`

VC ごとに狙う補題群や理論を指定する。

```mizar
while i <= n do
  invariant s = (i - 1) * i div 2;
  @vc_hint(establish: [arithmetic])
  @vc_hint(maintain: [arithmetic, ring_normalize])
  @vc_hint(decrease: [arithmetic])
  decreasing n - i + 1;
  ...
end;
```

対象は次のように分けられる。

- `establish`
- `maintain`
- `exit`
- `break`
- `continue`
- `decrease`

これにより、AI が「どの VC を解くための補題か」を局所的に理解しやすくなる。

### 4.2 `@invariant_shape`

invariant のテンプレート的な性質を宣言する。

```mizar
@invariant_shape(accumulator_over: V)
invariant total = Sum(V);
```

```mizar
@invariant_shape(frame: [current, visited])
invariant visited = S \ current;
```

意味論は持たせず、IDE/AI が invariant の役割を把握するためのタグとする。

主な用途:

- accumulator invariant
- frame invariant
- bound invariant
- relational invariant
- ghost coupling invariant

### 4.3 `@measure_hint`

`decreasing` の意図を明示する。

```mizar
@measure_hint(kind: structural)
decreasing term_size(t);
```

```mizar
@measure_hint(kind: numeric, source: "remaining interval length")
decreasing r - l + 1;
```

AI は termination 補題の雛形を選びやすくなる。

### 4.4 `@match_intent`

`match` が何を狙っているかを示すヒント。

```mizar
@match_intent(kind: structural_recursion)
match t do
  ...
end;
```

候補:

- `structural_recursion`
- `normalization`
- `destructor_elimination`
- `compiler_lowering`

これも意味論を持たないタグでよい。

### 4.5 `@pick_witness`

`the Element of S` の Pick に対して、実行系や AI が優先して探す witness の方針を与える。

```mizar
@pick_witness(strategy: smallest)
const x = the Element of S;
```

ただしこれは意味論に踏み込みやすいので慎重であるべき。
証明的には Pick は任意選択であり、実行上の戦略を source に埋め込むと仕様と実装が混ざる。

推奨:

- source 仕様には入れない
- extractor / MVM 設定に逃がす

## 5. AI/IDE 表示ヒント

ここが最も重要で、source 注釈よりも**検証器が出力する解決済み情報**を整備すべきである。

### 5.1 `@show_type`

手書き注釈としては簡易デバッグ用途。

```mizar
@show_type(expr: total + x)
```

IDE では hover で十分なので、source には必須ではない。

### 5.2 `@show_thesis`

Ch.16 §16.9 にある thesis tracking を source レベルでも起動できるようにする。

```mizar
@show_thesis
thus thesis;
```

これは教育・デバッグ・AI 対話のいずれにも有効。

### 5.3 `@show_vc`

algorithm 節の現在 VC を可視化する。

```mizar
@show_vc
while i <= n do
  invariant ...
  decreasing ...
  ...
end;
```

想定表示:

- establish VC
- maintain VC
- exit VC
- decrease VC

### 5.4 `@show_resolution`

その行の overload / template / `qua` 解決結果を表示する。

```mizar
@show_resolution
Product[R](s);
```

表示例:

- resolved functor: `algebra.monoid.Product`
- template args: `R qua MulMagma`
- inferred result type: `Element of R`

## 推奨する最小コア

全部を一度に入れる必要はない。AI 支援の観点からは、まず次の最小集合が強い。

1. `@[hint]`, `@[simp]`, `@[rewrite]`
2. `@proof_hint(...)`
3. `@type_hint(...)`
4. `@show_thesis`
5. `@show_vc`
6. `@show_resolution`

これだけでも、かなり実用性が上がる。

## source 注釈より重要な「解決済み情報の外部出力」

AI に本当に効くのは、source を増やすことより、検証器が次の情報を安定出力することである。

推奨 artifact:

1. `*.mizir.json`
2. LSP hover / code action / diagnostics
3. ATP 入出力の要約ログ

最低限ほしい JSON 項目:

```json
{
  "expr_id": "e124",
  "source_range": "20:14-20:23",
  "inferred_type": "Element of PolyRing[K,x]",
  "resolved_symbol": "algebra.poly.add",
  "template_args": ["K", "x"],
  "inserted_coercions": ["t qua AddMagma"],
  "active_thesis": "result = Eval(t, v)",
  "open_vcs": [
    {
      "kind": "vc.recursion.decrease",
      "goal": "term_size(p) < term_size(t)",
      "status": "proved_by_match"
    }
  ]
}
```

この形式なら、LLM は source を全文再解釈せずとも局所的に次の一手を選べる。

## cluster 情報について

cluster / registration / inheritance / `qua` による暗黙解決は、source annotation というよりも
**検証器が保持・出力する外部構造化情報**として扱うのが適切である。

そのため、cluster 解決の構造化保存と explanation API については別メモに分離した。

- [cluster_resolution_graph.md](./cluster_resolution_graph.md)

本メモでは、`@show_resolution` や `@proof_hint` などの source-level annotation を中心に扱い、
cluster 解決グラフ自体は verifier / IDE / LLM 連携のための外部情報とみなす。

## 診断コード設計

自然言語診断だけでなく、機械可読なコードを必ず付与する。

例:

- `type.expected`
- `type.inferred`
- `type.narrowing_requires_proof`
- `resolve.ambiguous_symbol`
- `resolve.no_viable_overload`
- `template.argument_omitted_not_inferable`
- `proof.by.search_exhausted`
- `proof.by.missing_fact`
- `vc.loop.establish`
- `vc.loop.maintain`
- `vc.loop.decrease`
- `vc.loop.break_preserve`
- `vc.loop.continue_preserve`
- `vc.recursion.decrease`
- `vc.postcondition.return`

これにより AI は診断種別ごとの修正戦略を学習できる。

## アルゴリズム節への具体的適用例

### 例1: `match` + structural recursion

```mizar
definition
  let K be Field;
  let x be Variable;
  let t be Element of PolyRing[K,x];
  let v be Element of K;

  @type_hint(result: Element of K)
  terminating algorithm eval_at(t, v) -> Element of K
    ensures result = Eval(t, v)
    @measure_hint(kind: structural)
    decreasing term_size(t)
  do
    @type_hint(p: Element of PolyRing[K,x], q: Element of PolyRing[K,x])
    var p, q;
    @match_intent(kind: structural_recursion)
    match t do
      case p + q do
        @proof_hint(lemmas: [poly_eval_add])
        return eval_at(p, v) + eval_at(q, v) by poly_eval_add;
      end;
      case p * q do
        @proof_hint(lemmas: [poly_eval_mul])
        return eval_at(p, v) * eval_at(q, v) by poly_eval_mul;
      end;
      otherwise
        return Eval(t, v);
      end;
    end;
  end;
end;
```

### 例2: loop verification

```mizar
var total := 0;
for x in S processed V do
  @invariant_shape(accumulator_over: V)
  invariant total = Sum(V);
  total := total + x;
end;
```

AI はこれを見て

- `V` が processed ghost set
- invariant が accumulation schema
- exit 時に `V = S`

を素早く把握できる。

## 非推奨事項

次のものは source 注釈としては入れすぎないほうがよい。

### 1. 強すぎる解決固定

例:

- overload を完全に手でロックする注釈
- cluster 展開の順序を固定する注釈

これらは言語の進化耐性を下げる。

### 2. 実装依存の Pick 戦略

Pick は論理上 arbitrary であるべきなので、`smallest` や `first` のような実行戦略を source に書くべきではない。

### 3. AI 専用の意味論

AI のためだけに本体意味論が変わる注釈は避ける。
AI 向け情報は、なるべく消去可能な metadata として表現すべきである。

## 段階的導入案

### Phase 1

- `@[hint]`, `@[simp]`, `@[rewrite]`
- `@proof_hint`
- `@show_thesis`
- 診断コードの安定化

### Phase 2

- `@type_hint`
- `@show_resolution`
- `@show_vc`
- 解決済み JSON artifact 出力

### Phase 3

- `@invariant_shape`
- `@measure_hint`
- ATP/LLM 向け explanation API

## まとめ

新Mizarで AI 適性を上げるには、source に大量の専用構文を足すよりも、

1. 少数の高価値アノテーションを用意し
2. verifier の内部解決結果を機械可読に外部出力し
3. 診断を安定コード化する

の 3 点が重要である。

特に有効なのは次である。

- `@type_hint`
- `@proof_hint`
- `@show_thesis`
- `@show_vc`
- `@show_resolution`
- `@[hint]`, `@[simp]`, `@[rewrite]`

これらは人間の可読性を大きく損なわず、新Mizarの「自然な数学文書」という強みを維持したまま、AI・IDE・ATP の支援性能を大きく引き上げる。
