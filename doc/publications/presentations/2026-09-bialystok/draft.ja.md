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

コードの導入文と図の説明を含め、本文の文を上から順に読む。コード、表、出典、
「後日の確認用」の節は、講演中は省略できる。ノート版は本文の文を「Read aloud」
に再掲し、補足説明と出典を分ける。冒頭はタイトルページのノートを使う。
3.3、5.2、8.2 など、選んだ例に時間をかける。この読み順で練習して時間を確認する。

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

- Bialystok にお招きいただき、ありがとうございます。今日は Mizar Evo を紹介します。
- Mizar は数学の証明を書き、コンピュータで検査するシステムです。
  Evo は証明の読みやすさを保ちながら、言語とツールを更新するプロジェクトです。
- 今日は概要を説明します。詳しい内容は、後で確認していただけるようレジュメに残しています。
- 設計はまだ確定していません。ご意見や反論をいただければと思います。

### Frame 0.2 - 最初の一瞥

現行 Mizar は親構造を指定します(exact MML excerpt):

```mizar
definition
  struct (1-sorted) addMagma (# carrier -> set, addF -> BinOp of the carrier
  #);
end;
```

Evo はさらに、フィールドを共通の Magma ビューに対応付けます(specification example):

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

- どちらの例も carrier と二項演算を記述しています。
  現行 Mizar も親の `1-sorted` を明示しています。
  Evo の例では、同じフィールドを `Magma` として見るビューを加えています。
- 出典: 現行 MML `algstr_0.miz` 37-40行。

### Frame 0.3 - 一文で言う提案

スライドテキスト:

```text
Preserve Mizar's mathematical vernacular.
Update the compiler, verifier, artifact, and publication layers.
```

- これが中心となる提案です。現行 Mizar の達成を土台とします。
- 言語標準はまだ草案です。MML 全体の移行は完了していません。
- AI 支援が証明検査に代わることはありません。
- Evo の例は仕様に従っていますが、まだ実装されていないものもあります。
  現在の実装範囲は、発表の終盤で説明します。

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

- この訪問の目的は、皆さんの Mizar の経験から学ぶことです。
- 互換性に必要な条件を確認し、移行実験に使う小さな MML 記事を選びたいと考えています。
- 自動定理証明器、略して ATP の結果をどう検査するかも議論できればと思います。
  もう1つの話題は、成果を Formalized Mathematics にどう結び付けるかです。
- 中心となる問いは、次のとおりです。

通底する問い:

```text
What must Mizar Evo preserve so that the Mizar community
still recognizes it as Mizar?
(Mizar コミュニティが Mizar と認め続けるために、何を保存しなければならないか)
```

発表者ノート:

- 日々の MML 作業をしていないと見えにくい制約があるかもしれません。
  私が見落としている反論もお聞かせください。

## Part 1. Why Now(なぜ今か)

### Frame 1.1 - Mizar が正しくやったこと

- まず、何を保ちたいかを説明します。
- Mizar の証明は宣言的で、数学として読めます。
- ソフト型、mode、attribute は豊かな数学の語彙を与えます。
- registration と cluster は、ほかの証明でも再利用できる自動化を提供します。
- Mizar Mathematical Library、略して MML は大規模で、丁寧に保守されています。
  Formalized Mathematics は、その成果を出版する場を提供しています。
- Evo の各提案は、これらの強みをどれだけ守れるかで判断したいと考えています。

発表者ノート:

- ここで参照しているのは MML 5.94.1493、1493記事です。

### Frame 1.2 - 圧力その1: スケール

- この研究の第1の理由は、ライブラリの大きさです。
- MML には、互いに依存する約1500の記事があります。
- 記事が依存、レビュー、再利用の単位です。
- ツールは記事環境を解決しますが、解決後の依存関係はソースには見えません。
- ライブラリが大きくなるほど、記事の改名や改訂が多くの作業に影響し得ます。
  Evo では、より小さな境界が保守に役立つかを検討します。

### Frame 1.3 - 圧力その2: ツーリングへの期待

- 第2の理由は、現在のツールに対する期待です。
- ソースが未完成でも、エディタからすぐにフィードバックを得たいと考えます。
- マニフェストとロックファイルによって、ビルドを再現できることを期待します。
- バージョン付きのパッケージと、閲覧できるドキュメントを期待します。
- これらはプログラミングツールでは一般的です。形式ライブラリの作業にも役立てたいと考えています。

### Frame 1.4 - 圧力その3: AI

- 第3の理由は、AI 支援です。
- AI ツールは証明の検索、説明、編集を支援できます。
- 必要なのは、ソースに結び付いた少量の構造化された文脈です。
  作業のたびにライブラリ全体のコピーを渡す必要はないはずです。
- AI の出力も検査しなければなりません。証明の妥当性が AI ツールの能力に依存してはいけません。
- ここで Mizar の読みやすいソースが役立ちます。
  安定した局所的なテキストパターンが、編集と検索を容易にします。

### Frame 1.5 - 3つの圧力、1つの設計

![実績ある設計にかかる3つの圧力](figures/three_pressures.pdf)

- この図は、ライブラリの大きさ、ツール、AI という3つの理由をまとめています。
- これらは、Mizar の設計が誤っていたことを意味しません。
- Mizar の読みやすい数学的言語を保ちながら、一部の境界を更新したい理由です。

### Frame 1.6 - 設計原則

スライドテキスト:

```text
Do not trade away readability to gain automation.
Use automation to protect and extend readability.
(自動化のために可読性を手放さない。自動化で可読性を守り、広げる)
```

- ここから3つの問いが得られます。人が証明を読めるでしょうか。
  ツールがその文脈を調べられるでしょうか。ライブラリが成長しても使える設計でしょうか。
- 次の8つの物語では、これらの問いを具体的な問題に当てはめます。

後日の確認用の詳細:

| 目標 | すべての機能に対する試験 |
|---|---|
| 可読性 | 証明テキストが依然として数学として読めるか |
| AI 対応 | ツールが監査可能な限定された文脈を見られるか |
| スケーラビリティ | ライブラリが成長しても境界が安定しているか |

## Part 2. Story 1: Dependencies You Can See(見える依存関係)

### Frame 2.1 - 痛み

この環境は依存関係を列挙しています(exact MML excerpt):

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

- 最初の物語は依存関係です。これらのリストは、記事に必要なライブラリの内容を Mizar に伝えます。
- この形式は何十年もライブラリの成長を支えてきました。
  問いは、より大きなライブラリで依存関係をどうレビューしやすくするかです。

発表者ノート:

- 出典: 現行 MML `algstr_0.miz` 15-25行。

### Frame 2.2 - なぜ痛いのか [deep dive]

要点:

- 1つの記号の出所が複数の役割リストに分散している。どの記事がどの記法・構成子・cluster を提供しているのか、レビュー担当者には見えない。
- Accommodator が環境を解決するが、解決結果はレビュー可能なソーステキストではない。
- ツールは記事より細かい粒度でキャッシュも無効化もできない。
- 定理を記事間で移動すると、未知の依存先を壊すリスクがある。

メッセージ:

- 暗黙の依存面は、すべての編集・レビュー・ツールに毎回かかる固定費であり、その額はライブラリとともに増える。

### Frame 2.3 - Evo の答え: import 前文

Evo は定義より前に import を置きます(specification example):

```mizar
import .function;
import mml.algebra.structure.sorted;

definition
  let S be 1-sorted;
  mode BinOpDef: BinOp of S is
    Function of [: S.carrier, S.carrier :], S.carrier;
end;
```

- ここでは、import は最初の非 import 項目より前に置かれています。
- import は初期の active lexicon を与えます。
  ローカル宣言は、その宣言位置以降で active lexicon を拡張します。
- import した項目は出自の FQN を保持します。
  パッケージとモジュールのパスが、各項目に安定した完全修飾名を与えます。

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

- 移行では、数学の内容と定理の同一性を保つことを目指します。
- モジュールも、人が読めるテキストであり、記事のように著者を持ちます。
- origin metadata は、移行した項目の出自を記録します。出版の話で再び取り上げます。
- 以下の問いは後日の確認用です。次は構造の話に移ります。

後日の確認用の問い:

- 移行中に、どの `environ` の役割は見た目の馴染みやすさを保つべきでしょうか。
- 現在の記事の依存関係で説明が最も難しく、生成する依存レポートの良い試験例になるものは何でしょうか。

## Part 3. Story 2: Structures Without Hidden Merges(隠れたマージのない構造体)

### Frame 3.1 - 痛み

この構造は、馴染みのある簡潔な形式を使っています(exact MML excerpt):

```mizar
definition
  struct (1-sorted) addMagma (# carrier -> set, addF -> BinOp of the carrier
  #);
end;
```

- 1つの宣言に、親との関係、フィールド、セレクタが含まれています。
- 親が複数ある場合は、継承したどのフィールドを共有するかも決まります。
- 加法と乗法のビューは、命名規約に依存します。
- この構文では、格納するデータと zero のような標準値を区別していません。
- Evo は、これらの選択をより明示的に書く方法を提案します。

発表者ノート:

- この短い形式は、構造を非形式的な数学の記述に近く保ちます。
- 出典: 現行 MML `algstr_0.miz` 37-40行。

### Frame 3.2 - なぜ痛いのか [deep dive]

要点:

- ダイアモンド継承では、どの継承セレクタが共有されるかを読者が追う必要がある。Evo はその経路に対する明示的なメンバー対応付けを提案する。
- 移行ツールは「このセレクタは固有データか、証明義務付きの標準値か、継承ビューか」を問えない。構文がそれを語らないからである。
- エラーは原因から遠い場所で、後続記事の型不一致として現れる。

メッセージ:

- MML の規模では、構造体継承はグラフ保守の問題である。そのグラフには明示的で検査可能な辺がふさわしい。

### Frame 3.3 - Evo の答え: field / property / attribute

Evo は field、property、attribute を区別します(specification example):

```mizar
definition
  struct AddLoopStr where
    field carrier -> set;
    field add -> BinOp of carrier;
    property zero -> Element of carrier;
  end;
end;
```

- field は、この例の `carrier` と `add` のようにデータを格納します。
  field はコンストラクタの引数であり、exact instance の等値性を決めます。
- この例の `zero` のような property の値は実装が与えます。宣言だけでは値はありません。
  `means` には存在と一意性の証明が必要で、`equals` は項を与えます。
- attribute は述語型の絞り込みです。cluster の伝播を支え、レイアウトを変えません。

発表者ノート:

- 重なり合う property 実装には `coherence` が必要です。

### Frame 3.4 - Evo の答え: 明示的な継承

Evo は親への対応付けをそれぞれ明示します(specification example):

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

- 親1つにつき、`inherit` 文を1つ書きます。
- ここでは `from` が、子の `add` フィールドを親の `binop` フィールドに対応付けます。
- メンバ型が構文上同一なら、名前が変わっても証明は不要です。
  それ以外の型には、部分型包含の `coherence` 証明が必要です。

### Frame 3.5 - Evo の答え: ダイアモンドが検査可能になる

この子構造には、2つの親構造があります(specification example):

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

図は継承の経路を示しています(sketch):

![合流を検査できるダイヤモンド](figures/diamond_inheritance.pdf)

- 同名・同型のメンバは、異なるルートからでも合流できます。
  メンバ型が異なる場合は、部分型包含の `coherence` 証明が必要です。
- 改名したビューは区別されます。不正な対応付けや証明の欠落は診断されます。

発表者ノート:

- 図は、`AddLoopStr` と `MulLoopStr` から `Magma` への、図示した対応付けを前提とします。
  コードは両者に共通する子 `DoubleLoopStr` を追加します。

### Frame 3.6 - 保存されるもの、問いたいこと

- Mizar と同じく、carrier、selector、`Element of` を使います。
- 隠れた選択を明示する必要がある場合に限り、aggregate は長くなります。
- 中心となる問いは、実際の代数の記事でこの形式が読みやすいかどうかです。
- 複数の継承経路を持つ構造で試したいと考えています。次は registration と cluster です。

後日の確認用の問い:

- `field` / `property` / `attribute` の区別は実際の代数の記事で読みやすいでしょうか。
  それとも、単純な場合にも注釈が多すぎるでしょうか。
- `ALGSTR` や位相の階層など、継承の多い MML の部分でも、`inherit` ごとに親1つでよいでしょうか。
- ダイヤモンドの試験例に最も適した MML の構造は何でしょうか。

## Part 4. Story 3: Automation You Can Audit(監査できる自動化)

### Frame 4.1 - 痛み

この registration は、証明済みの事実を Mizar が再利用できるようにします(exact MML excerpt):

```mizar
registration
  let M be addMagma;
  cluster right_add-cancelable left_add-cancelable -> add-cancelable for
Element
    of M;
  coherence;
end;
```

- registration によって attribute が自動伝播するため、証明を短く保てます。
- この考え方を保ち、自動化の手順をより見やすくしたいと考えています。

発表者ノート:

- 出典: 現行 MML `algstr_0.miz` 104-109行。

### Frame 4.2 - なぜ痛いのか [deep dive]

要点:

- 証明が失敗したとき、「なぜ検査器はこれが Group だと分からないのか」に局所的な答えがない。原因は環境のどこかにある。
- どの registration がどの順で発火したかは見えない。自動化は強力だが、その説明能力は強さに比例して伸びない。
- AI 支援にとっては状況がさらに悪い。cluster の状態を読む代わりに推測するしかない。

メッセージ:

- 自分を説明できない自動化は、たとえ健全でも、ライブラリ規模では保守の負債になる。

### Frame 4.3 - Evo の答え: ラベル付きで追跡可能な registration

Evo はすべての registration 項目にラベルを付けます(specification example):

```mizar
registration
  cluster EmptyImpliesFinite: empty -> finite for set;
  coherence proof ... end;

  cluster FiniteImpliesCountable: finite -> countable for set;
  coherence proof ... end;
end;
```

- ラベルは `by` で引用できます。診断とモジュールインターフェースにも現れます。
- 検証器は、グローバルな cluster graph を import で絞ったビューを使います。
- `explain-attribute` と解決トレースが、成功や失敗を説明します。
  例えば empty から finite、countable に至る手順を示せます。

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

- registration と cluster は言語に残り、証明は短く保たれます。
- cluster の適用時に、その証明を繰り返す必要はありません。
- reduction は等式を使って項を簡約します。
  ローカルなガードの証拠や明示的な等式の引用が必要な場合があります。
- 日々の MML 作業に最も役立つ説明は何かを知りたいと考えています。
  ここから、次の物語である、より強力な証明探索につながります。

後日の確認用の問い:

- cluster の説明では、失敗の説明、発火トレース、環境間の差分レポートのどれが日々の作業に最も役立つでしょうか。
- registration の使い方が最も複雑で、cluster graph の移行ベンチマークにすべき MML の記事群は何でしょうか。

## Part 5. Story 4: Powerful Search, Small Trust(強力な探索、小さな信頼)

### Frame 5.1 - 痛み

- ユーザーは、大きな `by` ステップや hammer 型の探索など、より強力な自動化を求めています。
- MizAR と MPTP は、すでに MML の前提を使って ATP 探索を行っています。
- Evo の目標は、探索が強力になっても証明検査を小さく保つことです。
- 探索結果だけでは、カーネルで検証された証明にはなりません。検査できる証拠が必要です。
- そこで、問いは次のようになります。

スライドテキスト:

```text
How do we get modern proof search
without trusting the searcher?
(探索器を信頼せずに、現代的な証明探索を得るには)
```

発表者ノート:

- これは MizAR、MPTP、hammer 研究を土台としています。
  ここで述べているのは Evo の証拠契約です。
  先行ツールが探索バックエンドを信頼しなければならないと主張しているわけではありません。

### Frame 5.2 - Evo の答え: 推論境界

![推論境界: 意味論、信頼しない探索、信頼する検査](figures/reasoning_boundary.pdf)

- この図は左から右に読めます。
- Mizar 側の段階で、名前解決、型推論、cluster 展開、オーバーロードの選択を行います。
  ATP はこれらを行いません。
- カーネルは渡された論理式と置換を検査し、信頼する SAT 検査を実行します。
  前提の選択や置換の発明はしません。
- それ以前の決定的な discharge にも、再生可能な証拠が必要です。

発表者ノート:

- 開発ポリシーでは `externally_attested` の結果を記録することがあります。
  これはカーネルで検証された証明とは区別されます。

### Frame 5.3 - 式と代入の証拠

![KernelEvidence とカーネルの SAT 検査](figures/certificate_replay.pdf)

- ここでは証拠の内容が分かります。ソースの論理式、置換、provenance、target と goal への binding です。
- カーネルはそれを検査し、インスタンス化した論理式と SAT 節を導出します。
  信頼するプロセス内 Rust SAT checker が UNSAT を返すことを要求します。
- バックエンドの resolution trace、SMT proof object、ログ、終了コードは、信頼される受理証拠ではありません。

発表者ノート:

- ハッシュは証拠を依存文脈に結び付けます。
  カーネルは binder 条件と、goal の反駁極性も検査します。

### Frame 5.4 - 同じ境界が AI を飼い慣らす

AI ツールは、このような引用の修正を支援できます(exact MML excerpt):

```mizar
theorem
  for F being non degenerated ZeroOneStr holds 1.F in NonZero F
proof
  let F be non degenerated ZeroOneStr;
  not 1.F in {0.F} by TARSKI:def 1;
  hence thesis by XBOOLE_0:def 5;
end;
```

- AI ツールは、欠けている、またはより適切な `by` 参照を提案できます。
- これは文の意味を保つ局所的な編集です。
- 検証器は、人間による編集と同じように検査します。
- AI が変更を提案し、検証器とカーネルが受理を判断します。

発表者ノート:

- AI アシスタントの能力が信頼基盤に入ることはありません。
- 出典: 現行 MML `struct_0.miz` 637-643行。

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

- 小さな信頼コアが受理を検査する、de Bruijn の規律を保ちます。
  この設計では、そのコアに SAT checker を含みます。
- 証明テキストは宣言的で読みやすいままです。自動化は論証の維持を支援します。
- 証拠形式について、ご意見をいただければと思います。
- 次は、大規模なライブラリの検査コストを見ます。

後日の確認用の問い:

- 論理式と置換による証拠の方式は、cluster や定義展開を含む Mizar 型の義務に対して説得力があるでしょうか。
- チームが最も監査しやすい証拠形式は何でしょうか。

## Part 6. Story 5: Verification That Scales(スケールする検証)

### Frame 6.1 - 痛み

- MML 全体の検査には何時間もかかります。
- 受理された記事が再利用の単位です。そのため、小さな編集でも期待以上の再検査が必要になり得ます。
- メモリ使用量は記事環境に従い、実際に使うインターフェースの外の内容も含みます。
- これらのコストは、記事を単位とすることから生じます。Evo は、より小さな再利用単位を提案します。

### Frame 6.2 - Evo の答え: フィンガープリントと差分検証

- Evo は fingerprint を使って、以前の結果を再利用できるか判断します。
- 関係するキャッシュキーはすべて一致する必要があります。データが欠けていればキャッシュミスです。
- 公開された文と受理状態が変わらなければ、証明本体を編集しても import 側を再ビルドしません。
- 独立したモジュール、義務、ATP 実行、カーネル検査は並列に実行できます。
  結果は canonical order で公開します。

![変更で何を再検証するかを示す fingerprint graph](figures/fingerprint_graph.pdf)

```text
Cache reuse is never proof authority.
A clean build must always be able to reproduce every acceptance.
(キャッシュ再利用は証明の権威ではない。クリーンビルドは常にすべての受理を再現できねばならない)
```

発表者ノート:

- キャッシュキーは、ソース、依存スライス、パッケージとロックファイル、ツールチェーン、
  スキーマ、ポリシー、registration、義務、証拠、witness を対象とします。

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

- キャッシュと並列処理は検査時間を変えますが、真理を変えてはいけません。
- 人は引き続き、記事の形式で完全なソーステキストをレビューします。
- 増分検証の結果とクリーンビルドを比較する試験が必要です。
- その試験には、実際の MML 保守作業を使いたいと考えています。次は template です。

後日の確認用の問い:

- どのようなクリーンビルドとの等価性試験があれば、現在の MML 保守チームが増分検証を信頼できるでしょうか。
- 改訂や改名など、現在のどの保守作業で増分コストを測るべきでしょうか。

## Part 7. Story 6: Templates For Generic Mathematics(汎用数学のためのテンプレート)

### Frame 7.1 - 痛みその1: scheme は柵の中にいる

この馴染みのある scheme は帰納法を表します(exact MML excerpt):

```mizar
scheme
  NatInd { P[Nat] } : for k being Nat holds P[k]
provided
A1: P[0] and
A2: for k be Nat st P[k] holds P[k + 1]
```

- scheme は、帰納法、分出、置換のような二階のパターンを支えます。
- 従来の `scheme` ブロックは定理スキーマを定義します。パラメータ付きの定義は別の言語形式を使います。
- Evo は、これらに共通の template システムを提案します。

発表者ノート:

- 出典: 現行 MML `nat_1.miz` 90行付近(2026年7月2日確認)。

### Frame 7.2 - 課題その2: 共通のテンプレート機構

- Mizar にはすでに、`Polynom-Ring L` のようなパラメータ付き構成があります。
- 目標は、定義と定理スキーマに共通の template システムを与えることです。
- パラメータの種類ごとに、明示的な規則は引き続き存在します。
- これらの規則が generic mathematics の記述と保守を容易にするか、移行例で確認する必要があります。

出典: [POLYNOM3, definition 10](https://mizar.uwb.edu.pl/version/current/html/polynom3.html)。

### Frame 7.3 - Evo の答え: テンプレート

この template は型パラメータから始まります(specification example):

```mizar
definition
  let T be type;
  struct MagmaStr[T] where
    field carrier -> T;
    field binop -> BinOp of T;
  end;
end;
```

- template は通常の `definition` ブロックです。先頭の `let` がパラメータを束縛します。
- パラメータには、型、値、述語、関手を使えます。
- 述語・関手パラメータは `attr`、`mode`、`struct`、`func`、`pred` 項目内に出現できません。
- 一部の template では、`Module over R` や `Subset of X` のような短い形式を使えます。
  制約付き template には角括弧が必要です。

発表者ノート:

- これらの短い形式は `Module[R]` と `Subset[X]` の表示です。

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

- scheme 型の推論能力は変わりません。
- `of` と `over` は、数学のテキストを読みやすく保ちます。
- 各インスタンス化は検査されます。template は新しい論理を加えません。
- 既存の MML scheme でこの設計を試したいと考えています。次の物語は証明と algorithm をつなぎます。

後日の確認用の問い:

- どの MML scheme を最初の移行対象にすべきでしょうか。
- 角括弧を canonical な同一性の形式とし、`of`/`over` を表示形式とすることでよいでしょうか。
- `func` と `pred` の template では、正規化した宣言引数型から各推論対象の型パラメータを一意に決める必要があります。
  `qua` ビューは推論しません。この規則では明示的な `[T]` 引数が多すぎるでしょうか。

## Part 8. Story 7: Verified Computation With Algorithms(アルゴリズムによる検証済み計算)

### Frame 8.1 - 痛み

- Mizar にはすでに、算術の自動化とプログラム正当性の証明があります。
- Evo は、契約付き algorithm のための言語形式を提案します。
- 目標は、1つの言語とツールチェーンで検証、実行、コード抽出をつなぐことです。
- 実行とコード抽出は、まだ今後の作業です。

スライドテキスト:

```text
Connect mathematical proofs with executable algorithms.
(数学の証明と実行可能なアルゴリズムを結び付ける)
```

MML の例: 算術 requirements の `NUMERALS`、プログラム正当性の `FIB_FUSC`
([Formalized Mathematics の目次](https://mizar.uwb.edu.pl/fm/contents.html)を参照)。

### Frame 8.2 - Evo の答え: 契約付きアルゴリズム

これは契約付きの Euclid のアルゴリズムです(specification example):

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

契約と不変条件は数学の `Gcd` を使うため、循環定義にはなりません。
ループが結果を計算します。`y` に対する `decreasing` 測度が停止性を証明します。

発表者ノート:

- この例は仕様 §20.12 の要約です。

### Frame 8.3 - Evo の答え: 計算による証明

仕様は計算による証明を認めています(specification example):

```mizar
theorem EuclidGcd12_8:  euclid_gcd(12, 8)  = 4  by computation;
theorem EuclidGcd100_75: euclid_gcd(100, 75) = 25 by computation;

theorem Fact10: factorial(10) = 3628800
proof
  thus thesis by computation(steps: 100000);
end;
```

- Mizar Virtual Machine、略して MVM は、ground な等式と述語を評価します。
  現在のパッケージで定義した computable な algorithm を使います。
  non-ground な論理式には古典的な証明が必要です。
- ステップ数、時間、深さの上限は省略可能です。それぞれ既定値は0で、無制限を意味します。
- ほかのパッケージの algorithm 本体は、ここでは実行できません。
  停止性が分かっていれば、その `ensures` 契約を利用できます。

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

- 各呼び出しは `requires` を満たさなければならない。通常の algorithm は部分正当性のみを証明する場合があり、functor に昇格されない。

### Frame 8.5 - 計算は真理を再定義しない [deep dive]

要点:

- 契約、不変条件、停止性測度が生成する検証条件は、通常の定理と同じ ATP+カーネル境界(物語4)を通る。
- `by computation` は制限を任意に設定できる MVM replay を使う。
- コード抽出(実行ターゲットへの出力)は検証済み成果物の厳密に下流であり、受理へ逆流することはない。

メッセージ:

- アルゴリズムはライブラリの表現力を広げる。定理が受理されるとは何か、には触れない。

### Frame 8.6 - 保存されるもの、問いたいこと

- algorithm は `definition` ブロック内に置きます。
- 検証済みの昇格によって、義務を証明した `terminating` algorithm は関手になれます。
- 証明では、現在のパッケージの computable な ground 呼び出しに `by computation` も使えます。
- 部分 algorithm の `ensures` には、その呼び出しが停止する証拠が必要です。
- 一階集合論の基礎は変わりません。
- 次は、この成果の出版と引用に移ります。

後日の確認用の問い:

- プログラミング中心の文化に変えずに、数学者への明確な利点を示せる計算例は何でしょうか。
- 数論、組合せ論、有限構造など、`by computation` が実際の証明をすぐに短くする MML の分野はあるでしょうか。
- コード抽出を行うなら、どの出力先を優先すべきでしょうか。

## Part 9. Story 8: A Library You Can Cite(引用できるライブラリ)

### Frame 9.1 - 痛み

- Formalized Mathematics は研究誌と形式ライブラリを結び付けています。記事を読んで引用できます。
- 雑誌の記事には、数学を説明しやすい順序があります。再利用するライブラリには、依存関係に基づく順序が必要です。
- そのため、ライブラリの構成を変えると、出版済みの記事へのリンクに影響し得ます。
  また、パッケージの再利用には、雑誌の引用に使う同一性がありません。
- Evo は、雑誌の記事とライブラリ項目の記録を分け、リンクすることを提案します。

### Frame 9.2 - Evo の提案: リンクされた記録

![記事とライブラリを結ぶ提案(sketch)](figures/fm_links.pdf)

- このスケッチは、引用、ライブラリ項目、検証成果物、MML の出自を結び付けます。
- 記録間のリンクを示しており、導出ではありません。

### Frame 9.3 - 誰が何を得るか [deep dive]

| 対象 | 得るもの |
|---|---|
| 読者 | 散文が主役のまま。形式ソースはワンクリック先 |
| 保守者 | リファクタリングが出版済み解説を書き換えなくなる |
| 著者 | 凍結した `pub` の記事同一性。ライブラリ内の所在は FQN |
| 計画中の AI ツール | 検索には散文、正確な文脈にはフィンガープリント |

### Frame 9.4 - 保存されるもの、問いたいこと

- Formalized Mathematics は、レビューと文章による説明を持つ雑誌として存続します。
- origin metadata は、移行した項目と MML の引用を結び付けます。移行の対応付けはまだ定義が必要です。
- 引用で読者に見せる識別子を議論できればと思います。
- これで8つの物語は終わりです。次に、設計全体を見ます。

後日の確認用の問い:

- ユーザー向け引用の主な同一性には、記事ラベル、ライブラリ FQN、origin id のどれを使うべきでしょうか。
- 既存の Formalized Mathematics 記事を移行後のモジュールにどう結び付けるべきでしょうか。
  今リンクを更新する、記事の改訂時に更新する、更新しない、のどれでしょうか。

## Part 10. Architecture In One Picture(一枚の絵のアーキテクチャ)

### Frame 10.1 - コア ATP 経路

![責任のグループを示した主要 ATP 経路](figures/pipeline.pdf)

- この図は、提案するパイプラインの ATP 経路を示しています。
- 決定的な discharge 後も未解決の義務だけが ATP に進みます。それ以前の discharge にも証拠が必要です。
- 各境界は、誰が事実を所有し、どの成果物が記録し、変更後に何を再検査するかを定めます。

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

- 8つの物語は、名前解決から出版まで、1つのパイプラインに属します。
- 各段階は、意味と証明検査について、それぞれの責任を保ちます。

後日の確認用の詳細:

| 物語 | パイプラインの段階 |
|---|---|
| 依存関係 | resolver、package manager |
| 構造と自動化 | checker(継承・cluster graph、trace) |
| 探索と信頼 | ATP layer、kernel、論理式・置換の証拠 |
| スケール | artifact、fingerprint、scheduler |
| template | elaborator(検査済みインスタンス化) |
| algorithm | VC generator、MVM |
| 出版 | artifact emitter、doc generation |

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

- 終わる前に、設計と実装を区別しておきます。
- 仕様書は24章と付録からなります。英語が正典で、日本語の対訳もあります。
- 実装済みの部分には、フロントエンド処理、一部の意味解析ケース、ATP 候補生成、カーネルの証拠検査があります。
- ソースから ATP とカーネル検査を通り、成果物を公開するまでの全経路は、まだ作業中です。
  実際の外部証明器を使った試験も引き続き必要です。
- MVM 実行、コード抽出、MML 全体の移行は今後の作業です。

発表者ノート:

- これは2026年9月時点の範囲です。コンポーネントの試験だけでは、パイプライン全体が動くとは言えません。
  `doc/design/todo.md` の Completion Gates を参照してください。

### Frame 11.1 - 移行は研究プログラムである

![ロードマップ](figures/roadmap_timeline.pdf)

- 2026年末に alpha を計画しています。中核フロントエンドの部分集合、import とモジュール解決の試作、
  診断、初期の成果物を対象とします。
- 2027年には、代表的な MML 記事を3〜5本、手作業とスクリプトで翻訳する計画です。すべての不一致を記録します。
- 2027年と2028年には、集合と関係の断片から、代数的構造とその依存関係へ広げる計画です。
- alpha の目標には、MML 全体の検証、最終的な互換層、安定した AI プロトコルは含みません。

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

- この訪問では、移行ベンチマーク記事を優先順に選べればと思います。
- 必要な互換性メタデータについて合意したいと考えています。
- 8つの物語、特に構造と cluster へのコメントも集めたいと考えています。
- 論文の最初の構成案と、反論の共有リストがあれば、次の作業を進めやすくなります。

### Frame 11.5 - 皆さんにお願いしたいこと

- 今日すべての設計上の問いを解決できるとは考えていません。
- 以下の問いは、後で確認していただけるようレジュメに残しています。
- 小さな MML の例を選んでいただければ、提案を実際に試す助けになります。

後日の確認用の問い:

- 小さいけれど、構造的に代表的な MML 記事は何でしょうか。
- 技術的な用途以外にも、Mizar コミュニティにとって大切な idiom は何でしょうか。
- 8つの物語のうち、最も間違っているのはどれでしょうか。その理由は何でしょうか。
- どのような移行結果があれば、Evo が真剣なプロジェクトだとコミュニティに認めてもらえるでしょうか。

## Part 12. Closing(結び)

### Frame 12.1 - 通底する問い、再び

- 最初に挙げた問いで締めくくりたいと思います。

スライドの問い:

```text
What must Mizar Evo preserve so that the Mizar community
still recognizes it as Mizar?
(Mizar コミュニティが Mizar と認め続けるために、何を保存しなければならないか)
```

- 最初に移行すべき小さな MML 記事はどれでしょうか。
- 最も変更が必要なのは、どの提案でしょうか。

### Frame 12.2 - 結び

- Mizar Evo は、大きなライブラリを支えるために必要な部分を更新すべきです。
- Mizar の数学的な同一性を定める部分は保つべきです。
- ご清聴ありがとうございました。ご質問やご意見をいただければと思います。

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
