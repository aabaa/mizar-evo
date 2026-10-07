# TPP 2026 発表叩き台

> **Status:** working draft  
> **目的:** TPP 2026 で Mizar Evolution の研究上の狙いを明示するための論旨整理。  
> **注意:** 本文は言語仕様ではない。事実・解釈・研究仮説を区別して扱う。

## 仮タイトル

**AI時代の定理証明支援系を再考する  
— FOL-native ATP と Mizar Evolution —**

別案:

**LLM Thinks, ATP Proves  
— Mizar Evolution における native hammer の設計思想 —**

---

## 0. 発表の一文

Mizar Evolution の中心的な研究仮説は、単に「Mizarを現代化する」ことではない。

> **LLM や人間は理論構築・証明戦略・補題設計を担い、  
> 大量の具体的な証明探索は、はるかに軽量な一階自動定理証明器 (ATP) に委ねる。  
> そのために、FOL ATP を後付けの補助機構ではなく native hammer として持つ
> 大規模形式数学環境を設計する。**

この仮説を、Mizar/MPTP/MizAR の20年以上の経験と、現在の Mizar Evolution
アーキテクチャを踏まえて検討する。

---

## 1. 問題意識: ATP は本当に ITP で十分に使われてきたか

現在の主要な ITP は、依存型理論や高階論理を基礎とするものが多い。一方、
Vampire や E に代表される強力な ATP は、長く一階論理を主要な活動領域として
発展してきた。

Isabelle/Sledgehammer は、この隔たりを越えて ATP を実用的に利用することに
成功した代表例である。典型的には、

```text
Isabelle/HOL goal
    -> premise selection
    -> HOL-to-ATP translation
    -> external ATP/SMT
    -> proof reconstruction / replay
    -> Isabelle acceptance
```

という経路を取る。

ここで問いたいのは、

> **「高階 ITP から FOL ATP を利用できるか」ではない。  
> それが可能であることは Sledgehammer が既に示している。  
> 問題は、これが ATP の能力を利用するための最適なアーキテクチャなのか、である。**

高階表現を必要とする数学は当然存在する。しかし、FOL で自然に扱える大規模な
数学領域について、最初から ATP-friendly な論理基盤を採用した ITP が十分に
比較検証されてきたとは言い難い。

### 発表上の注意

Sledgehammer を「失敗」と表現しない。むしろ、

- 高階環境から FOL ATP を利用する技術として大きな成果である;
- だからこそ、その成功を「FOL ATP 自体の限界」と混同してはいけない;
- FOL-native な対照系がほぼ存在しなかったため、アーキテクチャ比較が不十分だった;

という順に述べる。

---

## 2. MizAR が与える重要な実証的根拠

Mizar は集合論・一階述語論理を基礎とし、MPTP は MML を一階論理の大規模
ATP 問題へ変換する研究基盤を築いた。

2023年の **MizAR 60 for Mizar 50** は、MML 1147 から抽出された
57,897 個の theorem（unnamed top-level lemmas を含む）を対象に、大規模な
AI/ATP 実験を行った [1]。

論文の主要結果:

- **58.4%** の Mizar top-level lemmas を、ユーザの premise 指定なしの
  large-theory / hammering mode で自動証明;
- 人間の証明で使われた premise、または機械選択された premise を利用できる条件では
  **75%超**;
- strongest single method は hammering mode で **30秒以内に40%**;
- 58.4% は大規模 portfolio（最大420 CPU秒）による値。

重要なのは、これは **top-level lemma** を単位とした評価であることである。

### Sledgehammer の成功率との数字の単純比較を避ける

Sledgehammer の代表的な大規模評価では、Archive of Formal Proofs 等の
既存形式証明の途中に現れる **proof goals** を対象とするものがある [2,3]。
すなわち、人間が既に proof structure を与えた後の局所 goal を含む。

したがって、

```text
MizAR:       top-level theorem / lemma
Sledgehammer: goals arising inside existing structured proofs
```

という評価粒度の違いがあり、58% と 50--60% といった数字を直接比較してはならない。

むしろ発表で強調すべき点は:

> **MizAR は、FOL-native な大規模数学ライブラリに対して、top-level theorem を
> 相当な割合で完全自動化できることを既に示している。**

これは Mizar Evolution の設計仮説に対する強い先行根拠である。

---

## 3. それでも MizAR は完成形ではない

MizAR/MPTP は非常に重要だが、そのまま「現代的な FOL ITP + native hammer」
だったわけではない。

少なくとも以下は Mizar Evolution で再設計する余地がある。

- 言語・処理系・ATP が最初から同一アーキテクチャとして設計されていない;
- 現代的な package / namespace / incremental build / machine-readable API がない;
- ATP 問題生成、backend 実行、検証証拠の管理を、実装上の明確な境界として
  再設計できる;
- 現代の premise selection、neural guidance、LLM による高水準推論を統合できる;
- AI が生成した理論を、そのまま検証済みライブラリへ蓄積する研究環境として設計できる。

したがって Mizar Evolution は、

> **MPTP/MizAR で20年間実験されてきた「Mizar + FOL ATP」を、
> 後付け bridge ではなく theorem prover architecture の中心原理として設計し直す**

プロジェクトと位置づけられる。

---

## 4. AI 時代の役割分担

### 4.1 LLM に何をさせるか

LLM の強み:

- 問題の意味を理解する;
- 関連する数学理論を想起する;
- 定義を設計する;
- 証明方針を選択する;
- 中間補題を発明する;
- 失敗した方針を変更する;
- 自然言語数学と形式表現を接続する。

これは大域的・意味的・創造的な探索である。

### 4.2 ATP に何をさせるか

ATP の強み:

- 与えられた premise と goal に対し大量の記号的探索を行う;
- resolution / superposition 等を高速に繰り返す;
- LLM よりはるかに軽量な計算資源で多数の obligation を処理する;
- 同じ problem を複数 backend / strategy で独立に探索できる。

局所的な論理探索を毎回 LLM に文章として生成させる必要はない。

### 4.3 基本ループ

```text
          semantic / global reasoning
                 LLM
                  |
       theorem / definitions / lemmas
                  v
        Mizar Evolution library
                  |
          native FOL obligations
                  v
          Vampire / E / ...
                  |
        kernel-checkable evidence
                  v
           trusted verifier
```

より運用的には:

```text
top-level theorem
    |
    +--> cheap ATP first
            |
            +--> solved ------> verify / store
            |
            +--> failed
                    |
                    v
              ask expensive LLM
                    |
          propose decomposition / lemma
                    |
                    v
                 ATP again
```

つまり **LLM は常時呼ばなくてよい**。

> **LLM thinks. ATP proves. Mizar Evolution remembers and verifies.**

これを発表の中心メッセージ候補とする。

---

## 5. 「人間/LLM が証明を書く」の意味を変える

FOL-native hammer が十分に強ければ、人間や LLM が生成すべきものは
低水準 proof step ではなく、数学的な proof architecture になる。

例:

```text
Main theorem
  |
  +-- Lemma A
  +-- Lemma B
  +-- Lemma C
  |
  +-- Main follows
```

A/B/C の内部が数万 inference steps でも、人間や LLM にそれを書かせる必要はない。

成功すれば、形式証明の主な人間可読層は通常の数学に近い:

- 定義;
- 中間命題;
- 依存関係;
- 主要な構成;
- 失敗時に追加された補題。

これは Mizar の declarative proof tradition と自然に接続する。

---

## 6. Mizar Evolution の現在のアーキテクチャとの対応

現行設計では、概略:

```text
Source
 -> SurfaceAst
 -> Resolved / Typed representations
 -> CoreIr
 -> VcIr
 -> AtpProblem
 -> external ATP
 -> KernelEvidence
 -> trusted kernel check
 -> VerifiedArtifact
```

となっている。

参照:

- `doc/design/architecture/en/00.pipeline_overview.md`
- `doc/design/architecture/en/08.reasoning_boundary.md`
- `doc/design/architecture/en/09.atp_interface_protocol.md`
- `doc/design/architecture/en/10.atp_backend_integration.md`

ここで重要なのは:

1. **ATP は trusted computing base に入れない。**
2. proof search と acceptance を分離する。
3. ATP が成功したという報告自体を信用せず、kernel-checkable evidence を検査する。
4. surface language の高水準機能は ATP backend に押し付けず、
   elaboration / VC generation の前段で解決する。
5. 最終 proof obligation は可能な限り ATP-friendly にする。

この構造に global premise selection / native hammer を追加すれば、
今回の研究仮説を実験可能にできる。

---

## 7. 語彙的意味について

自然言語として意味のある identifier は、proof correctness の根拠にはしない。

`point` を `pencil` に rename しても、形式意味論が同じなら theorem の真偽は
変わってはならない。

一方で、

> **names do not determine truth, but names may guide search**

という立場は取り得る。

将来的な premise selection では、

- formal dependency;
- symbol occurrence;
- definition dependency;
- natural-language identifier;
- embedding / LLM prior;

を search guidance として組み合わせることができる。

ただし TPP 2026 では、これは中心主張ではなく将来方向として扱う。

---

## 8. 発表で立てる研究質問

### RQ1: Native hammer は top-level theorem をどこまで解けるか

Mizar Evolution へ移行した MML subset で、

- theorem statement のみ;
- automatic premise selection;
- fixed ATP portfolio;

から、top-level theorem success rate を測る。

### RQ2: 失敗の主因は何か

失敗を少なくとも以下へ分解する。

- premise selection failure;
- ATP search failure;
- missing intermediate lemma;
- unsupported / expensive encoding;
- resource limit;
- implementation limitation.

### RQ3: LLM を failure recovery に限定すると何が起きるか

直接 ATP で失敗した theorem に対してのみ LLM を呼び、

- intermediate lemma generation;
- premise suggestion;
- definition unfolding/folding suggestion;

を行う。

評価:

- additional solved rate;
- LLM calls per theorem;
- API/GPU cost;
- ATP CPU cost;
- wall-clock time.

### RQ4: 人間が proof decomposition を与えなくても実用になるか

主指標を local goal success ではなく、

> **top-level theorem success rate**

とする。

これが Mizar Evolution のユーザ体験を決める。

---

## 9. 最初の実験案

MML から比較的小さな依存閉包を選び、各 top-level theorem について次を測る。

### A. Native hammer baseline

- automatic premise selection;
- Vampire / E;
- 固定 timeout;
- LLM 不使用。

### B. Oracle / human-premise baseline

- 元 Mizar proof で使われた premise を利用できる条件;
- ATP search 自体の能力を見る。

### C. LLM-assisted recovery

A で失敗した theorem のみ、

1. LLM が必要な補題を最大 N 個提案;
2. 各補題を ATP で検証;
3. 証明済み補題を context に追加;
4. 主定理を ATP で再試行。

### 測定値

- top-level theorem solved rate;
- solved by ATP alone;
- solved after LLM decomposition;
- ATP CPU time;
- LLM token / API cost;
- generated lemmas per solved theorem;
- accepted evidence size;
- premise count;
- failure categories.

**比較対象の時間予算・problem granularity が異なる場合、Sledgehammer や MizAR の
公開 success rate と単純な順位比較はしない。**

---

## 10. 発表で主張しないこと

### 「FOL がすべての数学に最適」

主張しない。高階論理・依存型理論が自然な領域は存在する。

今回の問いは:

> FOL で自然に扱える大規模形式数学について、FOL-native architecture の潜在能力は
> 十分に調べられたか。

である。

### 「Sledgehammer は失敗」

主張しない。

Sledgehammer は異なる論理基盤をまたいで強力な ATP を利用する優れた技術である。
ただし、その成功を native FOL architecture の不要性の根拠にはできない。

### 「MizAR 58.4% と Sledgehammer 54% は直接比較できる」

主張しない。dataset、goal granularity、time budget、premise regime が異なる。

### 「FOL-native にすれば premise selection 問題が消える」

消えない。むしろ大規模ライブラリでは中心課題の一つであり続ける。

### 「LLM が不要」

逆。LLM の役割を、ATP が苦手な高水準 reasoning に集中させる。

---

## 11. TPP 2026 スライド案

### Slide 1 — Title

AI時代の定理証明支援系を再考する  
— FOL-native ATP と Mizar Evolution —

### Slide 2 — The observation

- LLM の数学能力が急上昇
- しかし formal proof の局所探索まで LLM に任せる必要があるか?
- 強力かつ軽量な ATP が既に存在

### Slide 3 — Two kinds of reasoning

左: LLM — semantics / theory / strategy  
右: ATP — exhaustive symbolic proof search

### Slide 4 — Current mainstream architecture

Higher-order / dependent ITP  
→ translation / hammer  
→ FOL ATP  
→ reconstruction

Sledgehammer を代表例として説明。

### Slide 5 — The missing control experiment

**What if ATP were native?**

「高階でも使える」ことと「高階経由が最適」は別問題。

### Slide 6 — Mizar as a natural test bed

- set theory / FOL
- declarative mathematics
- MML
- MPTP

### Slide 7 — MizAR 60

大きく **58.4%** を表示。

- top-level lemmas
- no user help in hammer setting
- >75% with premise assistance
- benchmark conditionsも明記

### Slide 8 — Important caveat

MizAR vs Sledgehammer の success rate を直接比較しない。

図:

```text
MizAR        : top-level lemma
Sledgehammer : proof goals inside structured developments
```

### Slide 9 — The architectural hypothesis

```text
LLM -> theorem / lemmas -> FOL-native hammer -> checked evidence
```

### Slide 10 — Cheap-first cascade

ATP first → failure only → LLM → new lemmas → ATP again

### Slide 11 — Mizar Evolution pipeline

現在の pipeline を簡略化して表示。

trusted / untrusted boundary を色分けする。

### Slide 12 — What changes for the user?

局所 proof step を書くのでなく、定義と重要補題を書く。

ATP が通れば `by` 相当で終了。  
失敗時のみ decomposition。

### Slide 13 — Evaluation plan

top-level theorem success rate を中心指標にする。

ATP-only / oracle-premise / LLM-recovery を比較。

### Slide 14 — Research claim

**The goal is not a better hammer for a higher-order ITP.  
The goal is to test an ITP architecture built around automated reasoning from the start.**

### Slide 15 — Closing

**LLM thinks. ATP proves. Mizar Evolution remembers and verifies.**

---

## 12. 発表準備時の要検証事項

最終スライド作成前に必ず一次資料で確認する。

- [ ] MizAR 60 の 58.4%, >75%, 40%/30 sec, 420 CPU sec の条件
- [ ] MML 1147 / 57,897 theorem の定義（unnamed top-level lemmas を含む）
- [ ] Sledgehammer評価で用いる dataset と「goal」の正確な定義
- [ ] Sledgehammer の特定 success rate を載せる場合は、同一論文内の評価条件を併記
- [ ] Mizar Evolution の現行 pipeline が発表時点の main branch と一致していること
- [ ] native hammer / automatic global premise selection の実装済み範囲と将来計画を区別
- [ ] OpenAI 等の最新AI数学成果を背景として使う場合は、発表直前に公開状況を再確認

---

## 13. Codexへの作業指示案

このファイルを入力としてスライドを作る際には、以下を守る。

1. 日本語の TPP 研究集会向け。専門家を想定し、背景説明を長くしすぎない。
2. Mizar Evolution の言語機能紹介ではなく、研究上の architectural hypothesis を主題とする。
3. MizAR 60 を最重要 empirical evidence として扱う。
4. Sledgehammer は straw man にしない。
5. success rate の比較では evaluation granularity を必ず明示する。
6. 「事実」「本発表の解釈」「今後検証する仮説」をスライド上で混同しない。
7. 現行実装については main branch の architecture documents を確認し、
   未実装機能を実装済みと書かない。
8. 最終的に 15--20 分版と 30--40 分版へ切り分けられる構成にする。
