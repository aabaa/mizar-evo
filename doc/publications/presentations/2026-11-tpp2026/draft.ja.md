# TPP 2026 発表叩き台

> **Status:** working draft  
> **講演時間:** 30分を標準。45分版は同じ物語に具体例を追加する。  
> **位置づけ:** Białystok 2026 セミナーを母体に、TPPでは **MizAR と higher-order hammer の自動証明性能の差**を出発点として、Mizar Evolution の設計思想を一本の物語として説明する。  
> **注意:** 本文は言語仕様ではない。確立した事実、解釈、研究仮説、将来構想を区別する。

---

# 0. 発表の中心ストーリー

TPP 2026 では、Mizar Evolution の機能を列挙するのではなく、次の問いから始める。

```text
MizAR と Sledgehammer の評価では
何をどの条件で計測しているのか？
                   |
                   v
      記述と証明支援を支える仕組みは何か？
      一階の基盤でも構成できるのか？
                   |
                   v
      HOL + ATP と FOL + ATP では
      複雑さの代償を払う場所が違う
                   |
                   v
  FOL は ATP と近いが、人間が直接書くには冗長
                   |
                   v
 Mizar は soft type / mathematical vernacular /
 scheme 等で FOL のまどろっこしさを隠してきた
                   |
                   v
 Mizar Evolution はその思想を template 等で現代化
                   |
                   +--------------------------+
                   |                          |
                   v                          v
       native FOL automation          verified algorithm
       をさらに強化                という第二の柱
                   |                          |
                   +------------+-------------+
                                v
      50年前には無かった software infrastructure
 namespace / package / incremental build / artifact / LSP / AI
                                |
                                v
                         Mizar Evolution
```

発表のメッセージは、

> **Mizar の数学的アイデンティティは残す。  
> しかし、50年前の software engineering と proof infrastructure は残さない。**

である。

---

# 1. 30分の構成

| 時間 | 内容 |
|---:|---|
| 0–3分 | **MizAR vs hammer benchmark** — 観察事実 |
| 3–7分 | **なぜ差が出るのか？** — architecture hypothesis |
| 7–12分 | **HOL+ATP vs FOL+ATP** — 関数を例に説明 |
| 12–16分 | **Mizarの役割** — FOLの煩雑さを言語で隠す |
| 16–20分 | **Template** — FOLのままgeneric mathematics |
| 20–24分 | **Algorithm** — tacticを越える検証可能な計算 |
| 24–27分 | **50年分のmodernization** — namespace / package / build / artifact |
| 27–29分 | **LLM + ATP + Mizar Evolution** |
| 29–30分 | **Roadmap / Closing** |

30分版では「機能紹介」をしない。  
各機能は、上記ストーリーに必要なものだけ見せる。

45分版では、

- 関数のHOL→FOL translation;
- `PermProduct[T]`;
- Euclid GCD;
- structure/view;
- package manifest;
- reasoning boundary;

を具体例として追加する。

---

# 2. Slide 1 — Title

第一候補:

**Mizar Evo の設計指針について**

— 自動証明・数学的記述・検証可能な計算を再接続する —

第二候補:

**Mizar Evolution: FOL-native Formal Mathematics for the AI Era**

---

# 3. Slide 2 — 出発点: MizAR と hammer benchmark

最初に結論を言わず、数字と評価単位を置く。

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

しかし、だからこそ次の問いが立つ。

---

# 4. Slide 3 — 一階の基盤を現代化する設計方針

大きく:

> **A first-order foundation supports abstraction and modern proof tools.**

日本語:

> **一階の基盤でも、数学的な抽象化と現代的な証明支援を組み立てられる。**

ここで設計方針を示す。

- 関数や構造を集合論で表し、型・言語機構で数学的な抽象化を支援する。
- 前提選択・ATP による探索・検査系での証明検査という構成は共通。
- Mizar Evolution は一階の基盤と既存ライブラリを継承し、言語と開発基盤を再整備する。

「仕組みが近い」は数学的な記述と証明支援の構成を指し、論理体系や表現方法の同一性を意味しない。

成功率や再構成の失敗率から一階の性能優位性を主張しない。設計の効果は、記述の利便性・自動証明・証明検査の観点で評価する。

---

# 5. Slide 4 — Sledgehammer: HOL + ATP

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

問い:

> **What if the theorem prover were first-order from the beginning?**

---

# 6. Slide 5 — 関数を例に HOL と FOL の違いを見る

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

# 7. Slide 6 — FOL / Set Theory では逆にどこで払うか

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

# 8. Slide 7 — HOL と FOL は「代償を払う場所」が違う

| | HOL ITP + ATP | FOL ITP + ATP |
|---|---|---|
| 人間の記述 | 高階機能を直接使えて簡潔 | 素朴には冗長 |
| 関数 | primitive higher-order object | set-theoretic first-order object |
| ATP接続 | encodingが必要 | 距離が短い |
| reconstruction | logic gapを戻す | 原理的に単純化可能 |
| 言語側の責務 | HOL abstraction | FOLの煩雑さを隠す abstraction |

中心メッセージ:

> **HOL and FOL do not eliminate complexity. They place it at different layers.**

Mizar Evolution の選択:

> **proof substrate はFOLに保ち、人間向けの複雑さはlanguage layerで吸収する。**

---

# 9. Slide 8 — Mizar はすでにその方向に50年進んできた

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

---

# 10. Slide 9 — Template: FOLのまま generic mathematics

Mizar Evolution の template:

```mizar
definition
  let T be type;

  struct MagmaStr[T] where
    field carrier -> T;
    field binop -> BinOp of T;
  end;
end;
```

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

# 11. Slide 10 — Algorithm は別の柱

ここで話題を分ける。

algorithm は、

> 「FOLでは関数が書きにくいから導入したHOL代替」

ではない。

問題は **algorithmic reasoning / computation** である。

## 歴史的背景

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

# 12. Slide 11 — Algorithm: tactic-like automation + verified computation

例:

```mizar
terminating algorithm euclid_gcd(a, b) -> Nat
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

---

# 13. Slide 12 — Algorithm の長期的な射程

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

# 14. Slide 13 — そして Mizar 自体も50年分老朽化した

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

# 15. Slide 14 — Old Mizar → Mizar Evolution

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

ここでは個別機能を説明しない。

30分版では、

> **「もちろん、言語だけでなく開発基盤も全部現代化する」**

と理解してもらえれば十分。

Białystok 版の詳細スライドを backup にする。

---

# 16. Slide 15 — Mizar Evolution 全体像

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

# 17. Slide 16 — AI時代にこの設計がどう効くか

LLM / human:

- theory construction;
- definition design;
- mathematical semantics;
- proof strategy;
- intermediate lemma invention;
- failure recovery.

ATP:

- first-order proof search;
- saturation / superposition;
- cheap repeated attempts;
- large portfolios.

Mizar Evolution:

- mathematical representation;
- verified library state;
- trusted checking;
- provenance / dependency;
- algorithm verification.

```text
top-level theorem
      |
   cheap ATP
    /     \
solved   failed
  |         |
verify     LLM
  |         |
store <- lemma / decomposition
```

一言:

> **LLM thinks. ATP proves. Mizar Evolution remembers and verifies.**

LLMの進歩はATPを不要にするのではなく、

> **大量に生成される数学を安価に検証するATPの価値をむしろ高める**

可能性がある。

---

# 18. Slide 17 — Roadmap

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

# 19. Slide 18 — Closing

最初の benchmark comparison に戻る。

```text
MizAR:
  top-level theorem
      -> FOL ATP

Sledgehammer:
  HOL goal
      -> translation
      -> FOL ATP
      -> reconstruction
```

最後に:

> **The question is not whether higher-order systems can use first-order ATPs. They clearly can.  
> The question is what becomes possible when automated first-order reasoning is a design principle from the beginning.**

日本語:

> **高階ITPからFOL ATPを使えるか、ではない。  
> 一階自動推論を最初から設計原理にしたITPでは、何が可能になるのか。**

そして Mizar Evolution の答え:

1. **FOL / set theory を proof substrate として残す。**
2. **Mizarの言語設計で、その記述上の不便を隠す。**
3. **template でgeneric mathematicsを拡張する。**
4. **algorithmで検証可能な計算を統合する。**
5. **50年分のsoftware infrastructureを現代的に再構築する。**
6. **LLMとATPを適材適所で組み合わせる。**

締めの一文:

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

45分だから機能を増やすのではなく、30分版の比較と設計思想を具体例で深くする。

---

# 21. 発表での主張レベル

## 事実として述べる

- Mizar / MML / MPTP / MizAR の既存構成;
- Sledgehammer の translation architecture;
- 公開benchmarkの条件と数字;
- Mizar Evolution の公開仕様;
- main branch の実装済み範囲。

## 研究仮説として述べる

- MizAR / hammer の性能差に architecture がどの程度寄与するか;
- native FOL hammer が現代的条件でどこまで伸びるか;
- LLM + cheap ATP cascade の費用対効果。

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

1. **冒頭は MizAR vs higher-order hammer benchmark。**
2. 数字の差を「勝敗」とせず、**なぜ差が出るのか**という研究質問にする。
3. HOL+ATP / FOL+ATP の違いを関数の具体例で説明する。
4. Mizarを「FOLのまどろっこしさを隠す言語」と位置づける。
5. template は、その思想をgeneric mathematicsへ拡張する機構として説明する。
6. algorithm はFOL/HOL比較から一旦分離し、**tactic-like automation + verified computation**として説明する。
7. FOL completeness → ATP tradition / LCF-HOL → tactic tradition は歴史的傾向として述べ、必然性とは言わない。
8. cryptography / quantum は future direction とラベルする。
9. 最後に **50年分のsoftware engineering modernization** を一枚で示す。
10. modernization は namespace / package / lockfile / incremental build / artifact / LSP / AI interface を中心にする。
11. 詳細なBiałystokの機能紹介はbackupへ回す。
12. 最後は roadmap で、現在・2027・2028+を明示する。
13. 実装済みと仕様のみの機能を混同しない。
14. 最終メッセージは:
    **Keep the foundation small. Keep the mathematics readable. Modernize everything else.**
