# Mizar Evo の設計指針について

Status: 日本語先行の改訂案。既存の6原則の番号を維持し、序論 §0、6つの設計指針 §1–§6、実装状況・結び §7 で構成。英語版は次段階。

対象: TPP 2026（第22回 Theorem Proving and Provers meeting）、理化学研究所 AIP 東京オフィス、2026年11月16日～17日。45分枠を仮定し、本編38分＋質疑7分（正式枠は要確認）。

内容の正典は `draft.ja.md`。発表原稿は `script.ja.md`。図は英語版と共通で、英語のラベルのまま使う（`figures/` と `../2026-09-bialystok/figures/`）。

## 書式上の約束

- 構造ラベル（`Title:`、`Subtitle:`、`Speaker note:`、`Source:`）と、コード例の状態ラベル（`exact MML excerpt`、`specification example`、`sketch`）は生成器が解釈するので英語のまま残す。
- 太字 `**...**` は読み上げる骨格。日本語台本の要点と対応させる。
- 表題は各表の直上に置き、本編・補足を通して「表1」から通し番号を付ける。
- 主張レベル: 歴史は出典と本人説明を区別。§1–§6 は既存システム・仕様・スケッチを表示。実装は 7.1–7.2 の固定コミットの限定範囲。「研究仮説」「将来構想」は本文中に明記する。
- 生成: `python3 build_beamer.py ja` が `tpp2026_ja.tex` を出力し、upLaTeX + dvipdfmx でコンパイルする（README 参照）。

## Part 0. Introduction

### Frame 0.1 - Title

Title:

```text
Mizar Evo の設計指針について
自動証明・数学的記述・検証可能な計算の再接続
```

Speaker note:

- **Mizar は証明検査系です。一階の集合論の上に、人間が読める数学の言葉と、MML の蓄積を持っています。**
- **Mizar Evolution、略して Mizar Evo は、その言語と道具を設計し直すプロジェクトです。歴史と開発経緯から6つの設計指針へ進み、現在の実装の境界と計画で結びます。**
- 話す内容にはすべて、事実・研究仮説・将来構想のラベルを付けます。

### Frame 0.2 - Mizar の実装史: 初期処理系から Mizar-rs、Evo へ

### 表1　Mizar の初期処理系と MML の成立

| 時期 | 実装史の主な節目 |
|---|---|
| 1973–75 | 数学文書の構想 → 命題論理の Mizar-PC（Algol / ODRA） |
| 1977 | Mizar-QC: 量化を導入（Pascal / CDC 6000） |
| 1982–84 | MSE: 小言語への整理。HPF: 数学的記法・暗黙の引数を拡充 |
| 1986 | Mizar-4: 複数パスで処理。同年末に PC / DOS へ移植 |
| 1988–89 | PC-Mizar、article 間参照。1989年1月1日に MML 開始 |

### 表2　PC-Mizar・Mizar-rs・Mizar Evo の概要

| 処理系 | 概要 |
|---|---|
| PC-Mizar（Pascal） | Parser → MSM → Analyzer → Checker の段階的処理 |
| Mizar-rs（Rust、ITP 2023） | 既存言語との互換性を目指し checker / analyzer を再実装 |
| | **全 MML 検査:** checker を差し替え、当時の全 MML を検査 |
| | **高速化:** 約5〜6倍（8 threads、実時間比較） |
| | **不具合発見:** 既存実装の健全性バグ4件を報告（MML 内で偽の証明への利用なし） |
| Mizar Evo（Rust） | 数学的記述と論理基盤を継承し、新しい言語仕様・処理系を構築 |

Source: Matuszewski & Rudnicki (2005); Carneiro (ITP 2023), Table 1。

Speaker note:

- Mizar-PC は命題論理版で、後年の PC-Mizar とは別。可読な記法・検査・ライブラリを複数の実験と再設計で育てた。表は代表的な節目で、一本道の後継関係ではない。詳細・URLは `draft.ja.md`「歴史と開発経緯」。
- Carneiro 2023 §1脚注2の1986は PC 移植の起点。PC-Mizar 実装は1988年。MSM は More Strict Mizar。
- 速度値は Mizar-rs の測定。実時間は checker のみ57.37→11.33分、analyzer 込み71.38→11.71分。健全性には両者が関わり、同論文 §5.4には互換性に伴う問題も残る。

### Frame 0.3 - 2023–2026年: 構想から言語設計へ

### 表3　Mizar Evo の開発経緯

| 時期 | 開発内容 |
|---|---|
| 2023年8月 | 初回 README は mizar-wishlist |
| 2024年9–10月 | import、namespace、structure 継承、型システム |
| 2025年5–6月 | mode、型推論、redefinition。依存構造・属性の草案を削除後、再導入 |
| 2026年1–4月 | EBNF・論理エンコード、registration、template、algorithm の意味論・契約 |
| 2026年5月 | 実装設計 → テストハーネス → minimal lexer |

- **Mizar の記法を拡張する構想から出発し、2023年の科研費申請準備を機に、言語仕様と処理系を再設計する方針を定めた。**
- ITP 2023 で Mario Carneiro 氏に実装協力を相談し、前向きな了承。CICM 2023 ではAdam Naumowicz 先生にwishlistについて相談。
- 当初の柱: 型の全称量化（generics）、namespace / package、ATP / traceability。Mizar-rs を利用するプロトタイプを構想。

科研費 基盤研究(C)・2024年度採択: 24K14897「集合論ベースの定理証明支援系の言語仕様の刷新」

Speaker note:

- Source: 2023年8月申請書と Git 差分。詳細は `draft.ja.md`「歴史と開発経緯」。
- 相談の年内順序・現在の参加確約は未確認。年表の日付はコミット日。

### Frame 0.4 - 設計の深化と AI Agentによる実装

### 言語設計の深化

- **template:** 当初の generics 構想から、scheme 相当を theorem に統一。構造 view・型引数推論へ拡張。
- **algorithm:** 日本の Mizar コミュニティの暗号理論研究者から、当初より アルゴリズムを証明したいという要望。楕円曲線の結合法則の証明でも、膨大な数式処理に苦労。

### 2026年5月からの実装と意見交換

- **5月: AI coding agents を使い Rust 実装を本格化。6–7月: crate 開発・自律開発手順・SAT-backed 検査と監査。**
- **設計 → エージェント実装 → テスト・監査 → 仕様再検討。**
- 9月: Białystok で Mizar チームにセミナー・意見交換。
- 9–10月: source 由来の VC を拡張しつつ、algorithm の部分性・契約を修正。

Speaker note:

- Source: 仕様18・20、Git 差分、2026年10月15日の補足説明。詳細は `draft.ja.md`。

### Frame 0.5 - 現行 Mizar の課題と6つの設計指針

### 表4　現行 Mizar の課題と6つの設計指針

| 領域 | 現行 Mizar から取り組む課題 | 新仕様の設計指針 |
|---|---|---|
| **§1 基盤** | 論理・MML の継承と処理系の刷新 | 一階論理・集合論を維持し、kernel を小さく保つ |
| **§2 記述** | 暗黙の type・registration・overload の追跡 | 数学的な抽象化を継承し、暗黙の選択を明示 |
| **§3 汎用化** | 定義・定理・scheme の個別機構 | template で共通化の仕組みを統一 |
| **§4 計算** | 証明と実行可能な手続きの接続 | algorithm の契約・不変条件・停止性を検査 |
| **§5 開発基盤** | 依存管理・配布・差分検証・IDE | module・namespace・package・差分ビルド・LSP |
| **§6 検査・自動化** | 外部探索と検査根拠の接続 | ATP を活用し、evidence のインスタンス化と SAT 検査 |

Speaker note:

- 課題と設計指針を同じ行に対応させる。以下の §1–§6 が各行に対応。一階の基盤と可読な記述は、継承すべき強み。
- 新仕様と実装済みの範囲は、末尾の「仕様と実装の状況」で区別する。
- Source: 仕様 01, 18, 20, 21, 23; Białystok 資料の課題別ストーリー。

## Part 1. Logical Foundation

### Frame 1.1 - 基盤の継承: 論理は保ち、処理系を再整備

```text
**数学的記述の層を継承し、**
**処理系と開発基盤を再構築する。**
```

- **一階論理と Tarski-Grothendieck 集合論を基盤として維持。**
- soft type、mode、attribute、registration、structure、宣言的証明を継承。
- **証明探索と検査を分離し、受理を小規模な kernel に集約。詳細は §6。**

Speaker note:

- 数学的な記述と証明支援に必要な機能は、一階の基盤の上にも構成できる。「仕組みが近い」はこれらの構成を指し、論理体系や表現方法の同一性を意味しない。
- 成功率や再構成の失敗率から一階の性能優位性を主張しない。設計の効果は、記述の利便性・自動証明・証明検査の観点で評価する。

### Frame 1.2 - 集合から位相空間まで

例1: Cantor の定理（現行 Mizar・証明省略） (sketch):

```mizar
theorem Cantor:
  for X being set holds card X in card (bool X)
proof ... end;
```

例2: コンパクト空間から Hausdorff 空間への連続全単射（証明省略） (sketch):

```mizar
theorem CompactBijection:
  for S being compact TopSpace,
      T being non empty T_2 TopSpace,
      f being Function of S,T
  st f is continuous one-to-one & rng f = [#] T
  holds f is being_homeomorphism
proof ... end;
```

- **集合・冪集合・基数から位相空間まで、一階の集合論の上で数学を記述する。**

Speaker note:

- Source: CARD_1:14、COMPTS_1:17（Backup 10）。各30秒程度で意味と記述の機構を指す。証明を省略した説明例で、実行結果ではない。
- Cantor: bool Xは冪集合。基数を始順序数で表すため、inはここでは濃度の大小。位相: T_2はHausdorff、rng f = [#] Tは全射、結論は同相写像。探索・検査の分離は§6。

## Part 2. Readable Mathematics

### Frame 2.1 - type・registration・overload の追跡

数学を自然に記述するため、処理系は多くの判断を自動的に行う。

- **type: 対象をどの型として使えるか。**
  属性や継承をたどる。例: Group → Magma（継承経路が一意の場合）。
- **registration: 必要な属性は、どの規則から得られたか。**
  例: empty → finite → countable という登録規則の連鎖。
- **overload: どの定義を選び、どのビューで使うか。**
  同じ記号の候補や、環の加法・乗法のビューを区別する。

**自然な記述と自動適用を保ちながら、暗黙の判断の根拠を追跡可能にする。**

Speaker note:

- Source: `doc/spec/en/08.type_inference.md` §8.1–8.2; `17.clusters_and_registrations.md` §17.7; `19.overload_resolution.md` §19.1–19.3。新仕様。type: 2.2–2.4、registration: 2.5、overload とビュー指定: 2.6。

### Frame 2.2 - type: field と property を宣言する

構造の宣言（definition 内の抜粋） (specification example):

```mizar
struct LoopStr where
  field carrier -> set;
  field binop -> BinOp of carrier;
  property unit -> Element of carrier;
end;

struct AddLoopStr where
  field carrier -> set;
  field add -> BinOp of carrier;
  property zero -> Element of carrier;
end;
```

**field は格納するデータ。property は名前と型を宣言し、実装は別に与える。**

Speaker note:

- Source: `doc/spec/en/05.structures.md` §5.2, 5.5.1; `sample_codes.md`, LoopStr・AddLoopStr。構造宣言は definition 内の抜粋。構成子に渡すのは field のみ。継承は2.3、mode と unit の実装は2.4。

### Frame 2.3 - type: 継承関係を後から宣言する

AddLoopStr の宣言後に、親への対応付けを追加 (specification example):

```mizar
definition
  inherit AddLoopStr extends LoopStr where
    field carrier from carrier;
    field add from binop;
    property zero from unit;
  end;
end;
```

- **構造の宣言と継承関係を分離。型を定義した後で Rust の trait を実装する構成に近い。**
- `from` で親の役割をリネーム: `binop` を `add`、`unit` を `zero` に対応付ける。
- 親ごとに一つの宣言。同じメンバー型なら証明不要、型を狭める場合は `coherence` を証明する。

Speaker note:

- Source: `doc/spec/en/05.structures.md` §5.3; `sample_codes.md`, AddLoopStr。Rust との類似は宣言の分離を指す。LoopStr の binop は field、unit は property。

### Frame 2.4 - type: Unital の unit を実装し、zero に引き継ぐ

has_unit は両側単位元の存在を表す（unit に依存しない） (sketch):

```mizar
definition
  mode UnitalDef: Unital is has_unit LoopStr;
  let M be Unital;
  property M.unit means
    for x being Element of M.carrier holds
      M.binop(it,x) = x & M.binop(x,it) = x;
  existence proof ... end;
  uniqueness proof ... end;
end;
```

- **A の LoopStr ビューが Unital なら、unit の実装を zero として利用できる。**
- `property zero from unit` の対応: `A.zero = (A qua LoopStr).unit`。zero の再実装は不要。

Speaker note:

- Source: `doc/spec/en/07.modes.md` §7.4.1, 7.8.2; `05.structures.md` §5.3, 5.8.3。仕様7の実装例を LoopStr に適用した sketch。has_unit の属性定義は draft.ja.md、証明本体は省略。存在は mode の前提から、一意性は二つの単位元の演算から示せる。A は AddLoopStr、その選択した LoopStr ビューに Unital の前提が必要。異なるビューの事実を混同しない。重なる実装がない例で、重なる場合は coherence が必要。

### Frame 2.5 - registration: 自動連鎖で見えにくくなる依存

**自動適用が連鎖し、利用箇所のソースだけでは依存する登録規則を追いにくい。**

```mizar
registration
  cluster EmptyImpliesFinite: empty -> finite for set;
  coherence proof ... end;
  cluster FiniteImpliesCountable: finite -> countable for set;
  coherence proof ... end;
end;
```

### 表5　新仕様で記録する登録規則の適用経路

| 出発点 | 自動的に得られる事実 | 記録する規則 |
|---|---|---|
| S is empty | S is finite | EmptyImpliesFinite |
| S is finite | S is countable | FiniteImpliesCountable |

- `empty` の宣言から `finite`・`countable` まで暗黙に得る。`by` で規則を引用する必要はない。
- **新仕様: 自動適用を保ち、規則のラベルと適用経路を記録する。**

Speaker note:

- Source: `doc/spec/en/17.clusters_and_registrations.md` §17.2, 17.4.1, 17.7; `23.package_management_and_build_system.md` §23.7.7。コードは新仕様のラベル付き登録例、表は empty set S の説明用トレース。利用箇所には規則の引用を書かず、別の登録規則が複数段適用されることを強調する。属性の解決は ATP 探索より前に行う。

### Frame 2.6 - overload: 同じ環の単位元を使い分ける

**同じ環 R でも、加法の単位元は 0、乗法の単位元は 1。**

- 加法の AddMagma: `0 + x = x + 0 = x`
- 乗法の MulMagma: `1 * x = x * 1 = x`

id. M は unital Magma M の単位元を返す (sketch, 定義省略):

```mizar
let R be Ring;
id. R;                   :: ambiguous Magma view
id. (R qua AddMagma);     :: R.zero
id. (R qua MulMagma);     :: R.one
```

- id. の定義が一つでも、R をどの Magma として使うかが曖昧になる。
- **qua で加法・乗法のビューを明示し、0 と 1 を区別する。**

Speaker note:

- Source: `doc/spec/en/sample_codes.md`, IdentityDef・LoopUnitalDef・RingLikeDef; `19.overload_resolution.md` §19.2.2–19.3.1; `05.structures.md` §5.8.3。ここで示す単位元の id. 定義に限定した sketch。各ビューが unital で、単位元が R.zero / R.one に対応し、Magma への経路が一意であることを仮定する。定義は draft.ja.md。

## Part 3. Generic Mathematics

### Frame 3.1 - 現行 MML: 型ごとに増える再定義・登録

**関数の和の本体は一つでも、結果型を保つ記述は型ごとに必要。**

### 表6　関数の和に付随する型ごとの再定義・登録（VALUED_1）

| 値の型 | 結果型の再定義 | 属性・全域性の登録 |
|---|---|---|
| 複素数 | `PartFunc of C,COMPLEX` | `complex-valued` ＋ `total` |
| 実数 | `PartFunc of C,REAL` | `real-valued` ＋ `total` |
| 有理数 | `PartFunc of C,RAT` | `RAT-valued` ＋ `total` |
| 整数 | `PartFunc of C,INT` | `INT-valued` ＋ `total` |
| 自然数 | `PartFunc of C,NAT` | `natural-valued` ＋ `total` |

- **対応する型が増えるたびに、再定義・登録・整合性の証明が増える。**
- 同じ「関数の和」と、その結果型を、一つの template で記述したい。

Speaker note:

- Source: MML [VALUED_1](https://mizar.uwb.edu.pl/version/current/html/valued_1.html), def 1 と直後の5種類の属性登録・PartFunc 再定義・total 登録。表は記述の整理で、逐語引用ではない。C は共通の定義域。各行は入力値の型を仮定し、total の登録は全域関数と必要な非空値域を仮定する。各再定義・登録に coherence 証明が伴う。
- VALUED_1:def 1 の本体は共通で、定義域の共通部分を使う。次の template 例は共通の非空定義域 I に限定する。

### Frame 3.2 - template: 本体と結果型を一元化する

汎用 functor (sketch, 存在・一意性の証明は省略):

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

### 表7　Template の具体化と結果型

| 呼び出し（実数 R・複素数 C）[^1] | 結果型 |
|---|---|
| `f + g` | `Function of I,R.carrier` |
| `u + v` | `Function of I,C.carrier` |

- **有理数・整数・自然数にも、同じ Add[T,I]。本体と結果型の記述を一元化する。**

[^1]: 明示形: `f +[R,I] g`、`u +[C,I] v`。省略は宣言型から一意に推論できる場合。継承経路が曖昧なら `qua` を明示。

Speaker note:

- `let R be RealAdd; let C be ComplexAdd;` と必要な登録を仮定。両構造の non empty AddMagma への経路は一意。f,g は Function of I,R.carrier、u,v は Function of I,C.carrier。宣言型から T・I が一意に決まる。有理数・整数・自然数も各加法構造・閉性・登録を前提に同じ Add を使う。synonym が中置の + を与える。
- Source: `doc/spec/en/11.symbol_management.md` §11.1.2; `18.templates.md` §18.2.2, 18.2.7, 18.7; `19.overload_resolution.md` §19.6.2。値域集合だけから加法構造を選べるとはしない。qua のビューは自動推論しない。

### Frame 3.3 - Scheme を template に統合 [deep dive]

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

- 述語パラメータにより、代入する述語ごとの一階定理の族を表現する。[^1]
- 明示的なインスタンス化: `defpred` に続けて `by NatInduction[P], Base, Step`。
- 関数子パラメータは集合ではなく schema レベルの記号として扱い、一階論理を維持する。

[^1]: 詳細: Białystok 資料 Story 6（`PermProduct[T]`、`qua` によるビュー）、および本資料 Backup 4。

Speaker note:

- Source: `doc/spec/en/18.templates.md` §18.2.3, 18.8.2, 18.9。型・値パラメータ化の構造・属性への適用例は次の3.4。

### Frame 3.4 - template: 構造と属性もパラメータ化する

struct: 体 K 上のベクトル空間のデータ (sketch, definition 内の抜粋):

```mizar
let K be Field;
struct VectorSpace[K] where
  field carrier -> non empty set;
  field add -> BinOp of carrier;
  field smul -> Function of [:K.carrier, carrier:], carrier;
end;
```

`of` / `over` は syntax sugar: `Vector of K` → `Vector[K]`（mode の例）。

attr: 空間 M における部分集合 S の稠密性 (sketch, 条件省略):

```mizar
let M be MetricSpace;
let S be Subset of M.carrier;
attr DenseDef: S is dense[M] means ...;
```

属性の接頭表記も syntax sugar: `M-dense` → `dense[M]`。

**型・値によるパラメータ化は struct・attr・mode・pred・func に共通。**

Speaker note:

- Source: `doc/spec/en/18.templates.md` §18.1, 18.2.6, 18.3, 18.5。definition 内の抜粋。VectorSpace[K] の法則と dense[M] の条件は省略。Vector は K を引数に取る mode と of 同義表記を仮定。構造でも VectorSpace over K は VectorSpace[K] の同義表記。述語・関数子パラメータは theorem・algorithm のみ。
- 35分版は15秒で構造と属性のパラメータ化を指す。

## Part 4. Verified Computation

### Frame 4.1 - 計算の課題: 検証可能な algorithm

手続きを Hoare 論理の規則で証明義務に分解する:

```text
{ requires }  algorithm body  { ensures }
```

P・Q: 事前・事後条件、B: 条件、I: 不変条件、S: 文。

### 表8　Hoare 論理の基本規則

| 構文 | 検査する条件（部分正当性）[^1] |
|---|---|
| 代入 `x := e` | `P ⇒ Q[x ← e]`（Q 中の自由な x を e で置換） |
| `if B do S1 else S2 end` | `{P ∧ B} S1 {Q}` と `{P ∧ ¬B} S2 {Q}` |
| `while B do S end` | 初期成立: `P ⇒ I`、保存: `{I ∧ B} S {I}`、終了: `I ∧ ¬B ⇒ Q` |
| `return e` | `P ⇒ Q[result ← e]` |

**各構文の規則を再帰的に適用し、一階論理の証明義務へ分解する。**

[^1]: 単純変数・純粋な式の基本形。途中の return / break / continue、型・定義域の条件は省略。

Speaker note:

- Source: `doc/spec/en/20.algorithm_and_verification.md` §20.1.4, 20.4.3, 20.5.1, 20.13.3。表は後ろ向きの規則（捕獲回避置換）。前向き生成は旧状態名で代入を処理し、ループ頭で変更箇所を havoc する。詳細は draft.ja.md。
- 既定は部分正当性。terminating の停止性は別途検査。検証済み手続きは証明支援にも用いる。

### Frame 4.2 - Algorithm: 契約、証明、計算

**契約付きのユークリッドの互除法** (specification example):

```mizar
terminating algorithm EuclidGcdDef: euclid_gcd(a, b) -> Nat
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

- **`terminating` は requires を満たすすべての入力での停止を要求。不変条件と y の厳密な減少を検査する。**[^1]
- **検証後は数学的な functor に昇格し、論理式・証明の中で使える。**

[^1]: ループ注釈は Dafny の `invariant` / `decreases` に類似。Evo は `invariant` / `decreasing` を使う。

Speaker note:

- Source: spec 20、§20.1.1, 20.5, 20.12。外側の definition/let を省略。
- 比較: [Dafny §8.15](https://dafny.org/latest/DafnyRef/DafnyRef.html#sec-loop-specifications)。

### Frame 4.3 - terminating と functor 昇格

### 表9　Algorithm の停止性と保証

| algorithm の形 | requires の下での意味 |
|---|---|
| terminating なし | 呼出しが戻れば契約が成立 |
| terminating を検証済み | requires の下で全域性。functor として利用 |

互除法は契約を通じて推論する (sketch):

```mizar
let a, b be Nat;
assume a >= 1 & b >= 1;
thus euclid_gcd(a,b) = Gcd(a,b) by EuclidGcdDef;
```

計算結果を使う証明 (sketch, MVM 未実装):

```mizar
thus euclid_gcd(12,8) = 4 by computation;
```

検証済み algorithm を MVM（仮想機械）で実行し、その結果を証明に使う。

Speaker note:

- Source: `doc/spec/en/20.algorithm_and_verification.md` §20.7.2–20.7.3, 20.9.1–20.9.2, 20.13.2; `16.theorems_and_proofs.md` §16.5.1。MVM 実行・コード抽出は今後の課題。
- requires と検証済み実行artifactが前提。既定の computation は MVM を信頼する computed 経路。kernel replay の条件は draft.ja.md。

### Frame 4.4 - 互除法: 何を証明するのか

1回の反復: 旧状態 `(x,y)`、`y > 0`。`r = x mod y` とすると、新状態 `(x',y') = (y,r)`。

### 表10　ユークリッドの互除法の証明義務

| 証明義務 | 数学的な根拠 |
|---|---|
| 不変条件の初期成立 | `x=a`, `y=b`。事前条件から正値性 |
| 不変条件の保存 | `Gcd(x,y) = Gcd(y,r)`、`y >= 1`、`r >= 0` |
| Nat 値の測度の減少 | `y' = r`、`0 <= r < y` |
| 終了時の事後条件 | `y=0`。`Gcd(a,b) = Gcd(x,0) = x` |

- **GCD・剰余の補題をライブラリから用い、定理と同じ枠組みで検査する。SAT 単独が算術を知っているわけではない。**
- 最後の `y := r` を `y := x` に変えると減少を失う。仕様に基づく説明例。

Speaker note:

- Source: `doc/spec/en/20.algorithm_and_verification.md` §20.5, 20.12, 20.13.3。証明義務の説明例であり、実行結果ではない。
- 長期的な対象は将来構想: 整数論・組合せ論、記号計算、最適化、暗号、量子アルゴリズム。MVM 実行・コード抽出も今後の課題。

### Frame 4.5 - 式変形を検証する: lambda と微分

関数を直接記述する（u は Real） (sketch):

```mizar
u |-> (sin u)*u
```

`differentiate` が積・sin などの式の構造を再帰的に処理する。

微分の証明に利用する（x は Real） (sketch, MVM 未実装):

```mizar
thus diff((u |-> (sin u)*u), x) = (cos x)*x + sin x
  by computation(differentiate(u |-> (sin u)*u));
```

- **`|->` は集合論的な関数を表し、`expr`（構文木）が式の構造と変数束縛を保持する。**
- 変換 algorithm の契約で、微分可能性と導関数の正しさを検証する。[^1]

[^1]: 微分則・意味変換・簡約の補題を仮定。仕様例であり、実行結果ではない。

Speaker note:

- Source: `doc/spec/en/13.term_expression.md` §13.10; `20.algorithm_and_verification.md` §20.5.3, 20.9.2, 20.12。
- differentiate は部分 algorithm。変数の宣言、集合論的な意味、因数分解の応用案は draft.ja.md。

## Part 5. Development Infrastructure

### Frame 5.1 - 環境部: 著者は何を取り込むのか

現行 Mizar の環境部、ALGSTR_0 を短縮 (sketch):

```mizar
environ
 vocabularies ... STRUCT_0 ...;
 notations ... STRUCT_0;
 constructors ... STRUCT_0 ...;
 registrations ... STRUCT_0;
 theorems STRUCT_0;
```

### 表11　環境部の依存指定と役割

| 一覧 | 取り込むもの |
|---|---|
| vocabularies / notations | 記号 / 記法 |
| constructors | 構成子 |
| registrations / theorems | 自動的な型の事実 / 引用する定理 |

- **役割別に必要な article を選ぶ。`notations`・`definitions` は article の順序にも意味がある。**

Speaker note:

- Source: Białystok `draft.md` Frame 2.1–2.2、ALGSTR_0 の引用。省略記号で一覧を短縮。取り込む登録は暗黙の型推論にも影響し、記号だけでは記法・型の事実は揃わない。
- 順序の Source: Adam Naumowicz, Towards Standardized Mizar Environments, CICM 2017, slide 13: <https://mizar.uwb.edu.pl/~softadm/imports/slides.pdf>。現行環境部の該当する一覧の話で、すべての import 指令の話ではない。

### Frame 5.2 - import と依存の追跡

新仕様: モジュールの import (specification example):

```mizar
import .function;
import mml.algebra.structure.sorted;
```

### 表12　import と依存関係の追跡

| 段階 | 著者が確認できるもの |
|---|---|
| import | モジュールの公開された定義・定理・登録 |
| 解決 | 元の完全修飾名と、登録の解決トレース |
| 再利用 | 差分ビルドで検証済みの依存指紋 |

- **公開項目をまとめて取り込み、実際に何を使ったかを追跡する。**
- package・lock file で依存バージョンを固定し、IDE 診断で解決した名前・トレースを確認する設計。

Speaker note:

- Source: `doc/spec/en/12.modules_and_namespaces.md` §12.3、`17.clusters_and_registrations.md`, Traceability、`23.package_management_and_build_system.md`。import 例は一対一の機械的な移行例ではない。詳細: Backup 7–8。

### Frame 5.3 - 全体像 [deep dive]

![Mizar Evo in one picture](figures/layer_stack.pdf)

- **豊かな数学的記述を、小規模な一階論理の基盤と現代的な開発環境で支える。**

Speaker note:

- 数学的な言語は elaboration を経て一階表現へ移る。外部 ATP の候補 evidence を kernel で検査し、検証済み成果物・ライブラリを IDE / LSP、AI、出版に利用する。

## Part 6. Checking And Automation

### Frame 6.1 - 自動化の課題: 証明探索と検査の分離

Isabelle/HOL: Sledgehammer が内部で検証する証明を提案 (sketch):

```text
have "Q a"
  sledgehammer
  by (metis allPQ pa)
```

Mizar Evo: 引用した前提で宣言的な証明ステップを記述 (sketch):

```mizar
assume AllPQ: for x being object holds P(x) implies Q(x);
assume Pa: P(a);
thus Q(a) by AllPQ, Pa;
```

- **共通する流れ: ゴールと前提 → 外部探索 → 内側の信頼できる受理判定。prover の成功報告だけでは受理しない。**
- Isabelle は Metis 等で内部の証明を再構成。Mizar Evo は論理式・置換の evidence をインスタンス化し、SAT で検査する。

Speaker note:

- Source: [Sledgehammer guide](https://isabelle.in.tum.de/doc/sledgehammer.pdf), §1, 5.2; spec 16; architecture 08, 10, 15。対応する前提を仮定する説明例。
- Evo は引用前提・局所仮定、Sledgehammer は theory context からの前提選択。共通の探索・検査の流れは evidence の同一性や実証結果を意味しない。

### Frame 6.2 - ATP の活用: 探索・検査・再利用

![ATP use with checking, storage, and retry](figures/llm_atp_loop.pdf)

- **ゴールと前提 → ATP 探索 → kernel 検査 → ライブラリへ保存・再利用。**
- 未解決時は補題・方針を修正して再探索。図は設計意図。

Speaker note:

- LLM は提案・修復を支援。受理は evidence の検査で決まり、その仕組みを次のページで示す。
- 現在の ATP 入力は引用した前提と局所仮定。ライブラリ全体の前提選択と反復の費用対効果は今後の評価対象。
- Source: `doc/spec/en/21.source_code_annotation_and_atp.md` §21.7.2; `doc/design/architecture/en/21.ai_agent_interface.md`。

### Frame 6.3 - Resolution 木と置換の抽出

Resolution ログからの候補抽出 (sketch):

![Resolution tree with composed substitutions and extracted evidence](figures/resolution_evidence.pdf)

- **非信頼の抽出器が枝上の置換を合成。F1 に `x := a`、F2 に `y := a` を保存する。**

Speaker note:

- 節の変数 x/y は別名にしている。Q の単一化で x:=y、P の単一化で y:=a。合成して F1[x:=a] と F2[y:=a] を回収する。
- ログを使う候補生成案。現行 architecture 10 は独立した instance finder。kernel は evidence を検査し、ログの各推論を信頼しない。
- Source: architecture 08, 10, 15, 16。スケッチであり、外部 prover の実行例ではない。

### Frame 6.4 - 保存する証拠から具体的な SAT 節へ

evidence の照合から SAT による受理まで (sketch):

![Evidence checks, ground instances and a five-variable ten-clause SAT refutation](figures/sat_evidence_check.pdf)

- **evidence を照合し、CNF を再生成。SAT で UNSAT を確認して R(a) を受理。**

Speaker note:

- 元の式、合成した置換、束縛文脈、対象 VC と反駁の極性を保存。instance と SAT 節は再生成する（Backup 5）。
- 青い各箱は含意を表す s/t の定義と4節。単位節 p・not r と合わせて5変数・10節。元の DIMACS は `draft.ja.md` §6に保持。実行ダンプではない。
- kernel は置換を探索せず、外部 Resolution の各ステップを replay しない。Source: architecture 08, 15, 16。

## Part 7. Status And Roadmap

### Frame 7.1 - 実装状況と規模

**Rust 製品コードの実装済み約27.1万行。完成時見積は約36–54万行（暫定）。**

### 表13　実装状況とコード規模

| 領域 | 実装済み | 完成時見積 | 限定実装と残る範囲 |
|---|---:|---:|---|
| lexer / parser / frontend | 34.4 | 36–43 | syntax を含む。実ソース読込・字句／構文解析を接続 |
| resolver / checker | 122.9 | 148–193 | source 由来の限定意味論。一般化・サービス接続は残る |
| Core / VC | 49.7 | 65–95 | IR・proof を含む。限定 source から検証義務へ |
| ATP / kernel | 22.3 | 30–48 | 符号化・内部候補・SAT 検査。実 ATP との統合は残る |
| build / driver / artifact | 41.3 | 56–85 | cache・共通基盤を含む。全工程の発行は未接続 |
| LSP | 0.5 | 6–19 | lifecycle・文書同期の基盤。意味情報との接続は残る |
| MVM / extraction / doc | 0 | 17–57 | runtime・抽出・文書生成は未実装 |

単位: kLoC（1,000行）。空行・コメント・テストを除く。完成時見積は実装済みを含む総量の概算。MML 移植は対象外。

Source: 基準 `08861708`、2026-10-15計測。残機能に基づく暫定見積。集計範囲・根拠は `draft.ja.md` §7。

Speaker note:

- 19 crates・178ファイル、271,118 LoC。製品コードの量であり完成率ではない。外部 prover・非 Rust コードも対象外。
- 追加88–266 kLoCを仮置き。既存部品の再利用を想定し、実績較正は未実施。講演直前に再計測・再見積する。
- 機能状況は同じ固定コミットの限定範囲。crate の完了表示を全言語の受理能力に読み替えない。

### Frame 7.2 - ソースから成果物まで: 接続の境界

```text
Driver:  source -> SourceLoad -> Frontend
                                    ... later services: pending
Tests:   source -> resolver/checker -> Core -> VC  (limited corpus)
Next:    evidence -> kernel acceptance -> witness -> artifact -> build/cache
```

- **SAT-backed kernel の節目は、あらゆる source proof の受理完成を意味しない。**
- **Algorithm の検証義務生成と、MVM 実行・code extraction・by computation は別の到達点。**
- 外部 prover を用いる一般的な source-to-artifact の実証結果は未主張。

Speaker note:

- Source: `doc/design/todo.md` Step 6–7; `crates/mizar-driver/src/registry/catalog.rs` の ModuleResolver 以後は ExternalDependencyGap; `crates/mizar-test/src/runner/proof_verification.rs` の限定 source 経路; 仕様20.9–20.10。
- 7.1 と同じ固定コミット。式・文脈・置換・来歴の生成、受理後の witness、artifact と再利用の接続を順に完成させる。

### Frame 7.3 - ロードマップ: 統合を優先する

- **最優先: source-to-semantics → VC → evidence → kernel → artifact → build の統合。**
- Algorithm の意味論・検証義務を整合し、限定的に動く経路を広げる。
- **2027年度の目標（予定）: 代表的 MML article の移植・互換性評価、native hammer のベースライン、top-level theorem 単位の評価、論文化。**
- MVM runtime・抽出は別途開発。MML 全体の移植は処理系完成と別課題。
- 科研費延長は検討中・未承認。学習的前提選択、LLM 修復、暗号・量子は長期構想。

Speaker note:

- Source: `doc/design/todo.md` の統合順序・MML port 境界と改訂ブリーフの研究目標。年度は目標であり完成の確約ではない。
- 35分版では1分30秒。最優先の接続と研究目標を述べ、長期構想の個別例は読み上げない。

### Frame 7.4 - おわりに

```text
**Keep the foundation small.**
**Keep the mathematics readable.**
**Modernize everything else.**
```

- **FOL・集合論と数学的な記述能力を継承し、言語・開発基盤を再設計する。**
- **ATP の探索と信頼できる検査を分ける。仕様は今も進化中。**
- **実装・健全性・MML 互換性について、専門家の意見を求めたい。**

## Backup 1. MizAR 60 の詳細

Source: Jakubův et al., MizAR 60 for Mizar 50, ITP 2023.

- データ: MPTP で出力した MML 1147、無名の top-level lemma を含む 57,897 件の定理。MizAR 40 と同一版を用いた比較。
- 学習・開発・holdout を90:5:5に分割。holdout 2,896件中1,690件（58.4%）を hammering mode で証明。ユーザの前提指定なし、420 CPU 秒（MizAR 40 は約40.6%）。
- 人または機械がライブラリから前提を選べる条件では 75% 超（MizAR 40 は 56%）。
- 最高性能の単一手法: hammering mode で 30 秒 40%。人間の前提指定ありで 120 秒 60%。
- 転移: 同手法を MML 1382 の新規 242 article、13,370 定理にも適用。
- 手法: ENIGMA と Deepire で誘導した E と Vampire、学習ベースの前提選択、数百万の ATP 証明で学習するループ。

## Backup 2. Sledgehammer 評価の詳細

### 表14　Sledgehammer と Magnushammer の評価条件

| 評価 | 対象・方式 | 成功率 |
|---|---|---|
| ITP 2022 | AFP 5,000ゴール、greedy 16構成 | 68.8% |
| Magnushammer（2023年公開） | PISA 1,000定理、Sledgehammer | 38.3% |
| 同じ PISA 評価 | 学習による前提選択を用いる Magnushammer | 59.5% |

- ITP 2022: 50 entry × 100ゴール。MePo、基準512事実、各構成30 CPU 秒。構成は評価集合から事後選択。再構成は評価対象外。
- PISA: Isabelle2021-1、Sledgehammer の timeout 30秒、ローカル5 prover、複数設定の成功集合を集約。Isabelle 内での検証を成功条件とする。
- 年代が近くても、局所ゴールと定理全体、探索成功と検証済み証明を混同しない。

Speaker note:

- Source: Desharnais et al., Seventeen Provers Under the Hammer, ITP 2022, sections 5, 5.6, Table 10; Mikuła et al., Magnushammer, arXiv:2303.04488（2023年公開）, Table 2, Appendix A.4。

## Backup 3. HOL から FOL への符号化と再構成

- 関数適用: 関数変数の適用を `app(F, X)` で表現。定数の適用はカリー化を維持、または平坦化。
- ラムダ抽象: lambda lifting は定義式付きの新しい定数を導入する。コンビネータ変換が代替。
- 型: 多相な HOL の型を guard、tag、または単相化で符号化。選択は健全性・完全性・prover 性能に影響する。
- 真偽値: 論理式中の真偽値を返す項には追加の符号化が必要。
- 再構成: 使用した補題による `metis`、SMT 証明の `smt` replay、生成した Isar テキスト。再構成の失敗は別途集計。

Source: Meng and Paulson 2008; Blanchette, Böhme, Popescu, Smallbone 2016; Blanchette, Kaliszyk, Paulson, Urban 2016; Schurr, Fleury, Desharnais 2021.

## Backup 4. Template のインスタンス化: 一つの証明、多くのビュー

有界な型パラメータと汎用的な定理 (specification example, 証明は省略):

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

- 環から Magma への継承経路は二つ。`qua` によるビューの選択が、記法を決定する。

Speaker note:

- Source: `doc/spec/en/18.templates.md`, section 18.2.2.

## Backup 5. Kernel evidence と信頼できる SAT 検査

![KernelEvidence and the kernel's SAT check](../2026-09-bialystok/figures/certificate_replay.pdf)

- evidence は元の論理式・明示的な代入・来歴・対象とゴールの対応を保持。kernel はこれらを検査し、決定的な SAT 問題を生成。信頼できるプロセス内 SAT 検査器で UNSAT を確認する。
- バックエンドの証明トレース、SMT の証明オブジェクト、ログ、終了コードは診断用のみ。

Speaker note:

- Source: `doc/design/architecture/en/08.reasoning_boundary.md`, `15.kernel_certificate_format.md`.

## Backup 6. ATP の中心経路

![The core ATP path, with responsibility groups](../2026-09-bialystok/figures/pipeline.pdf)

- 決定的な discharge 後の未解決義務のみを ATP に送る。前段の discharge にも再検査可能な evidence が必要。
- 各境界で、情報の管理主体・記録する成果物・変更後の再検査範囲を定める。

Speaker note:

- Source: `doc/design/architecture/en/00.pipeline_overview.md`; Białystok 資料 Part 10。

## Backup 7. 差分検証

![The fingerprint graph: what a change re-verifies](../2026-09-bialystok/figures/fingerprint_graph.pdf)

- 証明本体の編集時、公開された主張と受理状態が不変なら、依存側の再ビルドは不要。インタフェース変更時は依存コーンを再検証。
- キャッシュ再利用には、関連キーの完全一致が必要。データ欠落時はキャッシュミス。再利用自体は証明の正当性を保証せず、クリーンビルドで全受理結果を再現する。

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

- 再現可能なビルドの要件: 固定したソース・lockfile・ツールチェーン・検査器設定・決定的な ATP evidence。
- バージョン管理された再利用により、article 集合間の手作業コピーを解消。集約モジュールで、一分野を一つの import として公開できる。

Source: `doc/spec/en/23.package_management_and_build_system.md`; Białystok 資料 Story 1。

## Backup 9. 質疑用の具体例

本編38分には追加せず、質問に応じて以下を参照:

1. 単一のゴールで HOL から FOL への関数の符号化を説明（`app`、部分適用、lambda lifting）。
2. Mizar の関数表現: `Function of X,Y`、`f.x`、集合論的対象としての soft type。
3. Template: 加法と乗法のビューを持つ `PermProduct[T]`（Backup 4）。
4. Algorithm: ユークリッドの互除法、`by computation`、関数子への昇格。
5. Structure とビュー: `AddMagma`、`MulMagma`、`Magma`（Białystok Story 2）。
6. 現代的な基盤: `mizar.pkg`、namespace、指紋グラフ（Backup 7-8）。
7. 検査: evidence のインスタンス化と SAT 検査（本編 6.4、詳細は Backup 5）。

## Backup 10. 出典と帰属

本講演の例が参照する MML:

### 表15　MML の参照箇所

| 目的 | 出典 | 参照箇所 | 使用箇所 |
|---|---|---|---|
| Cantor の定理 | `CARD_1` | 定理14 | Frame 1.2 例1 |
| 連続全単射と同相性 | `COMPTS_1` | 定理17 | Frame 1.2 例2 |

Source URLs:

- <https://mizar.uwb.edu.pl/version/current/html/card_1.html#T14>
- <https://mizar.uwb.edu.pl/version/current/html/compts_1.html#T17>

帰属についての注記:

- 参照した MML のテキストは GPL-3.0-or-later / CC-BY-SA-3.0-or-later。表14に挙げたスライド例は説明用に構成し、原文引用とは区別する。
- ベンチマークの数値は Backup 1-2 と `references.bib` を参照。最終版の作成前に、出版社の情報で書誌事項を確認。

## Backup 11. 評価条件の比較

### 表16　MizAR 60 と Sledgehammer の評価条件

| | MizAR 60 (ITP 2023) | Sledgehammer / AFP (ITP 2022) |
|---|---|---|
| 報告された成功率 | 58.4%（1,690 / 2,896） | 68.8%（3,440 / 5,000） |
| 評価対象 | MML 1147 の定理・補題。全57,897件のうち holdout 2,896件 | AFP の50 entry から選んだ局所ゴール5,000件 |
| 前提選択 | ライブラリ全体から学習的に選択 | MePo。基準512事実、構成により増減 |
| 時間予算 | ポートフォリオ合計 420 CPU 秒 | greedy 16構成 × 各30 CPU 秒 = 480 CPU 秒 |
| 成功の判定 | ATP 証明の発見（hammering mode） | 外部 prover の証明発見。Isabelle 内の再構成は評価対象外 |

- **評価対象・前提選択・時間予算・構成選択が異なる。成功率だけでは設計上の優劣を判定できない。**

Speaker note:

- 「ハンマー」は goal を自動証明器に送る道具。「前提」は prover が使ってよい事実。
- Source: Jakubův et al. 2023, sections 6.2, 6.5; Desharnais et al. 2022, sections 5, 5.6, Table 10（greedy 構成は同じ評価集合から事後選択）。
- 公表年と実験時期は異なる。MizAR の主要実験は2020–2021年。AFP 評価は2022年1月の Isabelle と2021年12月の AFP を使用。
- MizAR の 75% は、人または機械がライブラリから前提を選んだ条件。質問用に取っておく。

## Backup 12. 評価単位: 定理全体と局所ゴール

![What each benchmark counts](figures/evaluation_units.pdf)

- **MizAR: 著者による前提指定なしに、ライブラリから定理全体を自動証明できるか。**
- **AFP 研究: 証明中の局所ゴールを、その文脈で自動証明できるか。多くは人手で記述された証明の内部に位置する。**
- いずれも妥当な評価であるが、対象とする課題は異なる。

## Backup 13. 一階 ATP への二つの接続経路

![Two paths from an interactive prover to a first-order ATP](figures/two_paths.pdf)

- **Sledgehammer の一階・SMT 経路: 高階のゴールを翻訳し、得られた証明を Isabelle 内で再構成する。**
- **MizAR: 一階・集合論の問題を ATP へ送る。**

Speaker note:

- どちらも既存システムの説明（Blanchette, Kaliszyk, Paulson, Urban 2016; Jakubův et al. 2023）。オレンジの箱が翻訳と再構成の層。
- 図は一階・SMT 接続の概形。ITP 2022 の評価は、高階形式を直接扱う prover も含む。
- MizAR の見出しの数字は ATP 証明を数える。Mizar checker は、推論が検査器の強さの範囲なら、得られた `by` ステップを再検査する。

## Backup 14. 高階論理における関数

HOL の文の概形 (sketch, Isabelle/HOL 風の記法):

```hol
f :: 'a => 'b        x :: 'a

P (f x)
```

- **HOL: 関数型と関数適用を論理に組み込む。**
- 関数を引数・戻り値とする関数、部分適用、ラムダ抽象を直接記述できる。
- **高階の構造を用いた簡潔な数学的記述が可能。**
- **E や Vampire の一階推論を利用するには、高階構造を一階表現へ符号化する必要がある。**

## Backup 15. HOL の符号化と再構成

符号化の手順の概形 (sketch):

```fol
F X                   ->  app(F, X)
(%x. t) ...           ->  fresh constant + defining axioms   (lambda lifting)
polymorphic types     ->  type guards or type tags
Boolean-valued terms  ->  extra encoding
```

- **関数変数の適用は `app` で明示。ラムダ抽象は lambda lifting またはコンビネータで、型は guard または tag で符号化する。**
- **ATP の証明を Isabelle 内で再構成（reconstruction）: `metis`、`smt` の replay、生成した Isar テキストを利用。**
- 高階 ITP と一階 ATP の接続を実現する技術。
- 接続の仕組みを示す説明であり、実測上の性能の優劣を示すものではない。

Speaker note:

- Source: Meng and Paulson 2008（高階節から一階節への翻訳）; Blanchette, Böhme, Popescu, Smallbone 2016（型の符号化）; Blanchette, Kaliszyk, Paulson, Urban 2016（サーベイ）; Schurr, Fleury, Desharnais 2021（再構成）。
- Sledgehammer が実際に使う符号化は prover とオプションで変わる。この枚は概形。

## Backup 16. 複雑さを担う層の違い

![Where HOL and FOL systems pay for complexity](figures/where_you_pay.pdf)

- **HOL は高階構造を ATP 向けに符号化。Mizar は集合論の細部を言語層で抽象化。**
- **Mizar Evo: 一階の基盤を保ち、数学的な記述を現代化。**

Speaker note:

- HOL と FOL では複雑さを担う層が異なる。HOL は関数型・ラムダを論理に組み込み、一階 ATP との接続では符号化と再構成を行う。
- 集合論の直接記述に伴う対・定義域などの細部は、Mizar の soft type、mode、attribute、registration、scheme で抽象化する。

## Backup 17. 設計上のトレードオフ

### 表17　HOL / FOL における設計上のトレードオフ

| | HOL ITP + ATP | FOL ITP + ATP |
|---|---|---|
| 記述 | 高階機能による簡潔な記述 | 直接記述すると冗長 |
| 関数 | 基本的な高階の対象 | 一階の集合論的対象 |
| ATP 接続 | 高階構造の一階・SMT 符号化 | 一階問題の生成 |
| 再構成 | Isabelle 内での再構成 | 対応可能な Mizar ステップの再検査 |
| 言語層の役割 | HOL の抽象 | 一階の細部の抽象化 |

- 表現と検査の経路を比較した表であり、実測上の性能優位性を示すものではない。
