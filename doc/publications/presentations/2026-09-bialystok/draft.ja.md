# Mizar Evo: Bialystok Mizar チーム向け第三稿

状態: 問題駆動の構成に全面刷新した第三稿。

予定: 2026年9月 Bialystok の Mizar チーム訪問時の議論用。

英語版主原稿: `draft.md`(こちらが正典。本書はレビュー高速化のための日本語対訳版)。

## 作業上の中心命題

Mizar Evo は「既存 Mizar の置き換え」ではなく、「Mizar の伝統を継承する再設計」として提示する。中心命題は次の形。

> Mizar Evo は、Mizar の可読な数学的言語(mathematical vernacular)を維持しつつ、
> 大規模な形式数学を予測可能な自動化、AI 支援、再現可能な検証で維持できるよう、
> 言語境界、検証器パイプライン、成果物モデル、出版ワークフローを再構築する。

発表は8つの「問題駆動の物語」で構成する。各物語は、現行 Mizar の実務における実際のコストから始まり、Mizar Evo の解決策をコードで示し、何が保存されるかを明示する。文法記法(EBNF)は意図的に排除した。言語はサンプルコードのみで示し、細部の正典は `doc/spec/en/` の仕様書とする。

## コードのステータス表記規約

すべてのコード例に次の3種類のステータスラベルを付す。

- 「exact MML excerpt(MML 原文抜粋)」: 現行 MML からの逐語引用。記事名と行番号を付し、出典・ライセンス注記は発表者ノートに保持する。
- 「specification example(仕様書の例)」: `doc/spec/en/` の Mizar Evo 言語仕様から採録、または直接に翻案したもの。
- 「sketch(スケッチ)」: 仕様として未確定の説明用素材。

## 出典の状況

### リポジトリ内の出典

- `doc/spec/en/05.structures.md`, `06.attributes.md`, `07.modes.md`
- `doc/spec/en/12.modules_and_namespaces.md`
- `doc/spec/en/17.clusters_and_registrations.md`
- `doc/spec/en/18.templates.md`
- `doc/spec/en/20.algorithm_and_verification.md`
- `doc/spec/en/21.source_code_annotation_and_atp.md`
- `doc/spec/en/23.package_management_and_build_system.md`
- `doc/spec/en/sample_codes.md`
- `doc/design/architecture/en/00.pipeline_overview.md`
- `doc/design/architecture/en/08.reasoning_boundary.md`
- `doc/design/architecture/en/15.kernel_certificate_format.md`
- `doc/design/architecture/en/21.ai_agent_interface.md`

### 外部出典

注記がない限り 2026年6月18日 確認。

- Mizar ホームページ: Mizar 8.1.15 と MML 5.94.1493(2025年5月30日付)。
  <https://mizar.uwb.edu.pl/>
- `ALGSTR_0` プレーンテキスト(逐語抜粋: 15-25行、37-40行、104-109行):
  <https://mizar.uwb.edu.pl/version/current/mml/algstr_0.miz>
- `STRUCT_0` プレーンテキスト(逐語抜粋: 637-643行):
  <https://mizar.uwb.edu.pl/version/current/mml/struct_0.miz>
- `NAT_1` プレーンテキスト(scheme `NatInd`、90行付近。2026年7月2日確認):
  <https://mizar.uwb.edu.pl/version/current/mml/nat_1.miz>

MML プレーンテキストは GPL-3.0-or-later / CC-BY-SA-3.0-or-later の配布条件を明記している。最終版デッキでは記事名・出典 URL・行番号を発表者ノートに必ず保持する。

仕様書の例は `doc/spec/en/` に従う。必要な import と先行宣言を前提とし、`...` は省略した証明テキストを表す。カーネル証拠は第21章 §21.7 と `doc/design/architecture/en/15.kernel_certificate_format.md` の現行設計に従う。

## デッキの形

全13セクション: 開幕、動機、8つの物語、アーキテクチャ総括、ロードマップ、結び。詳細版は持ち帰り用に維持する。45分の講演では選んだ例を説明し、ほかは簡単に紹介する。質疑応答が45分に含まれるかは未確認。

`[deep dive]` のフレームは要点を失わずに省略できる。各物語の問いは後日のレビュー用に残し、最後の討論では2〜3問を選ぶ。

## Part 0. Opening(開幕)

### Frame 0.1 - タイトル

タイトル:

```text
Mizar Evo
Readable, AI-Ready, Scalable Formal Mathematics
```

サブタイトル:

```text
Eight problems, eight proposals
A discussion with the Bialystok Mizar team, September 2026
```

発表者ノート:

- 感謝から始める。
- これは「Mizar が Mizar であり続けているか」を判断できる最適の聴衆との設計レビューである、と述べる。
- 何も凍結されていない。訪問の目的は反論を集めることである。
- Frame 3.3、5.2、8.2 を説明し、ほかは簡単に紹介する。deep dive は必要に応じて使う。詳しい内容は持ち帰り用レジュメで確認してもらう。

### Frame 0.2 - 最初の一瞥

旧 Mizar(exact MML excerpt):

```mizar
definition
  struct (1-sorted) addMagma (# carrier -> set, addF -> BinOp of the carrier
  #);
end;
```

Mizar Evo(specification example):

```mizar
definition
  struct AddMagma where
    field carrier -> set;
    field add -> BinOp of carrier;
  end;

  inherit AddMagma extends Magma where
    field carrier from carrier;
    field add from binop;    :: renamed view
  end;
end;
```

発表者ノート:

- 出典: 現行 MML `algstr_0.miz` 37-40行。
- 現行 Mizar も `(1-sorted)` で親を明示している。Evo は共通の `Magma` ビューへの明示的な対応付けを加える。どちらの例も carrier と二項演算を記述し、Evo の例はこの追加のビューも示す。

### Frame 0.3 - 一文で言う提案

スライドテキスト:

```text
Preserve Mizar's mathematical vernacular.
Modernize the compiler, verifier, artifact, and publication layers.
```

この発表が主張しないこと:

- MML 全体の移行が完了しているとは主張しない。
- 言語標準が最終確定しているとは主張しない。
- AI 支援は証明検査の代替ではない。
- 現行 Mizar システムの達成は「問題」ではなく「基準線」である。

Evo の例は仕様に従うが、実装がまだ対応していないものもある。現在の範囲は Frame 11.0 を参照。

### Frame 0.4 - 例の読み方 [deep dive]

すべてのコード例にラベルを付す:

| ラベル | 意味 |
|---|---|
| exact MML excerpt | 現行 MML の逐語引用。記事名と行番号付き |
| specification example | Mizar Evo 言語仕様からの例 |
| sketch | 説明用。仕様として未確定 |

読み方の規則:

- 本発表に EBNF は登場しない。言語はサンプルコードのみで示す。文法とエッジケースの正典は `doc/spec/en/` である。
- 例は必要な import と先行宣言を前提とする。`...` は省略した証明テキストを表す。

### Frame 0.5 - この訪問で得たいもの

要点:

- MML 作業の外からは見えない互換性制約を特定する。
- 小さいが代表性のある移行ベンチマーク記事を選ぶ。
- ATP 探索とカーネル検査の信頼境界をレビューする。
- Formalized Mathematics とパッケージライブラリの結び付け方を議論する。
- 我々がまだ思い付いていない反論を集める。

通底する問い:

```text
What must Mizar Evo preserve so that the Mizar community
still recognizes it as Mizar?
(Mizar コミュニティが Mizar と認め続けるために、何を保存しなければならないか)
```

## Part 1. Why Now(なぜ今か)

### Frame 1.1 - Mizar が正しくやったこと

要点:

- 数学として読める宣言的な証明テキスト。
- ソフト型、モード、豊かな形容詞語彙。
- attribute、registration、cluster による再利用可能な自動化。
- 成熟した精選ライブラリ(MML 5.94.1493: 1493記事)。
- 形式化記事を軸とする出版文化(Formalized Mathematics)。

メッセージ:

- これらの強みが設計の基準線である。本発表のすべての提案は、これらを守れるかどうかで判定される。

発表者ノート:

- 聴衆に自分たちのシステムを講義しない。このフレームは1分間の共通認識の確認であり、チュートリアルではない。

### Frame 1.2 - 圧力その1: スケール

要点:

- MML は相互依存する約1500記事にまで成長した。
- 依存・レビュー・再利用の単位は記事全体である。
- 記事環境はツールが解決するが、解決結果の依存面はレビュー対象のソースには現れない。
- ライブラリ全体に及ぶ保守作業(改名、リファクタリング、改訂)のリスクはライブラリの規模とともに増える。

メッセージ:

- 現行 Mizar が間違っているのではない。記事単位の境界が、より小さなライブラリのために設計されたものだ、ということである。

### Frame 1.3 - 圧力その2: ツーリングへの期待

要点:

- エディタには即時・部分的・耐障害なフィードバックが期待される。
- ビルドはマニフェストとロックファイルから再現可能であることが期待される。
- 再利用はバージョン付きのパッケージ粒度で機能することが期待される。
- ドキュメントは生成・リンク・閲覧可能であることが期待される。

メッセージ:

- この期待水準は主流言語のエコシステムが作った。新しいユーザーはその期待とともに来るし、形式ライブラリもその水準で評価される。

### Frame 1.4 - 圧力その3: AI

要点:

- AI エージェントはすでに検索・説明・修復に有用である。
- 彼らに必要なのは、ライブラリ全体のダンプではなく、有界で構造化されソース位置に結び付いたコンテキストである。
- AI の出力が証明の真偽を定義してはならない。検証は支援者の能力から独立でなければならない。
- ここでは可読なソースが強みになる。安定した局所的テキストパターンこそ、AI の編集と検索が最も得意とするものである。

メッセージ:

- Mizar の可読性は懐古的な資産ではない。安全な AI 支援を可能にするものそのものである。

### Frame 1.5 - 3つの圧力、1つの設計

![実証済みの設計にかかる3つの圧力](figures/three_pressures.pdf)

メッセージ:

- 3つの圧力のどれも、Mizar の設計が間違っていたと言うものではない。3つ合わせて、境界を進化させるべきだと言っている。

### Frame 1.6 - 設計原則

スライドテキスト:

```text
Do not trade away readability to gain automation.
Use automation to protect and extend readability.
(自動化のために可読性を手放さない。自動化で可読性を守り、広げる)
```

3本柱と単一のテスト:

| 柱 | すべての機能に課すテスト |
|---|---|
| 可読性 | 証明テキストは依然として数学として読めるか |
| AI 対応 | ツールは有界で監査可能なコンテキストを見られるか |
| スケーラビリティ | ライブラリが成長しても境界は安定か |

発表者ノート:

- 続く8つの物語は、この原則を具体的な痛み1つずつに適用したものである。

## Part 2. Story 1: Dependencies You Can See(見える依存関係)

### Frame 2.1 - 痛み

旧 Mizar(exact MML excerpt):

```mizar
environ

 vocabularies XBOOLE_0, SUBSET_1, BINOP_1, ZFMISC_1, STRUCT_0, ARYTM_3,
      FUNCT_1, FUNCT_5, SUPINF_2, ARYTM_1, RELAT_1, MESFUNC1, ALGSTR_0, CARD_1;
 notations TARSKI, XBOOLE_0, SUBSET_1, ZFMISC_1, BINOP_1, FUNCT_5, ORDINAL1,
      CARD_1, STRUCT_0;
 constructors BINOP_1, STRUCT_0, ZFMISC_1, FUNCT_5;
 registrations ZFMISC_1, CARD_1, STRUCT_0;
 theorems STRUCT_0;

begin :: Additive structures
```

発表者ノート:

- 出典: 現行 MML `algstr_0.miz` 15-25行。
- この部屋の全員が、このブロックを試行錯誤で編集した経験を持つ。
- `environ` が悪いという話ではない。これが数十年のライブラリ成長を支えた。問題は、今日のスケールでのコストである。

### Frame 2.2 - なぜ痛いのか [deep dive]

要点:

- 1つの記号の出所が複数の役割リストに分散している。どの記事がどの記法・構成子・cluster を提供しているのか、レビュー担当者には見えない。
- Accommodator が環境を解決するが、解決結果はレビュー可能なソーステキストではない。
- ツールは記事より細かい粒度でキャッシュも無効化もできない。
- 定理を記事間で移動すると、未知の依存先を壊すリスクがある。

メッセージ:

- 暗黙の依存面は、すべての編集・レビュー・ツールに毎回かかる固定費であり、その額はライブラリとともに増える。

### Frame 2.3 - Evo の答え: import 前文

Mizar Evo(specification example):

```mizar
import .function;
import mml.algebra.structure.sorted;

definition
  let S be 1-sorted;
  mode BinOpDef: BinOp of S is
    Function of [: S.carrier, S.carrier :], S.carrier;
end;
```

決定性を与える規則:

- すべての import は最初の非 import アイテムより前に置く。
- import が初期の有効な語彙(active lexicon)を供給する。局所宣言は宣言位置以降で語彙を拡張し、インポートされたアイテムは出典の FQN を保持する。
- 安定した完全修飾名(FQN)はパッケージとモジュールのパスから導出される。

### Frame 2.4 - Evo の答え: パッケージ [deep dive]

Mizar Evo(specification example):

```toml
[package]
name    = "algebra"
version = "2.3.1"
edition = "2025"

[dependencies]
mml_core = "^1.0"
topology = { version = "^0.9", features = ["metric"] }
```

要点:

- 再現可能なビルドには、固定したソース、ロックファイル、ツールチェーン、検証器設定(決定的な ATP 証拠を含む)が必要である。
- バージョン付き再利用(SemVer)が、記事集合間のその場しのぎのコピーを置き換える。

### Frame 2.5 - 環境の移行 [deep dive]

![environ から import への移行対応](figures/environ_migration.pdf)

メッセージ:

- 移行は改名だけではない。レポートは各モジュールが提供する構文・意味論・自動化を示さなければならない。

### Frame 2.6 - 保存されるもの、問いたいこと

保存されるもの:

- 数学と定理の同一性には手を触れない。
- 記事スタイルの執筆文化は残る。モジュールは依然として読み物である。
- 移行では origin メタデータを保持する(物語8で再訪する)。

Bialystok への問い:

- 移行中、どの `environ` の役割は見た目の親しみを残すべきか。
- 現行のどの記事依存が最も説明困難で、生成される依存レポートの最良のテストケースになるか。

## Part 3. Story 2: Structures Without Hidden Merges(隠れたマージのない構造体)

### Frame 3.1 - 痛み

旧 Mizar(exact MML excerpt):

```mizar
definition
  struct (1-sorted) addMagma (# carrier -> set, addF -> BinOp of the carrier
  #);
end;
```

要点:

- 親リンク、フィールド、セレクタの配置が1つのコンパクトな宣言に同居している。
- 親が複数のとき、継承フィールドのマージは暗黙である。
- 改名されたビュー(加法版と乗法版)は命名慣習に依存している。
- 格納データと標準的な値(零元、単位元)が区別されない。

発表者ノート:

- 出典: 現行 MML `algstr_0.miz` 37-40行。
- このコンパクトさが利点だったことを認める。構造体は非形式の数学的記述に近いままでいられた。

### Frame 3.2 - なぜ痛いのか [deep dive]

要点:

- ダイアモンド継承では、どの継承セレクタが共有されるかを読者が追う必要がある。Evo はその経路に対する明示的なメンバー対応付けを提案する。
- 移行ツールは「このセレクタは固有データか、証明義務付きの標準値か、継承ビューか」を問えない。構文がそれを語らないからである。
- エラーは原因から遠い場所で、後続記事の型不一致として現れる。

メッセージ:

- MML の規模では、構造体継承はグラフ保守の問題である。そのグラフには明示的で検査可能な辺がふさわしい。

### Frame 3.3 - Evo の答え: field / property / attribute

Mizar Evo(specification example):

```mizar
definition
  struct AddLoopStr where
    field carrier -> set;
    field add -> BinOp of carrier;
    property zero -> Element of carrier;
  end;
end;
```

| 概念 | 意味 | 帰結 |
|---|---|---|
| `field` | 格納データ | 構成子引数、exact instance の等価性 |
| `property` | 実装から得られる派生値 | `means`: existence/uniqueness、`equals`: 直接の項 |
| `attribute` | 述語型の細別 | cluster 伝播に参加。レイアウトではない |

property 宣言だけでは値は得られない。実装が値を与えなければならない。

発表者ノート:

- 説明する: 格納する field、派生値の property、attribute を区別する。構成子の引数は field のみ。実装の定義域が重なる場合は `coherence` が必要。

### Frame 3.4 - Evo の答え: 明示的な継承

Mizar Evo(specification example):

```mizar
definition
  struct AddMagma where
    field carrier -> set;
    field add -> BinOp of carrier;
  end;

  inherit AddMagma extends Magma where
    field carrier from carrier;
    field add from binop;    :: renamed
  end;
end;
```

要点:

- `inherit` 文は親1つにつき1つ。対応付けと改名はソーステキストである。
- 継承メンバーの型が同一でない場合、部分型包含の `coherence` 証明が必要。同名・同型の対応付けでは不要。
- 加法版・乗法版という命名慣習が、検査されるビューになる。

### Frame 3.5 - Evo の答え: ダイアモンドが検査可能になる

Mizar Evo(specification example):

```mizar
definition
  struct DoubleLoopStr where
    field carrier -> set;
    field add -> BinOp of carrier;
    field mul -> BinOp of carrier;
    property zero -> Element of carrier;
    property one -> Element of carrier;
  end;

  inherit DoubleLoopStr extends AddLoopStr;
  inherit DoubleLoopStr extends MulLoopStr;
end;
```

継承図(sketch):

![検査可能な結合を持つダイアモンド](figures/diamond_inheritance.pdf)

メッセージ:

- 同名・同型のメンバーは、ルート宣言が異なっていても結合できる。それ以外の結合には部分型包含の `coherence` 証明が必要。
- 改名ビューは別々に保つ。無効な対応付けや証明の欠落は診断になる。

発表者ノート:

- 図は `AddLoopStr` と `MulLoopStr` から `Magma` への、図中の親対応付けを前提とする。コードは両者を親とする子構造体を追加する。

### Frame 3.6 - 保存されるもの、問いたいこと

保存されるもの:

- 構造体は Mizar の構造体のまま: carrier、セレクタ、`Element of`。
- 集約のコンパクトさを手放すのは、そこに隠れた決定が存在した箇所だけである。

Bialystok への問い:

- `field` / `property` / `attribute` の区別は実際の代数の記事で読みやすいか。単純な場合に注釈過剰にならないか。
- `inherit` 1文につき親1つという制約は、`ALGSTR` 系や位相のような継承の多い MML 領域で受け入れ可能か。
- ダイアモンドのテストケースとして最適な MML 構造体はどれか。

## Part 4. Story 3: Automation You Can Audit(監査できる自動化)

### Frame 4.1 - 痛み

旧 Mizar(exact MML excerpt):

```mizar
registration
  let M be addMagma;
  cluster right_add-cancelable left_add-cancelable -> add-cancelable for
Element
    of M;
  coherence;
end;
```

発表者ノート:

- 出典: 現行 MML `algstr_0.miz` 104-109行。
- registration は Mizar の最良のアイデアの1つである。形容詞は静かに伝播し、証明は短く保たれる。痛みは機構ではなく、その不透明さにある。

### Frame 4.2 - なぜ痛いのか [deep dive]

要点:

- 証明が失敗したとき、「なぜ検査器はこれが Group だと分からないのか」に局所的な答えがない。原因は環境のどこかにある。
- どの registration がどの順で発火したかは見えない。自動化は強力だが、その説明能力は強さに比例して伸びない。
- AI 支援にとっては状況がさらに悪い。cluster の状態を読む代わりに推測するしかない。

メッセージ:

- 自分を説明できない自動化は、たとえ健全でも、ライブラリ規模では保守の負債になる。

### Frame 4.3 - Evo の答え: ラベル付きで追跡可能な registration

Mizar Evo(specification example):

```mizar
registration
  cluster EmptyImpliesFinite: empty -> finite for set;
  coherence proof ... end;

  cluster FiniteImpliesCountable: finite -> countable for set;
  coherence proof ... end;
end;
```

要点:

- すべての registration 項目に必須のラベルが付く。`by` で引用でき、診断で報告され、モジュールインターフェースの一部になる。
- 検証器はグローバルな cluster グラフを import でフィルタしたビューを使う。
- `explain-attribute` と resolution trace が、属性を導出できる理由や解決が失敗する理由を示す。例: empty -> finite -> countable。

### Frame 4.4 - Evo の答え: 向き付き簡約 [deep dive]

Mizar Evo(specification example):

```mizar
registration
  let n be Nat;
  reduce NatAddZero: n + 0 to n;
  reducibility
  proof
    let n be Nat;
    thus n + 0 = n by mml.number.natural.Nat_add_zero;
  end;
end;
```

要点:

- 簡約(reduction)は等式証明に裏付けられた向き付きの単純化である。
- 右辺は厳密に小さくなければならず、インポートされた規則が循環を作れない。規則選択は決定的(パターン包摂 → ガード特異性 → FQN タイブレーク)。
- 向きのない同一視のイディオムは、監査可能な `reduce` 項目になる。

### Frame 4.5 - 保存されるもの、問いたいこと

保存されるもの:

- registration と cluster は第一級のまま。証明は短いまま。
- cluster の適用には証明の繰り返しは要らない。簡約には局所的なガード証拠か、明示的な等式引用が要ることがある。

Bialystok への問い:

- 日々の摩擦を最も減らす cluster 説明はどれか: 失敗の説明、発火トレース、環境間の差分レポート。
- registration を最も酷使する MML 記事群はどれで、cluster グラフの移行ベンチマークにすべきはどれか。

## Part 5. Story 4: Powerful Search, Small Trust(強力な探索、小さな信頼)

### Frame 5.1 - 痛み

要点:

- ユーザーはより強い自動化を求める: より大きな `by` ステップ、hammer 型の探索。
- MizAR と MPTP は既に MML の前提を使った ATP 探索を行っている。
- 証明検査を小さく保ちながら、証明探索を強化したい。
- Evo は探索に検査可能な証拠を要求する。探索結果だけでは kernel-verified な証明を確立できない。

スライドテキスト:

```text
How do we get modern proof search
without trusting the searcher?
(探索器を信頼せずに、現代的な証明探索を得るには)
```

発表者ノート:

- MizAR・MPTP・hammer 系研究を紹介する。既存ツールが探索バックエンドを信頼せざるを得ないとは主張せず、Evo の証拠に関する契約を説明する。

### Frame 5.2 - Evo の答え: 推論境界

![推論境界: 意味論、信頼しない探索、信頼する検査](figures/reasoning_boundary.pdf)

鍵となる規則:

- ATP は名前解決も型推論も cluster 展開もオーバーロード選択も行わない。
- カーネルは与えられた式と代入を検査し、信頼された SAT 検査を実行する。前提を選ばず、代入を発明しない。
- ATP 前の決定的な討ち取り(discharge)にも再生可能な証拠が要る。「前の段階が済んだと言ったから」では何も受理されない。

発表者ノート:

- 説明する: 探索が証拠を提案し、カーネルがそれを検査する。
- 開発ポリシーは `externally_attested` の結果を記録できる。これはカーネル検査済みの証明とは別である。

### Frame 5.3 - 式と代入の証拠

![KernelEvidence とカーネルの trusted SAT check](figures/certificate_replay.pdf)

要点:

- `KernelEvidence` はソース式、明示的な代入、出典情報、対象/目標への束縛を含む。
- カーネルはこの証拠を検査し、インスタンス化された式と SAT 節を導出し、信頼されたプロセス内 Rust SAT checker による UNSAT を要求する。
- バックエンドの resolution トレース、SMT 証明オブジェクト、ログ、終了コードは、受理の証拠として信頼しない。

発表者ノート:

- ハッシュは証拠を依存関係コンテキストに束縛する。カーネルは束縛変数の条件と目標の反駁極性も検査する。

### Frame 5.4 - 同じ境界が AI を飼い慣らす

旧 Mizar(exact MML excerpt):

```mizar
theorem
  for F being non degenerated ZeroOneStr holds 1.F in NonZero F
proof
  let F be non degenerated ZeroOneStr;
  not 1.F in {0.F} by TARSKI:def 1;
  hence thesis by XBOOLE_0:def 5;
end;
```

メッセージ:

- 引用修復(欠けている、あるいはより鋭い `by` 参照の提案)は、安全な AI 編集の典型である。ソース局所的で、意味を保存し、人間の編集と同じように検証器が検査する。

発表者ノート:

- 出典: 現行 MML `struct_0.miz` 637-643行。
- エージェントは提案し、検証器とカーネルが決める。支援者の能力は信頼基盤に決して入らない。

### Frame 5.5 - 編集クラス: Green / Yellow / Red [deep dive]

| クラス | 例 | ポリシー |
|---|---|---|
| Green | 引用の追加、`qua` の挿入、情報注釈 | 自動提案可。ただし必ず検証される |
| Yellow | import の追加、局所補題、registration | 人間のレビュー付きで提案 |
| Red | 定理の弱化、定義の変更、公理の追加 | 通常のエージェントには禁止 |

禁止される修復(sketch):

```mizar
theorem
  for x be Nat holds x + 0 = x or x = 0;
```

メッセージ:

- 証明を楽にするために主張を弱めるのは Red 編集である。エージェントにできるのは、せいぜい人間の明示的な unsafe-edit レビューを要請するフラグを立てることまでである。

### Frame 5.6 - 保存されるもの、問いたいこと

保存されるもの:

- de Bruijn の規律: 受理は SAT checker を含む小さな信頼コアを使う。
- 証明テキストは宣言的で可読なまま。自動化は論証を保守するのであって、置き換えるのではない。

Bialystok への問い:

- 式/代入の証拠方式は、cluster や定義展開を含む Mizar 型の証明義務に対して説得力があるか。
- チームが最も監査してもよいと思える証拠フォーマットはどれか。

## Part 6. Story 5: Verification That Scales(スケールする検証)

### Frame 6.1 - 痛み

要点:

- MML 全体の検証は時間単位で測るバッチ処理である。
- 再利用境界は受理済み記事であり、小さな変更が必要以上を再検証する。
- メモリは実際に使うインターフェースではなく、記事環境に従って増える。
- これは現行設計の欠陥ではない。ライブラリが大きくなったときに記事粒度が意味するもの、そのものである。

### Frame 6.2 - Evo の答え: フィンガープリントと差分検証

要点:

- キャッシュキーはソース、依存スライス、パッケージ/ロックファイル、ツールチェーン、スキーマ、ポリシー、registration、義務、証拠、witnessを含む。
- 関係するすべてのキーが一致するときだけ再利用し、データが欠ければ cache miss になる。
- 定理の証明本体を変更しても、公開された statement と受理 status が変わらなければ importer は再ビルドしない。
- 独立なモジュール、証明義務、ATP 実行、カーネル検査は並列に走り、結果は正準順で公開される。

![フィンガープリントグラフ: 変更が何を再検証するか](figures/fingerprint_graph.pdf)

規則:

```text
Cache reuse is never proof authority.
A clean build must always be able to reproduce every acceptance.
(キャッシュ再利用は証明の権威ではない。クリーンビルドは常にすべての受理を再現できねばならない)
```

### Frame 6.3 - Evo の答え: メモリ契約 [deep dive]

```text
resident memory should scale with:
  active source and typed AST
  imported public interfaces
  import-filtered indexes
  active module obligations
  in-flight proof checks and ATP runs
  small caches

not with:
  imported proof bodies
  imported proof witnesses
  private lemmas outside the interface
  registration data outside the import closure
```

メッセージ:

- これは常駐メモリのモデルであり、測定済みの性能保証ではない。特定の query に必要なときだけ証明本体と trace を lazy にロードする。

### Frame 6.4 - 保存されるもの、問いたいこと

保存されるもの:

- クリーンビルドの意味論: キャッシュと並列化が変えるのは速度であって、真偽ではない。
- 記事スタイルのレビュー: 人間が読むものは依然として完全なソーステキストである。

Bialystok への問い:

- 今日 MML を保守しているチームにとって、差分検証を信頼できるものにするクリーンビルド等価性テストとは何か。
- 現行のどの保守作業(改訂、改名)を差分コストのベンチマークにすべきか。

## Part 7. Story 6: Templates For Generic Mathematics(汎用数学のためのテンプレート)

### Frame 7.1 - 痛みその1: scheme は柵の中にいる

旧 Mizar(exact MML excerpt):

```mizar
scheme
  NatInd { P[Nat] } : for k being Nat holds P[k]
provided
A1: P[0] and
A2: for k be Nat st P[k] holds P[k + 1]
```

要点:

- scheme は二階的なパターン(帰納法、分出、置換)を担い、実際に機能している。しかし別規則を持つ別機構である。
- 従来の `scheme` ブロックは定理スキーマを定義する。パラメータ付き定義には別の言語形式を使う。

発表者ノート:

- 出典: 現行 MML `nat_1.miz` 90行付近(2026年7月2日確認)。

### Frame 7.2 - 課題その2: 共通のテンプレート機構

要点:

- Mizar には既に `Polynom-Ring L` のようなパラメータ化された構成がある。
- scheme とパラメータ付き定義は異なる機構を使う。
- Evo は定義と定理スキーマに共通のテンプレート機構を提案し、パラメータの種類ごとの規則を明示する。

メッセージ:

- 汎用的な数学を記述・保守しやすくすることが目的である。規則が役立つかは移行例で確かめる必要がある。

出典: [POLYNOM3, definition 10](https://mizar.uwb.edu.pl/version/current/html/polynom3.html)。

### Frame 7.3 - Evo の答え: テンプレート

Mizar Evo(specification example):

```mizar
definition
  let T be type;
  struct MagmaStr[T] where
    field carrier -> T;
    field binop -> BinOp of T;
  end;
end;
```

要点:

- テンプレートは、先頭の `let` がパラメータ(型・値・述語・functor)を束縛する、通常の `definition` ブロックである。
- predicate と functor のパラメータは `attr`、`mode`、`struct`、`func`、`pred` 項目では使えない。
- 適格なテンプレートには短い形式が使える: `Module over R` は `Module[R]`、`Subset of X` は `Subset[X]` である。制約付きテンプレートにはブラケットが必要である。

### Frame 7.4 - 有界パラメータと汎用定理 [deep dive]

Mizar Evo(仕様 §18.2.2 の例; 証明は省略):

```mizar
definition
  let T be type extends commutative associative unital Magma;
  theorem PermProduct[T]:
    for s being FinSequence of T,
        p being Permutation of dom s
    holds Product[T](s) = Product[T](s * p)
  proof ... end;
end;
```

メッセージ:

- 境界条件は、可換性、結合律、単位元を要求する。
- `Product[T]` は単位元から始め、選択した演算で列を fold する。これらの定義方程式はライブラリが証明する。

### Frame 7.5 - 1つの証明、多くのインスタンス化 [deep dive]

インスタンス化(仕様の例; 必要な registration を仮定):

```mizar
PermProduct[commutative associative unital AddMagma]
PermProduct[commutative associative unital MulMagma]

let R be commutative Ring;
PermProduct[R qua AddMagma]        :: R の加法ビュー
PermProduct[R qua MulMagma]        :: R の乗法ビュー
```

メッセージ:

- 環は2つの経路で Magma に到達する。`qua` がビューを選ぶ。ビューは記法も決める。総称の `*` は加法の場合には `+` として表示される。
- 必要な属性は、選択したビュー上で成立していなければならない。

### Frame 7.6 - scheme は普通のテンプレートになる [deep dive]

Mizar Evo(specification example):

```mizar
definition
  let P be pred(Nat);
  theorem NatInduction[P]:
    P(0) & (for n being Nat st P(n) holds P(n+1))
    implies for n being Nat holds P(n)
  proof ... end;
end;
```

要点:

- 述語パラメータは馴染みの `defpred` の慣習に従う。
- 旧 scheme には直接的・機械的な移行先がある。
- 定理のインスタンス化は明示的なブラケット構文を使うので、ツールと成果物は証明がどのインスタンスを使ったかを正確に見られる。

### Frame 7.7 - 保存されるもの、問いたいこと

保存されるもの:

- scheme 型の推論は能力を落とさず生き残る。
- `of` / `over` の言い回しが数学的散文の可読性を保つ。
- 一階の規律: 各インスタンス化を検査する。テンプレートは新しい論理を追加しない。

Bialystok への問い:

- 最初の移行対象にすべき MML の scheme はどれか。
- ブラケットを正準の同一性形式とし、`of`/`over` を表示形式とすることは受け入れ可能か。
- `func` と `pred` テンプレートでは、正規化した宣言引数型が、推論する各型パラメータを一意に定めなければならない。`qua` ビューは推論しない。この規則は明示的な `[T]` 引数を要求しすぎるか。

## Part 8. Story 7: Verified Computation With Algorithms(アルゴリズムによる検証済み計算)

### Frame 8.1 - 痛み

要点:

- Mizar には既に算術の自動処理とプログラムの正当性証明がある。
- Evo は契約付きアルゴリズムの言語構文を提案し、検証・実行・コード抽出を結び付ける。
- 問いは、これらの段階を1つの言語とツールチェーンでどう結び付けるかである。

スライドテキスト:

```text
Connect mathematical proofs with executable algorithms.
(数学の証明と実行可能なアルゴリズムを結び付ける)
```

MML の例: 算術 requirements は `NUMERALS`、プログラムの正当性は `FIB_FUSC`。
[Formalized Mathematics の目次](https://mizar.uwb.edu.pl/fm/contents.html)を参照。

### Frame 8.2 - Evo の答え: 契約付きアルゴリズム

Mizar Evo(specification example。仕様 20.12 節から凝縮):

```mizar
definition
  let a, b be Nat;
  terminating algorithm euclid_gcd(a, b) -> Nat
    requires a >= 1 & b >= 1
    ensures result = Gcd(a, b)
  do
    var x := a;  var y := b;
    while y <> 0 do
      invariant x >= 1 & y >= 0 & Gcd(a, b) = Gcd(x, y);
      decreasing y;
      const r := x mod y;  x := y;  y := r;
    end;
    return x;
  end;
end;
```

メッセージ:

- `ensures` と不変条件は数学側の `Gcd` を引用する。循環はない。

発表者ノート:

- 説明する: 契約は数学で記述する。アルゴリズムが計算し、ライブラリの functor `Gcd` が仕様を与える。停止性は `y` に対する `decreasing` 測度から来る。

### Frame 8.3 - Evo の答え: 計算による証明

Mizar Evo(specification example):

```mizar
theorem EuclidGcd12_8:  euclid_gcd(12, 8)  = 4  by computation;
theorem EuclidGcd100_75: euclid_gcd(100, 75) = 25 by computation;

theorem Fact10: factorial(10) = 3628800
proof
  thus thesis by computation(steps: 100000);
end;
```

要点:

- Mizar Virtual Machine(MVM)が対象とするのは ground な等式と述語だけであり、現行パッケージで定義された計算可能なアルゴリズムを使って評価する。
- ステップ、時間、深さの制限は任意で、それぞれの既定値は `0`(無制限)である。
- 他パッケージのアルゴリズムは不透明である。停止性が分かっている場合は `ensures` 契約を使えるが、ここで本体を実行することはできない。

### Frame 8.4 - Evo の答え: 停止性が再帰を買う [deep dive]

Mizar Evo(specification example):

```mizar
definition
  let n be Nat;
  terminating algorithm factorial(n) -> Nat
    decreasing n
  do
    if n = 0 do return 1; end;
    return n * factorial(n - 1);
  end;
end;
```

要点:

- 通常の `func` 定義は定義による拡張であり、決して再帰しない。
- `requires` を満たす入力について contract と termination のすべての義務を証明すると、`terminating` アルゴリズムは functor に昇格する。
- これが再帰が数学の層に入る唯一の扉であり、その扉は証明の形をしている。

発表者ノート:

- 各呼び出しは `requires` を満たさなければならない。通常の algorithm では停止性の証明は必須ではなく、functor に昇格されない。

### Frame 8.5 - 計算は真理を再定義しない [deep dive]

要点:

- 契約、不変条件、停止性測度が生成する検証条件は、通常の定理と同じ ATP+カーネル境界(物語4)を通る。
- `by computation` は制限を任意に設定できる MVM replay を使う。
- コード抽出(実行ターゲットへの出力)は検証済み成果物の厳密に下流であり、受理へ逆流することはない。

メッセージ:

- アルゴリズムはライブラリの表現力を広げる。定理が受理されるとは何か、には触れない。

### Frame 8.6 - 保存されるもの、問いたいこと

保存されるもの:

- アルゴリズムは `definition` ブロックに置く。証明では検証済みの昇格、または現行パッケージの計算可能な ground 呼び出しに対する `by computation` を使える。
- 部分アルゴリズムの `ensures` を使うには、その呼び出しが停止する証拠が必要である。
- 一階の集合論的基礎は無傷のまま残る。

Bialystok への問い:

- 文化をプログラミングへ傾けずに数学者に価値を示せる計算例はどれか。
- `by computation` が実在の証明を直ちに短くする MML 領域(数論、組合せ論、有限構造)はあるか。
- 抽出ターゲットとして最初に重要なのはどれか。そもそも要るか。

## Part 9. Story 8: A Library You Can Cite(引用できるライブラリ)

### Frame 9.1 - 痛み

要点:

- Formalized Mathematics は Mizar に希有なものを与えている: 形式ライブラリの上の、学術的で引用可能な出版層である。
- しかし記事の同一性とライブラリの編成は強く結合している。ライブラリのリファクタリングは出版済み記事の構造に負荷をかけ、パッケージ再利用にはジャーナル向けの同一性が存在しない。
- 解説は物語の順序を欲し、再利用は依存の順序を欲する。1つの構造で両方は最適化できない。

### Frame 9.2 - Evo の提案: リンクされた記録

![提案する記事-ライブラリ リンクモデル(スケッチ)](figures/fm_links.pdf)

メッセージ:

- リンクされた記録が、雑誌の引用、ライブラリ内の所在、検証成果物、MML の出自を結び付ける。

### Frame 9.3 - 誰が何を得るか [deep dive]

| 対象 | 得るもの |
|---|---|
| 読者 | 散文が主役のまま。形式ソースはワンクリック先 |
| 保守者 | リファクタリングが出版済み解説を書き換えなくなる |
| 著者 | 凍結した `pub` の記事同一性。ライブラリ内の所在は FQN |
| 計画中の AI ツール | 検索には散文、正確な文脈にはフィンガープリント |

### Frame 9.4 - 保存されるもの、問いたいこと

保存されるもの:

- Formalized Mathematics は査読と解説を備えた本物のジャーナルであり続ける。
- 提案する origin メタデータは移行項目を MML 引用へリンクする。移行マッピングはまだ定義されていない。

Bialystok への問い:

- 利用者向けの引用で第一とすべき同一性はどれか: 記事ラベル、ライブラリ FQN、origin id。
- 既存の Formalized Mathematics 記事は移行後のモジュールにどうリンクすべきか: 遡及的に、改訂時に、それとも紐付けないか。

## Part 10. Architecture In One Picture(一枚の絵のアーキテクチャ)

### Frame 10.1 - コア ATP 経路

![責務グループ付きコア ATP 経路](figures/pipeline.pdf)

メッセージ:

- すべての境界は、誰がその事実を所有し、どの成果物がそれを記録し、変更時に何を再計算すべきかを述べる。それこそが要点である。

発表者ノート:

- 図は ATP 経路を示す。決定的な討ち取りの後も開いている義務だけが ATP に進み、先行する討ち取りにも証拠が要る。

### Frame 10.2 - 責務の分割 [deep dive]

| 層 | 責務 |
|---|---|
| フロントエンド | 字句解析、構文解析、リカバリ |
| リゾルバ | import、名前、ラベル、名前空間 |
| チェッカ | ソフト型、cluster、registration、オーバーロード |
| エラボレータ | コア論理表現 |
| VC 生成器 | 証明とアルゴリズムの義務 |
| ATP 層+カーネル | 信頼されない探索と、検査による受理 |
| 成果物エミッタ | ツールと依存先のための安定した出力 |

### Frame 10.3 - 8つの物語はどこに住むか

| 物語 | パイプライン上の住所 |
|---|---|
| 依存関係 | リゾルバ、パッケージマネージャ |
| 構造体と自動化 | チェッカ(継承・cluster グラフ、トレース) |
| 探索と信頼 | ATP 層、カーネル、式/代入の証拠 |
| スケール | 成果物、フィンガープリント、スケジューラ |
| テンプレート | エラボレータ(検査されるインスタンス化) |
| アルゴリズム | VC 生成器、MVM |
| 出版 | 成果物エミッタ、ドキュメント生成 |

メッセージ:

- 8つの物語は8つの別プロジェクトではない。8つのユーザー可視の痛みから見た、1本のパイプラインである。

### Frame 10.4 - 信頼境界のテスト [deep dive]

スライドテキスト:

```text
Reject what must not pass
before
Accept everything that should pass
(通してはならないものを拒否することが先。通すべきものを全部通すことは後)
```

要点:

- 健全性バグはパーサの穴より重い。カーネル近傍のテストは、不正な証拠と失敗する証拠を先に重視する。
- 受理言語のカバレッジは、その盾の後ろで育てる。

## Part 11. Roadmap And Collaboration(ロードマップと協働)

### Frame 11.0 - プロジェクトの現在地

要点:

- 仕様: 24章+付録。英語が正典で、日本語対訳がある。
- 実装済みの部分: フロントエンド処理、一部の意味解析ケース、ATP 候補の生成、kernel evidence の検査。
- 作業中: ソース入力から ATP 探索、カーネル検査を経て成果物公開に至る経路全体の統合。実際の外部証明器を使うテストも引き続き必要。
- 保留: MVM 実行とコード抽出。
- MML 全体の移行は今後の課題。

メッセージ:

- 2026年末に予定する alpha は、進行中の作業におけるマイルストーンである。

発表者ノート:

- これは2026年9月時点の範囲である。個々のコンポーネントのテストだけでは、パイプライン全体の動作は確立されない。`doc/design/todo.md` の Completion Gates を参照。

### Frame 11.1 - 移行は研究プログラムである

![ロードマップ年表](figures/roadmap_timeline.pdf)

フェーズ:

1. 2026年末、アルファ: コアサブセットのフロントエンドとパーサ、import とモジュール解決のプロトタイプ、構造化診断、初期の成果物。
2. 2027年、移行ラボ: 代表的な MML 記事3〜5本を手作業とスクリプトで翻訳し、すべての不一致を分類済み課題として記録する。
3. 2027〜2028年、拡大: 基礎的な集合・関係の断片、次いで代数構造と、成功した断片の依存錐へ。

アルファの非目標:

- MML 全体の検証、最終互換レイヤ、安定 AI プロトコル。

### Frame 11.2 - 何を測るか [deep dive]

要点:

- 翻訳済み記事数と行数。受理されたパーサのサブセット。
- 解決された import と未解決の依存。
- 決定的に閉じた義務と、kernel evidence を伴う ATP で閉じた義務。
- モジュールあたりのメモリと実時間。差分ビルドとクリーンビルドの比。
- 人間の判断を要した互換性決定の件数。

### Frame 11.3 - 互換性ポリシーとリスク [deep dive]

ポリシー:

- MML の定理同一性を保つ origin mapping を定義する。
- 移行に役立つところでは互換エイリアスを残す。
- 旧挙動からのすべての乖離を、理由とテストとともに記録する。

| リスク | 緩和策 |
|---|---|
| 互換性作業がプロジェクトを飲み込む | 代表スライスを先に。ビッグバン翻訳はしない |
| registration の挙動が変わる | トレース成果物と比較レポートを早期に |
| パッケージ配置がジャーナルのリンクを壊す | origin メタデータ、記事-ライブラリ識別子 |
| AI 編集が移行ミスを隠す | 通常のエージェントには Red 編集を禁止したまま、検証器成果物を必須に |

### Frame 11.4 - 2026年9月が生むべきもの

要点:

- 優先順位付きの移行ベンチマーク記事リスト。
- 互換性メタデータの要件についての合意。
- 8つの物語、特に構造体と cluster へのレビューノート。
- 最初の論文アウトライン。
- 共有された反論とリスクのリスト。

### Frame 11.5 - 皆さんにお願いしたいこと

問い:

- 小さいが構造的に代表性のある MML 記事はどれか。
- 技術的な利便を超えて、文化的に不可欠なイディオムはどれか。
- 8つの物語のうち最も間違っているのはどれで、それはなぜか。
- どんな移行結果が出れば、コミュニティは Evo が本気だと確信するか。

## Part 12. Closing(結び)

### Frame 12.1 - 通底する問い、再び

スライドの問い:

```text
What must Mizar Evo preserve so that the Mizar community
still recognizes it as Mizar?
(Mizar コミュニティが Mizar と認め続けるために、何を保存しなければならないか)
```

発表者ノート:

- 開幕の例(依然として Mizar として読める構造体)に立ち返る。
- 2〜3問を選ぶ: 何を残すべきか、最初に移行する小さな MML 記事は何か、最も修正が必要な提案はどれか。
- ほかの問いは、後日レジュメで検討してもらう。

### Frame 12.2 - 結び

最終スライド:

```text
Mizar Evo should be modern where scale demands it,
and conservative where Mizar's mathematical identity depends on it.
(スケールが要求するところでは現代的に、
 Mizar の数学的アイデンティティがかかるところでは保守的に)
```

## Backup A. 用意済みの逐語例

この発表で使う逐語抜粋:

| 目的 | 出典 | 行 | 使用フレーム |
|---|---|---:|---|
| 構造体定義 | `algstr_0.miz` | 37-40 | Frame 0.2, 3.1 |
| 記事環境 | `algstr_0.miz` | 15-25 | Frame 2.1 |
| registration/cluster | `algstr_0.miz` | 104-109 | Frame 4.1 |
| 証明の引用 | `struct_0.miz` | 637-643 | Frame 5.4 |
| 帰納法 scheme | `nat_1.miz` | 90付近 | Frame 7.1 |

出典 URL:

- `ALGSTR_0`: <https://mizar.uwb.edu.pl/version/current/mml/algstr_0.miz>
- `STRUCT_0`: <https://mizar.uwb.edu.pl/version/current/mml/struct_0.miz>
- `NAT_1`: <https://mizar.uwb.edu.pl/version/current/mml/nat_1.miz>

出典表示の注記:

- MML プレーンテキストは GPL-3.0-or-later / CC-BY-SA-3.0-or-later の条件を明記している。記事名・URL・行番号を発表者ノートに保持すること。

## Backup B. 仕様参照マップ

使用仕様: 2026年9月10日時点。文法と意味論は[リポジトリ](https://github.com/aabaa/mizar-evo)の `doc/spec/en/00.index.md` を参照。
オンラインのファイルは講演後に変更されることがある。

| トピック | 仕様の出典(`doc/spec/en/` 配下) |
|---|---|
| モジュールと import | `12.modules_and_namespaces.md` |
| 構造体と継承 | `05.structures.md` |
| attribute とモード | `06.attributes.md`, `07.modes.md` |
| registration と簡約 | `17.clusters_and_registrations.md` |
| テンプレートと scheme | `18.templates.md` |
| アルゴリズムと MVM | `20.algorithm_and_verification.md` |
| ATP とカーネル証拠 | `21.source_code_annotation_and_atp.md` |
| パッケージと成果物 | `23.package_management_and_build_system.md` |
| 章横断の文法 | `appendix_a.grammar_summary.md` |
| ライブラリのスケッチ集 | `sample_codes.md` |

## Backup C. 図版リスト

最終版デッキに必要な図(`figures/*.tex`、TikZ standalone。各図は `figures/` 内で `pdflatex` でビルド):

1. 実証済みの設計にかかる3つの圧力(Part 1)。[作成済み: `figures/three_pressures.pdf`、Frame 1.5 で使用]
2. environ から import への移行(物語1)。[作成済み: `figures/environ_migration.pdf`、Frame 2.5 で使用]
3. 構造体継承とダイアモンドの整合性(物語2)。[作成済み: `figures/diamond_inheritance.pdf`、Frame 3.5 で使用]
4. 推論境界: 意味論 / ATP 探索 / カーネル検査(物語4)。[作成済み: `figures/reasoning_boundary.pdf`、Frame 5.2 で使用]
5. KernelEvidence と trusted SAT check(物語4)。[作成済み: `figures/certificate_replay.pdf`、Frame 5.3 で使用]
6. 差分検証のフィンガープリントグラフ(物語5)。[作成済み: `figures/fingerprint_graph.pdf`、Frame 6.2 で使用]
7. Formalized Mathematics の記事-ライブラリ リンクモデル(物語8)。[作成済み: `figures/fm_links.pdf`、Frame 9.2 で使用]
8. 責務グループ付きコア ATP 経路(Part 10)。[作成済み: `figures/pipeline.pdf`、Frame 10.1 で使用]
9. ロードマップ年表(Part 11)。[作成済み: `figures/roadmap_timeline.pdf`、Frame 11.1 で使用]

## Backup D. 論文アウトラインの種

論文タイトル案:

```text
Mizar Evo: Readable, AI-Ready, and Scalable Formal Mathematics
```

節構成案:

1. 序論: なぜ今 Mizar に進化が必要か。
2. 基準線としての Mizar: 可読性、MML、Formalized Mathematics。
3. 設計原則と3本柱。
4. 言語の進化: 依存関係、構造体、registration、テンプレート。
5. 検証器アーキテクチャ、kernel evidence、trusted SAT checker。
6. 検証済み計算と MVM。
7. AI 安全な証明開発。
8. パッケージベースのライブラリと出版ワークフロー。
9. 移行計画と評価指標。
10. 関連研究と協働のアジェンダ。

## Backup E. レビュー用チェックリスト

Beamer 化の前にこのチェックリストを使う:

- すべての物語は機能の発表ではなく、実際のコストから始まっているか。
- 現行の慣行への批判はすべて、その慣行がなぜ有用だったかを認めているか。
- すべてのコード例にステータスラベル(exact MML excerpt / specification example / sketch)が付いているか。
- すべての逐語抜粋に記事名と行番号の出典があるか。
- すべての物語は、聴衆が実際に答えられる問いで終わっているか。
- Red の AI 編集は明確に禁止されているか。
- 移行の主張は測定可能か。
- すべてのフレームから EBNF が消えているか。

## Backup F. 想定反論

物語自身が扱わない反論への準備回答:

| 反論 | 準備回答 |
|---|---|
| なぜ現行 Mizar の漸進的改良ではだめなのか | 既存ツールにも役立つ提案があるかもしれない。Evo はモジュール、証明の証拠、ライブラリ成果物の変更をまとめて検討する。どの変更が有用かは移行例で判断したい。 |
| 既存 MML 記事の著者性とクレジットはどうなるか | 移行中の同一性とクレジットを保つ origin mapping を提案する。マッピング形式はまだ開いている。`pub` 名前空間は出版済み記事を変更せずに保つ。 |
| これはコミュニティのフォークではないのか | コミュニティへの提案であり、この訪問がその最初のレビューである。名前空間のガバナンスモデルは `mml` ルートを Mizar チームが統治する前提である。 |
| GPL / CC-BY-SA の義務はどうするのか | 移行は MML 内容のライセンスと帰属を保存する。ツールチェーンのライセンスは議論に開かれている。 |
| AI の話は誇大宣伝ではないのか | AI 支援は信頼基盤に決して入らない選択的レイヤであり、すべての提案は AI なしでも成立する。 |
| なぜ既存チェッカではなく新しいカーネルなのか | 式/代入の証拠には SAT checker を含む小さな信頼コアが要る。Mizar Evo の仕様が義務を定め、旧挙動は移行比較の指針になる。 |

発表者ノート:

- 提起された場合にのみ使う。先回りして提示しない。
