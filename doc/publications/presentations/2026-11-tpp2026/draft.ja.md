# TPP 2026 発表叩き台

> **Status:** working draft  
> **講演時間:** 30分を標準。45分版は補助スライドを追加して構成する。  
> **位置づけ:** Białystok 2026 セミナーを母体とし、TPPでは「なぜ FOL を核に残すのか」を出発点に Mizar Evolution 全体の設計思想を説明する。  
> **注意:** 本文は言語仕様ではない。確立した事実、設計判断、研究仮説を区別して扱う。

---

## 仮タイトル

第一候補:

**Mizar Evolution: FOLを核にしたAI時代の形式数学**  
— 強力な自動化、Template、Algorithm、そして大規模ライブラリ —

第二候補:

**なぜ今、一階述語論理なのか**  
— Mizar Evolution の設計思想 —

第三候補:

**Mizar Evolution: 数学者のための集合論と自動証明を再接続する**

第一候補を推奨する。TPPでは native hammer 単独ではなく、言語・検証・ライブラリを一体として説明する。

---

# 0. 最初に伝えるべきこと: なぜ FOL を残すのか

Mizar Evolution は、単に「古い Mizar を現代化する」プロジェクトではない。

最初の設計判断は、**証明論理を高階化せず、一階述語論理と集合論を基礎として残す**ことである。

この判断には二つの積極的な理由がある。

## 0.1 LLM に頼らなくても強力な自動化が使える

Vampire、E などの一階自動定理証明器 (ATP) は、長年の研究によって高度に発達している。

- proof search に特化している;
- LLM よりはるかに軽量に大量の探索を行える;
- 同じ問題を多数の strategy / prover で反復できる;
- 結果を独立した検証系で確認できる;
- LLM の能力や挙動に proof correctness を依存させる必要がない。

したがって Mizar Evolution が FOL を native substrate に持つことは、

> **AIを使わなくても強い自動化を持ち、AIを使う場合にもAIを証明探索の唯一の担い手にしない**

という設計につながる。

これは「FOL は単純だから実装しやすい」という消極的理由ではない。

> **FOL を選ぶこと自体が、軽量で成熟した ATP を最大限利用するための architecture 上の選択である。**

## 0.2 数学者に馴染みのある集合論の上に立つ

Mizar は Tarski–Grothendieck set theory と一階述語論理を基礎とする。

多くの通常の数学では、

- 数;
- 集合;
- 写像;
- 代数構造;
- 位相空間;
- 数列;
- 関数空間;

を集合論的対象として理解できる。

したがって Mizar Evolution は、プログラミング言語的な型理論を先に学ばなければ数学を書けない環境ではなく、

> **数学者が通常の数学で使ってきた集合論的な世界観を、そのまま形式化の基盤にする**

ことを目指す。

ここは「数学者は全員集合論を意識している」という主張ではない。

主張はより限定的である。

> **集合論は数学の標準的な共通基盤として長い実績を持ち、Mizar はその上で大規模な数学ライブラリを構築してきた。**

## 0.3 この二つを同時に取りたい

```text
mathematician-friendly set-theoretic foundation
                    +
mature lightweight first-order ATP ecosystem
                    =
          FOL-native formal mathematics
```

TPP 2026 の発表は、ここから始める。

---

# 1. しかし FOL だけでは足りない

FOL を基礎に残すと、当然代償がある。

特に不足するのは次の三つである。

1. **generic mathematics**  
   型・述語・関数にパラメータ化された数学を自然に記述したい。

2. **computation**  
   反復、再帰、有限探索、symbolic procedure を数学の中で扱いたい。

3. **large-scale software engineering**  
   1500篇規模のライブラリを現代的に分割・配布・再検証したい。

Mizar Evolution の設計方針は、

> **これらを解決するために基盤論理を高階化するのではなく、  
> FOL core の外側に明示的な言語機構を置く**

ことである。

```text
        rich mathematical / software surface
                     |
          -------------------------
          |           |           |
       template    algorithm   modules / packages
          |           |           |
          -------- elaboration ----
                     |
              first-order core
                     |
                 native ATP
```

---

# 2. 30分講演の流れ

| 時間 | 内容 |
|---:|---|
| 0–5分 | **なぜ FOL を残すのか** — 集合論と軽量ATP |
| 5–8分 | FOL の制約と設計方針 |
| 8–13分 | **Template** — 高階化せず generic mathematics |
| 13–18分 | **Algorithm** — 論理を拡張せず計算を取り込む |
| 18–22分 | structure / view / registration — 数学的表現力 |
| 22–25分 | namespace / package / incremental build — 大規模化 |
| 25–28分 | **Native hammer + LLM** — 役割分担 |
| 28–30分 | 全体像・研究課題・結論 |

**30分を標準とする。**

45分版では Białystok 版の具体例を追加する。  
本編そのものを長くしすぎない。45分版は「別の話を足す」のではなく、各節の例を深くする。

---

# 3. Slide 1 — Title

**Mizar Evolution: FOLを核にしたAI時代の形式数学**  
— 強力な自動化、Template、Algorithm、そして大規模ライブラリ —

サブタイトル候補:

> A small logical foundation with a rich mathematical environment.

---

# 4. Slide 2 — Why FOL? その1: AIなしでも強い

大きく一文:

> **Strong automation should not require a large language model.**

図:

```text
                 proof obligation
                       |
              +--------+--------+
              |                 |
          Vampire              E
              |                 |
              +--------+--------+
                       |
                  checked result
```

要点:

- ATP は proof search の専用エンジン;
- LLMより軽量;
- 多数の obligation を繰り返し解ける;
- solverを交換・並列化できる;
- correctness は solver 自身を信頼せず再検査できる。

発表で強調:

> **LLM が非常に強くなっても、論理探索まで毎回 LLM にやらせる必要はない。**

---

# 5. Slide 3 — Why FOL? その2: 数学者の集合論

Mizar の基盤:

```text
first-order logic
       +
Tarski–Grothendieck set theory
```

要点:

- 数学対象を集合論的に扱う;
- MML はこの基盤で長期に構築されてきた;
- mathematical vernacular と相性がよい;
- foundation と programming language を必要以上に同一化しない。

ここで Lean / Coq を攻撃しない。

言うべきこと:

> **型理論には型理論の利点がある。Mizar Evolution は別の極として、  
> 集合論とFOLを基盤とする形式数学環境を現代化する。**

---

# 6. Slide 4 — FOL の代償

ここで初めて弱点を認める。

```text
FOL is a good proof substrate,
but not by itself a complete modern mathematical language.
```

不足:

- generic abstraction;
- recursive / iterative computation;
- modular library engineering;
- reusable mathematical views;
- scalable automation metadata.

そして問い:

> **基盤論理を強くせずに、これらをどこまで表現できるか。**

---

# 7. Slide 5 — Template: FOL の弱さを補う

Mizar Evolution の template は generics mechanism。

仕様例:

```mizar
definition
  let T be type;
  struct MagmaStr[T] where
    field carrier -> T;
    field binop -> BinOp of T;
  end;
end;
```

template が扱うもの:

- type parameters;
- predicate parameters;
- functor parameters;
- constrained parameters;
- parameterized definitions;
- theorem schemas;
- algorithms.

重要:

> **template は基盤論理をHOLへ変えない。**

template は meta-level schema であり、instance を決めた後に first-order representation へ落とす。

---

# 8. Slide 6 — Scheme も Template に統合する

現行 Mizar の scheme:

- induction;
- recursion;
- predicate/function schema.

Evo:

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

### 設計上の意味

- 高階的な数学的パターンを surface language では自然に記述;
- kernel の proof logic は FOL のまま;
- template instance を artifact / dependency として追跡できる。

一言:

> **High-level abstraction without changing the foundational logic.**

「higher-order abstraction」という表現は誤解を招く場合があるので、発表ではこちらを推奨。

---

# 9. Slide 7 — Algorithm: FOL の弱さを別の方向から補う

もう一つの不足は computation。

仕様例:

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

algorithm で扱う:

- mutable local state;
- if / loop / recursion;
- requires / ensures;
- invariants;
- termination;
- concrete execution;
- code extraction.

---

# 10. Slide 8 — Algorithm は真理を増やさない

重要な境界:

```text
algorithm
   |
contracts / invariants / termination
   |
verification conditions
   |
FOL
   |
ATP + kernel
```

したがって:

> **algorithm は計算能力を増やすが、基盤論理を拡張しない。**

- partial correctness by default;
- termination を証明した algorithm は mathematical functor として利用可能;
- `by computation` は verified execution;
- code extraction は受理の下流。

この節は「プログラミング言語化」ではなく、

> **proof と computation の境界を明示する**

ためのものとして説明する。

---

# 11. Slide 9 — 数学的表現力は foundation だけで決まらない

Białystok版の structure / view / registration を1枚に圧縮する。

Mizar Evolution は、

- structure;
- field / property;
- mode;
- attribute;
- cluster / registration;
- explicit inheritance / view;
- overload resolution;
- reduction;

を高水準の数学言語として持つ。

例:

```mizar
inherit AddMagma extends Magma where
  field carrier from carrier;
  field add from binop;
end;
```

意味:

- 同じ構造を異なる数学的 view で再利用;
- additive / multiplicative duplication を減らす;
- proof search 前に meaning を確定する;
- elaboration後は論理的 obligation へ落とす。

中心文:

> **A small foundation does not require a poor surface language.**

---

# 12. Slide 10 — 大規模ライブラリには software engineering が要る

MML 規模では theorem prover は単なるcheckerではない。

必要:

- explicit import;
- namespaces;
- packages;
- semantic versioning;
- lock file;
- reproducible build;
- incremental verification;
- dependency fingerprints;
- verified artifacts;
- diagnostics / LSP;
- documentation;
- machine-readable API.

Białystok版の

- Dependencies You Can See;
- Verification That Scales;
- A Library You Can Cite;

をここへ統合する。

一言:

> **Once the library is large, theorem proving is also software engineering.**

---

# 13. Slide 11 — ここまでを一枚で

```text
        Mathematical surface
  --------------------------------
  structures   templates   algorithms
  attributes   views       computation
  registrations
  --------------------------------
                |
           elaboration
                |
          first-order core
                |
    verification conditions
                |
       native ATP portfolio
                |
        checked evidence
                |
      verified artifacts
                |
 packages / library / IDE / AI
```

中心文:

> **Richness above, simplicity below.**

このスライドが発表全体の要約。

---

# 14. Slide 12 — Sledgehammer と正面から比較する

ここは TPP 2026 の重要な対照実験として、Sledgehammer と MizAR を**明示的に比較する**。

比較条件が同一ではないため成功率を単純な勝敗には使わない。しかし、その非対称性自体が重要である。

## Sledgehammer: higher-order ITP から FOL ATP を使う

典型的な経路:

```text
Isabelle/HOL goal
  -> relevance filtering / premise selection
  -> HOL-to-FOL/SMT encoding
  -> external ATP / SMT
  -> proof reconstruction / replay
  -> Isabelle acceptance
```

代表的な評価:

- Judgment Day は **7つの Isabelle theory から生じる proof goals** を対象にした。
- 原研究では 1240 goals に対し、E/SPASS/Vampire の並列実行で 30秒 **47%**。
- 改良後の評価では all provers combined で **63.6%**。
- ただし選択された Isabelle formalizations の goals の約 **40% は “trivial”**、すなわち引数なしの標準 Isabelle tactic で直接解ける。
- それらを除いた **1144 nontrivial goals** では、first-order ATPs が **36.8%**、SMTを加えた all provers が **44.3%**。
- AFP 全体を用いた 2015 年の評価では、各 prover の再構成込み success は概ね **50%前後**、外部 prover を oracle として組み合わせても **60.7%**。

重要なのは「Sledgehammer が弱い」ということではない。

> **Sledgehammer は、HOL から FOL/SMT へ翻訳しながらここまで自動化した。**

その engineering achievement は大きい。

しかし、ここから直ちに

> 「ITP で ATP を使う最適な方法は higher-order logic から翻訳することだ」

とは言えない。

## MizAR: FOL-native library から直接 ATP を使う

MizAR 60 (2023):

- MML 1147 から抽出した 57,897 theorems（unnamed top-level lemmas を含む）。
- ユーザによる premise 指定なしの hammer setting で **58.4%**。
- human-written Mizar proof が使った premises のみに制限して prover を助ける条件では **75%超**。
- strongest single hammer method でも 30秒で約 **40%**。
- portfolio の時間予算は Sledgehammer benchmark と同一ではないため、数字だけの勝敗には使わない。

しかし問題の粒度はむしろ MizAR 側が厳しい。

```text
Sledgehammer:
  theorem
    -> human-written structured proof
       -> local goal
          -> hammer

MizAR:
  top-level theorem
       -> hammer
```

したがって発表では、次を明言する。

> **MizAR が top-level theorem の約60%を自動証明した一方、Sledgehammer の代表的評価は人間が既に分解した proof goals に対するものである。  
> ベンチマーク条件が異なるため因果関係までは主張できないが、FOL-native architecture を現代的条件で正面から比較していないこと自体が研究上の空白である。**

### 発表の強い問い

> **If first-order ATPs are already this effective behind a higher-order translation layer, what happens when the theorem prover is designed around them natively?**

日本語:

> **高階論理から変換して使ってもこれだけ強いATPを、最初からnativeに使うITPを作ったらどうなるのか。**

この問いこそ Mizar Evolution の研究動機の一つである。

### ここで主張できること / できないこと

**主張できる:**

- Sledgehammer の success rate は modern ATP 自体の上限ではない。
- Sledgehammer と MizAR は problem granularity が大きく異なる。
- MizAR は top-level theorem automation が実用的な水準に達し得ることを示した。
- FOL-native ITP を現代の ATP / premise selection / LLM と組み合わせて再評価する価値がある。

**まだ主張できない:**

- MizAR の高い成功率の原因が FOL foundation だけである。
- 同じ theorem corpus を用いれば必ず FOL-native が HOL hammer を上回る。
- 58.4% と 44.3% をそのまま「14.1ポイント差」と解釈できる。

したがって最終的な決着は Mizar Evolution 上での controlled experiment に委ねる。

---

# 15. Slide 13 — Native hammer: FOL を選んだ利益を回収する

高階 hammer:

```text
HOL goal
  -> premise selection
  -> encoding
  -> FOL ATP
  -> reconstruction
```

Mizar Evolution:

```text
Mizar Evo goal
  -> first-order obligation
  -> ATP
  -> checked evidence
```

問い:

> **高階 ITP から ATP を使えるか、ではなく、  
> ATP を native substrate とする ITP はどこまで自動化できるか。**


高階 hammer:

```text
HOL goal
  -> premise selection
  -> encoding
  -> FOL ATP
  -> reconstruction
```

Mizar Evolution:

```text
Mizar Evo goal
  -> first-order obligation
  -> ATP
  -> checked evidence
```

問い:

> **高階 ITP から ATP を使えるか、ではなく、  
> ATP を native substrate とする ITP はどこまで自動化できるか。**

---

# 16. Slide 14 — MizAR は「下限」ではなく設計仮説の先行実証

MizAR の結果は単なる historical curiosity ではない。

Mizar Evolution が目指す

```text
large mathematical library
        +
first-order logical substrate
        +
automatic premise selection
        +
strong external ATP
```

という構成のかなりの部分を、MPTP/MizAR は既に実験している。

ただし Mizar Evolution ではさらに、

- language / verifier / ATP interface を最初から同一 architecture として設計;
- current Vampire / E / learned guidance;
- package / namespace / incremental artifact;
- kernel-checkable evidence;
- LLM による failure recovery / lemma invention;

を統合する。

したがって MizAR 60 の約60%は「完成値」ではなく、

> **古いMizar + 外付けMPTPでもここまで行った**

という出発点として提示する。

---

# 17. Slide 15 — AI時代の役割分担

ここで今回の議論を結論として置く。

LLM / human:

- theory construction;
- definition design;
- mathematical semantics;
- proof strategy;
- lemma invention;
- failure diagnosis.

ATP:

- first-order proof search;
- repeated cheap attempts;
- large portfolios;
- proof obligation discharge.

Mizar Evolution:

- mathematical representation;
- library state;
- trusted verification;
- artifacts and provenance.

```text
             LLM / human
          theory / strategy
                 |
       definitions / lemmas
                 |
           Mizar Evolution
                 |
            cheap ATP first
                 |
        +--------+--------+
        |                 |
      solved            failed
        |                 |
      verify        ask LLM again
        |                 |
       store <--- new lemma
```

中心メッセージ:

> **LLM thinks. ATP proves. Mizar Evolution remembers and verifies.**

ただし「ATPが全証明を必ず解く」という意味ではない。  
ATPで閉じない部分にだけ、LLMまたは人間が高水準の分解を追加する。

---

# 18. Slide 16 — なぜ今この設計なのか

AIが強くなるほど、

- theorem / conjecture / proof idea の生成量は増える;
- verification workload も増える;
- すべてを大型LLMで再探索するのは高価;
- independent symbolic verification の価値が上がる;
- library integration / provenance / dependency management が重要になる。

したがって、

> **LLMの進歩はATPを不要にするのではなく、  
> むしろ軽量で厳密なATPを大量に使う理由を増やす可能性がある。**

ここを TPP 2026 の新しいメッセージとする。

---

# 19. Slide 17 — 研究として何を検証するか

Mizar Evolution は思想だけではなく、測定可能な仮説として評価する。

## RQ1

**Native hammer は MML の top-level theorem をどこまで解けるか。**

## RQ2

FOL-native でも失敗する原因は何か。

- premise selection;
- missing lemma;
- search timeout;
- encoding;
- unsupported semantics.

## RQ3

ATP failure の場合だけ LLM を使う cascade は、どれだけ成功率を上げるか。

測る:

- ATP-only success;
- ATP + oracle premise;
- ATP + LLM decomposition;
- wall time;
- ATP CPU;
- LLM calls / tokens / cost.

## RQ4

template / algorithm / view を使った高水準記述が、FOL core へ安定して elaboration できるか。

---

# 20. Slide 18 — Closing

最終図:

```text
              mathematician / LLM
                     |
        rich mathematical language
       /        |          |       \
 structure   template   algorithm   package
       \        |          |       /
                FOL core
                   |
             native ATP
                   |
           trusted checking
                   |
             large library
```

締め:

> **Mizar Evolution は、FOL の弱さを否定しない。  
> その代わり、論理を小さく保つことで得られる自動証明と検証の強さを活かし、  
> 不足する抽象化・計算・大規模開発能力を明示的な言語機構として積み上げる。**

最後の一文:

> **The question is not how expressive the foundational logic can become,  
> but how much mathematics we can build while keeping the proof substrate small and automatable.**

---

# 21. 45分版で追加するもの

45分になった場合でも本編の順序は変えない。

追加候補:

1. **Structure / view の実例**  
   Białystok版 AddMagma / MulMagma / Magma の例。

2. **Template の実例**  
   `PermProduct[T]` と additive / multiplicative view。

3. **Algorithm の実例**  
   Euclidean GCD と `by computation`。

4. **MizAR / Sledgehammer の評価粒度**  
   top-level lemma と local proof goal の違いを図示。

5. **trusted boundary**  
   ATP結果を直接信頼せず KernelEvidence を検査する現行 architecture。

6. **package / incremental build**  
   manifest / fingerprint graph。

「45分だから新しい機能を増やす」のではなく、**30分版の主張を具体例で深くする**。

---

# 22. 発表で主張しないこと

- FOL が全数学に最適。
- HOL / DTT が不要。
- Sledgehammer は失敗。
- MizAR と Sledgehammer の success rate を直接比較できる。
- FOL-native なら premise selection が不要。
- LLM は不要。
- template は基盤論理に二階量化を追加する。
- algorithm の実行結果を無条件に theorem として採用する。

---

# 23. 事実確認が必要な項目

最終スライド前に一次資料で確認:

- [ ] Mizar の foundation の表現: FOL + Tarski–Grothendieck set theory
- [ ] MizAR 60: 58.4%, >75%, strongest method 40% / 30 sec, portfolio budget
- [ ] MML 1147 / 57,897 theorem の正確な定義
- [ ] Sledgehammer benchmark における "goal" の定義
- [ ] template の current semantics と implementation status
- [ ] algorithm の current semantics と implementation status
- [ ] view / inheritance の current syntax
- [ ] Mizar Evolution main branch の実装状況
- [ ] native global premise selection の実装済み範囲と将来計画

---

# 24. Codexへの作業指示

このファイルから TPP 2026 スライドを生成するとき:

1. **FOLを残す意義を最初の5分で明示する。**
2. FOLの二つの積極的理由を必ず分けて説明する:
   - LLMに依存しない強力・軽量なATP;
   - 数学者に馴染みのある集合論的基盤。
3. その後に「FOLの不足をどう補うか」として template / algorithm を出す。
4. Białystok 2026 資料を具体例の主要ソースとして再利用する。
5. native hammer は全体設計の一部として後半に出す。
6. 機能カタログにしない。すべてを「small core + rich surface」という一つの原則に結び付ける。
7. 30分版を先に完成させる。45分版は backup / deep-dive slide を追加して作る。
8. 現行仕様と実装状況を main branch から再確認し、未実装を実装済みと書かない。
9. MizAR / Sledgehammer の数字は benchmark 条件を併記し、直接順位づけしない。
10. 最終スライドは **LLM thinks / ATP proves / Mizar Evolution remembers and verifies** に接続する。
