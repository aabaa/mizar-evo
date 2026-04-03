# Cluster 解決グラフの構造化保存

## 概要

新Mizarにおける AI 支援の難所のひとつは、cluster / registration / inheritance / `qua` による暗黙解決である。
これは本質的に「到達可能性を伴う推論の連鎖」なので、内部では**グラフとして表現**するのが自然である。

ただし、AI に巨大な全体グラフをそのまま渡すのは重い。
したがって方針としては次の二段構えがよい。

1. 内部では **global cluster graph** を静的に保持する
2. AI/IDE には、問い合わせごとに切り出した**局所サブグラフ**と**説明トレース**を返す

## 1. global cluster graph

コンパイラ・検証器は、import 解決後の環境に対して cluster 連鎖を静的にインデックス化する。

頂点の候補:

- radix type
- mode
- attribute
- parameterized mode instance
- coercion view
- template-instantiated type context

辺の候補:

- cluster registration
- struct inheritance
- mode expansion
- coercion / widening
- `qua` による view selection
- template constraint discharge

概念的には、次のようなグラフになる。

```text
empty set
  --cluster--> finite set
  --cluster--> countable set

Ring
  --qua(AddMagma)--> AddMagma
  --qua(MulMagma)--> MulMagma

commutative Ring
  --inherits--> Ring
  --attribute--> commutative
```

この global graph は source ごとに毎回再構築してもよいが、ライブラリ部分はキャッシュしてよい。

## 2. AI/IDE に渡すのは局所サブグラフ

AI が欲しいのは「世界全体」ではなく、ある地点での解決理由である。
そのため、公開 API は巨大グラフの dump ではなく、クエリ単位の説明情報にする。

代表的クエリ:

- `T has attribute A` は成り立つか
- 式 `e` の型はなぜ `M` に上がるのか
- なぜこの overload 候補が選ばれたのか
- `qua` が必要なのはなぜか
- どの cluster が不足していて失敗したのか

返す情報は、次の 2 種類が有用である。

### path view

実際に使われた最小限の解決経路。

```json
{
  "query": "S is countable",
  "status": "resolved",
  "path": [
    {
      "kind": "cluster",
      "from": "empty set",
      "to": "finite set",
      "source": "mml.sets.cluster_empty_finite"
    },
    {
      "kind": "cluster",
      "from": "finite set",
      "to": "countable set",
      "source": "mml.sets.cluster_finite_countable"
    }
  ]
}
```

### neighborhood view

近傍の候補、競合、失敗要因を含む説明。

```json
{
  "query": "PermProduct[R]",
  "status": "ambiguous",
  "candidates": [
    {
      "target": "R qua AddMagma",
      "reason": "Ring inherits AddGroup then Magma"
    },
    {
      "target": "R qua MulMagma",
      "reason": "Ring inherits MulMonoid then Magma"
    }
  ],
  "suggested_fix": "insert qua coercion"
}
```

AI にとって重要なのは、全体グラフよりもこちらである。

## 3. explanation extractor

global graph の上に、説明抽出器を設ける。
これは verifier の内部推論結果から、AI/IDE 向けの安定表現を組み立てる層である。

役割:

- 成功時: 実際に使った経路を返す
- 失敗時: 到達不能点と不足前提を返す
- 曖昧時: 競合経路と分岐理由を返す

失敗時に特に重要な分類:

- 到達不能
- 複数候補が同順位で曖昧
- 必要な attribute が未登録
- template 引数が未確定
- `qua` 指定が必要

AI にとっては、成功証跡よりむしろこちらの情報が修正提案に効く。

## 4. 保存形式

内部表現は自由だが、外部出力には以下の 3 層を分けるとよい。

1. `cluster-db`
   - ライブラリ全体の静的インデックス
2. `resolution-trace`
   - 各問い合わせで実際に使った経路
3. `diagnostic-explanation`
   - 失敗理由・競合候補・修正提案

たとえば `cluster-db` は build cache に保持し、source 変更時には差分更新できる。

## 5. AI に渡す単位

AI に与える情報単位は、原則として次の順に絞る。

1. 現在行の `resolution-trace`
2. 必要なら `neighborhood view`
3. さらに必要なら関連 cluster の小部分グラフ
4. 最後の手段として module 単位の graph slice

全 graph の丸投げは避けるべきである。

## 6. 望ましい API の形

LSP / CLI / build artifact のいずれでも、次の問い合わせがあると強い。

- `explain-type <expr-id>`
- `explain-attribute <type> <attribute>`
- `explain-overload <expr-id>`
- `explain-qua <expr-id>`
- `explain-failure <diagnostic-id>`

返答は自然言語ではなく、まず構造化 JSON で返す。
IDE や LLM はその上に表示文を組み立てればよい。

## 7. 結論

cluster 問題への対策として有効なのは、単に注釈を増やすことではなく、

1. cluster 解決を**静的グラフとして構造化保存**し
2. 問い合わせ単位で**局所サブグラフ**を抽出し
3. 成功経路・曖昧性・失敗理由を**説明トレース**として返す

ことである。

この設計なら、Mizar の暗黙推論は「ブラックボックス」ではなく「説明可能な検索系」になる。
AI 支援との相性も大きく改善する。
