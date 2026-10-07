# Mizar Evolution: なぜ今、一階述語論理なのか

Status: `slides.md`（英語デッキ原稿）の日本語版。フレーム番号と構成は英語版と同一。

対象: TPP 2026（第22回 Theorem Proving and Provers meeting）、理化学研究所 AIP 東京オフィス、2026年11月16-17日。30分枠。

内容の正典は `draft.ja.md`。発表原稿は `script.ja.md`。図は英語版と共通で、英語のラベルのまま使う（`figures/` と `../2026-09-bialystok/figures/`）。

## 書式上の約束

- 構造ラベル（`Title:`、`Subtitle:`、`Speaker note:`、`Source:`）と、コード例の状態ラベル（`exact MML excerpt`、`specification example`、`sketch`）は生成器が解釈するので英語のまま残す。
- 太字 `**...**` は読み上げる骨格。英語版と同じ箇所を太字にする。
- 主張レベル: 無印は事実（既存システム、公開ベンチマーク、Mizar Evo 仕様、main branch）。「研究仮説」「将来構想」は本文中に明記する。
- 生成: `python3 build_beamer.py` が `tpp2026_ja.tex` を出力し、upLaTeX + dvipdfmx でコンパイルする（README 参照）。

## Part 0. Opening

### Frame 0.1 - Title

Title:

```text
Mizar Evolution: なぜ今、一階述語論理なのか
自動証明・読める数学・検証できる計算を再接続する
```

Subtitle:

```text
TPP 2026、理化学研究所 AIP 東京オフィス、2026年11月
```

Speaker note:

- **Mizar は証明検査系です。一階の集合論の上に、人間が読める数学の言葉で書かれた50年分のライブラリを持っています。**
- **Mizar Evolution、略して Mizar Evo は、その言語と道具を設計し直すプロジェクトです。今日は機能を並べません。自動証明器をめぐる一つの謎から始めて、なぜ論理を一階のままにするのかを説明します。**
- 話す内容にはすべて、事実・研究仮説・将来構想のラベルを付けます。

### Frame 0.2 - 主張の読み方 [deep dive]

| レベル | 意味 | 表示 |
|---|---|---|
| 事実 | 既存システム、公開ベンチマーク、Mizar Evo の仕様と main branch | 無印 |
| 研究仮説 | Mizar Evo が検証するために作られている主張 | 本文中に明記 |
| 将来構想 | 日程も設計もまだ決まっていない目標 | 本文中に明記 |

コード例のラベルは Białystok 資料と同じ: exact MML excerpt、specification example、sketch。

- 仕様と実装は別物です。終盤の一枚でその境界を示します。

## Part 1. Two Hammers, Two Numbers

### Frame 1.1 - 似て見える二つの数字

| | MizAR 60 (ITP 2023) | Sledgehammer on AFP (CICM 2015) |
|---|---|---|
| 見出しの数字 | 58.4% を証明 | 60.7% を証明 |
| 数える単位 | MML 1147 の top-level theorem と lemma（57,897件） | AFP 開発内部の proof goal（6,934件） |
| 前提 | ライブラリ全体から学習的に選択 | 開発内から MePo フィルタで選択 |
| 時間予算 | ポートフォリオ合計 420 CPU 秒 | 各 prover 30 秒、4 prover |
| 結果の扱い | ATP 証明が見つかった（hammering mode） | union を oracle として信頼。一行再構成は各 prover 約50% |

- **二つのハンマー、二つの見出しの数字。ほとんど同じに見えます。**
- **一つのベンチマークとして比べてはいけません。単位も、前提も、時間予算も違います。**

Speaker note:

- 「ハンマー」は goal を自動証明器に送る道具。「前提」は prover が使ってよい事実。
- Source: Jakubův et al. 2023, results 1-3; Blanchette et al. 2015, Section "Proof Automation with Sledgehammer", Figure 13.
- MizAR の 75% は、人または機械がライブラリから前提を選んだ条件。質問用に取っておく。

### Frame 1.2 - それぞれの数字は何を数えているか

![What each benchmark counts](figures/evaluation_units.pdf)

- **MizAR の問い: この定理を丸ごと、著者の助けなしに、ライブラリから機械が証明できるか。**
- **AFP 研究の問い: 現れた場所でこの goal を機械が閉じられるか。多くは人がすでに書いた証明の内側にあります。**
- どちらも正当な問いです。しかし同じ問いではありません。

### Frame 1.3 - 一階 prover へ至る二つの経路

![Two paths from an interactive prover to a first-order ATP](figures/two_paths.pdf)

- **Sledgehammer は高階の goal を一階論理か SMT の問題に翻訳します。そのあと証明を Isabelle の内側に組み立て直します。**
- **MizAR は、最初から一階である問題を prover に渡します。検査系と ATP 問題の距離が短いのです。**

Speaker note:

- どちらも既存システムの説明（Blanchette, Kaliszyk, Paulson, Urban 2016; Jakubův et al. 2023）。オレンジの箱が翻訳と再構成の層。
- MizAR の見出しの数字は ATP 証明を数える。Mizar checker は、推論が検査器の強さの範囲なら、得られた `by` ステップを再検査する。

### Frame 1.4 - 問い

```text
**測り方の違いなのか、設計の帰結なのか**
```

- **研究仮説: 一階のライブラリでは、検査系自身の論理と ATP 問題の距離が短い。高階の処理系はその距離を翻訳と再構成で渡る必要があり、そこに代償がある。**
- 事実: Sledgehammer は、高階の処理系が一階 ATP を非常にうまく使えることを示しました。
- 研究仮説: 「高階プラス翻訳」が ATP を使う最良の設計かどうかは、まだ分かりません。比較対象になる現代的な一階 ITP がほとんど無いからです。
- **Mizar Evo は、現代的な検査系・ライブラリ・prover でこの仮説を試すために作っています。**

## Part 2. Where Complexity Lives

### Frame 2.1 - 高階論理における関数

HOL の文の概形 (sketch, Isabelle/HOL 風の記法):

```hol
f :: 'a => 'b        x :: 'a

P (f x)
```

- **HOL では、関数型と関数適用が論理そのものの一部です。**
- 関数を引数にする、関数を返す関数、部分適用、ラムダ。すべて直接書けます。
- **数学を書く人にとって、これはとても便利です。**
- **しかし E や Vampire は一階の prover です。高階の構造は、prover に見せる前に符号化しなければなりません。**

### Frame 2.2 - 代償は ATP との境界で現れる

符号化の手順の概形 (sketch):

```fol
F X                   ->  app(F, X)
(%x. t) ...           ->  fresh constant + defining axioms   (lambda lifting)
polymorphic types     ->  type guards or type tags
Boolean-valued terms  ->  extra encoding
```

- **変数である関数の適用は、明示的な `app` 記号になります。ラムダは外に持ち上げるか、コンビネータにします。型は guard か tag になります。**
- **prover が成功したら、証明を Isabelle に戻さなければなりません。`metis` の呼び出し、`smt` の replay、生成した Isar テキスト。この工程が再構成（reconstruction）です。**
- これは優れた工学的成果です。同時に、一階の処理系には不要な層でもあります。
- HOL は便利さを先に受け取り、ATP との境界で支払います。

Speaker note:

- Source: Meng and Paulson 2008（高階節から一階節への翻訳）; Blanchette, Böhme, Popescu, Smallbone 2016（型の符号化）; Blanchette, Kaliszyk, Paulson, Urban 2016（サーベイ）; Schurr, Fleury, Desharnais 2021（再構成）。
- Sledgehammer が実際に使う符号化は prover とオプションで変わる。この枚は概形。

### Frame 2.3 - 集合論における関数: Mizar の基礎

関数適用は定義された集合論的関係 (exact MML excerpt):

```mizar
  func f.x -> set means
  :Def2:
  [x,it] in f if x in dom f otherwise it = {};
```

型付き関数は部分関数の上の soft type (exact MML excerpt):

```mizar
definition
  let X,Y;
  mode Function of X,Y is quasi_total PartFunc of X,Y;
end;
```

- **ここでは関数は普通の一階の対象、つまり対の集合です。適用は論理に組み込まれているのではなく、定義されています。**
- **基礎の論理に高階である必要はどこにもありません。**
- 代償: 素朴に集合論を書くと、定義域、グラフ、関数性、所属関係がすべて本文に現れます。

Speaker note:

- Source: 現行 MML `funct_1.miz` 138-140 行（`FUNCT_1:def 2`）、`funct_2.miz` 87-90 行（2026年10月7日確認）。URL: <https://mizar.uwb.edu.pl/version/current/mml/funct_1.miz>, <https://mizar.uwb.edu.pl/version/current/mml/funct_2.miz>。GPL-3.0-or-later / CC-BY-SA-3.0-or-later。
- `quasi_total`（FUNCT_2 の 36-44 行）は、Y が空でなければ定義域が X 全体であることを言う。

### Frame 2.4 - Mizar の50年の答え: 細部を言語に隠す

著者が実際に書くもの (sketch, 現行 Mizar と Mizar Evo で有効):

```mizar
let X, Y be set;
let f be Function of X, Y;
let x be Element of X;
...  f.x  ...
```

- **同じ集合論的な関数です。しかし著者が書くのは `Function of X,Y` と `f.x` であって、対や定義域ではありません。**
- **soft type、mode、attribute、registration、scheme、宣言的証明は、同じものの書きやすい別表記ではありません。一階の集合論を、読める数学へ持ち上げる言語設計です。**
- **Mizar はこの方向に50年歩いてきました。Mizar Evo は歩き続けます。**

### Frame 2.5 - どこで払うか

![Where HOL and FOL systems pay for complexity](figures/where_you_pay.pdf)

```text
**HOL と FOL は複雑さを消してはいない。**
**置く場所が違うだけである。**
```

- **HOL は ATP との境界で払う。FOL はキーボードの前で払い、Mizar の言語がその支払いを引き受ける。**
- **Mizar Evo の選択: 基礎の論理は一階のまま保ち、人間向けの複雑さは言語に隠す。**

### Frame 2.6 - トレードオフを一枚の表で [deep dive]

| | HOL ITP + ATP | FOL ITP + ATP |
|---|---|---|
| 記述 | 高階の機能、短い本文 | 素朴に書くと長い |
| 関数 | 基本的な高階の対象 | 一階の集合論的対象 |
| ATP 接続 | 符号化が必要 | 距離が短い |
| 再構成 | 論理のギャップを戻る必要 | 原理的には単純 |
| 言語が提供すべきもの | HOL の抽象 | 一階の細部を隠す抽象 |

- この表は解釈であって、測定ではありません。測定は Mizar Evo が2027年に出すべきものです。

## Part 3. Modernizing Mizar's Answer

### Frame 3.1 - 論理は保ち、言語を現代化する

```text
**数学の層は保つ。**
**その下と周りの道具を作り直す。**
```

- **Mizar Evo は、一階論理と Tarski-Grothendieck 集合論を基礎の論理として保ちます。**
- soft type、mode、attribute、registration、structure、宣言的証明も保ちます。
- **隠れていた選択を明示し、generic な仕組みを統一し、自動化を追跡できるようにし、全体を現代的なコンパイラ構成に載せます。**
- 続く数枚で「言語を現代化する」の中身を示します。template、algorithm、信頼境界、基盤です。

### Frame 3.2 - Template: 高階論理なしの generic な数学

**template はパラメータ付きの definition ブロック** (specification example):

```mizar
definition
  let T be type;
  struct MagmaStr[T] where
    field carrier -> T;
    field binop -> BinOp of T;
  end;
end;
```

- **パラメータは型、値、述語、関数子を取れます。古典的な Mizar の scheme は、述語パラメータ付きの定理として同じ仕組みに入ります。**
- **template は論理に無制限の二階量化を追加しません。インスタンス化はそれぞれ検査され、一階の証明義務を生みます。**
- generic な数学は表層言語に住み、基礎の論理には住みません。

Speaker note:

- Source: `doc/spec/en/18.templates.md`, sections 18.1-18.2 and 18.8.

### Frame 3.3 - Scheme は普通の template になる [deep dive]

述語パラメータ付き定理としての帰納法 (specification example):

```mizar
definition
  let P be pred(Nat);
  theorem NatInduction[P]:
    P(0) & (for n being Nat st P(n) holds P(n+1))
    implies for n being Nat holds P(n)
  proof ... end;
end;
```

- 述語パラメータは、代入する述語ごとに一つ、一階の定理の族を与えます。
- インスタンス化は明示的です: `defpred` のあとに `by NatInduction[P], Base, Step`。
- 関数子パラメータは schema レベルの記号であって、集合ではありません。これが論理を一階に保ちます。
- 詳細: Białystok 資料 Story 6（`PermProduct[T]`、`qua` によるビュー）、および本資料 Backup 4。

### Frame 3.4 - Algorithm は第二の柱

- **Mizar Evo の algorithm は、高階関数の代わりではありません。別の必要、つまりアルゴリズムについての推論と、検査できる計算に答えるものです。**
- 歴史的な傾向であって必然ではありません: 一階論理には完全な推論系があり、その周りに resolution から superposition、saturation へと自動探索の文化が育ちました。
- LCF と HOL の系統には、プログラムできる証明構成の文化、tactic と tactical が育ちました。
- **Mizar は宣言的証明と、処理系に組み込まれた自動化を使ってきました。ユーザがプログラムできる tactic 言語は持ちませんでした。**
- Mizar Evo の algorithm は、上に載せた tactic 言語ではありません。契約を持つ手続きであり、処理系がそれを検証します。

### Frame 3.5 - Algorithm: 契約、証明、計算

**契約付きのユークリッドの互除法** (specification example):

```mizar
terminating algorithm euclid_gcd(a, b) -> Nat
  requires a >= 1 & b >= 1
  ensures result = Gcd(a, b)
do
  var x := a;  var y := b;
  while y <> 0 do
    invariant x >= 1 & y >= 0 & Gcd(a, b) = Gcd(x, y);  decreasing y;
    const r := x mod y;  x := y;  y := r;
  end;
  return x;
end;
```

- **契約、不変条件、停止性の測度は一階の証明義務になり、定理と同じように検査されます。**
- **役割は二つ。検証された自動化手続きと、`by computation` で実行する検査済みの計算。状態: 仕様化済み。MVM による実行とコード抽出は後の作業です。**

Speaker note:

- Source: `doc/spec/en/20.algorithm_and_verification.md`, section 20.12（外側の `definition` ブロックと `let a, b be Nat;` を省略）。

### Frame 3.6 - Algorithm の将来の対象 [deep dive]

将来構想であり、現在の能力ではない:

- 整数論・組合せ論のアルゴリズム、記号計算、最適化手続き。
- さらに先: 暗号アルゴリズムとプロトコル、量子アルゴリズムと古典・量子ハイブリッド。

すべてに共通する一つの流れ:

1. 手続きを書く。2. 契約を述べる。3. 不変条件と停止性を与える。4. 証明義務を生成する。5. ATP で証明し kernel で検査する。6. 具体的な入力で実行する。7. 将来はコードを抽出する。

- **algorithm は tactic と応用アルゴリズムの間のギャップを小さくします。上の対象は将来構想です。**

### Frame 3.7 - 探索は外、信頼は内

![The reasoning boundary: semantics, untrusted search, trusted checking](../2026-09-bialystok/figures/reasoning_boundary.pdf)

- **一階 ATP は強力な探索器です。信頼できる検査器ではありません。**
- **Mizar 側が名前、型、cluster、オーバーロードを受け持ちます。prover は探索を受け持ちます。kernel は受理を受け持ち、渡された論理式と代入を、小さく信頼できる SAT 検査で確かめます。**
- prover の終了コードは証明ではありません。だから一階の自動化を設計原理にしても、信頼基盤は大きくなりません。

Speaker note:

- Source: `doc/design/architecture/en/08.reasoning_boundary.md`; Białystok 資料 Story 4; 本資料 Backup 5 に evidence の中身。

### Frame 3.8 - そして Mizar 自身も50年経っている

| MML 50年で見えたこと | Mizar Evo |
|---|---|
| article が依存の単位 | 明示的 import を持つモジュール |
| グローバルな名前管理 | namespace、完全修飾名 |
| 配布とバージョン管理が弱い | package、SemVer、lock file |
| フルビルド | 依存指紋による差分ビルド |
| ATP は外付け、自動化が見えにくい | 第一級の ATP パイプライン、解決トレース、kernel evidence |
| IDE 連携と機械可読な入出力が弱い | LSP、構造化診断、エージェント向けインタフェース |

- **Mizar の数学的な考え方を保つことと、1970年代のソフトウェア構成を保つことは、別のことです。**
- **もちろん開発基盤全体も現代化します。詳細は Białystok 資料にあります。ここでは一枚の表で十分です。**

Speaker note:

- 批判ではない。50年前には一般的でなかったソフトウェア工学を、形式数学の環境に持ち込む話。
- 詳細: Białystok 資料 Story 1, 3, 5, 8; 本資料 Backup 6-8。

## Part 4. The AI Era

### Frame 4.1 - 全体像

![Mizar Evo in one picture](figures/layer_stack.pdf)

```text
**上に豊かな数学。下に小さな一階論理。**
**周りに現代的な基盤。**
```

### Frame 4.2 - LLM が考え、ATP が証明し、Mizar Evo が記憶し検証する

![The LLM, ATP, and Mizar Evo division of labor](figures/llm_atp_loop.pdf)

- **LLM: 理論、定義、方針、補題、失敗からの回復。ATP: 安価で反復できる一階の探索。Mizar Evo: 表現、検証済みライブラリ、信頼できる検査、各事実の来歴。**
- 研究仮説: 機械が大量の数学を生成する時代には、ATP による安価な検査の価値が上がる。このループの費用と効果はまだ測られていません。
- 現在の仕様: ATP が受け取るのは引用された前提と局所仮定だけです。ライブラリ全体を使うハンマーは2027年の研究です。

Speaker note:

- Source: `doc/spec/en/21.source_code_annotation_and_atp.md`, section 21.7.2, item 4（グローバルライブラリからの自動前提選択は無し）; `doc/design/architecture/en/21.ai_agent_interface.md`（編集クラス）。

### Frame 4.3 - プロジェクトの現在地（2026年10月）

- **仕様: 24章と付録。英語が正典。**
- **main branch に実装済み: Rust フロントエンド（字句解析、構文解析、構文木）。alpha コーパス上の名前解決と型検査。証明義務の生成と決定的な discharge。ATP 問題の符号化と候補 evidence。SAT に基づく kernel の evidence 検査。キャッシュ、指紋、ビルドスケジューリングの各マイルストーン。**
- **進行中: ソースから検証済み成果物までの end-to-end 統合。LSP サーバ。ドキュメント生成。**
- **後の作業: MVM の実行、コード抽出、ライブラリ全体の前提選択、MML の移行。**
- この講演では、外部 prover を使った end-to-end の結果は主張しません。

Speaker note:

- Source: `doc/design/todo.md`, Crate Status（2026年10月7日時点）。講演前に再確認。

## Part 5. Roadmap And Closing

### Frame 5.1 - ロードマップ

![Roadmap](figures/roadmap_tpp.pdf)

- **2026年: 仕様、kernel までの Rust パイプライン、template 処理、alpha の end-to-end 実行を仕上げる。**
- **2027年: 代表的な MML article を移行し、native hammer のベースラインを作り、MizAR と同じ単位、top-level theorem でベンチマークする。**
- 2028年以降: 移行の拡大、学習ベースの前提選択、LLM による失敗回復、MVM と抽出。暗号と量子は将来構想のまま。

### Frame 5.2 - 二つの経路に戻る

```text
**問いは、高階の処理系が一階 ATP を使えるか、ではない。**
**明らかに使える。**
**問いは、一階の自動推論を最初から設計原理にしたとき、**
**何が可能になるか、である。**
```

Mizar Evo の答え、六つ:

1. 一階論理と集合論を基礎の論理として残す。
2. Mizar の言語設計で一階の細部を隠す。
3. template で generic な数学を広げる。
4. algorithm で検査できる計算を加える。
5. 50年分のソフトウェア基盤を作り直す。
6. LLM と ATP を、それぞれが強い場所で組み合わせる。

### Frame 5.3 - おわりに

```text
**Keep the foundation small.**
**Keep the mathematics readable.**
**Modernize everything else.**
```

- **基盤は小さく。数学は読めるように。それ以外はすべて現代化する。**
- **ありがとうございました。特に、2027年のベンチマークを「同じものを同じ単位で比べる」設計にする方法について、ご意見をいただければ幸いです。**

## Backup 1. MizAR 60 の詳細

Source: Jakubův, Chvalovský, Goertzel, Kaliszyk, Olšák, Piotrowski, Schulz, Suda, Urban, MizAR 60 for Mizar 50, ITP 2023.

- データ: MPTP で出力した MML 1147、無名の top-level lemma を含む 57,897 件の定理。MizAR 40 の評価と同じ版なので比較できる。
- top-level lemma の 58.4% を large-theory（hammering）mode で、ユーザの助けなしに、CPU 時間 420 秒に制限したポートフォリオで証明（MizAR 40 は約 40.6%）。
- 人または機械がライブラリから前提を選べる条件では 75% 超（MizAR 40 は 56%）。
- 最強の単一手法: hammering mode で 30 秒 40%。人間の前提ありで 120 秒 60%。
- 転移: 最強手法は MML 1382 の新規 242 article、13,370 定理でも動く。
- 手法: ENIGMA と Deepire で誘導した E と Vampire、学習ベースの前提選択、数百万の ATP 証明で学習するループ。

## Backup 2. Sledgehammer 評価の詳細

| Prover | One-line / + Isar / + Oracle（6,934 goal に対する %） |
|---|---|
| E | 49.7 / 51.4 / 52.5 |
| SPASS | 49.4 / 50.5 / 52.0 |
| Vampire | 49.5 / 51.0 / 51.8 |
| Z3 | 49.6 / 50.0 / 53.7 |

- 設定: ランダムに選んだ AFP の 128 theory、各 100 goal まで。Isabelle2014。MePo フィルタ。各 prover 30 秒をスライスに分割。再構成は 2 秒以内に成功する必要。
- 組み合わせて oracle として信頼すると、goal の 60.7% を証明。
- Judgement Day（2010）: 7 theory の 1,240 subgoal、E・SPASS・Vampire を 30 秒で 46%。2015年の予備評価では 6 prover で 75%。
- theory ごとの成功率は 10% から 100% までばらつく。Sledgehammer は Judgement Day に合わせて調整されてきた。

Speaker note:

- Source: Blanchette, Haslbeck, Matichuk, Nipkow, Mining the Archive of Formal Proofs, CICM 2015, Section "Proof Automation with Sledgehammer", Figure 13; Böhme and Nipkow, IJCAR 2010.

## Backup 3. HOL から FOL への符号化と再構成

- 関数適用: 変数である関数を引数に適用すると `app(F, X)` になる。定数の適用はカリー化のままか平坦化。
- ラムダ抽象: lambda lifting は定義式付きの新しい定数を導入する。コンビネータ変換が代替。
- 型: 多相な HOL の型は guard、tag、または単相化で符号化する。選択は健全性、完全性、prover の性能に影響する。
- 真偽値: 論理式の中の真偽値を取る項には別の符号化が必要。
- 再構成: 使われた補題による `metis`、SMT 証明の `smt` replay、生成した Isar テキスト。評価では再構成の失敗を別に数える。

Source: Meng and Paulson 2008; Blanchette, Böhme, Popescu, Smallbone 2016; Blanchette, Kaliszyk, Paulson, Urban 2016; Schurr, Fleury, Desharnais 2021.

## Backup 4. Template のインスタンス化: 一つの証明、多くのビュー

有界な型パラメータと generic な定理 (specification example, 証明は省略):

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

インスタンス化 (specification example, 必要な registration がある前提):

```mizar
PermProduct[commutative associative unital AddMagma]
PermProduct[commutative associative unital MulMagma]
let R be commutative Ring;  PermProduct[R qua AddMagma]  :: additive view
```

- 環は二つの経路で Magma に届く。`qua` がビューを選び、ビューが記法も決める。

Speaker note:

- Source: `doc/spec/en/18.templates.md`, section 18.2.2.

## Backup 5. Kernel evidence と信頼できる SAT 検査

![KernelEvidence and the kernel's SAT check](../2026-09-bialystok/figures/certificate_replay.pdf)

- evidence は元の論理式、明示的な代入、来歴、対象と goal の束縛を持つ。kernel はそれらを検査し、決定的な SAT 問題を作り、信頼できるプロセス内 SAT 検査器に UNSAT を要求する。
- バックエンドの証明トレース、SMT の証明オブジェクト、ログ、終了コードは診断用のみ。

Speaker note:

- Source: `doc/design/architecture/en/08.reasoning_boundary.md`, `15.kernel_certificate_format.md`.

## Backup 6. ATP の中心経路

![The core ATP path, with responsibility groups](../2026-09-bialystok/figures/pipeline.pdf)

- 決定的な discharge のあとも開いている義務だけが ATP に行く。前段の discharge にも再生できる evidence が要る。
- 各境界は、誰が事実を所有し、どの成果物が記録し、変更後に何を再検査するかを定める。

Speaker note:

- Source: `doc/design/architecture/en/00.pipeline_overview.md`; Białystok 資料 Part 10。

## Backup 7. 差分検証

![The fingerprint graph: what a change re-verifies](../2026-09-bialystok/figures/fingerprint_graph.pdf)

- 証明本体の編集は、公開された主張と受理状態が変わらなければ、importer を再ビルドしない。インタフェースの変更は依存コーンを再検証する。
- 関係するキャッシュキーはすべて一致する必要がある。データが無ければキャッシュミス。キャッシュ再利用は証明の権威ではない。クリーンビルドはすべての受理を再現しなければならない。

Speaker note:

- Source: `doc/design/architecture/en/11.artifact_and_incremental_build.md`, `18.dependency_fingerprint.md`; Białystok 資料 Story 5。

## Backup 8. Package manifest

Mizar Evo (specification example):

```toml
[package]
name    = "algebra"
version = "2.3.1"
edition = "2025"

[dependencies]
mml_core = "^1.0"
topology = { version = "^0.9", features = ["metric"] }
```

- 再現可能なビルドには、固定したソース、lockfile、ツールチェーン、検査器の設定が要る。決定的な ATP evidence も含む。
- バージョン付きの再利用が article 集合間の手作業コピーを置き換える。集約モジュールは一つの import で一分野をまとめて公開できる。

Source: `doc/spec/en/23.package_management_and_build_system.md`; Białystok 資料 Story 1。

## Backup 9. 45分版での追加

物語は変えない。次の順で具体例を深める:

1. 一つの goal で HOL から FOL への関数の符号化を追う（`app`、部分適用、lambda lifting）。
2. Mizar の関数表現: `Function of X,Y`、`f.x`、集合論的対象としての soft type。
3. Template: 加法と乗法のビューを持つ `PermProduct[T]`（Backup 4）。
4. Algorithm: ユークリッドの互除法、`by computation`、関数子への昇格。
5. Structure とビュー: `AddMagma`、`MulMagma`、`Magma`（Białystok Story 2）。
6. 現代的な基盤: `mizar.pkg`、namespace、指紋グラフ（Backup 7-8）。
7. 信頼境界: ATP 探索、KernelEvidence、検査された受理（Backup 5）。

## Backup 10. 出典と帰属

この講演で使う MML の正確な引用:

| 目的 | 出典 | 行 | 使用箇所 |
|---|---|---:|---|
| 関数適用 `f.x` | `funct_1.miz` | 138-140 | Frame 2.3 |
| soft type としての `Function of X,Y` | `funct_2.miz` | 87-90 | Frame 2.3 |

Source URLs:

- `FUNCT_1`: <https://mizar.uwb.edu.pl/version/current/mml/funct_1.miz>
- `FUNCT_2`: <https://mizar.uwb.edu.pl/version/current/mml/funct_2.miz>

帰属についての注記:

- MML のテキストは GPL-3.0-or-later / CC-BY-SA-3.0-or-later。article 名、URL、行番号を発表者ノートに残す。
- ベンチマークの数字は Backup 1-2 と `references.bib` を参照。最終版の前に書誌情報を出版社で確認する。
