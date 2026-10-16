# TPP 2026 発表叩き台

> **Status:** working draft  
> **講演時間:** 45分枠を仮定し、本編36分30秒（35–38分、最大40分）＋質疑7分＋余裕1分30秒。正式な持ち時間は主催者に要確認。
> **位置づけ:** Białystok 2026 セミナーを母体に、TPPでは歴史・開発経緯から **現行 Mizar の6つの課題と、新仕様の対応**へ接続し、Mizar Evolution の設計指針を説明する。
> **注意:** 本文は言語仕様ではない。確立した事実、解釈、研究仮説、将来構想を区別する。

---

# 0. 序論・発表の中心ストーリー

歴史・開発経緯（0.2–0.4）を前置し、§0.5 で現行 Mizar の課題と新仕様の6つの設計指針を1枚に対応させる。各行を §1–§6 で扱い、実装状況と計画（7.1–7.3）で結ぶ。

| 課題 | 新仕様の対応 |
|---|---|
| **§1 基盤:** 論理・MML の継承と処理系の刷新の両立 | 一階論理・集合論を維持し、kernel を小さく保つ |
| **§2 記述:** 暗黙の type・registration・overload の追跡 | 数学的な抽象化を継承し、暗黙の選択を明示 |
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

## 36分30秒の構成

スライドの表には、内容を示す表題と、本編・補足を通した通し番号（表1–表17）を直上に付す。

| 時間 | 内容 | フレーム |
|---|---|---|
| 0–9分 | 歴史・構想・2024–25年の設計・AI開発から6原則へ | 0.1–0.5 |
| 9–11分 | §1 一階論理・集合論・MML の継承 | 1.1–1.2 |
| 11–16分 | §2 type → registration → overload | 2.1–2.6 |
| 16–19分 | §3 Template・記法・型引数推論・構造と属性の例 | 3.1–3.4 |
| 19–24分 | §4 契約・停止性・functor 昇格・互除法・式変形 | 4.1–4.5 |
| 24–27分 | §5 環境部・import・開発基盤 | 5.1–5.3 |
| 27–32分 | §6 探索と検査・evidence・CNF・SAT | 6.1–6.4 |
| 32–36分30秒 | 実装状況・ロードマップ・結び | 7.1–7.3 |
| 36分30秒–43分30秒 | 質疑（本編外） | 必要な Backup |
| 43分30秒–45分 | 余裕時間 | — |

35分版は 3.3（40秒）を省略し、3.4を15秒の概観に短縮（20秒短縮）、0.4 の設計深化の説明を30秒に短縮する（計1分30秒短縮）。年表 0.3 と evidence/SAT の説明は残す。図・コードを指す時間を含む設計値であり、実測はリハーサルで確認する。日本語版を先に確定し、英語版への反映は次段階とする。

## 歴史と開発経緯（0.2–0.4）

序論は表紙を含む5枚。実装史と処理系比較、構想と設計年表、設計深化とAI実装をそれぞれ1枚にまとめ、6原則の対応表（0.5）へ接続する。以下は各スライドの詳細原稿。

**0.2 — 実験的処理系から PC-Mizar へ。** 主要な実装上の節目を選ぶ。小言語の分岐や未完成版もあり、単一の処理系がそのまま拡張され続けた年表ではない。

| 時期 | 処理系・実装上の変化 | 出典節 |
|---|---|---|
| 1973–75年 | 数学文書を計算機で扱う構想から、命題論理の Mizar-PC へ。ODRA 1204 / Algol 1204 | §2–3 |
| 1977年 | Mizar-QC: 量化を導入。CDC 6000 版を Pascal で実装 | §4 |
| 1978–81年 | Mizar-MS、FC、2: 型・関数記法などを試行 | §5–7 |
| 1982年 | Mizar-MSE: 等号付き多ソート述語論理の小言語。Mizar-3 は別の未完成実験版 | §8–9 |
| 1983–84年 | Mizar-HPF: 数学的記法と hidden parameters を拡充 | §10 |
| 1986年 | Mizar-4: メモリ制約からファイルで連携する複数パスへ。同年末に PC / DOS へ移植 | §11 |
| 1988–89年 | PC-Mizar を実装、article 間参照を導入。1989年1月1日を MML の公式な起点とする | §13 |

出典: [Matuszewski and Rudnicki, MIZAR: the first 30 years, 2005, pp. 3–24](https://mizar.uwb.edu.pl/people/romat/MatRud2005.pdf)。**MIZAR-PC の PC は propositional calculus。後年の PC-MIZAR と区別する。** Carneiro 2023 §1脚注2は PC Mizar の起点を1986年と要約しているが、上表では歴史論文に沿い、PC移植（1986）と PC-MIZAR の実装（1988）を分ける。

**0.2 — PC-Mizar から Mizar-rs、Evo へ。** Carneiro 2023 §2が説明する Pascal 実装は、Parser → MSM（More Strict Mizar）→ Analyzer → Checker の複数パスを持つ。MSM は歴史上の Mizar-MS / MSE とは別の、中間構文と処理段階の名前。Analyzer は型・証明構造を処理し、Checker は局所的な証明義務を検査するため、健全性には両者が関わる。

Mizar-rs（Mario Carneiro, ITP 2023）は既存言語との互換性を目指す、公開された Rust 再実装である。成果を次のように示す。

- **互換実装と全 MML 検査:** 健全性に関わる checker / analyzer を再実装。checker は既存の処理系に差し替え可能で、当時の MML 全体を検査した。
- **高速化:** 会議版 Table 1 の8スレッド測定では、MML 検査の実時間が checker のみで57.37分→11.33分（5.06倍）、checker + analyzer で71.38分→11.71分（6.10倍）。本編は「約5〜6倍」と要約する。同表の環境は Intel Core i7-1165G7 @ 2.80GHz。Evo の測定値ではなく、あらゆる環境での速度保証でもない。
- **実装の簡潔化:** 同表の checker + analyzer の比較は Pascal 56,926行に対し Rust 20,096行。これは論文の比較対象の行数であり、信頼基盤を小さな証明 kernel に置き換えたという意味ではない。
- **既存系への還元:** 複数のメモリエラーと健全性バグ4件を発見・報告（§5）。4件は MML 内で偽の証明に利用されていなかった。別の非致命的な不具合に依存していた46 articles には、欠落した `BOOLE` requirement を補った（§5.5）。46件の定理が偽だったという報告ではない。

**成果の境界と Evo への接続。** §5.4の flex-and の問題は Rust 版にも引き継がれ、会議版の時点では MML 互換性のため `unsound` flag を残している。再実装によってすべての健全性問題を解決したとはしない。外部の小さな信頼基盤へ証明を出力することは §7の将来課題。Mizar Evo では言語仕様を再設計し、Rust で処理系を構築する。探索と検査の分離を目指す。Evo へのコード直接流用は未確認。

出典: [Carneiro, Reimplementing Mizar in Rust, ITP 2023, LIPIcs 268, 10:1–10:18](https://doi.org/10.4230/LIPIcs.ITP.2023.10), abstract・§1–2・§5–7、特に Table 1（p. 10:16）。[arXiv 2304.08391](https://arxiv.org/abs/2304.08391) の改訂版と会議版は区別する。会議版 abstract の11.8分・4.8倍は Table 1 と異なるため、本編・台本の性能値は同表に統一する。


**0.3 — 構想から言語設計へ。** 年表（列見出しは「開発内容」）を上に、構想・相談・研究の柱の解説を下に置く。 日付はコミット日であり、発案日や仕様確定日ではない。代表的な差分を示す。

| 時期 | 作業・設計の変化 | 出典（Git差分） |
|---|---|---|
| 2023年8月 | 初回 README は `mizar-wishlist`。公開履歴の起点（研究構想は申請書を参照） | [7085d0de](https://github.com/aabaa/mizar-evo/commit/7085d0de) |
| 2024年9–10月 | 目次・import、namespace、structure 継承、型システムの記述 | [3cf79c79](https://github.com/aabaa/mizar-evo/commit/3cf79c79), [59bd0c55](https://github.com/aabaa/mizar-evo/commit/59bd0c55), [53657c47](https://github.com/aabaa/mizar-evo/commit/53657c47), [710a71cc](https://github.com/aabaa/mizar-evo/commit/710a71cc) |
| 2025年5–6月 | mode、型推論、redefinition を具体化。依存構造・属性の草案を一度削除し、Template との関係を整理して再導入 | [1cd7a6bb](https://github.com/aabaa/mizar-evo/commit/1cd7a6bb), [3b251bca](https://github.com/aabaa/mizar-evo/commit/3b251bca), [0fe0a280](https://github.com/aabaa/mizar-evo/commit/0fe0a280), [b4a6ea8d](https://github.com/aabaa/mizar-evo/commit/b4a6ea8d), [3ec16025](https://github.com/aabaa/mizar-evo/commit/3ec16025) |
| 2026年1–2月 | EBNF・論理エンコードを含む技術仕様へ拡充。registration・Template を体系化 | [b8c9a98c](https://github.com/aabaa/mizar-evo/commit/b8c9a98c), [a8bfe43b](https://github.com/aabaa/mizar-evo/commit/a8bfe43b), [42c809b9](https://github.com/aabaa/mizar-evo/commit/42c809b9) |
| 2026年3–4月 | Template の制約、Algorithm の意味論・契約・snapshot / claim を具体化 | [10ae2070](https://github.com/aabaa/mizar-evo/commit/10ae2070), [502e4e4c](https://github.com/aabaa/mizar-evo/commit/502e4e4c), [73ff614e](https://github.com/aabaa/mizar-evo/commit/73ff614e), [414d6f98](https://github.com/aabaa/mizar-evo/commit/414d6f98) |
| 2026年5月 | 実装設計からテストハーネス、minimal lexer へ。17–18日が継続的 Rust 実装の起点 | [e6abaf93](https://github.com/aabaa/mizar-evo/commit/e6abaf93), [20c91e5b](https://github.com/aabaa/mizar-evo/commit/20c91e5b), [1e5b6a8f](https://github.com/aabaa/mizar-evo/commit/1e5b6a8f) |
| 2026年6–7月 | 各 crate と自律開発手順、SAT-backed 検査・健全性監査を整備 | [941f22e7](https://github.com/aabaa/mizar-evo/commit/941f22e7), [43674a22](https://github.com/aabaa/mizar-evo/commit/43674a22), [f75af877](https://github.com/aabaa/mizar-evo/commit/f75af877) |
| 2026年9–10月 | source 由来の意味論・VC の限定実装を拡張しつつ、Algorithm の部分性・契約を修正 | [e9e75954](https://github.com/aabaa/mizar-evo/commit/e9e75954), [8737e8ba](https://github.com/aabaa/mizar-evo/commit/8737e8ba), [8f7f08d2](https://github.com/aabaa/mizar-evo/commit/8f7f08d2) |

初回 README と申請書は旧名に対応するが、GitHub 上の改名・移行時期は要追加検証。2023年8月〜2024年8月の活動量はこの抽出から断定しない。2024–25年を未着手期間として扱わない。

**0.3 — 構想の起点。** Mizar の記法を拡張する構想から出発し、2023年の科研費申請準備を機に、言語仕様と処理系を再設計する方針を定めた。同年 ITP で Mario Carneiro 氏に「仕様書ができたら実装を手伝ってほしい」と相談し、前向きな了承を得た。CICM 2023 ではAdam Naumowicz 先生にwishlistについて相談した。年内の厳密な順序、現在の参加確約、共同設計の合意は主張しない。

研究課題名・番号はスライド下部の注記に置く: 「集合論ベースの定理証明支援系の言語仕様の刷新」（24K14897、2024年度採択）。申請時の柱は型の全称量化（generics / template）、namespace / package、ATP と traceability。Mizar-rs を利用する新仕様プロトタイプの計画だった。「型の全称量化」は当初の課題名であり、現行 Template が基盤論理を高階化するという意味ではない。

**0.4 — 仕様設計の深化。** template の出発点には、当初から generics の構想があった。そこから従来 Mizar の `scheme` 相当を template 内の `theorem` として表す仕組みに統一し、定義と証明を共通のパラメータの下で記述する設計へ整理した。さらに、代数的構造への制約や、同じ環の加法・乗法に一つの定理を適用する要求に応えるため、制約付き型パラメータ・構造 view・記法の対応付けを具体化し、型引数推論も整備した。これらは一階論理の基盤上で具体化する仕組みである。統一の整理は [084e6f9f](https://github.com/aabaa/mizar-evo/commit/084e6f9f)、制約と構造 view の拡張は [10ae2070](https://github.com/aabaa/mizar-evo/commit/10ae2070)、引数推論の具体化は [7a69c372](https://github.com/aabaa/mizar-evo/commit/7a69c372) の差分を参照。

algorithm には、当初から二つの要求があった。日本の Mizar コミュニティには暗号理論の専門家が多く、アルゴリズムの証明を行いたいという要望があった。また、楕円曲線の結合法則を数式ベースで証明する際に膨大な数式処理を要した経験などから、計算を証明に組み込む要求も以前からあった。これらを背景として、実行可能な手続きと数学的推論を接続する algorithm を設計した。契約による部分正当性と正常に返る保証を区別し、後者を検証した場合に functor として使うため、停止性・部分性・昇格条件を具体化した。研究上の背景は2026年10月15日の補足説明に基づく。要求が当初からあったことと、現行の詳細な意味論が当初から決まっていたことは区別する。

研究計画・報告の出典（非公開原本、発表には研究内容のみ使用）: Dropbox の `基盤C/1.draft/24000611-022-40780242-60050-0001.pdf`（2023年8月、研究目的・方法・準備状況）、`基盤C/R7年度実施報告書/24K14897_2024_F-7-1.pdf`、`基盤C/R8年度実施報告書/24K14897_2025_F-7-1.pdf`（各§5・§7・§8）。いずれもブリーフ記載の科研費フォルダ配下。現行の設計範囲は [Template仕様](../../../spec/en/18.templates.md) と [Algorithm仕様](../../../spec/en/20.algorithm_and_verification.md) を参照。

**0.4 — AI Agentによる実装。** AI coding agents の進歩を背景に、当初想定した実装協力を待たず2026年5月から Rust 実装を本格化した。設計→エージェント実装→テスト・健全性監査→仕様再検討という循環が生まれた。AI は実装・テスト作成を支援する。速度倍率や AI 起因コミット比率は未計測。2026年9月には Białystok で Mizar チームにセミナーを実施して意見交換した。個別の発言・合意・設計変更との因果関係は未確認。秋にも algorithm の仕様修正が続き、設計が十分固まっていなかった面がある。

出典の区分: 方針転換・相談・AI 利用の経験・セミナー実施は 改訂ブリーフ `doc/publications/tmp/tpp2026_revision_brief_ja.md` の本人説明。[9月原稿](../2026-09-bialystok/draft.md) と [README](../2026-09-bialystok/README.md) は発表内容の資料であり、現地議論の議事録ではない。実装開始と秋の仕様修正は上の Git 差分を参照。

**この検討を経て、現在の6つの設計指針になった。** ここから既存の §1–§6 の技術的説明へ接続する。

---

## タイトル

**Mizar Evo: Design Principles**

日本語表記: **Mizar Evo の設計指針について**

— 自動証明・数学的記述・検証可能な計算を再接続する —

---

## 補足: MizAR と hammer benchmark

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

# 1. 基盤: 一階の基盤を現代化する設計方針

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

**1.2 — 集合から位相空間まで。** 約1分で二つの定理の書き方を示す。前半は Cantor の定理（[CARD_1:14](https://mizar.uwb.edu.pl/version/current/html/card_1.html#T14)）で、集合 X より冪集合の濃度が大きいことを `card X in card (bool X)` と記述する。MMLでは基数を始順序数として表すため、ここでの `in` は基数の大小を表す。集合・冪集合・基数も一階の集合論上の対象として扱う例である。

後半はコンパクト空間から Hausdorff 空間への連続全単射が同相写像になる定理（[COMPTS_1:17](https://mizar.uwb.edu.pl/version/current/html/compts_1.html#T17)）。`TopSpace` という構造、`compact`・`T_2` という属性、`Function of S,T` という型を使って通常の数学の前提と結論を記述する。Sはコンパクトな位相空間、Tは空でない Hausdorff 位相空間とし、連続性・単射性と `rng f = [#] T` による全射性を保つ。`[#] T` はTの台集合を指す。

両例とも現行 Mizar の定理を説明用に整形し、変数名とラベルを調整した記述例である。`proof ... end;` は証明省略を表し、原文の逐語引用、Mizar Evoでの検証結果、ソース互換性の実証とはしない。各例は約30秒で数学的な意味と記述に現れる機構を指し、証明の展開は行わない。1.1の「数学的記述の層を継承し、処理系を再構築する」という方針へ接続し、探索と検査の詳細は§6に置く。出典確認日: 2026年10月15日。

---

## 補足: Sledgehammer の接続経路

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

## 補足: 関数を例に HOL と FOL の表現を見る

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

## FOL / Set Theory では逆にどこで払うか

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

## 補足: HOL と FOL の表現と検査経路

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

# 2. 記述: type・registration・overload の追跡

Mizar の soft type、attribute、registration、構造と継承は、一階の集合論の上で数学を読みやすく記述するための言語機構である。
自然な記述を支える自動処理を保ちながら、その判断の根拠を追跡可能にすることを、この節の主題とする。

§2.1 は、次の三つの問いから導入する。

- type: 属性や継承から、対象をどの型として使えるか。例: Group → Magma（継承経路が一意の場合）。
- registration: 必要な属性が、どの規則の連鎖で得られたか。例: empty → finite → countable。
- overload: 同じ記号の候補からどの定義を選び、構造をどのビューで使うか。例: 環の加法・乗法のビューを区別する。

新仕様の説明であり、すべての判断を著者に明示指定させる方針ではない。継承経路が曖昧な場合には `qua` でビューを指定する。
Source: `doc/spec/en/08.type_inference.md` §8.1–8.2; `17.clusters_and_registrations.md` §17.7; `19.overload_resolution.md` §19.1–19.3。

§2.2–2.4 は type の説明として、構造の宣言、継承関係、mode と property の実装の順に示す。
§2.2 で sample_codes の LoopStr・AddLoopStr を宣言し、§2.3 で `field add from binop`・`property zero from unit` の対応付けを宣言する。field は格納するデータであり、property の宣言は名前と型を与える。Rust trait との類似は型と実装関係の宣言を分離する構成に限る。
Source: `doc/spec/en/05.structures.md` §5.2–5.3; `sample_codes.md`。

§2.4 で mode Unital を定義し、親側の `unit` を実装する（説明用 sketch、証明省略）。`has_unit` は `unit` を使わず両側単位元の存在で定義する属性とする。

```mizar
definition
  let M be LoopStr;
  attr HasUnitDef: M is has_unit means
    ex e being Element of M.carrier st
      for x being Element of M.carrier holds
        M.binop(e,x) = x & M.binop(x,e) = x;
  mode UnitalDef: Unital is has_unit LoopStr;
end;

definition
  let M be Unital;
  property M.unit means
    for x being Element of M.carrier holds
      M.binop(it,x) = x & M.binop(x,it) = x;
  existence proof ... end;
  uniqueness proof ... end;
end;
```

存在は mode の前提から、一意性は二つの両側単位元を互いに演算することで示せる。スライドでは has_unit の定義と証明本体を省略する。
AddLoopStr の A について、選択した `A qua LoopStr` が Unital なら、そのビューに unit の実装が適用される。継承時の `property zero from unit` の対応により `A.zero = (A qua LoopStr).unit` となり、zero の別実装は不要である。任意の AddLoopStr に単位元があるとは主張しない。名前の異なる継承は reduct/view を経由するため、異なるビューの属性や実装を無条件に混同しない。
既存の重なる実装はない例とし、重なる場合には `coherence` が必要。`equals` による項からの実装も可能だが、この例では `means` に集中する。
Source: `doc/spec/en/07.modes.md` §7.4.1, 7.8.2; `05.structures.md` §5.3, 5.5.1, 5.8.3。

§2.5 は registration の自動的な連鎖と、それによる依存の見えにくさを課題として先に示す。
`empty` な集合 S と宣言するだけで、登録規則から `finite`、さらに `countable` が暗黙に得られる。利用箇所で各規則を `by` に書く必要はない。登録が連鎖すると、最終的な型の事実がどの規則から得られたかを、その箇所のソースだけで追うことが難しくなる。
その解決策として、新仕様の `EmptyImpliesFinite`・`FiniteImpliesCountable` のラベル付き登録と適用経路の記録を示す。ラベルは必須だが、自動適用は保ち、利用箇所での明示引用は不要。規則名だけでなく、empty → finite → countable という依存の連鎖を説明できることを中心にする。
Source: `doc/spec/en/17.clusters_and_registrations.md` §17.2, 17.4.1, 17.7; `23.package_management_and_build_system.md` §23.7.7。

§2.6 は overload の説明として、同じ環 R に共存する加法と乗法の単位元を使い分ける例を示す。加法の AddMagma ビューでは零元 R.zero、乗法の MulMagma ビューでは単位元 R.one が得られる。§2.4 の unit・zero の実装と対応付けを受ける。

`sample_codes.md` の IdentityDef にある、unital Magma の単位元を返す functor `id. M` を用いる（carrier のアクセスを明記した sketch、証明省略）:

```mizar
definition
  let M be unital Magma;
  func IdentityDef: id. M -> Element of M.carrier means
    for x being Element of M.carrier holds
      M.binop(x,it) = x & M.binop(it,x) = x;
  existence proof ... end;
  uniqueness proof ... end;
end;
```

ここで id. は演算の単位元を返す functor である。利用箇所では加法・乗法のビューを選ぶ:

```mizar
let R be Ring;
id. R;                   :: ambiguous Magma view
id. (R qua AddMagma);     :: R.zero
id. (R qua MulMagma);     :: R.one
```

この例の id. は上記の定義を対象とする。定義が一つでも R から Magma への継承経路は複数あるので、暗黙の widening は曖昧となる。qua で選んだ AddMagma / MulMagma から Magma への経路は一意とする。
選択した各 Magma ビューが unital であり、その単位元が R.zero / R.one に対応することを、構造の定義・継承・登録等から得ている前提の例である。異なる経路の属性は無条件に混同しない。加法側は Group、乗法側は unital であればよく、乗法側を Group と仮定しない。qua 自体は既存 Mizar にもある記法である。
Source: `doc/spec/en/sample_codes.md`, IdentityDef・LoopUnitalDef・RingLikeDef; `19.overload_resolution.md` §19.2.2–19.3.1; `05.structures.md` §5.8.3。

---

# 3. 汎用化: Template と generic mathematics

本編では、functor の「関数の和」を例にする。
§3.1 では、現行 MML の VALUED_1 で点ごとの加法の本体を一度定義しても、複素数・実数・有理数・整数・自然数ごとに結果型の再定義と登録が必要になる負担を示す。
5種類それぞれに PartFunc の結果型、値の属性、全域性の登録があり、その整合性を coherence で証明する。対応する型を増やすたびに、同じ演算を型付きで使うための記述・証明が増える点を中心にする。
§3.2 では加法構造 T と添字集合 I をパラメータ化し、一つの Add[T,I] から各型の関数の和と具体的な結果型を得ることで、この記述を一元化する。各数体系の加法構造やその性質の証明は必要であり、型固有の数学まで省略するという意味ではない。

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

§3.2 では non empty AddMagma を一意に継承する実数・複素数の加法構造 R・C と必要な登録を仮定する。有理数・整数・自然数にも、それぞれの加法構造と同じ条件を用意すれば同じ Add[T,I] を適用でき、関数の和の本体と結果型の記述を型ごとに作り直す必要はない。f,g の宣言型は `Function of I,R.carrier`、u,v は `Function of I,C.carrier` とし、正規化した宣言型から T・I が一意に決まる例として `f + g`・`u + v` と具体的な結果型を表に示す。
本例は共通の非空定義域 I に限定する。VALUED_1:def 1 の異なる定義域の共通部分を扱う構成すべてを置換するとは主張しない。
synonym で中置表記を与える。明示形 `f +[R,I] g`・`u +[C,I] v` と推論の条件は脚注に収める。値域集合だけから構造・演算を選ぶとは主張せず、継承経路が曖昧な場合の qua は明示する（仕様 §11.1.2, §18.2.7, §19.6.2）。

Source: `doc/spec/en/18.templates.md`, sections 18.2.2, 18.7, 18.10.1;
`doc/spec/en/sample_codes.md`, AddMagma;
MML VALUED_1:def 1 と、その直後に並ぶ5種類の値の属性・PartFunc の結果型再定義・全域性の登録
(<https://mizar.uwb.edu.pl/version/current/html/valued_1.html>)。

§3.3 は帰納法による scheme の theorem への統合に集中する。§3.4 を独立させ、型・値によるパラメータ化が struct・attr・mode・pred・func に共通であることを、構造と属性の短い例で示す。

構造の例は、体 K 上の VectorSpace[K] のメンバー宣言（加法・スカラー倍の法則は省略）:

```mizar
definition
  let K be Field;
  struct VectorSpace[K] where
    field carrier -> non empty set;
    field add -> BinOp of carrier;
    field smul -> Function of [:K.carrier, carrier:], carrier;
  end;
end;
```

属性の例は、空間 M における部分集合 S の稠密性:

```mizar
definition
  let M be MetricSpace;
  let S be Subset of M.carrier;
  attr DenseDef: S is dense[M] means
    for x being Element of M, eps being positive Real holds
      ex s being Element of S st dist_M(x,s) < eps;
end;
```

スライドでは definition 外枠と dense[M] の条件を省略する。VectorSpace[REAL_FIELD] などでは体を切り替え、dense[M] では稠密性を判定する空間を指定する。同じ集合 S の稠密性でも空間 M が変われば判定が変わり得るため、M は属性の引数として必要である。
`of` / `over` は角括弧によるパラメータ化の syntax sugar であり、対応する同義表記がある場合、`Vector of K` は `Vector[K]`、上の構造なら `VectorSpace over K` は `VectorSpace[K]` に対応する（Vector は K を引数に取る mode を仮定した記法例）。属性も `S is M-dense` は `S is dense[M]` の syntax sugar である。どちらも同じ template の読みやすい表記である。
型・値パラメータは定義ブロックの各項目で使える。述語・関数子パラメータを使える項目は theorem・algorithm に限定される。
§3は45秒・60秒・40秒・35秒に配分し、19:00の§4開始を維持する。35分版の3.4は15秒で二つのパラメータ化を指し、定義の説明を省略する。
Source: `doc/spec/en/18.templates.md` §18.1, 18.2.6, 18.3–18.7。

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

# 4. 計算: Algorithm は別の柱

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

## Algorithm: tactic-like automation + verified computation

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

§4.1 は Hoare 論理の契約 `{requires} body {ensures}` を、基本構文ごとの証明義務に分けて示す。P を事前条件、Q を事後条件、B を条件、I を不変条件、S を文とする。

- 代入 `x := e`: `P ⇒ Q[x ← e]`。事後条件 Q の自由な x を式 e で、変数捕獲を避けて置換する。
- 分岐 `if B do S1 else S2 end`: `{P ∧ B} S1 {Q}` と `{P ∧ ¬B} S2 {Q}` をそれぞれ検査する。
- 反復 `while B do S end`: `P ⇒ I`（初期成立）、`{I ∧ B} S {I}`（保存）、`I ∧ ¬B ⇒ Q`（終了）を示す。
- `return e`: `P ⇒ Q[result ← e]` を示す。

各構文の規則を本体に再帰的に適用し、最終的に一階論理式の証明義務へ分解する。表は単純変数・純粋な式についての基本形で、途中の return / break / continue と型・定義域の条件を省略する。代入の置換は後ろ向きの説明であり、仕様 §20.1.4・20.13.3 の前向き生成では旧状態名を導入し、代入対象と alias し得る事実を更新する。ループ頭では変更され得る事実を havoc し、不変条件と条件式から本体を検査する。入口の可変値に関する事実をそのまま全反復に持ち越さない。
4.1 は45秒の概観とし、while の3条件を次の互除法の例につなげる。
§4.2–4.4 は terminating が requires を満たすすべての入力での停止を要求し、検証後に functor として使えることを説明する。既定は部分正当性。可変状態・ループを含む互除法は、全域性と契約の公理を通じて推論する。§4.4 の本編は契約の引用と計算結果の利用に絞り、定義方程式の生成は扱わない（詳細は仕様 §20.7.3, 20.13.6）。
定義ラベル EuclidGcdDef と呼び出す名前 euclid_gcd を分け、証明では `by EuclidGcdDef` で検証済み昇格公理を引用する。
§4.4 の下段では、`by computation` を「検証済み algorithm を具体的な入力で MVM（Mizar Virtual Machine）により実行し、計算結果を証明に用いる機能」と説明する。

```mizar
thus euclid_gcd(12,8) = 4 by computation;
```

上段の一般の a,b に対する契約の引用に対し、下段は具体値の等式を計算で確かめる。評価するのは algorithm の euclid_gcd であり、数学的な func Gcd は実行対象ではない。requires は計算結果に依存せず別途証明し、実行には検証済みの実行artifactを用いる。既定の computation は MVM の正常終了報告も信頼する computed 経路である。`replay: true` は検証済み定義方程式による再生と最小核の検査を要求し、定義的でないループを持つこの互除法には適用できない（仕様 §20.9.2）。
このコードは仕様に基づくスケッチで、現在の実行結果を示すものではない。MVM 実行・コード抽出は今後の課題。§4.4 は契約の引用と具体的な計算結果の利用を1分で説明する。
ループ注釈は Dafny の invariant・decreases に類似し、Evo では invariant・decreasing と書く。比較の出典は [Dafny Reference Manual](https://dafny.org/latest/DafnyRef/DafnyRef.html#sec-loop-specifications) §8.15.1–8.15.2。採用元の断定には使わない。
Source: `doc/spec/en/20.algorithm_and_verification.md` §20.1.1, 20.2–20.3, 20.7.2–20.7.3, 20.9.1–20.9.2, 20.13; `16.theorems_and_proofs.md` §16.5.1。

§4.5 では lambda と検証済み式変形を結びつける。`u |-> (sin u)*u` は、u を Real と宣言した場合に全実数上の集合論的関数を表す。通常の論理式では関数の構成・一意性を通じて一階の式へ翻訳し、式の構造を扱う場面では `expr of Function of REAL,REAL` として束縛変数の同一性・型・文脈を保持する。

スライドの証明抜粋は、仕様 §20.12 の微分例に基づく。

```mizar
reserve u for Real;  :: theorem より前
:: proof 内
let x be Real;
thus diff((u |-> (sin u)*u), x) = (cos x)*x + sin x
  by computation(differentiate(u |-> (sin u)*u));
```

`differentiate` は式の構造を調べる部分 algorithm で、入力と結果はいずれも全実数上の関数を表す expr である。変数・定数・和・積・sin・cos の対応する分岐を微分則で検証し、通常終了した場合の契約は、すべての実数 u で入力関数が微分可能で、その微分値が結果の関数値に等しいことを保証する。証明には入力の型・全実数での適用条件、意味変換、点ごとの等式と簡約の補題を用いる。スライドには微分則・意味変換・簡約の補題を仮定することを脚注に示す。対応しない形では失敗し得る。コードは仕様上の利用例で、MVM での実行結果を主張しない。

利点は、数学で使う関数表記をそのまま変換手続きの入力にし、変数束縛を保って構造を処理し、結果を数学的な主張へ戻せることにある。多項式の因数分解なら `u |-> u*u+2*u+1` から `u |-> (u+1)*(u+1)` への変換で、全実数入力について関数値が一致することを契約にする、という将来応用が考えられる。§4.5 では仕様に具体例がある微分を中心に示す。
Source: `doc/spec/en/13.term_expression.md` §13.10; `20.algorithm_and_verification.md` §20.5.3, 20.9.2, 20.12。

§4 はコード（4.2）、証明義務（4.3）、検証後の利用（4.4）、式変形（4.5）の順に進める。配分は45・75・75・60・45秒とし、24:00の§5開始を維持する。4.3 では互除法の本体を外枠省略で再掲し、初期化・不変条件・測度・return に付けた (1)–(4) を証明義務の表と対応させる。旧状態 (x,y) と更新後 (y,r) を区別し、コードを指しながら保存・減少・終了時の結論を説明する。

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

## Algorithm の長期的な射程

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

# 5. 開発基盤: 長年の蓄積を支える開発基盤の更新

ここで論理・証明の話から software infrastructure へ移る。

メッセージ:

> **Mizar の数学的思想を保存することと、既存の software architecture を保存することは別である。**

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

## Old Mizar → Mizar Evolution

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

§5.1 では、環境部に vocabularies・notations・constructors・registrations・requirements・definitions・equalities・expansions・theorems・schemes の10種類の指令があることを表で示す。ALGSTR_0 のコードはそのうち5種類を示す抜粋で、すべての指令を使う必要があるという意味ではない。requirements は特別な処理を有効にする名前を指定する。
同じ資源も記号・記法・構成子・登録など役割ごとに指定する。そのうえ notations・definitions は列挙順が設計上意味を持ち、上書きの優先関係によって採用される記法や定義・再定義が切り替わる。ここで上書きは選択上の優先を指す。取り込む対象の選択と、この上書き・順序依存の管理を、アーティクル執筆の最大の難所として強調する。
例として CICM 2017 資料では、YELLOW_3 の定義展開を修正する並べ替えが別の記法の型情報に影響し、YELLOW_9 では discrete の記法解決が取り込む順序で変わる。スライド本文では個別例を省き、追加・並べ替えが既存の記述や証明へ波及することを口頭で述べる。
Source: Adam Naumowicz, Towards Standardized Mizar Environments, CICM 2017, slides 7, 13, 19–24 (<https://mizar.uwb.edu.pl/~softadm/imports/slides.pdf>)。
続けて仕様の import 例を示す。旧一覧との一対一の翻訳ではなく、モジュールの公開項目を取り込み、完全修飾名・解決トレースで利用項目を追跡する設計として説明する。
package・バージョン管理・差分検証の詳細は全体像と Backup 7–8 で補足する。

---

## Mizar Evolution 全体像

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

# 6. 検査・自動化: Evidence のインスタンス化と SAT 検査

§6.1 は、Sledgehammer と同様の native hammer を Mizar Evo の処理系・IDE に統合する計画を示す。発表では探索を依頼する入力を `thus Q(a) by atp;` と暫定的に表し、補題候補の選択と ATP 試行を経て `thus Q(a) by library.AllPQ, Pa;` を提案する流れを示す。`by atp` は今回の発表用の暫定構文で、現行仕様で確定した呼び出し方法ではない。将来は引数を持たせる想定だが、具体的な形式は未定。局所仮定 Pa: P(a) とゴール Q(a) に対し、ライブラリの説明用補題 library.AllPQ（すべての x で P(x) implies Q(x)）を候補として提案する例にする。
仕様 §22.4.4 では、未解決の義務について利用可能なライブラリの事実を順位付けし、設定された再試行で候補を追加して ATP を試す。引用案はソースに反映してから再検証・受理する。§23.5.2 は候補を missing_facts.json に記録し、LSP は引用を追加する操作を提供する。初期の順位付けは記号ベースで、学習的手法は後の拡張である。候補提示と最小核による受理を区別し、prover の成功報告だけで証明済みとはしない。スライドの矢印は支援の流れであり、暫定構文のコードも実行結果ではない。
通常の検証に渡す前提は明示引用と局所仮定（仕様 §21.7.2）であり、ライブラリの候補探索は引用案を作る支援段階である。Sledgehammer/Metis は内部で証明を再構成し、Evo は論理式・置換の evidence をインスタンス化・SAT 検査する。両者の探索結果や evidence が同一であるとは主張しない。
Source: [Sledgehammer user guide](https://isabelle.in.tum.de/doc/sledgehammer.pdf), §1, 5.2; `doc/spec/en/21.source_code_annotation_and_atp.md` §21.7.2; `22.error_handling_and_diagnostics.md` §22.4.4; `23.package_management_and_build_system.md` §23.5.2, LSP; `doc/design/architecture/en/08.reasoning_boundary.md`, `09.atp_interface_protocol.md`。

§6 では、ATP の探索・kernel 検査・ライブラリへの保存と再利用・再試行の流れ図を先に示す。
図は設計意図であり、LLM の提案・修復も示すが、反復の費用対効果は未評価。支援で得た補題候補を引用に反映し、通常の検証で evidence を検査する流れにつなげる。
続いて Backup 5 の検査の仕組みを、3段階で示す。

1. **Evidence:** 元の論理式、明示的な代入、来歴、対象ゴールを保持する。
2. **Kernel:** 対象との対応・来歴・代入を検査し、論理式のインスタンスを導出する。
3. **SAT:** インスタンスとゴールの否定を決定的に符号化し、信頼できる SAT 検査器で UNSAT を確認した場合に受理する。

UNSAT は、前提とゴールの否定が同時には成立しないことを意味する。
インスタンス化済みの論理式や SAT 問題を外部入力として信頼せず、kernel が生成する。
バックエンドの証明トレース・ログ・終了コードは診断用であり、受理の根拠にはしない。

§6.3 は F1: `forall x. (P(x) implies Q(x))`、F2: `forall y. (Q(y) implies R(y))`、H: `P(a)` からゴール `R(a)` を反駁する Resolution 木を図示する。左に導出木、右に抽出する evidence を配置する。Q の単一化で `x:=y`、続く P の単一化で `y:=a`、最後は基礎節 R(a) と not R(a) から空節を得る。枝上の置換を合成し、元の F1 に `x:=a`、F2 に `y:=a` を回収する。
ログから置換を抽出する候補生成案として説明する。現行 architecture 10 の独立 instance finder も同じ受理用 evidence を生成できる。保存するのは元の式への出所の結び付け、置換と束縛文脈、対象 VC と反駁の極性であり、外部の導出ステップを kernel に replay させない。
§6.4 は来歴・対象・capture avoidance の検査、基礎インスタンスの生成、命題変数への写像、CNF 化を順に示す。`p=P(a)`・`q=Q(a)`・`r=R(a)` に対し、含意を表す補助変数 s・t を導入する Tseitin 符号化を用い、図の青い2箱に s/t の定義と各4節を示し、単位節 p・not r と合わせた5変数・10節から UNSAT と受理までを示す。s・t と p が真で、not r が要求されるため充足不能。
図のノード・矢印は説明用の設計スケッチ。元のインスタンス化・符号化の詳細（図と同じ5変数・10節）は次の通り。

```text
save: F1,F2,H (source bindings); F1[x:=a], F2[y:=a]; goal R(a), refute
check: source/context, binders, capture avoidance
instances: P(a) implies Q(a); Q(a) implies R(a); P(a); not R(a)
atoms: p=P(a)=1, q=Q(a)=2, r=R(a)=3
logical CNF: (not p or q) & (not q or r) & p & not r
Tseitin: s=4 iff (not p or q); t=5 iff (not q or r)
SAT clauses (DIMACS example; 0 ends each clause):
p cnf 5 10
1 0       -3 0
1 4 0     -2 4 0     -1 2 -4 0     4 0
2 5 0     -3 5 0     -2 3 -5 0     5 0
```

現行エンコーダも OR に補助変数を導入する。ここで示すのは solver 入力の具体的な構成例で、実装の実行ダンプや外部 prover の実証結果ではない。instance の不足はゴールが偽であることを意味せず、kernel は置換を探し直さない。

ATP や LLM は探索を支援できるが、受理の境界はこの共通の検査に置く。
Source: `doc/design/architecture/en/08.reasoning_boundary.md`, `10.atp_backend_integration.md`, `15.kernel_certificate_format.md`, `16.substitution_and_binding.md`。

---

# 7. 実装状況・ロードマップ・結び

実装状況の固定点: **2026-10-14 11:32 JST、`0886170858542b03ef2ce16ae61819e186f6781c`**（専用 worktree の基準）。実行デモの測定値ではなく、この版の設計・ソース・既存テストから読む範囲。発表直前に日付と SHA を更新する。

## 7.1 — 仕様・限定実装・計画を分ける

| 領域 | 仕様で定義済み（Specified） | 限定範囲の実装（Implemented）／残る計画（Planned） |
|---|---|---|
| lexer / parser / frontend | 語彙環境・構文木・回復 | 実ソースの読込・字句解析・構文解析を Driver で接続済み |
| resolver / checker | 名前・型・登録・overload・Template | alpha コーパスと source 由来の限定意味論。一般の意味解析サービス接続は残る |
| Core / VC | 論理への変換、契約・不変条件・停止性の義務 | 限定 source から Core / VC・決定的 discharge。全言語・全 Algorithm の検証完了ではない |
| ATP / kernel | 候補生成と evidence の受理を分離 | 問題符号化・候補の内部機構と SAT-backed kernel。実 ATP 候補抽出・受理・witness の統合は残る |
| build / driver / artifact | 差分ビルド・検証済み成果物 | scheduler・指紋・保存機構と frontend までの接続。全工程の artifact 発行は未接続 |
| LSP | 診断・意味情報・編集支援 | 範囲変換と lifecycle / stdio 基盤。build・意味情報の接続は残る |
| MVM / extraction / MML | 計算による証明・実行・抽出を仕様化 | runtime・抽出は未実装。MML 移植は別途進める計画 |

Source: [全体 TODO](../../../design/todo.md) Crate Status・Step 6–8、各行の crate TODO。特に [Driver registry](../../../../crates/mizar-driver/src/registry/catalog.rs)、[VC TODO](../../../design/mizar-vc/en/todo.md)、[ATP TODO](../../../design/mizar-atp/en/todo.md)、[kernel TODO](../../../design/mizar-kernel/en/todo.md)、[LSP TODO](../../../design/mizar-lsp/en/todo.md)。

### 実装規模（同じ固定コミットを2026年10月15日に計測）

**実装済み LoC は現在存在する製品コードの量であり、機能完成率ではない。** Git 管理下の `crates/*/src/**/*.rs` を対象に、空行・コメントのみの行・テストを除いた物理コード行を数える。全OS・feature分岐を対象とし、括弧だけの行と文字列内の非空行もコード行に含む。マクロ展開後の行数ではない。`mizar-test` 全体、`tests/`・`tests.rs`、テスト専用の `determinism_suite.rs`、構文木上の `#[cfg(test)]` 等のテスト専用項目を除外した。19 crates・178ファイルで **271.118 kLoC（1 kLoC = 1,000行）**。依存ライブラリ、外部 prover、生成物、非 Rust コード、MML 本体は含めない。

p.30 の内訳（kLoC。各表示値は丸め、合計は丸め前の値から算出）:

| p.30 の領域 | 実装済み | 完成時見積 | 集計した crate と実装済み（`mizar-` 接頭辞省略） |
|---|---:|---:|---|
| lexer / parser / frontend | 34.4 | 約36–43 | lexer 6.845、parser 17.480、frontend 5.714、syntax 4.329 |
| resolver / checker | 122.9 | 約148–193 | resolve 21.456、checker 101.461 |
| Core / VC | 49.7 | 約65–95 | core 27.750、vc 11.984、ir 4.889、proof 5.101 |
| ATP | 9.6 | 約16–28 | atp 9.553。問題の符号化・候補生成・backend 接続 |
| kernel | 12.8 | 約15–20 | kernel 12.751。SAT 符号化・接続1.947を含む |
| build / driver / artifact・共通基盤 | 41.3 | 約56–85 | build 7.472、driver 6.470、artifact 11.694、cache 6.003、session 3.987、diagnostics 5.700 |
| LSP | 0.5 | 約6–19 | lsp 0.479 |
| MVM / extraction / doc | 0 | 約17–57 | runtime・コード抽出・文書生成の専用実装は未着手。既存の契約・VC・artifact は上の担当行に含む |

ATP と kernel を分け、最小核の議論に関係する検査部の集計範囲を示す。`mizar-kernel` 単体は **12.751 kLoC**。このうち `sat_encoding.rs` は1.665、`sat_checker.rs` は0.282で、SAT 符号化・solver 接続の計 **1.947 kLoC** は kernel 本体の内数である。この1.947に証拠の解析・来歴・置換検査や SAT solver 本体は含まれない。

実際の SAT 判定は `batsat 0.6.0` に依存し、さらに `bit-vec 0.5.1` を使う。同じ空行・コメント・テスト除外の方法で公開ライブラリの `src/**/*.rs` を数えると、それぞれ **3.886 / 0.993 kLoC**、計 **4.879 kLoC**。kernel と合わせた検査部のソース規模は **17.630 kLoC（表示17.6）**。SAT 依存は上の製品コード表と完成時見積には含めず、検査部の内訳として別枠で示す。製品コードの計測は10月15日、kernel の再集計・SAT 依存の計測は10月16日で、コードの固定点は変えていない。

これは **最小 TCB 全体の行数ではない**。kernel の全モジュールには legacy 監査用の処理も含め、SAT 依存も未使用の公開機能を削らず製品ソース全体を数えた。一方、依存先の Core・session、ハッシュ等の推移依存、標準ライブラリ・コンパイラはこの17.630には含めない。Core・session は表の別行に計上済みであり、TCB のみを呼び出し到達性で切り出した計測は未実施。

Source: [kernel manifest](../../../../crates/mizar-kernel/Cargo.toml)、[SAT 符号化](../../../../crates/mizar-kernel/src/sat_encoding.rs)、[SAT 接続](../../../../crates/mizar-kernel/src/sat_checker.rs)、[SAT 依存の選定](../../../design/mizar-kernel/en/sat_dependency_audit.md)。依存版は固定版の Cargo.lock、行数はローカル Cargo registry にある当該版の公開ソースに基づく。

**完成時見積は実装済みを含む総量の暫定レンジ**（1 kLoC = 1,000行）。既存の正式な LoC 見積はなく、今回、残機能の実装規模を仮置きした。対象は現行仕様の実用処理系・統合の Rust 部分。architecture 20 の LOC 目標はテスト群の規模であり、ここには転用しない。MML 移植量、外部 prover、VS Code 拡張等の非 Rust 部分は別計画・対象外とする。

追加量の仮定と主な残機能は次の通り。件数に一定係数を掛けた予測ではなく、既存部品を再利用する場合の概算であり、実績による較正は未実施。

- 構文・frontend **+2–8 kLoC**: 反映構文・Algorithm の構文拡張、意味情報との接続（全体 TODO の Algorithm / Reflected-syntax）。
- resolver / checker **+25–70 kLoC**: 限定 source profile の一般化、一般の式・証明・宣言、Template・登録・imported semantic payload（[Checker TODO](../../../design/mizar-checker/en/todo.md)）。
- Core / VC / IR / proof **+15–45 kLoC**: 一般 source lowering、ループ・再帰・停止性・snapshot / claim、witness 接続（[Core TODO](../../../design/mizar-core/en/todo.md)、VC・Proof TODO）。
- ATP **+6–18 kLoC**: instance finder、実 backend 出力の抽出・選択、候補生成と受理経路の接続（ATP TODO、全体 TODO Step 7）。
- kernel **+2–7 kLoC**: 一般の evidence 生産・受理経路との接続に必要な検査・境界処理の拡充。単独 SAT solver の新規実装量ではなく、依存 SAT は別枠。named equality instances と既存 SAT 検査は実装済みで二重加算しない（kernel TODO、全体 TODO Step 7）。
- build・共通基盤 **+15–43 kLoC**: 実サービス・artifact / cache 統合、runtime artifact、診断公開、CLI・MCP / agent interface の Rust 部分（全体 TODO Step 6–8・Complete processor）。
- LSP **+6–18 kLoC**: build bridge、診断公開、hover / navigation / completion、code actions・計算結果表示（LSP TODO 6–25・EDITOR-COMPUTED-RESULTS）。
- MVM / extraction / doc **+17–57 kLoC**: runtime・実行制限・ghost erasure・抽出 backend **15–50**、文書生成 **2–7**（仕様20.9–20.10、[文書生成 TODO](../../../design/mizar-doc/en/todo.md)）。未着手で実装方式・backend の選択に依存するため、特に不確実性が大きい。

合計の追加想定は **88–266 kLoC**、実装済みを含む完成規模は **約360–540 kLoC**。丸め前の総量は359.118–537.118 kLoCで、全体の表示は10 kLoC単位、行別の完成見積は1 kLoC単位の概数。一般化による削除・置換を追加実装でおおむね相殺する仮定であり、下限・上限の保証ではない。LoC の比から完成率や残工数を求めない。実装方針が変わった時点と講演直前に再計測・再見積する。

### 実装範囲の補足（質疑用）

本編は7.1の実装状況から7.2の優先課題へ直接進む。内部の接続範囲を問われた場合は、以下を説明する。

- Driver の実サービス経路: **実ソース → SourceLoad → Frontend**。ModuleResolver 以後の意味解析・証明・artifact サービスは依存する出力の実装待ち。
- テストハーネスの限定経路: **source → resolver / checker → Core → VC**。内部候補・kernel の検査機構があることと、一般の source proof を端から端まで受理できることは別。
- 次の接続: 本物の式・文脈・置換・来歴 → evidence → kernel 受理 → witness → artifact → build / cache。
- **Algorithm verification と実行を区別**する。VC 生成や SAT-backed kernel の節目は、MVM runtime・code extraction・`by computation` の実行完成を意味しない。

Source: [Step 6–7](../../../design/todo.md)、[semantic harness](../../../../crates/mizar-test/src/runner/proof_verification.rs)、[Algorithm仕様 §20.9–20.10](../../../spec/en/20.algorithm_and_verification.md)。外部 prover を用いた一般的な source-to-artifact の実証結果は主張しない。

## 7.2 — 優先課題と研究目標（計画）

最優先は source-to-semantics → VC → evidence → kernel → artifact → build の統合。限定経路を広げ、未対応入力を受理しない境界を保つ。Algorithm の意味論と検証義務の整合も継続する。

**2027年度の目標（予定）**は、代表的な MML article を用いた移植・互換性評価、native hammer のベースラインと top-level theorem 単位の評価、設計・実装成果の論文化。既存 MML の移植は処理系完成とは別の研究・整備課題であり、ライブラリ全体の移行完了を約束しない。科研費延長は検討中で未承認。

MVM runtime・抽出・`by computation` の実行は別途必要な開発として扱う。学習的前提選択、LLM による修復、暗号・量子アルゴリズムへの展開は長期構想であり、期日付きの完成予定ではない。

---

## 7.3 結び

冒頭の6つの課題に対する新仕様の対応を振り返る。
一階の基盤と数学的な言語を継承し、template、algorithm、開発基盤、
evidence と SAT 検査を通して処理系を再整備する。

仕様は進化中である。討論では実装・健全性・MML 互換性について専門家の意見を求め、記述の利便性、自動化の評価も扱う。
ベンチマーク比較は、必要に応じて補足資料を参照する。

> **Keep the foundation small. Keep the mathematics readable. Modernize everything else.**

---

# 質疑用の補足

HOL→FOL 符号化、`PermProduct[T]`、structure/view、package manifest、指紋グラフ、検査境界の詳細は Backup に保持する。本編を45分に延ばすための追加素材とはせず、質疑で必要な箇所だけ参照する。

---

# 発表での主張レベル

## 事実として述べる

- Mizar / MML / MPTP / MizAR の既存構成;
- Sledgehammer の translation architecture;
- 公開benchmarkの条件と数字;
- Mizar Evolution の公開仕様;
- §7 の固定コミットにおける限定実装の範囲。

## 研究仮説として述べる

- native hammer の前提選択と証明探索を、現代的条件でどう評価するか。

これらの実験結果を、現時点の設計動機や既存システムに対する優位性としては扱わない。

## 将来構想として述べる

- cryptographic protocol verification;
- quantum algorithm verification;
- large-scale autonomous theory generation loop.

---

# 最終確認事項

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
- [ ] 発表直前のコミット SHA・日付と実装状況を更新
- [ ] native global premise selection の実装済み範囲

---

- [ ] 正式な発表枠・質疑時間、35分版／36分30秒版の実測
- [ ] 2023年の相談の年内順序、旧リポジトリの改名・移行時期
- [ ] Białystok の具体的な発言・設計への影響（本人確認が得られるまで追記しない）
- [ ] Mizar-rs の直接流用の有無（ソース・依存・ライセンスを確認するまで主張しない）
- [ ] 科研費延長の承認状況（現時点では未承認）

# Codexへの作業指示

TPP 2026 スライドを作成するとき:

1. 歴史・年表・開発経緯を前置し、§0.5 で課題と設計指針を1枚の表に対応させ、番号を §1–§6 の話題に合わせる。
2. §1 基盤、§2 記述、§3 汎用化、§4 計算、§5 開発基盤、§6 検査・自動化の順に説明する。
3. 一階論理・集合論と可読な数学的言語を、継承すべき強みとして扱う。
4. MizAR / Sledgehammer の評価条件と HOL/FOL 比較は補足に置き、性能優位性を設計動機にしない。
5. ATP 活用の流れ図を §6 に残し、続けて evidence → インスタンス化 → SAT 検査を示す。
6. 仕様と実装状況を区別し、将来構想にはラベルを付ける。
7. 最後に実装状況とロードマップを示し、MML 移行・利便性・自動化の評価を討論する。
8. 最終メッセージは:
   **Keep the foundation small. Keep the mathematics readable. Modernize everything else.**
