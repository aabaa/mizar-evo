# TPP 2026 発表叩き台

> **Status:** working draft  
> **講演時間:** 30分を標準。45分版は同じ物語に具体例を追加する。  
> **位置づけ:** Białystok 2026 セミナーを母体に、TPPでは **現行 Mizar の6つの課題と、新仕様の対応**を出発点として、Mizar Evolution の設計指針を説明する。
> **注意:** 本文は言語仕様ではない。確立した事実、解釈、研究仮説、将来構想を区別する。

---

# 0. 発表の中心ストーリー

§0 で現行 Mizar の課題と新仕様の6つの設計指針を1枚に対応させ、各行を §1–§6 で扱う。

| 課題 | 新仕様の対応 |
|---|---|
| **§1 基盤:** 論理・MML の継承と処理系の刷新の両立 | 一階論理・集合論を維持し、kernel を小さく保つ |
| **§2 記述:** 暗黙の型・登録・オーバーロードの追跡 | 数学的な抽象化を継承し、暗黙の選択を明示 |
| **§3 汎用化:** 定義・定理・scheme の共通化 | template による汎用化機構の統一 |
| **§4 計算:** 証明と実行可能な手続きの接続 | algorithm の契約・不変条件・停止性の検査 |
| **§5 開発基盤:** 依存管理・配布・差分検証・IDE 連携 | module・namespace・package・差分ビルド・LSP |
| **§6 検査・自動化:** 外部探索の結果を再検査可能な根拠として受け渡す | evidence → インスタンス化 → SAT 検査 |

論理基盤と可読な数学的記述は、継承すべき強みである。
課題は、その強みを保ちつつ現代化するための設計上の要請として扱う。

成功率の比較や HOL/FOL の接続経路は補足資料に置く。
一階の方が ATP に有利だという性能上の主張を、再設計の理由にしない。
§6 に ATP の探索・検査・保存・再試行の流れ図を置き、その後で evidence のインスタンス化と SAT 検査を説明する。

> **Mizar の数学的アイデンティティを継承し、処理系と開発基盤を再整備する。**

---

# 1. 30分の構成

| 時間 | 内容 |
|---:|---|
| 0–2分 | §0 課題と設計指針を1枚で対応 |
| 2–4分 | §1 基盤 — 一階論理・集合論・MML の継承 |
| 4–10分 | §2 記述 — 登録の連鎖・構造と継承・定理の再利用 |
| 10–14分 | §3 汎用化 — template の共通化・記法・型引数推論 |
| 14–20分 | §4 計算 — Hoare 論理・停止性・functor 昇格 |
| 20–23分 | §5 開発基盤 — 環境部の役割・順序から import へ |
| 23–28分 | §6 検査・自動化 — Sledgehammer との共通点・反駁と CNF |
| 28–30分 | 章番号なし: 実装状況、ロードマップ、結び |

以下の節は説明素材を保持する。実際の順序・番号は `slides.md` と `slides.ja.md` に従う。

45分版では、

- 関数のHOL→FOL translation;
- `PermProduct[T]`;
- Euclid GCD;
- structure/view;
- package manifest;
- reasoning boundary;

を具体例として追加する。

---

# 2. Title

第一候補:

**Mizar Evo の設計指針について**

— 自動証明・数学的記述・検証可能な計算を再接続する —

第二候補:

**Mizar Evolution: FOL-native Formal Mathematics for the AI Era**

---

# 3. 補足: MizAR と hammer benchmark

質疑で用いる評価条件。導入の設計動機には使わない。

## MizAR 60

MizAR 60 (ITP 2023):

- MML 1147;
- 元コーパスは無名の top-level lemma を含む57,897件。学習・開発・holdout を90:5:5に分割;
- user premise helpなしの hammering modeで holdout 2,896件中1,690件、**58.4%**;
- human-written proof が使った premises を利用できる条件では **75%超**;
- strongest single method は 30秒で約 **40%**;
- full portfolio は最大 420 CPU秒。

強調:

> **MizAR の評価単位は top-level theorem / lemma である。**

## Sledgehammer / hammer benchmark

代表的な評価では、既存の Isabelle development の途中に生じる **proof goals** を対象とする。

- 本文は Seventeen Provers Under the Hammer（ITP 2022）を参照。AFP の50 entry から5,000ゴール;
- MePo、基準512事実。評価集合から事後選択した greedy 16構成、各30 CPU秒で **68.8%**（3,440 / 5,000）;
- 外部 prover の探索成功を数え、Isabelle 内での再構成は評価対象外。高階形式を扱う prover も含む;
- 補足: Magnushammer（2023年公開）では PISA の1,000定理に対し Sledgehammer **38.3%**、Magnushammer **59.5%**。こちらは Isabelle 内での検証済み証明を数える。

重要:

> **58.4% と 68.8% は同一条件の比較ではない。**

time budget、dataset、premise regime、goal granularity、構成選択の方法が違う。
公表年と実験時期も区別する。MizAR の主要実験は2020–2021年で、AFP 評価は2022年1月の Isabelle と2021年12月の AFP を使用。

異なる条件での公表値であり、一階・高階の基盤の性能優位性は導かない。

---

# 4. 一階の基盤を現代化する設計方針

大きく:

> **A first-order foundation supports abstraction and modern proof tools.**

日本語:

> **一階の基盤でも、数学的な抽象化と現代的な証明支援を組み立てられる。**

ここで設計方針を示す。

- 関数や構造を集合論で表し、型・言語機構で数学的な抽象化を支援する。
- 前提選択・ATP による探索・検査系での証明検査という構成は共通。
- Mizar Evolution は一階の基盤と既存ライブラリを継承し、言語と開発基盤を再整備する。
- 証明探索と検査を分離し、受理を小規模な kernel に集約する。§1 で方針を示し、詳細を §6 で説明する。

「仕組みが近い」は数学的な記述と証明支援の構成を指し、論理体系や表現方法の同一性を意味しない。

成功率や再構成の失敗率から一階の性能優位性を主張しない。設計の効果は、記述の利便性・自動証明・証明検査の観点で評価する。

---

# 5. 補足: Sledgehammer の接続経路

典型的な経路:

```text
Isabelle/HOL goal
        |
premise selection
        |
HOL -> FOL / SMT encoding
        |
external ATP / SMT
        |
proof reconstruction / replay
        |
Isabelle theorem
```

説明:

- Sledgehammer は **高階論理からFOL/SMTへ翻訳しながらATPを実用化した**。
- これは大きなengineering achievement。
- ただし、そのための encoding / reconstruction layer が必要。

符号化と再構成は、HOL のゴールを外部の一階 prover に接続する仕組みとして説明する。

---

# 6. 補足: 関数を例に HOL と FOL の表現を見る

## HOL

概念的には:

```text
f : α -> β
x : α

P (f x)
```

関数型とapplicationが論理に組み込まれる。

自然に扱える:

- function as argument;
- function returning function;
- partial application;
- lambda abstraction.

**人間が書く側では非常に便利。**

しかし E / Vampire のような first-order ATP へ渡す場合、この高階性を一階表現へ変換する必要がある。

例えば variable function application:

```text
F X
  |
  v
app(F, X)
```

さらに問題によって、

- lambda lifting / combinator encoding;
- partial application encoding;
- Boolean term encoding;
- polymorphic type encoding;

等が必要になる。

図:

```text
HOL convenience
      |
      | pay later
      v
HOL-to-FOL translation
      |
      v
first-order ATP
```

> **HOLでは表現の便利さを先に得て、ATP接続時に代償を払う。**

---

# 7. FOL / Set Theory では逆にどこで払うか

Mizar では関数そのものを集合論上のobjectとして扱う。

```mizar
let X, Y be set;
let f be Function of X, Y;
let x be Element of X;

... f.x ...
```

論理的には、

- `f` は一階量化できるobject;
- `Function of X,Y` は soft type / predicate;
- application は集合論的関数として解釈される。

そのため proof substrate を高階化する必要はない。

しかし素朴な set-theoretic FOL をそのまま書くと、

- domain;
- range;
- graph;
- functionality;
- membership;
- application relation;

などが表に出て非常に煩雑。

ここに Mizar の存在意義がある。

> **Mizar は FOL / set theory の論理的な単純さを保ちつつ、その記述上の煩雑さを mathematical vernacular と soft typing で隠す。**

```text
Mizar source
  Function of X,Y
  Element of X
  f.x
      |
      | language hides
      | set-theoretic plumbing
      v
set-theoretic FOL
      |
      v
first-order ATP
```

---

# 8. 補足: HOL と FOL の表現と検査経路

| | HOL ITP + ATP | FOL ITP + ATP |
|---|---|---|
| 人間の記述 | 高階機能を直接使えて簡潔 | 素朴には冗長 |
| 関数 | primitive higher-order object | set-theoretic first-order object |
| ATP接続 | HOL-to-FOL / SMT encoding | 一階問題の生成 |
| reconstruction | Isabelle 内での再構成 | 対応可能な Mizar ステップの再検査 |
| 言語側の責務 | HOL abstraction | FOLの煩雑さを隠す abstraction |

中心メッセージ:

> **HOL and FOL do not eliminate complexity. They place it at different layers.**

Mizar Evolution の選択:

> **proof substrate はFOLに保ち、人間向けの複雑さはlanguage layerで吸収する。**

---

# 9. Mizar はすでにその方向に50年進んできた

現行 Mizar が持つもの:

- soft types;
- modes;
- attributes;
- structures;
- registrations / clusters;
- schemes;
- declarative proofs;
- mathematical notation.

これらは単なるsyntactic sugarではない。

> **FOL / set theory を、人間が数学として読み書きできる層へ持ち上げるための language design**

として見る。

Mizar Evolution はこの思想を捨てない。

むしろ、

- ambiguous / implicit な部分を明示化;
- generic mechanism を整理;
- automation trace を可視化;
- modern compiler architecture に載せ直す。


§2.2 は `EmptyImpliesFinite`・`FiniteImpliesCountable` のラベルを付けた登録で、empty → finite → countable の自動的な連鎖と適用経路の記録を示す。
ラベルは必須だが、自動適用のための明示引用は不要。暗黙に得られた型の事実を、規則名と経路で説明できることを中心にする。
Source: `doc/spec/en/17.clusters_and_registrations.md` §17.2, 17.7。

§2.3–2.3b は sample_codes の AddLoopStr・LoopStr・Group・Ring の階層を使う。
field は格納するデータ、property は別実装で一意な値を与える。後付けの inherit で `field add from binop`・`property zero from unit` を対応付ける。Rust trait との類似は型と実装関係の宣言を分離する構成に限る。
ダイアモンドのメンバー起点と継承経路を追跡し、型が同じなら共有を自動検査、異なる型には coherence を要求する。ビューは混同しない。
Group 上の RightUnit の概形を `RightUnit[R qua AddLoopStr]` に適用し、`R.add(x,R.zero)=x` を得る再利用例を示す。選択した環の加法ビューに Group の前提が必要で、乗法側を Group と主張しない。
Source: `doc/spec/en/05.structures.md` §5.2–5.4; `07.modes.md` §7.8.2; `sample_codes.md`。

演算選択は継承ビューの例として扱う (sketch):

```mizar
let R be commutative Ring;
f(R);                  :: ambiguous Magma view
f(R qua AddMagma);      :: use addition
f(R qua MulMagma);      :: use multiplication
```

f は Magma の演算を使う functor の概形。環の加法・乗法という二つの継承パスから、
利用するビューを qua で明示する。必要な定義・継承・登録を前提とする。
qua 自体は既存 Mizar にもある記法であり、例は新仕様の継承パス選択と曖昧性説明を示す。
自動適用した登録は適用経路を記録し、依存する規則を追跡可能にする。

Source: `doc/spec/en/19.overload_resolution.md`, section 19.3.1;
`17.clusters_and_registrations.md`, Traceability。

---

# 10. Template: FOLのまま generic mathematics

本編では、functor の「関数の和」を例にする。
現行 MML の VALUED_1 は点ごとの加法を定義し、数の種類ごとに結果型の再定義・登録を持つ。
新仕様では、値の型と添字集合をパラメータ化し、同じ構成の定義と結果型を共通化する。

構成例 (sketch, 存在・一意性の証明は省略):

```mizar
definition
  let T be type extends non empty AddMagma;
  let I be non empty set;
  let f, g be Function of I, T.carrier;
  func AddDef: Add[T,I](f,g) -> Function of I,T.carrier means
    for i being Element of I holds it.i = T.add(f.i,g.i);
  synonym f +[T,I] g for Add[T,I](f,g);
end;
```

T は加法を持つ型、I は共通の非空添字集合。必要な import・構造・登録を前提とする。
同じ構成の定義族が対象であり、意味の異なる同名演算は区別する。
テンプレート本体を制約の下で一度検査し、各用途へ具体化する。

§3.1a では non empty AddMagma を一意に継承する実数・複素数の加法構造 R・C と必要な登録を仮定する。f,g の宣言型は `Function of I,R.carrier`、u,v は `Function of I,C.carrier` とし、正規化した宣言型から T・I が一意に決まる例として `f + g`・`u + v` と具体的な結果型を表に示す。
本例は共通の非空定義域 I に限定する。VALUED_1:def 1 の異なる定義域の共通部分を扱う構成すべてを置換するとは主張しない。
synonym で中置表記を与える。明示形 `f +[R,I] g`・`u +[C,I] v` と推論の条件は脚注に収める。値域集合だけから構造・演算を選ぶとは主張せず、継承経路が曖昧な場合の qua は明示する（仕様 §11.1.2, §18.2.7, §19.6.2）。

Source: `doc/spec/en/18.templates.md`, sections 18.2.2, 18.7, 18.10.1;
`doc/spec/en/sample_codes.md`, AddMagma;
MML VALUED_1:def 1 と結果型の再定義・登録
(<https://mizar.uwb.edu.pl/version/current/html/valued_1.html>)。

predicate / functor parameterも扱う。

```mizar
definition
  let P be pred(Nat);

  theorem NatInduction[P]:
    P(0) &
    (for n being Nat st P(n) holds P(n+1))
    implies
    for n being Nat holds P(n);
end;
```

重要:

> **template は基盤論理に unrestricted second-order quantification を追加しない。**

- schema / generics mechanism;
- instance を決める;
- first-order obligation へ落とす;
- classical Mizar scheme を統合する。

```text
generic description
      |
template instantiation
      |
first-order obligation
```

つまり、

> **高階化しなくても、必要なgeneric mathematicsをsurface languageで表現する。**

---

# 11. Algorithm は別の柱

ここで話題を分ける。

algorithm は、

> 「FOLでは関数が書きにくいから導入したHOL代替」

ではない。

問題は **algorithmic reasoning / computation** である。

## 歴史的背景（補足）

FOLには完全な推論系があり、

```text
formula
  -> complete calculus
  -> automated search
  -> resolution / superposition / saturation
```

という研究文化が発達した。

一方 LCF/HOL 系では、

```text
interactive proof
  -> proof-state transformation
  -> tactic / tactical
```

という programmable proof construction が大きく発展した。

ここでの主張は歴史的傾向であり、

- FOLではtacticが存在し得ない;
- 完全性からtacticが数学的に不要;

という意味ではない。

Mizar は declarative proof と system-provided automation を中心とし、user-defined tactic language を持たなかった。

Mizar Evolution の algorithm は、この歴史に対する単なる「tactic languageの追加」ではない。

---

# 12. Algorithm: tactic-like automation + verified computation

例:

```mizar
terminating algorithm EuclidGcdDef: euclid_gcd(a, b) -> Nat
  requires a >= 1 & b >= 1
  ensures result = Gcd(a, b)
do
  var x := a;
  var y := b;

  while y <> 0 do
    invariant Gcd(a,b) = Gcd(x,y);
    decreasing y;
    ...
  end;

  return x;
end;
```

algorithm の役割は二重。

§4.1 は Hoare 論理の契約 `{requires} body {ensures}` と、if/else・while・for・代入・return の擬似コードに近い記述を説明する。
§4.2–4.2a は terminating が requires を満たすすべての入力での停止を要求し、検証後に functor として使えることを説明する。既定は部分正当性。定義的な断片には定義方程式も生成するが、可変状態・ループを含む互除法には全域性と契約の公理のみを与える。
定義ラベル EuclidGcdDef と呼び出す名前 euclid_gcd を分け、証明では `by EuclidGcdDef` で検証済み昇格公理を引用する。
ループ注釈は Dafny の invariant・decreases に類似し、Evo では invariant・decreasing と書く。比較の出典は [Dafny Reference Manual](https://dafny.org/latest/DafnyRef/DafnyRef.html#sec-loop-specifications) §8.15.1–8.15.2。採用元の断定には使わない。
Source: `doc/spec/en/20.algorithm_and_verification.md` §20.1.1, 20.2–20.3, 20.7.2–20.7.3, 20.13; `16.theorems_and_proofs.md` §16.5.1。

## A. automation procedure

- symbolic transformation;
- normalization;
- finite search;
- decision procedure;
- proof-development helper;

のような手続きを書ける。

## B. algorithm verification

algorithm 自身が検証対象。

```text
algorithm
   |
requires / ensures
invariant / decreasing
   |
verification conditions
   |
FOL
   |
ATP + kernel
```

したがって、

> **automation procedure itself can be specified and verified.**

§4.3 では互除法の1反復を旧状態 `(x,y)` から新状態 `(y,r)`、`r=x mod y` として追う。
初期成立は事前条件、保存は `Gcd(x,y)=Gcd(y,r)`、停止性は Nat 値の測度と `0<=r<y`、終了時は `Gcd(x,0)=x` から説明する。
必要な GCD・剰余の補題はライブラリから供給する。SAT 単独が算術を扱うという説明にはしない。仕様の説明例と実行結果を区別する。

---

# 13. Algorithm の長期的な射程

単なる theorem prover の convenience feature ではない。

対象候補:

- number-theoretic algorithms;
- combinatorial search;
- symbolic algorithms;
- optimization procedures;
- cryptographic algorithms;
- cryptographic protocols;
- eventually quantum algorithms;
- hybrid classical / quantum algorithms.

同じ枠組みで、

1. procedure を書く;
2. contract を述べる;
3. invariant / termination を与える;
4. VC を生成;
5. ATP で証明;
6. concrete input で実行;
7. 将来は code extraction;

までつなぐ。

暗号・量子は **future direction** と明示する。

> **algorithm は tactic と application algorithm の境界を狭める。**

---

# 14. そして Mizar 自体も50年分老朽化した

ここで論理・証明の話から software infrastructure へ移る。

メッセージ:

> **Mizar の数学的思想を保存することと、1970年代由来の software architecture を保存することは別である。**

Mizar は50年にわたり大規模な数学ライブラリを支えてきた。

一方、現在の利用者が当然期待する仕組みには不足がある。

- namespace;
- explicit module boundary;
- package management;
- semantic versioning;
- lock file;
- reproducible build;
- incremental verification;
- dependency fingerprint;
- machine-readable artifact;
- LSP / IDE integration;
- stable API for AI agents;
- modern diagnostics;
- parallel build / verification.

ここは批判ではなく、

> **50年前には一般的でなかった software engineering を、現在の形式数学環境に導入する**

という位置づけにする。

---

# 15. Old Mizar → Mizar Evolution

一枚の表で highlight のみ。

| 50年の蓄積で見えた課題 | Mizar Evolution |
|---|---|
| article中心の大きな依存境界 | module / explicit import |
| globalな名前管理 | namespace / fully qualified name |
| distribution / version管理が弱い | package / SemVer / lock file |
| full rebuild中心 | incremental build / dependency fingerprint |
| automationが見えにくい | resolution trace / explanation |
| ATPが外付け | first-class ATP pipeline |
| build resultが内部的 | stable verified artifact |
| IDEとの接続が弱い | LSP / structured diagnostics |
| AIとの機械可読I/Oが弱い | agent interface / machine-readable context |
| generic mechanismが分散 | unified template |
| proofとexecutable procedureが分離 | verified algorithm / MVM |

§5.1 では ALGSTR_0 の環境部を短縮して、初見の人にも役割別の依存指定を説明する。
記号・記法・構成子・登録・定理で同じ article が複数回現れ、著者が article と必要な役割を判断する負担を示す。
現行環境部では、特に notations・definitions の article 順が意味を持つことも述べる。Source: Adam Naumowicz, Towards Standardized Mizar Environments, CICM 2017, slide 13 (<https://mizar.uwb.edu.pl/~softadm/imports/slides.pdf>)。
続けて仕様の import 例を示す。旧一覧との一対一の翻訳ではなく、モジュールの公開項目を取り込み、完全修飾名・解決トレースで利用項目を追跡する設計として説明する。
package・バージョン管理・差分検証の詳細は全体像と Backup 7–8 で補足する。

---

# 16. Mizar Evolution 全体像

```text
                mathematician / LLM
                       |
            mathematical vernacular
      /          |           |           \
 structure    template    algorithm     module
 mode         scheme      computation   package
 attribute
      \          |           |           /
                  elaboration
                       |
                    FOL core
                       |
                native ATP layer
                       |
               checked evidence
                       |
                verified artifact
                       |
          large versioned library
             /        |        \
           IDE       AI       publication
```

中心文:

> **Preserve the mathematical layer. Rebuild the infrastructure underneath and around it.**

別案:

> **Rich mathematics above, small first-order proof substrate below, modern infrastructure around it.**

---

# 17. Evidence のインスタンス化と SAT 検査

§6.1 は同じ前提・ゴールを使った Isabelle/HOL の Sledgehammer/Metis と Mizar Evo の by の概形を並べ、外部探索と内側の受理判定を分離する共通点を示す。Metis は内部で再証明し、Mizar Evo は論理式・置換の evidence を検査する。前提選択の範囲は異なる。
Source: [Sledgehammer user guide](https://isabelle.in.tum.de/doc/sledgehammer.pdf), §1, 5.2; `doc/design/architecture/en/08.reasoning_boundary.md`。

§6 では、ATP の探索・kernel 検査・ライブラリへの保存と再利用・再試行の流れ図を先に示す。
図は設計意図であり、LLM の提案・修復も示すが、反復の費用対効果は未評価。
続いて Backup 5 の検査の仕組みを、3段階で示す。

1. **Evidence:** 元の論理式、明示的な代入、来歴、対象ゴールを保持する。
2. **Kernel:** 対象との対応・来歴・代入を検査し、論理式のインスタンスを導出する。
3. **SAT:** インスタンスとゴールの否定を決定的に符号化し、信頼できる SAT 検査器で UNSAT を確認した場合に受理する。

UNSAT は、前提とゴールの否定が同時には成立しないことを意味する。
インスタンス化済みの論理式や SAT 問題を外部入力として信頼せず、kernel が生成する。
バックエンドの証明トレース・ログ・終了コードは診断用であり、受理の根拠にはしない。

§6.3 は F1: `forall x. (P(x) implies Q(x))`、F2: `forall y. (Q(y) implies R(y))`、H: `P(a)` からゴール `R(a)` を反駁する Resolution 木を示す。Q の単一化で `x:=y`、続く P の単一化で `y:=a`、最後は基礎節 R(a) と not R(a) から空節を得る。枝上の置換を合成し、元の F1 に `x:=a`、F2 に `y:=a` を回収する。
ログから置換を抽出する候補生成案として説明する。現行 architecture 10 の独立 instance finder も同じ受理用 evidence を生成できる。保存するのは元の式への出所の結び付け、置換と束縛文脈、対象 VC と反駁の極性であり、外部の導出ステップを kernel に replay させない。
§6.3a は来歴・対象・capture avoidance の検査、基礎インスタンスの生成、命題変数への写像、CNF 化を順に示す。`p=P(a)`・`q=Q(a)`・`r=R(a)` に対し、含意を表す補助変数 s・t を導入する Tseitin 符号化を用い、5変数・10節の全節列を DIMACS 表記で示す。s・t と p が真で、not r が要求されるため充足不能。
現行エンコーダも OR に補助変数を導入する。ここで示すのは solver 入力の具体的な構成例で、実装の実行ダンプや外部 prover の実証結果ではない。instance の不足はゴールが偽であることを意味せず、kernel は置換を探し直さない。

ATP や LLM は探索を支援できるが、受理の境界はこの共通の検査に置く。
Source: `doc/design/architecture/en/08.reasoning_boundary.md`, `10.atp_backend_integration.md`, `15.kernel_certificate_format.md`, `16.substitution_and_binding.md`。

---

# 18. Roadmap

ロードマップは最後の実装メッセージとして簡潔に。

## 2026

- 24章の言語仕様;
- Rust frontend / parser / resolver / checker;
- ATP / kernel architecture;
- template processing;
- alpha end-to-end pipeline を完成へ。

## 2027

- representative MML articles の移行;
- native hammer baseline;
- top-level theorem benchmark;
- package / artifact / incremental verification;
- algorithm verification;
- MML migration tooling.

## 2028+

- MML migration の拡大;
- semantic / learned premise selection;
- LLM-assisted failure recovery;
- large-scale benchmark;
- MVM / extraction;
- advanced verified algorithms.

長期:

```text
AI-generated theory
      |
Mizar Evolution
      |
native ATP verification
      |
verified mathematical library
      |
next theory generation
```

さらに capability が整った段階で、

- cryptographic algorithms / protocols;
- quantum algorithms;

の検証へ展開する。

---

# 19. Closing

冒頭の6つの課題に対する新仕様の対応を振り返る。
一階の基盤と数学的な言語を継承し、template、algorithm、開発基盤、
evidence と SAT 検査を通して処理系を再整備する。

討論では MML の移行、記述の利便性、自動化の評価を扱う。
ベンチマーク比較は、必要に応じて補足資料を参照する。

> **Keep the foundation small. Keep the mathematics readable. Modernize everything else.**

---

# 20. 45分版で追加する具体例

45分にする場合もストーリーは変えない。

追加候補:

1. **HOL→FOL function encoding**
   - `app(F,X)`
   - partial application
   - lambda lifting.

2. **Mizar function representation**
   - `Function of X,Y`
   - `f.x`
   - soft type / set-theoretic object.

3. **Template**
   - `PermProduct[T]`
   - additive / multiplicative view.

4. **Algorithm**
   - Euclidean GCD
   - `by computation`.

5. **Structure / view**
   - AddMagma / MulMagma / Magma.

6. **Modern infrastructure**
   - `mizar.pkg`
   - namespace;
   - fingerprint graph.

7. **Trusted boundary**
   - ATP search;
   - KernelEvidence;
   - checked acceptance.

45分だから機能を増やすのではなく、30分版の課題と設計指針を具体例で深くする。

---

# 21. 発表での主張レベル

## 事実として述べる

- Mizar / MML / MPTP / MizAR の既存構成;
- Sledgehammer の translation architecture;
- 公開benchmarkの条件と数字;
- Mizar Evolution の公開仕様;
- main branch の実装済み範囲。

## 研究仮説として述べる

- native hammer の前提選択と証明探索を、現代的条件でどう評価するか。

これらの実験結果を、現時点の設計動機や既存システムに対する優位性としては扱わない。

## 将来構想として述べる

- cryptographic protocol verification;
- quantum algorithm verification;
- large-scale autonomous theory generation loop.

---

# 22. 最終確認事項

発表前に一次資料で再確認する。

- [ ] MizAR 60 の 58.4%, >75%, 40%/30sec, 420 CPU sec の正確な条件
- [ ] MML 1147 / 57,897 theorem の定義
- [ ] Judgment Day / AFP の benchmark 条件と goal granularity
- [ ] HOL→FOL translation の説明 (`app`, lambda lifting, type encoding)
- [ ] FOL completeness と ATP / resolution tradition の歴史的表現
- [ ] LCF tactic history の表現
- [ ] Mizar に user-defined tactic language がないことの表現
- [ ] template current semantics
- [ ] algorithm current semantics
- [ ] package / namespace / build current specification
- [ ] main branch の実装状況
- [ ] native global premise selection の実装済み範囲

---

# 23. Codexへの作業指示

TPP 2026 スライドを作成するとき:

1. §0 で課題と設計指針を1枚の表に対応させ、番号を §1–§6 の話題に合わせる。
2. §1 基盤、§2 記述、§3 汎用化、§4 計算、§5 開発基盤、§6 検査・自動化の順に説明する。
3. 一階論理・集合論と可読な数学的言語を、継承すべき強みとして扱う。
4. MizAR / Sledgehammer の評価条件と HOL/FOL 比較は補足に置き、性能優位性を設計動機にしない。
5. ATP 活用の流れ図を §6 に残し、続けて evidence → インスタンス化 → SAT 検査を示す。
6. 仕様と実装状況を区別し、将来構想にはラベルを付ける。
7. 最後に実装状況とロードマップを示し、MML 移行・利便性・自動化の評価を討論する。
8. 最終メッセージは:
   **Keep the foundation small. Keep the mathematics readable. Modernize everything else.**
