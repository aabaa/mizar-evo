# Mizar Evo: Bialystok Mizar チーム向け第二稿

状態: Beamer 化前の第二稿。

予定: 2026年9月 Bialystok の Mizar チーム訪問時の議論用。

英語版主原稿: `draft.md`。

## 作業上の中心命題

Mizar Evo は「既存 Mizar の置き換え」ではなく、「Mizar の伝統を継承する再設計」として提示するのがよいと思います。中心命題は次の形です。

> Mizar Evo は、Mizar を特徴づけてきた可読な数学的言語を維持しつつ、大規模な形式化を AI 支援、予測可能な自動化、再現可能な検証で維持できるように、言語境界、検証器パイプライン、成果物モデル、出版ワークフローを再構成する。

この発表の目的は、Bialystok チームに完成済みの設計を承認してもらうことではありません。現行 Mizar、MML 保守、Formalized Mathematics の制約をよく知る立場から、設計方針をレビューしてもらうことです。

## 参照した情報

### リポジトリ内の参照文書

この第二稿は、主に以下を参照しています。

- `doc/spec/en/01.introduction.md`
- `doc/spec/en/05.structures.md`
- `doc/spec/en/06.attributes.md`
- `doc/spec/en/12.modules_and_namespaces.md`
- `doc/spec/en/18.templates.md`
- `doc/spec/en/23.package_management_and_build_system.md`
- `doc/spec/en/sample_codes.md`
- `doc/design/architecture/en/00.pipeline_overview.md`
- `doc/design/architecture/en/08.reasoning_boundary.md`
- `doc/design/architecture/en/15.kernel_certificate_format.md`
- `doc/design/architecture/en/20.test_strategy.md`
- `doc/design/architecture/en/21.ai_agent_interface.md`

### 今回確認した外部情報

確認日: 2026年6月18日。

- Mizar 公式トップページ。Current Mizar Version 8.1.15、MML Version
  5.94.1493、May 30, 2025 の記載を確認。
  <https://mizar.uwb.edu.pl/>
- Mizar System ページ。Accommodator、Verifier、Exporter、Library
  Committee への提出という現行ワークフローを確認。
  <https://mizar.uwb.edu.pl/system/index.html>
- Mizar Mathematical Library ページ。MML が Mizar Articles からなり、
  built-in notions と Tarski-Grothendieck set theory を基礎にすることを
  確認。
  <https://mizar.uwb.edu.pl/library/>
- Formalized Mathematics ページ。ジャーナル情報、volume 34 (2026) の
  articles in press、巻号一覧を確認。
  <https://mizar.uwb.edu.pl/fm/>
- 現行 HTML-linked `ALGSTR_0`。旧 Mizar 側の algebraic structure 例の
  形を確認するために使用。
  <https://mizar.uwb.edu.pl/version/current/html/algstr_0.html>
- 現行 plain-text MML source。短い exact excerpt と行番号確認に使用。
  - `ALGSTR_0`: <https://mizar.uwb.edu.pl/version/current/mml/algstr_0.miz>
  - `STRUCT_0`: <https://mizar.uwb.edu.pl/version/current/mml/struct_0.miz>

旧 Mizar のコード例は短い exact excerpt に留めています。plain-text file には GPL-3.0-or-later / CC-BY-SA-3.0-or-later の配布条件が記載されているため、final deck では article attribution、source URL、line numbers を speaker note に残すのが安全です。

## Bialystok でレビューしてもらいたいこと

1. Mizar Evo は Mizar として認識できるだけの同一性を保っているか。
2. 言語変更は、スケール、ツール、移行の必要性から見て妥当か。
3. Mizar 側意味解析、ATP 探索、小さい kernel 検査の境界は技術的に説得力があるか。
4. 最初の MML 移行ベンチマークとして、どの断片を選ぶべきか。
5. package-oriented library と Formalized Mathematics をどう接続すべきか。

## 防衛すべき主張

| 主張 | なぜ重要か | 見せるべき根拠 |
|---|---|---|
| Mizar の可読性は守るべき中核資産である。 | tactic-heavy な証明支援系との差別化であり、人間によるレビューの土台でもある。 | side-by-side のコード例、出版例。 |
| 可読性は AI 親和性にも効く。 | AI agent は安定した検索可能な局所テキストパターンを必要とする。 | safe edit classes、patch flow、citation repair 例。 |
| スケールには module、artifact、package 境界が必要である。 | article-level dependency だけでは大規模な発展と再利用が難しい。 | import 例、package manifest、dependency fingerprint。 |
| ATP 探索は強力に使うが信頼しない。 | AI や ATP の出力をそのまま proof truth にしてはいけない。 | ATP -> certificate -> kernel flow。 |
| kernel は探索ではなく証拠検査を担当する。 | trusted base を小さくし、de Bruijn 的な説明を明確にできる。 | certificate fields、rejection categories。 |
| Formalized Mathematics と library はリンクしつつ分離すべきである。 | 論文には叙述、library には再利用可能な module 構造が必要。 | article-to-library link model。 |

## 推奨 Beamer 構成

第二稿は conference talk というより、技術訪問・共同レビュー向けです。ここからは詳細な 5 部構成の review deck として扱います。repository/source-status notes は Markdown 上では有用ですが、Beamer deck の冒頭 slides には入れません。

次のいずれにも展開できます。

- 60-75 枚程度の本編 + 充実した backup slides。
- 120-150 枚程度の workshop 的な詳細 deck。

推奨構成:

| Part | Topic | 目的 | 目安枚数 |
|---|---|---|---:|
| 1 | Concept | readability、AI-readiness、scalability、non-goals | 15-20 |
| 2 | Language specification | grammar、syntax deltas、migration examples、review questions | 70+ |
| 3 | Architecture | trust、ATP、SAT kernel、artifacts、LSP、AI boundary | 25-30 |
| 4 | Roadmap | MML migration phases、compatibility policy、benchmarks | 10-15 |
| 5 | Formalized Mathematics | library/article split、publication links、open questions | 8-12 |
| Backup | detailed examples | ソース、図、policy tables | 20+ |

## Part 0. Opening

### Frame 0.1 - Title

タイトル:

```text
Mizar Evo
Readable, AI-Ready, Scalable Formal Mathematics
```

サブタイトル:

```text
Discussion draft for the Bialystok Mizar team
September 2026
```

Speaker note:

- まず感謝を述べる。
- この発表は、Mizar Evo が Mizar として認識できるかを最も適切に判断できる人たちとの design review である、と明言する。

### Frame 0.2 - 一文での提案

Slide text:

```text
Preserve Mizar's mathematical vernacular.
Modernize the compiler, verifier, artifact, and publication layers.
```

説明:

- 「Mizar を Lean のようにする」話ではない。
- 「MML を置き換える」話でもない。
- Mizar らしい可読性を、現代のスケールと自動化の圧力の下でも維持する話である。

### Frame 0.3 - 今回の訪問で得たいもの

箇条書き:

- 外部からは見えにくい compatibility constraints の特定。
- 小さいが代表性のある migration examples の選定。
- ATP と kernel checking の trust boundary のレビュー。
- Formalized Mathematics と package library の接続設計の相談。

### Frame 0.4 - この発表が主張しないこと

箇条書き:

- Full MML migration が完了しているとは言わない。
- 最終言語仕様が固定済みだとは言わない。
- AI-facing protocol は proof checking の代替ではない。
- 現行 Mizar の成果を問題扱いするものではない。

### Frame 0.5 - 発表全体を通す問い

```text
What must Mizar Evo preserve so that the Mizar community still recognizes it
as Mizar?
```

## Part 1. Current Mizar As The Baseline

### Frame 1.1 - なぜ現行 Mizar から始めるか

要点:

- 再設計は Mizar の強みから始めなければ意味がない。
- Mizar は、可読な形式数学テキストという難題にすでに長く取り組んできた。

### Frame 1.2 - 現行の運用モデル

Mizar System ページに基づく概念図:

```text
source article
  -> Accommodator builds an article-specific environment
  -> Verifier checks the text
  -> Exporter extracts accepted facts and definitions
  -> Library review and inclusion in MML
```

発表での解釈:

- このモデルは歴史的に成功している。
- 一方で、article-centered、database-centered な運用である。
- Mizar Evo は verification と review を保ちつつ、dependency、artifact、package 境界を明示する。

### Frame 1.3 - Verified Article Library としての MML

要点:

- MML は Mizar articles から構成される。
- 基礎には built-in notions と Tarski-Grothendieck set theory がある。
- 他の記事は検証済みの帰結として後続記事から利用できる。

発表上の角度:

- MML は単なる source code ではなく、curated mathematical corpus である。
- 移行計画は、article history、theorem identity、review practice を尊重する必要がある。

### Frame 1.4 - Publication Layer としての Formalized Mathematics

要点:

- Formalized Mathematics は MML と接続した確立済みの publication venue である。
- verifier input だけでなく、scholarly exposition を担っている。
- Mizar Evo はこの橋を壊すのではなく、強化すべきである。

### Frame 1.5 - 現行 Mizar の強み

箇条書き:

- 宣言的な proof text。
- soft typing と mathematical modes。
- attributes、registrations、clusters。
- 成熟した curated library。
- formal articles を中心にした出版文化。

### Frame 1.6 - 現行モデルへの圧力

箇条書き:

- article environment では、正確な局所依存関係が見えにくいことがある。
- 大規模 library には、より細かい cache と artifact 境界が必要。
- 現代的 editor workflow には、部分的で resilient な解析が必要。
- AI agents には bounded かつ structured な context が必要。
- publication と reusable library layout は最適化対象が異なる。

### Frame 1.7 - 設計ルール

```text
Do not trade away readability to gain automation.
Use automation to protect and extend readability.
```

## Part 2. Design Concept

### Frame 2.1 - 三本柱

設計は、三つの制約を同時に満たしているかで評価する。

| Pillar | 意味 | なぜ重要か |
|---|---|---|
| Readability | proof script を数学的叙述に近い形に保ち、重要な曖昧性では局所的な disambiguation を明示する。 | 人間の reviewer が hidden automation を再実行しなくても議論を確認できる。 |
| AI-readiness | verifier context、source spans、obligations、safe edit classes を、小さく監査可能な interface として公開する。 | AI は search、explanation、migration、patch proposal を支援できるが、proof authority にはならない。 |
| Scalability | imports、artifacts、packages、caches、publication links に安定した境界を与える。 | theorem meaning を変えずに、大規模 library maintenance を予測可能にする。 |

三本柱は互いに補強し合う:

```text
Readable source
  -> better human review
  -> better AI retrieval and edits
  -> clearer artifacts and dependency traces
  -> more reliable large-library maintenance
```

設計テスト:

- feature は、readable mathematics を保ちながら automation または scale を改善する場合にだけよい。

### Frame 2.2 - Anti-Goal

```text
Mizar Evo should not make proofs shorter by making the argument invisible.
```

説明:

- tactic-heavy workflow との重要な対比点。
- 自動化は有用だが、受理された proof artifact は inspectable であるべき。

### Frame 2.3 - First-Order Core を選ぶ意味

箇条書き:

- first-order set-theoretic core は通常の数学的実践と相性がよい。
- first-order reasoning では ATP が強い。
- soft types、modes、attributes、registrations は Mizar-facing な構造として維持できる。
- kernel は high-level elaboration ではなく evidence checking に集中できる。

設計理由:

- 現行 Mizar の基礎を保ちつつ、ATP が強い層で ATP を使える。
- dependent-type elaboration や tactic execution を、通常の数学的テキストの信頼説明に含めずに済む。

### Frame 2.4 - Modernization の内容

箇条書き:

- explicit module imports。
- parameterized definitions と theorem schemas のための templates。
- package manifests and lockfiles。
- stable fully-qualified names。
- deterministic build artifacts。
- incremental and parallel verification。
- editor 向け LSP。
- AI agents 向け planned MCP-style context surface。
- publication から library objects への versioned links。

### Frame 2.5 - Modernization が意味しないもの

箇条書き:

- declarative proof style の放棄ではない。
- ATP success をそのまま信頼することではない。
- 人間の mathematical review を AI で置き換えることではない。
- すべての旧記事を一度に新 style へ強制することではない。

## Part 3. Lean As A Design Contrast

### Frame 3.1 - なぜ Lean と比較するか

要点:

- Lean は現代 proof assistant ecosystem の大きな成功例である。
- その成功は design space を明確にしてくれる。
- Mizar Evo は Lean から学ぶが、Lean の core trade-off をそのまま採用するわけではない。

### Frame 3.2 - 比較の framing

```text
This is not "Lean is wrong".
It is "Mizar Evo chooses different constraints".
```

### Frame 3.3 - Trade-Off Table

| Topic | Lean direction | Mizar Evo direction |
|---|---|---|
| Foundation | dependent type theory | soft types を伴う first-order set-theoretic core |
| Proof style | tactics and terms | declarative Mizar-style text |
| Syntax | expressive macros and notation | disciplined, stable surface syntax |
| Automation | elaboration、type classes、tactics、simplifiers | Mizar-side semantics + ATP + checked certificates |
| AI context | proof states and tactic traces | source text、obligations、artifacts、safe patches |

### Frame 3.4 - Readability Contrast

慎重な主張:

- tactics は強力で、Lean 実践では非常に重要。
- 一方で、tactic script は最終的な数学的議論を procedure の背後に隠すことがある。
- Mizar Evo は、proof text が数学的叙述としても publication source としても有用であり続けることを重視する。

### Frame 3.5 - Scalability Contrast

慎重な主張:

- Lean の elaboration と type-class ecosystem は表現力の高い library を可能にする。
- Mizar Evo は name resolution、cluster resolution、overload resolution、ATP dispatch を分離し、traceable に保つ。
- 狙いは、予測可能な局所 context と reproducible artifacts である。

### Frame 3.6 - AI Contrast

慎重な主張:

- Lean agent は proof state に対して tactics や terms を生成することが多い。
- Mizar Evo agent は、小さな source edits を扱うべきである。
  citations、explicit coercions、imports、local assertions、explanations など。
- verifier が各 edit を独立に受理または拒否する。

### Frame 3.7 - Discussion Question

```text
Which Lean-style conveniences would be valuable in Mizar Evo, and which would
we intentionally reject to preserve readability?
```

## Part 4. Language Changes And Migration Examples

### Frame 4.1 - 言語変更の方針

箇条書き:

- recognizable mathematical surface を保つ。
- dependencies と names をより明示する。
- structure definition、inheritance mapping、template parameterization を分ける。
- tools に stable identities と source spans を与える。
- migrated MML material には compatibility metadata を持たせる。

### Frame 4.1a - Mizar から Evo への言語差分 inventory

これは checklist であり、訪問前に全項目を最終仕様として固定するという意味ではない。migration semantics に関わる artifact/workflow 差分は、次の inventory slide に分ける。

| 領域 | 既存 Mizar の baseline | Evo 側の差分 | 変更理由 |
|---|---|---|---|
| Article environment | `environ` の vocabularies、notations、constructors、registrations、requirements | 明示的な `import` prelude と module/package namespace | dependency surface を再現可能・レビュー可能にする |
| Module interfaces | article export は library tooling に媒介され、source-level interface boundary は少ない | `export`、`public`/`private`、opaque imports、separate compilation が public API surface を定義する | reusable interface を安定させ、proof bodies と helpers を private に保つ |
| Symbol identity | article-local names と MML article labels | path-derived FQNs と module-qualified labels | migration、rename、API compatibility、citation repair を追跡する |
| Symbolic notation and aliases | 広い symbolic constructor vocabulary と、mathematical aliases としての synonyms/antonyms | arbitrary symbols は主に `func` と `pred` に集中し、type constructors は identifier-like にする。synonym/antonym equivalence は保つ | alternative notation を失わずに lexing、parsing、editor tooling、AI edits を安定させる |
| Operator precedence and parse domains | notation は強力だが、precedence choices が parser/verifier behavior の中に暗黙化しやすい | 明示的な operator metadata、term/formula の別 precedence domains、parse-before-overload discipline | grouping を deterministic、reviewable、overload choice から独立にする |
| Lexical activation | vocabulary と notation の availability は article environment によって駆動される | import-prelude pre-scan、source-position active lexicon、no forward references、declaration 後の operator metadata | tokenization と dot/operator disambiguation を deterministic にする |
| Soft type foundation | set-theoretic objects 上の Mizar soft types | soft typing を保ちつつ、radix/mode heads、`object`/`set`、type erasure、widening/narrowing、`reconsider` obligations を明示する | Mizar の foundation を認識可能に保ち、verifier obligations を見えるようにする |
| Structures | parent structure、fields、selectors がひとつの宣言で強く結びつく | `struct`、`field`、`property`、`inherit` を別概念にする | layout、derived canonical values、inherited views を分ける |
| Inheritance | inherited fields は structure syntax の中でかなり暗黙的 | `inherit Child extends Parent where ...` が mapping、renaming、narrowing、coherence を記録する | multiple inheritance と diamond cases を監査可能にする |
| Modes and attributes | adjectives と clusters を伴う soft type vocabulary | modes は type abbreviation/refinement として残し、attributes は type-refining predicates として扱う | Mizar style を保ちつつ、classification と data layout を分ける |
| `sethood` and comprehensions | Fraenkel-style set formation は set-theoretic side conditions に依存する | mode `sethood` と comprehension checks を explicit type-checker obligations にする | ATP search の前に set-theoretic foundations を保存する |
| Definitions and correctness | `func`/`pred` definitions は familiar な `equals`/`means` styles を使う | `equals`、`means`、`existence`、`uniqueness`、`assume` side conditions を explicit obligations として保つ | Mizar の definitional idiom を保ちつつ obligations を監査可能にする |
| Predicate and functor properties | algebraic properties は concise automatic reasoning を支える | `commutativity`、`symmetry`、`irreflexivity` などを proof-backed property declarations にする | compact mathematical notation を保ちつつ、automation が使える事実を正確に記録する |
| Overload and `redefine` | overloaded symbols と redefinitions は Mizar の中心的 idiom である | root/family overload resolution、`coherence with`、refinement joins を記録・説明可能にする | source-order ambiguity を避け、refined result facts を安定させる |
| Registrations, clusters, and reductions | 強力な automatic propagation と simplification だが局所的説明が難しい場合がある | labeled registration items、import-filtered resolution graph、reduction index、trace artifacts | automation を残しながら inference と rewriting を説明可能にする |
| Schemes and generics | classical `scheme` と `of`/`over` parameterization | first-class templates。bracket form を canonical にし、constraints と inference を明示し、`of`/`over` は shorthand とする | instantiation choices を隠さずに parameterized definitions と theorem schemas を統合する |
| Declarative proof skeleton | `let`、`assume`、`thus`、`hence`、`thesis`、diffuse reasoning を持つ Jaśkowski-style `proof ... end` text | readable proof skeleton を保ち、extracted obligations、thesis states、source spans を metadata として出す | Mizar の proof prose を守りつつ proof state を toolable にする |
| Proof status | verified theorem items が通常の publication unit | `open`、`assumed`、`conditional` theorem statuses | 未解決作業、仮定、clean verified results を分ける |
| Proof citations | visible facts への `by` references | grouped/bulk citations、used-axiom recording、citation refinement | 小さな verifier-checked repair と AI-assisted edits を支える |
| Type views | 既存の `qua`、implicit widening、局所的な disambiguation idioms | source-level `qua` は保ち、resolved view/coercion metadata artifacts を出す | overload と inherited-view choices を検査可能にする |
| Algorithms | 数学的 proof language であり、一般的な verified programming surface ではない | `algorithm`、contracts、invariants、termination、MVM、`by computation` | theorem truth を変えずに verified computation を扱う |

Speaker note:

- 重要な review question は「syntax change が何個あるか」ではなく、「どの Mizar-facing idea を保存し、どの hidden dependency を明示するか」である。

### Frame 4.1b - Mizar から Evo への artifact/workflow 差分 inventory

これらは純粋な syntax ではないが、migration tools、reviewers、editors、packages が何を信頼できるかに関わる。

| 領域 | 既存 Mizar の baseline | Evo 側の差分 | 変更理由 |
|---|---|---|---|
| Annotations | comments と informal tool hints | `@proof_hint`、`@show_type`、`@show_resolution`、`@latex` などの semantic-neutral annotations | logical meaning を変えずに tools を導く |
| Diagnostics and LSP | verifier messages は主に human-facing output | stable diagnostic codes、primary/secondary spans、fix suggestions、lazy explanations、LSP records | humans、editors、AI tools にとって failures を actionable にする |
| ATP boundary | proof search と verifier behavior が安定した artifact boundary として見えにくい | ATP dispatch と independently checked certificates / rejection categories | trusted base を小さく保つ |
| Packages and dependency resolution | article/MML workflow | `mizar.pkg`、lockfile、SemVer、features、compatibility checks、cached verifier artifacts | reproducible な package-oriented development を可能にする |
| Incremental verification | accepted article output が主な reuse boundary | dependency slices、VC anchors、witness hashes、cache keys。cache reuse は proof authority ではない | clean-build equivalence を保ちながら verification を scale させる |
| Documentation generation | source comments と publication pages は reusable API docs とは別 | `mizar doc`、`:::` doc comments、label/FQN cross-links、`@latex`、verified artifacts からの docs | docs を proof gate にせず browsable API documentation を生成する |
| Code extraction | source language に一般的な verified extraction workflow はない | terminating algorithms、ghost erasure、target-neutral runtime IR、extractor configuration | executable output を verified computation の下流に置く |
| Formalized Mathematics link | article identity と library identity が密接 | publication metadata を reusable library modules へリンクする | citation value を守りつつ package reuse を支える |

Speaker note:

- これらは artifact delta だが、migrated source change が説明可能であり続けるかを決めるため、language discussion の近くに置く。

### Frame 4.1b.01 - Language Specification Review Contract

Message:

- この章は圧縮してはいけない部分である。
- Bialystok review では、language surface を単なる design story ではなく仕様として扱う。
- 各 syntax change は、Mizar author にとっての readability、MML migration cost、parser/verifier/tools にとっての determinism という三つの観点で見る。
- 以下の例は最終 tutorial ではなく review target である。Mizar らしくない、あるいは隠れた Mizar idiom が抜けているなら、それこそが欲しい feedback である。

### Frame 4.1b.02 - Grammar Sources For This Review

Canonical specification sources:

この表の path はすべて `doc/spec/en/` の下にある。

| Topic | Source |
|---|---|
| Cross-chapter grammar | `appendix_a.grammar_summary.md` |
| Operator precedence | `appendix_b.operator_precedence.md`, `10.functors.md` |
| Lexical structure | `02.lexical_structure.md` |
| Structures | `05.structures.md` |
| Attributes and modes | `06.attributes.md`, `07.modes.md` |
| Terms and formulas | `13.term_expression.md`, `14.formulas.md` |
| Statements and proofs | `15.statements.md`, `16.theorems_and_proofs.md` |
| Registrations and templates | `17.clusters_and_registrations.md`, `18.templates.md` |

Review rule:

- Slides は surface grammar の要約である。edge cases と well-formedness rules については spec files が authority である。

### Frame 4.1b.03 - Surface Syntax Review Map

Review ではこの grammar stack を順に確認する:

1. lexical classes と active lexicon;
2. module/import prelude と item activation;
3. definition blocks と visibility;
4. type expressions、attributes、modes、structures;
5. functors、predicates、notation、properties、overload families;
6. operator precedence と term/formula boundary;
7. terms、formulas、comprehensions、`the`、`sethood`、`qua`;
8. theorem/proof statements と citations;
9. registrations、reductions、templates、annotations、algorithms。

Why this order matters:

- Mizar-style readability は局所 syntax だけでは決まらない。いつ names が見えるか、どの notation が active か、どの automation が発火してよいかに依存する。

### Frame 4.1b.04 - Compilation Unit Syntax

Grammar sketch:

```ebnf
compilation_unit ::= import_prelude export_prelude
                     { annotated_declaration } ;

import_prelude   ::= { import_stmt } ;
export_prelude   ::= { export_stmt } ;

declaration      ::= definition_block
                   | reserve_decl
                   | registration_block
                   | claim_block
                   | [ visibility ] theorem_item
                   | [ visibility ] notation_decl ;

visibility       ::= "private" | "public" ;
```

Example:

```mizar
import algebra.group as group;
export algebra.group;

public theorem left_cancel:
  for G being Group, a, b, c being Element of G
  st a * b = a * c holds b = c;
```

Review point:

- file は再現可能な dependency surface から始まり、その後に通常の Mizar items が続く。これは article-environment roles を置き換えるが、language を generic programming module system にするためではない。

### Frame 4.1b.05 - Import Prelude And Item Activation

Grammar sketch:

```ebnf
import_stmt ::= "import" module_alias_decl
                { "," module_alias_decl } ";" ;
export_stmt ::= "export" module_path { "," module_path } ";" ;

module_alias_decl ::= module_path [ "as" module_identifier ]
                    | module_branch_import ;
```

Example:

```mizar
import algebra.group as group;
import algebra.ring.{Ring, Ideal};
export algebra.group;
```

Semantic rule:

- すべての `import` statements は最初の non-import item より前に置く。
- imported symbols が body の active lexicon を seed する。
- current module の declarations は、その item が完了してから active になる。
- 後続の import-shaped statement は dynamic environment update ではなく syntax error である。

Why:

- lexing、parsing、AI context extraction を deterministic にするため。

### Frame 4.1b.06 - Lexical Classes

Grammar sketch:

```ebnf
identifier       ::= ( letter | "_" )
                     { letter | digit | "_" | "'" } ;
constructor_name ::= identifier | readable_constructor_name ;
user_symbol      ::= symbol_char { symbol_char } ;
```

Example:

```mizar
struct ZeroStr where
  field carrier -> set;
end;

func + (x, y: Integer) -> Integer equals int.add(x, y);
```

Policy:

- `mode`、`struct`、`attr` names は constructor names を使う。
- `field` と `property` names は identifiers である。
- arbitrary symbolic notation は `func` と `pred` に集中させる。
- tokenization は source position で active な longest match を使う。

Review point:

- これは legacy Mizar から見て最も目に見える syntax difference の一つである。実際の algebraic / set-theoretic examples で確認する必要がある。

### Frame 4.1b.07 - Reserved Symbols And Dot Disambiguation

Reserved punctuation includes:

```text
, . .. ; : := ( ) [ ] { } .{ = <> & -> .= .* @[ ...
```

Dot policy:

- `.` は grammar position と active lexicon により、selector syntax、namespace qualification、user functor symbol のいずれにもなりうる。
- parser は resolver が分類できるだけの surface form を保存する。

Bracket policy:

- `[` と `]` は template arguments と built-in bracket functor syntax のために予約する。
- `x[y]` のような postfix indexing は黙って許可しない。

Example:

```mizar
let R be Ring;
set U = R.carrier;
set e = algebra.group.identity(R qua Group);
```

Why:

- ambiguous punctuation により migration が parser guesses に依存してはいけない。

### Frame 4.1b.08 - Type Expression Syntax

Grammar sketch:

```ebnf
type_expression ::= attribute_chain type_head ;
type_head       ::= radix_type | mode_type ;

attribute_chain ::= { [ "non" ] attribute_ref } ;
attribute_ref   ::= [ param_prefix ]
                    [ struct_ref_name "." ] attribute_name
                    [ "(" argument_list ")" ] ;

radix_type      ::= "object" | "set"
                  | struct_ref_name [ type_args ] ;
mode_type       ::= mode_ref_name [ type_args ] ;
```

Example:

```mizar
reserve G for non empty Group;
let x be Element of G;
let H be commutative subgroup of G;
```

Review point:

- Mizar の soft type style は保存する。ただし grammar は radix type、mode type、attribute chain の分離を明示する。

### Frame 4.1b.09 - Definition Block Syntax

Grammar sketch:

```ebnf
definition_block   ::= "definition"
                         { definition_content }
                       "end" ";" ;

definition_content ::= { annotation }
                       ( definition_parameter_decl
                       | assumption
                       | correctness_condition
                       | property_item
                       | [ visibility ] definitional_item
                       | [ visibility ] theorem_item
                       | [ visibility ] registration_item ) ;
```

Example:

```mizar
definition
  let G be Group;
  func UnitDef: unit(G) -> Element of G
    means for x being Element of G holds it * x = x;
  existence by group.unit_exists;
  uniqueness by group.unit_unique;
end;
```

Review point:

- `definition ... end` の envelope は familiar なまま保つ。
- 新しい surface elements は、Mizar の definition-oriented reading style を壊さず obligations を明確にしているかで評価する。

### Frame 4.1b.10 - Structure Declaration Syntax

Grammar sketch:

```ebnf
struct_def    ::= "struct" struct_def_name [ type_params ]
                  "where" struct_member { struct_member }
                  "end" ";" ;

struct_member ::= field_decl | property_decl ;
field_decl    ::= "field" identifier "->" type_expression
                  [ ":=" term_expression ] ";" ;
property_decl ::= "property" identifier "->" type_expression ";" ;
```

Example:

```mizar
struct UnitalMagmaStr where
  field carrier -> non empty set;
  field mult -> BinOp of carrier;
  property unit -> Element of carrier;
end;
```

Review point:

- `field` は structure value の intrinsic data を宣言する。
- `property` は canonical derived slot または obligation を宣言する。
- この syntax は layout と mathematically determined values を意図的に分ける。

### Frame 4.1b.11 - Property Implementation Syntax

Grammar sketch:

```ebnf
property_means_impl  ::= "property" identifier "." identifier
                         "means" formula_definiens ";"
                         existence_block uniqueness_block ;
property_equals_impl ::= "property" identifier "." identifier
                         "equals" term_definiens ";" ;
```

Example:

```mizar
definition
  let M be UnitalMagma;
  property M.unit means
    for x being Element of M.carrier holds
      M.mult(it, x) = x & M.mult(x, it) = x;
  existence by magma.unit_exists;
  uniqueness by magma.unit_unique;
end;
```

Review point:

- `struct` 内の `property` は virtual slot であり、stored field data ではない。
- implementation は、必要な assumptions を attributes で保証する mode に対して後から与える。
- `means` は `existence` と `uniqueness` を伴い、`equals` は value を直接与える。

### Frame 4.1b.12 - Inheritance Declaration Syntax

Grammar sketch:

```ebnf
inherit_def ::= "inherit" inherit_child "extends" parent_type
                [ "where" inherit_member { inherit_member }
                  [ coherence_block ] "end" ] ";" ;

field_redef    ::= "field" identifier [ "->" type_expression ]
                   "from" ( identifier | "it" ) ";" ;
property_redef ::= "property" identifier [ "->" type_expression ]
                   "from" identifier ";" ;
```

Example:

```mizar
inherit GroupStr extends UnitalMagmaStr where
  field carrier from carrier;
  field mult from mult;
  property unit from unit;
  coherence by group.inherit_unital_magma;
end;
```

Review point:

- 各 `inherit` statement は parent をちょうど一つだけ指定する。
- renaming、narrowing、coherence は source-visible items である。
- diamond inheritance は hidden merge ではなく checked mapping problem になる。

### Frame 4.1b.13 - Attribute And Mode Syntax

Grammar sketch:

```ebnf
attr_def ::= "attr" label ":" subject "is" attr_pattern
             "means" formula_definiens ";" ;

mode_def ::= "mode" label ":" mode_def_name [ type_params ]
             "is" attribute_chain radix_type ";"
             [ mode_property ] ;
```

Example:

```mizar
attr AssocDef:
  G is associative means
    for x, y, z being Element of G holds (x * y) * z = x * (y * z);

mode GroupDef: Group is associative invertible UnitalMagmaStr;
```

Review point:

- Attributes は unary type-refining predicates である。
- Modes は reusable type classifications に名前を付ける。
- これにより Mizar の adjective-rich prose を保ちつつ、cluster participation を明示する。

### Frame 4.1b.14 - Predicate And Functor Syntax

Grammar sketch:

```ebnf
pred_def ::= "pred" label ":" pred_pattern
             "means" formula_definiens ";" ;

func_def ::= "func" label ":" func_pattern "->" type_expression
             ( "means" formula_definiens
             | "equals" term_definiens ) ";" ;
```

Example:

```mizar
pred DividesDef:
  x divides y means ex z being Integer st y = x * z;

func InvDef: inverse(G, x) -> Element of G
  means x * it = unit(G) and it * x = unit(G);
```

Definition patterns:

- Predicate and functor patterns は symbolic notation または phrase notation を使える。
- symbols は declaration item 完了後にだけ active になる。
- overloaded definitions は accidental source-order behavior ではなく、roots、argument types、refinement rules によって解決する。

### Frame 4.1b.15 - Synonyms, Antonyms, And `redefine`

Grammar sketch:

```ebnf
synonym_def  ::= "synonym" alt_pattern "for" original_pattern ";" ;
antonym_def  ::= "antonym" alt_pattern "for" original_pattern ";" ;

redefine_attr ::= "redefine" "attr" label ":" subject "is" attr_pattern
                  "means" formula_definiens ";"
                  "coherence" [ "with" label ] justification ";" ;
redefine_pred ::= "redefine" "pred" label ":" pred_pattern
                  "means" formula_definiens ";"
                  "coherence" [ "with" label ] justification ";" ;
redefine_func ::= "redefine" "func" label ":" func_pattern
                  "->" type_expression
                  ( "means" formula_definiens | "equals" term_definiens ) ";"
                  "coherence" [ "with" label ] justification ";" ;
```

Example:

```mizar
definition
  let a, b be Real;
  pred LessDef: a < b means real.lt(a, b);
  synonym b > a for a < b;
  antonym a >= b for a < b;
end;

definition
  let x be non negative Real;
  redefine func AbsNonNeg: |.x.| -> non negative Real equals x;
  coherence with AbsGeneral by real.abs_nonneg;
end;
```

Review point:

- Synonyms と antonyms は mathematical phrasing と natural negation を保存する。
- これらは semantic aliases であり、新しい root ではない。
- `redefine` は `coherence with` により同じ root を sharpen する。rival overload ではない。

### Frame 4.1b.16 - Predicate And Functor Properties

Grammar sketch:

```ebnf
pred_property ::= ( "symmetry" | "asymmetry" | "connectedness"
                 | "reflexivity" | "irreflexivity" )
                  justification ";" ;

func_property ::= ( "commutativity" | "idempotence"
                 | "involutiveness" | "projectivity" )
                  justification ";" ;
```

Example:

```mizar
definition
  let x, y be Real;
  pred LessDef: x < y means x is_less_than y;
  asymmetry by real.lt_asym;
  irreflexivity by real.lt_irrefl;
end;

definition
  let X, Y be set;
  func UnionDef: X \/ Y -> set means
    for z being object holds z in it iff z in X or z in Y;
  commutativity by set.union_comm;
  idempotence by set.union_idem;
end;
```

Review point:

- Properties は comments でも trusted hints でもない。
- 各 property は proof obligation とともに受理される。
- この設計により、algebraic notation を compact に保ちながら、automatic matching、rewriting、ATP が使える facts を正確に記録する。

### Frame 4.1b.17 - `equals`, `means`, And Correctness Blocks

Key distinction:

| Form | Meaning | Obligations |
|---|---|---|
| `equals` | direct term definition を与える | 多くの場合 well-definedness は即時 |
| `means` | value または relation を特徴づける | `existence`、`uniqueness`、`coherence`、`compatibility` が必要になりうる |
| `assume` | local definition-side condition を追加する | generated obligation の一部として記録する |

Grammar sketch:

```ebnf
existence_block     ::= "existence" justification ";" ;
uniqueness_block    ::= "uniqueness" justification ";" ;
coherence_block     ::= "coherence" justification ";" ;
compatibility_block ::= "compatibility" justification ";" ;
```

Example:

```mizar
func InvDef: inverse(G, x) -> Element of G
  means x * it = unit(G) and it * x = unit(G);
existence by group.inverse_exists;
uniqueness by group.inverse_unique;
```

Review point:

- syntax は Mizar の definitional idiom を保存しつつ、migration tools が隠してはいけない proof obligations を見えるようにする。

### Frame 4.1b.18 - Operator Declarations And Precedence

Grammar sketch:

```ebnf
infix_operator_decl   ::= "infix_operator" "(" string_literal ","
                          infix_assoc "," nat_literal ")" ";" ;
prefix_operator_decl  ::= "prefix_operator" "(" string_literal ","
                          nat_literal ")" ";" ;
postfix_operator_decl ::= "postfix_operator" "(" string_literal ","
                          nat_literal ")" ";" ;
infix_assoc           ::= "left" | "right" | "none" ;
```

Example:

```mizar
infix_operator("+", left, 80);
infix_operator("*", left, 90);
infix_operator("^", right, 95);
prefix_operator("-", 85);
postfix_operator("!", 95);

a + b * c       :: a + (b * c)
a ^ b ^ c       :: a ^ (b ^ c)
a %% b %% c     :: error if %% is non-associative
```

Design reason:

- term precedence と formula precedence は別 domain である。
- precedence values は `0` から `255` で、高い値ほど強く結合する。
- declaration のない symbolic functor は precedence `64`、non-associative grouping を default とする。
- operator declarations は後続 token にだけ作用し、forward declarations ではない。また target spelling はその時点で active functor でなければならない。
- parsing は operator metadata から grouping を決め、overload resolution はその後で semantic root を選ぶ。
- 同じ visible user symbol に対する imported metadata が衝突する場合は link-time conflict とする。
- `=`、`<>`、`in` のような built-in predicate symbols は core module 由来であり、user declaration では override できない。
- `qua` は language-fixed であり、term-level で最も低い precedence を持ち、left-associative である。

### Frame 4.1b.19 - Term Expressions, Selectors, And `qua`

Grammar sketch:

```ebnf
term_expression ::= operator_expression
                    { "qua" type_expression } ;

term_postfix    ::= "." field_name [ "(" [ term_list ] ")" ]
                  | "with" "(" field_update_list ")" ;

field_update    ::= selector ":=" term_expression ;
```

Example:

```mizar
let x be object;
reconsider y = x qua Element of G;
set C = G.carrier;
set H = G with (carrier := C);
```

Review point:

- field/property access は readable に保つ。
- `qua` は intended view の記録であり、inference failure ではないので source-level syntax として残す。
- selector syntax と namespace syntax は source spans と explanation artifacts 付きで解決する必要がある。

### Frame 4.1b.20 - Term Primary Forms: Constructors, Sets, And `the`

Grammar sketch:

```ebnf
term_primary      ::= variable_identifier
                    | "it"
                    | numeral
                    | "(" term_expression ")"
                    | struct_constructor
                    | set_expression
                    | choice_expression
                    | inline_functor_application
                    | template_functor_application
                    | bracket_functor_application ;

struct_constructor ::= struct_name [ type_args ]
                       "(" [ field_assignment_list ] ")" ;
inline_functor_application ::= inline_func_name "(" [ term_list ] ")" ;
template_functor_application ::= functor_symbol template_args
                                 [ "(" [ term_list ] ")" ] ;
bracket_functor_application ::= user_symbol term_list user_symbol
                              | "[" term_list "]" ;
set_expression    ::= "{" [ term_list ] "}"
                    | "{" term_expression "where"
                      typed_var_list [ ":" formula ] "}" ;
choice_expression ::= "the" type_expression ;
```

Example:

```mizar
set p = Point(x: 3, y: 4);
set evens = { n where n is Element of NAT : n mod 2 = 0 };
set witness = the Element of X;
```

Review point:

- これらは ordinary mathematical terms であり、algorithmic runtime constructs ではない。
- `the T` は `T` の non-emptiness evidence に依存する。
- set comprehensions は generator range が set であると分かる場合だけ許可する。

### Frame 4.1b.21 - `sethood` And Fraenkel Safety

Grammar sketch:

```ebnf
mode_property    ::= "sethood" justification ";" ;
set_comprehension ::= "{" term_expression "where"
                       typed_var_list [ ":" formula ] "}" ;
```

Example:

```mizar
definition
  mode FiniteOrdinalDef: FiniteOrdinal is ordinal finite set;
  sethood by ordinal.finite_ordinal_sethood;
end;

set S = { f where f is FiniteOrdinal : f is non empty };
```

Review point:

- `sethood` は mode の全 instances が proper class ではなく set をなすことを証明する。
- type checker は sethood witness または element-of-set range がない場合、`{ f(x) where x is T : P(x) }` を拒否する。
- これにより ATP proof search の前に Tarski-Grothendieck set-theoretic boundary を見える形で保つ。

### Frame 4.1b.22 - Formula Syntax

Grammar sketch:

```ebnf
formula           ::= quantified_formula | iff_formula ;
universal_formula ::= "for" quantified_vars [ "st" formula ]
                      ( "holds" formula | quantified_formula ) ;
existential_formula ::= "ex" quantified_vars "st" formula ;

iff_formula       ::= implies_formula
                      [ "iff" ( implies_formula | quantified_formula ) ] ;
implies_formula   ::= or_formula
                      [ "implies" ( implies_formula | quantified_formula ) ] ;
or_formula        ::= and_formula
                      { "or" ( and_formula | quantified_formula )
                      | "or" "..." "or"
                        ( and_formula | quantified_formula ) } ;
and_formula       ::= not_formula
                      { "&" ( not_formula | quantified_formula )
                      | "&" "..." "&"
                        ( not_formula | quantified_formula ) } ;
not_formula       ::= "not" ( not_formula | quantified_formula )
                    | atomic_formula
                    | "(" formula ")"
                    | "contradiction"
                    | "thesis" ;

atomic_formula      ::= predicate_application
                      | inline_predicate_application
                      | is_assertion ;
is_assertion        ::= term_expression "is" [ "not" ]
                        is_assertion_body ;
```

Example:

```mizar
for x being Element of G st x is invertible holds
  ex y being Element of G st x * y = unit(G);
```

Review point:

- Mizar-style quantified prose は保存する。
- attribute assertion と type assertion は、resolution が分類するまで意図的に surface syntax を共有する。
- `& ... &` や `or ... or` のような repeated connective forms は explicit formula syntax であり、parser accident ではない。

### Frame 4.1b.23 - Term/Formula Boundary And Formula Precedence

Precedence domains:

| Domain | Source of precedence | Review point |
|---|---|---|
| Terms | user operator metadata plus fixed term syntax | parse before overload |
| Atomic formulas | predicates, equality, membership, `is` assertions | bridge from terms to formulas |
| Formulas | fixed hierarchy for `not`, `&`, `or`, `implies`, `iff`, quantifiers | no user-defined formula precedence |

Example:

```mizar
a + b = c + d
:: atomic formula over two parsed terms

not x > 0 & y > 0
:: (not (x > 0)) & (y > 0)

a or b implies c
:: (a or b) implies c

a iff b iff c
:: error: iff is non-associative
```

Review point:

- Atomic formulas は term parsing と formula parsing の境界である。
- `a < b < c` のような predicate-chain notation は、この境界で解決し、term-operator associativity として扱わない。
- これにより grouping は diagnostics で説明可能になり、AI edits に対しても安定する。

### Frame 4.1b.24 - Theorem Item And Status Syntax

Grammar sketch:

```ebnf
theorem_item   ::= [ theorem_status ] theorem_role
                   label_identifier ":" formula
                   [ justification ] ";" ;

theorem_status ::= "open" | "assumed" | "conditional" ;
theorem_role   ::= "theorem" | "lemma" ;
```

Example:

```mizar
open theorem cancellation_candidate:
  for a, b, c being Element of G st a * b = a * c holds b = c;

assumed lemma choice_profile:
  for X being non empty set holds ex x being object st x in X;
```

Review point:

- `open`、`assumed`、`conditional` は source-visible である。未完了または policy-controlled material が ordinary verified theorem material のように見えてはいけないため。
- publication profiles は許容する status を制限できる。

### Frame 4.1b.25 - Proof Statement Syntax

Grammar sketch:

```ebnf
proof      ::= "proof" reasoning "end" ;
reasoning  ::= { annotated_statement } ;
statement  ::= [ "then" ] linkable_statement
             | standalone_statement ;

conclusion ::= ( "thus" | "hence" ) proposition
               justification ";" ;
```

Example:

```mizar
proof
  let x be Element of G;
  assume A1: x = unit(G);
  thus thesis by A1, group.unit_left;
end;
```

Preserved proof vocabulary:

- `let`、`assume`、`given`、`consider`、`take`;
- `thus`、`hence`、`thesis`、`by`;
- `now ... end`、`hereby ... end`、`per cases`、`case`、`suppose`;
- `deffunc`、`defpred`、`reconsider`。

Review point:

- readable Jaśkowski skeleton は optional ではない。新しい artifacts は tools を支えるが、source proof は Mizar mathematics として読めるべきである。

### Frame 4.1b.26 - Proof Statement Details

Grammar sketch:

```ebnf
iterative_equality ::= [ label_identifier ":" ]
                       term_expression "=" term_expression
                       simple_justification
                       ".=" term_expression simple_justification
                       { ".=" term_expression simple_justification } ";" ;

statement ::= [ "then" ] linkable_statement
            | standalone_statement ;
```

Example:

```mizar
A1: f.(x + y) = f.x + f.y by Additive
             .= g.x + f.y by A2
             .= g.x + g.y by A3;

assume A2: f = g;
then f.x + f.y = g.x + f.y by FuncEq;
```

Review point:

- `.=` は justified equality chain であり、assignment ではない。
- `then` は直前の statement への依存を記録する。
- statement-level の `such that` と `and` は labeled assumptions を作り、formula-level の `st` と `&` はひとつの formula の中に残る。

### Frame 4.1b.27 - Citation And Reference Syntax

Grammar sketch:

```ebnf
justification ::= simple_justification | proof | computation_proof ;
simple_justification ::= [ "by" references ] ;

reference ::= label_identifier [ template_args ]
            | qualified_reference [ template_args ]
            | grouped_reference
            | bulk_reference ;

grouped_reference ::= namespace_path ".{"
                      grouped_item { "," grouped_item } "}" ;
bulk_reference    ::= namespace_path ".*" ;
```

Example:

```mizar
thus thesis by group.unit_left, group.{assoc, inverse_left};
then thesis by algebra.group.*;
```

Review point:

- citation repair は小さな source edit のまま保つ。
- bulk/grouped forms は便利でよいが、dependencies を不可視にしてはいけない。artifact が actually used facts を記録する。

### Frame 4.1b.28 - Registrations, Clusters, And Reductions

Grammar sketch:

```ebnf
registration_block ::= "registration"
                         { registration_content }
                       "end" ";" ;

registration_item ::= existential_registration
                    | conditional_registration
                    | functorial_registration
                    | reduction_registration ;

conditional_registration ::= "cluster" label ":"
                             antecedent_adjectives "->"
                             consequent_adjectives
                             "for" type_expression ";"
                             "coherence" justification ";" ;
```

Example:

```mizar
registration
  cluster nonempty_group:
    associative unital -> non empty for Magma;
  coherence by group.nonempty_from_unit;
end;
```

Review point:

- Registrations は Mizar の強みとして first-class に残す。
- syntax には labels と traceability を加え、automatic propagation を reviewable かつ import-filtered にする。

### Frame 4.1b.29 - Reduction Registration Semantics

Grammar sketch:

```ebnf
reduction_registration ::= "reduce" label ":"
                           term_expression "to" term_expression ";"
                           "reducibility" justification ";" ;
```

Example:

```mizar
registration
  let n be Nat;
  reduce NatAddZero: n + 0 to n;
  reducibility by nat.add_zero;
end;
```

Design reason:

- reduction は equality proof に支えられた oriented simplification rule である。
- 右辺は language の simplification order で厳密に小さくなければならず、imported rules が rewrite cycles を作れない。
- normalization は deterministic である。rule selection は specificity-first で、必要なら FQN tie-break を使う。
- classic な unoriented `identify` idea はここでは採用しない。simplifying equivalence は auditable な `reduce` として書く。

### Frame 4.1b.30 - Template Syntax

Grammar sketch:

```ebnf
template_definition ::= definition_block ;
template_parameter_decl ::= definition_parameter_decl ;

let_type ::= "type" [ "extends" bound_type ]
           | type_expression ;

template_item ::= attr_def | mode_def | struct_def
                | pred_def | func_def | algorithm_def
                | theorem_item | registration_item ;

template_args ::= "[" template_arg { "," template_arg } "]" ;
```

Example:

```mizar
definition
  let T be type;
  struct MagmaStr[T] where
    field carrier -> T;
    field binop -> BinOp of T;
  end;
end;
```

Review point:

- Templates は `scheme` の cosmetic replacement ではない。
- parameterized definitions、theorem schemas、structures、registrations、algorithms の共通 syntax である。
- `of` と `over` は readable shorthands として残し、bracket form は identity と tooling のための canonical form にする。

### Frame 4.1b.31 - Template Constraints And Inference

Grammar sketch:

```ebnf
template_args ::= "[" template_arg { "," template_arg } "]" ;
let_constraint ::= "such" "that" formula ;
let_type       ::= "type" [ "extends" bound_type ]
                 | type_expression ;
```

Example:

```mizar
definition
  let F be Field;
  let V be VectorSpace[F] such that V is finite_dimensional;
  mode BasisDef: Basis[V] is finite linearly_independent Subset of V.carrier;
end;

Product[Ring](s)       :: explicit template argument
Product(s)             :: valid only when the argument type determines Ring
M of A, B              :: greedy shorthand for M[A, B] when arity matches
```

Review point:

- bracket arguments は tools のために identity を保存する canonical form である。
- `of` と `over` は readable shorthands として残すが、greedy parsing は deterministic でなければならない。
- `such that` constraint は use-site obligation であり、hidden global assumption ではない。

### Frame 4.1b.32 - Annotation Syntax

Example:

```mizar
@show_type(total + x)

@show_resolution
Product[R](s);

@proof_hint(max_axioms: 32, solver: vampire)
thus result = unit(G) by group.identity_unique;

@latex("\\mathbb{Z}")
func IntSetDef: INT -> set equals IntegerSet;
```

Review point:

- Annotations は source syntax だが、特定の proof-development profile が search hint として扱う場合を除き semantic-neutral でなければならない。
- editors、documentation、proof search、AI agents を導く。
- source に存在するだけで proof evidence にはならない。
- `@show_type(...)` や `@show_resolution` などの fixed diagnostic annotations は state を表示するだけで、declaration に logical content を付与しない。

### Frame 4.1b.33 - Algorithm Syntax Boundary

Example:

```mizar
definition
  let x, y be Integer;

  algorithm max2(x, y) -> Integer
    ensures (result = x or result = y) & x <= result & y <= result
  do
    if x <= y do
      return y;
    end;
    return x;
  end;
end;
```

Review point:

- Algorithms は language surface の一部だが、theorem truth を再定義してはいけない。
- Algorithms は `definition` blocks の中で宣言され、public interface は body ではなく signature と contracts である。
- contracts、invariants、termination measures、`by computation` は、同じ trusted boundary を通る verification obligations を生成する。

### Frame 4.1b.34 - Syntax Migration Review Questions

Questions for Bialystok:

- どの legacy Mizar constructs は cultural / migration reasons によりほぼ同じ syntax を保つ必要があるか。
- どの constructs は current implicit form が dependencies を隠しているため、より explicit にしてよいか。
- `field` / `property` / `attribute` distinctions は実際の algebraic examples で読みやすいか。
- one-parent-per-`inherit` は multiple-inheritance-heavy な MML fragments に受け入れられるか。
- explicit operator-precedence rules は existing Mizar notation users の期待に合うか。
- Fraenkel-style set formation に対して、`sethood` checks は普通の set notation を重くしすぎず十分に見えるか。
- 既存 proofs が暗黙に依存している predicate/functor properties のうち、どれを最初に migrate すべきか。
- oriented `reduce` は implicit simplification や古い identification idioms の readable replacement になっているか。
- `of` と `over` を source shorthands として残すなら、template brackets を canonical identity syntax として受け入れられるか。
- template inference はどこで止め、explicit `[T]` arguments や `qua` views を要求すべきか。
- この surface review から抜けている proof statement forms は何か。

### Frame 4.1c.01 - Detail: Article Environment

- Current: `environ` は各 article の vocabulary、notation、constructors、registrations、requirements、visible theorem base を与える。
- Evo: 同じ役割を explicit imports、prelude policy、package metadata、reproducible dependency reports に分解する。
- Review question: 現行の `environ` roles のうち、migration 中も見た目として残すべきものはどれか。generated reports へ移せるものはどれか。

### Frame 4.1c.02 - Detail: Module Interfaces

- Current: article acceptance と MML inclusion が後続 article から使えるものを決めるが、source file には public/private interface syntax が少ない。
- Evo: `export`、`public`、`private`、opaque imports、separate compilation が reusable API surface を定義する。
- Review question: legacy article facts のうち、public module API にすべきものと private proof support にすべきものはどれか。

### Frame 4.1c.03 - Detail: Symbol Identity

- Current: theorem や constructor の identity は article names、labels、MML inclusion history と結びついている。
- Evo: path-derived FQNs と module-qualified labels により、local import spelling から独立した identity を持たせる。
- Review question: old citations を保存しつつ package/module reorganization を許すには、どの identity metadata が必要か。

### Frame 4.1c.04 - Detail: Symbolic Notation And Aliases

- Current: Mizar は rich mathematical notation を持ち、synonyms と antonyms により alternative phrasing や natural negation を表せる。
- Evo: arbitrary operator symbols は `func` と `pred` に集中し、mode、attribute、structure names は identifier-like に保つ。synonym/antonym equivalence は保存する。
- Review question: どの legacy symbolic constructors には compatibility spelling が必要で、どれは clearer constructor names へ移すべきか。

### Frame 4.1c.05 - Detail: Lexical Activation

- Current: article environment がどの symbols と notations を利用可能にするかを決め、多くの効果は局所テキストから見えにくい。
- Evo: import prelude を pre-scan し、active lexicon を source-position dependent にし、declaration は item 完了後にだけ有効にする。
- Review question: no-forward-reference や dot/operator disambiguation の変化について、migration でどこに generated diagnostics が必要か。

### Frame 4.1c.06 - Detail: Soft Type Foundation

- Current: Mizar の soft type system は、set-theoretic objects 上に重なる中心的な identity feature である。
- Evo: soft typing を保ちつつ、`object`/`set`、radix と mode heads、type erasure、widening/narrowing、`reconsider` obligations を見えるようにする。
- Review question: どの type obligations を source に見せ、どれを verifier metadata に残すべきか。

### Frame 4.1c.07 - Detail: Structures

- Current: structure declarations は parent structure、fields、selectors、constructor layout を compact legacy syntax にまとめる。
- Evo: `struct`、`field`、`property`、`inherit` により、layout、canonical derived data、inherited views を分ける。
- Review question: legacy selector を移行するとき、それは intrinsic structure data か、derived property か、inherited view か。

### Frame 4.1c.08 - Detail: Inheritance

- Current: inheritance は強力だが、inherited field mapping は structure syntax と naming convention から読み取ることが多い。
- Evo: 各 `inherit` statement がひとつの parent relation を記録し、mapping、renaming、narrowing、coherence evidence を含める。
- Review question: diamond inheritance と coherence obligations を、聴衆を圧倒せずに示すにはどの algebraic examples がよいか。

### Frame 4.1c.09 - Detail: Modes And Attributes

- Current: modes と adjectives により、Mizar text は mathematical prose に近くなる。
- Evo: modes は type abbreviations/refinements として残し、attributes は clustering が使う explicit type-refining predicates として扱う。
- Review question: current adjectives のうち、cultural vocabulary として正確に保存すべきものはどれか。

### Frame 4.1c.10 - Detail: Definitions And Correctness

- Current: `func` と `pred` definitions は familiar な `equals` と `means` styles を使い、correctness obligations を伴う。
- Evo: これらの style を保ち、existence、uniqueness、coherence、compatibility、`assume` side conditions を explicit obligations と artifact entries にする。
- Review question: どの correctness obligations を migration diff の中で source-visible にすべきか。

### Frame 4.1c.11 - Detail: Overload And `redefine`

- Current: overloaded symbols と redefinitions は、多くの関連型にまたがる compact mathematical notation を支える。
- Evo: ordinary overload roots、same-root `redefine` families、`coherence with`、refinement joins を分けて記録する。
- Review question: familiar notation だけを見ている user に、changed overload choice を tools はどう説明すべきか。

### Frame 4.1c.12 - Detail: Registrations, Clusters, And Reductions

- Current: registrations と clusters は Mizar の大きな automation strength だが、その局所効果は inspect しにくいことがある。
- Evo: labeled registration items、import-filtered cluster graphs、reduction indexes、replayable traces により propagation と rewriting を説明可能にする。
- Review question: migrated article を信頼するために、どの cluster explanations が必須か。

### Frame 4.1c.13 - Detail: Schemes And Generics

- Current: classical `scheme` と `of`/`over` parameterization は多くの reusable reasoning patterns を担う。
- Evo: first-class templates が parameterized definitions、structures、attributes、functors、predicates、algorithms、theorem schemas を統合する。
- Review question: template-style presentation の最初の migration target にすべき existing schemes はどれか。

### Frame 4.1c.14 - Detail: Declarative Proof Skeleton

- Current: `let`、`assume`、`thus`、`hence`、`thesis`、diffuse reasoning を持つ Jaśkowski-style proof text は Mizar readability の中心である。
- Evo: readable skeleton を保ちつつ、extracted obligations、thesis states、source spans を metadata として出す。
- Review question: proof を tactic-driven に見せずに user を助ける proof-state metadata はどの程度か。

### Frame 4.1c.15 - Detail: Proof Status

- Current: normal publication unit は verified theorem item である。
- Evo: `open`、`assumed`、`conditional` により、unsettled propositions、deliberate assumptions、non-clean material に依存する results を分ける。
- Review question: published material や package-distributed material で non-clean items をどこまで許容する policy がよいか。

### Frame 4.1c.16 - Detail: Proof Citations

- Current: `by` citations は visible facts を名前で参照し、proof steps を compact に保つ。
- Evo: grouped citations、bulk citations、used-axiom recording、citation refinement が small verifier-checked repairs を支える。
- Review question: migration tool はいつ citations を minimize し、いつ author の original citation style を保存すべきか。

### Frame 4.1c.17 - Detail: Type Views

- Current: `qua`、widening、narrowing、local disambiguation は、user が intended type views を選ぶために使われる。
- Evo: source-level `qua` を保ち、inserted coercions、selected views、overload metadata を inspectable artifacts にする。
- Review question: migration 中にどの implicit choices を source に出し、どれを hover/explanation data に残すべきか。

### Frame 4.1c.18 - Detail: Algorithms

- Current: Mizar は主に mathematical proof language であり、general verified programming surface ではない。
- Evo: `algorithm`、contracts、invariants、termination measures、MVM execution、`by computation` が verified computation と proof を接続する。
- Review question: mathematical proof language から注意を逸らさずに価値を示せる computational examples はどれか。

### Frame 4.1c.19 - Detail: Annotations

- Current: comments と informal tool conventions は reader を助けるが、stable verifier-facing interface ではない。
- Evo: semantic-neutral annotations が proof search を導き、inferred state を表示し、notation を render する。ただし theorem truth は変えない。
- Review question: どの annotations は標準化するほど有用で、どれは external tooling に残すべきか。

### Frame 4.1c.20 - Detail: Diagnostics And LSP

- Current: verifier output は主に human-facing text である。
- Evo: stable diagnostic codes、primary/secondary spans、fix suggestions、lazy explanation artifacts、LSP records により errors を actionable にする。
- Review question: current MML maintainers の migration friction を最も下げる diagnostic explanations はどれか。

### Frame 4.1c.21 - Detail: ATP Boundary

- Current: proof automation は有用だが、trusted boundary は portable artifact protocol として提示されていない。
- Evo: ATPs が search し、certificate artifacts が evidence を記録し、kernel がその evidence を独立に accept/reject する。
- Review question: どの certificate failures は user-facing にすべきで、どれは implementation/debugging details か。

### Frame 4.1c.22 - Detail: Packages And Dependency Resolution

- Current: MML は articles と library inclusion を中心に組織される。
- Evo: `mizar.pkg`、lockfiles、SemVer、features、compatibility checks、cached artifacts が reproducible package-oriented development を支える。
- Review question: standard library、published articles、third-party packages、local experiments は namespace policy 上どこに置くべきか。

### Frame 4.1c.23 - Detail: Incremental Verification

- Current: accepted article output が主な reuse boundary である。
- Evo: dependency slices、VC anchors、witness hashes、cache keys により reuse を可能にする。ただし cache reuse は proof authority ではない。
- Review question: incremental verification を信頼するためには、どの clean-build equivalence tests が必要か。

### Frame 4.1c.24 - Detail: Documentation Generation

- Current: source comments、MML browsing、Formalized Mathematics pages は関連しているが別々の reading needs を満たす。
- Evo: `mizar doc` は verified artifacts、labels、FQNs、documentation comments、`@latex` annotations を使って API documentation を生成する。
- Review question: どの documentation warnings は optional lint でよく、どれは publication profiles を block すべきか。

### Frame 4.1c.25 - Detail: Code Extraction

- Current: Mizar articles から verified executable code を抽出する general source-level workflow はない。
- Evo: terminating algorithms、ghost erasure、target-neutral runtime IR、extractor configuration により、executable output を downstream artifact にする。
- Review question: first extraction benchmarks に適した target languages と mathematical domains は何か。

### Frame 4.1c.26 - Detail: Formalized Mathematics Link

- Current: article identity、publication exposition、library reuse は密接に接続している。
- Evo: publication metadata が articles を reusable library modules、package versions、stable theorem identities にリンクする。
- Review question: citation value を保存しながら reusable library を独立に進化させるにはどうすべきか。

### Frame 4.2 - Current Article Environment の exact excerpt

現行 MML からの exact excerpt:

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

Speaker note:

- Source: current MML `algstr_0.miz`, lines 15-25。
- `environ` が悪いと言いたいのではない。
- dependency classes を explicit module/package data に翻訳する必要がある、という話。

### Frame 4.3 - Mizar Evo Import Prelude

```mizar
import std.algebra.structure.sorted;
import std.function.binop;
import std.algebra.structure.magma;
```

主な違い:

- file-scoped import prelude。
- package/module namespace。
- import closure が build-system input になる。
- path と module に由来する stable FQNs。

設計理由:

- prelude を file-scoped にすることで、lexing、disambiguation、AI context extraction が body を読む前に安定した imported base environment を一度だけ構築できる。
- どの import が symbol、notation、registration、theorem を提供したかを tool が再現可能に説明できる。

### Frame 4.4 - Environment から Imports への移行表

| Current Mizar role | Evo target |
|---|---|
| vocabularies | exported symbols and lexical metadata |
| notations | imported notation metadata |
| constructors | visible definitions and constructors |
| registrations | import-scoped registration index |
| requirements | package or prelude policy |
| article labels | module-qualified theorem identities |

### Frame 4.5 - 単なる rename ではない理由

箇条書き:

- Current environments は複数の dependency role を混ぜている。
- syntax-facing、semantic、automation-facing な依存が混在する。
- migration には、各 import module が何を提供するかを説明する report が必要。
- 新しい境界の目的は cosmetic modernization ではなく、旧来の implicit role を package/artifact dependency にする前にレビュー可能にすること。

### Frame 4.6 - Legacy Structure Example の exact excerpt

現行 plain-text `ALGSTR_0` で確認した shape:

```mizar
definition
  struct (1-sorted) addMagma (# carrier -> set, addF -> BinOp of the carrier
  #);
end;
```

Speaker note:

- Source: current MML `algstr_0.miz`, lines 37-40。
- 短い exact excerpt としてこのまま slide に置ける。speaker note には attribution を残す。

### Frame 4.7 - Evo Structure Example

```mizar
definition
  struct AddMagma where
    field carrier -> set;
    field add -> BinOp of carrier;
  end;

  struct AddLoopStr where
    field carrier -> non empty set;
    field add -> BinOp of carrier;
    property zero -> Element of carrier;
  end;

  inherit AddMagma extends 1-sorted;
end;
```

主張:

- Evo は field declarations と inheritance obligations を明示する。
- ただし構想は Mizar structures に近いまま保つ。
- Evo は `field` と `property` も分ける。field は独立した structure component、property は structure に付随する一意に決まる数学的データまたは obligation である。

設計理由:

- structure inheritance は、どの field が共有・rename・要求されるのかを users が見られる場合に強い。
- obligations を明示すると、migration risk を verifier、LSP、reviewer が議論できる対象にできる。
- fields と properties を分けることで、constructor layout、selector identity、mathematical laws を同じ種類の dependency として扱わずに済む。

### Frame 4.7a - なぜ Field と Property を分けるか

設計上の区別:

| Concept | 意味 | 分ける理由 |
|---|---|---|
| `field` | structure value が持つ内在的 component | constructor shape、selector identity、extensional equality、storage-like artifacts に効く |
| `property` | structure に付随する一意に決まる値または obligation | semantic obligations、coherence、inheritance constraints、proof/artifact fingerprints に効く |
| `attribute` | `empty`、`trivial`、`add-cancelable` などの predicate-style refinement | structure layout ではなく registration/cluster propagation に参加する |

要点:

- carrier や operation は、それが変わると object 自体が変わるので field である。
- zero、unit、degree などの canonical value は、数学的 structure から決まるものとして扱う場合 property にでき、coherence evidence を伴う。
- attribute は stored data でも selector でもなく、automation が伝播できる reusable logical refinement である。
- この分離により、migration tool は「この legacy item は structure data か、proof obligation を伴う canonical value か、伝播される predicate か」を問えるようになる。

Speaker note:

- Source design vocabulary: `doc/spec/en/05.structures.md` は `field` と `property` を分け、`doc/spec/en/06.attributes.md` は attributes を clustering が使う type-refining predicates と定義する。

### Frame 4.7b - なぜ `struct` と `inherit` を分けるか

設計上の区別:

| Declaration | 何を所有するか | 分ける理由 |
|---|---|---|
| `struct` | local type constructor、fields、properties | object 自身の layout と selectors を定義する |
| `inherit` | ひとつの parent structure との関係 | field/property mapping、renaming、narrowing、coherence evidence を記録する |

要点:

- structure declaration は「この object は何でできているか」に答える。
- inheritance declaration は「この object をあの parent としてどう見るか」に答える。
- 分けることで multiple inheritance が明示的になり、各 parent が監査可能なひとつの `inherit` statement を持つ。
- diamond inheritance は declaration order の隠れた副作用ではなく、source span と coherence obligation を持つ diagnostic problem になる。
- artifact fingerprint も明確になる。field layout の変更と inherited view/coherence proof の変更は同じ event ではない。

Speaker note:

- Source design vocabulary: `doc/spec/en/05.structures.md` は `inherit` statement ごとに parent をひとつにし、renaming、narrowing、coherence には explicit `where` block を使う。

### Frame 4.8 - Renaming And Views

```mizar
definition
  struct Magma where
    field carrier -> set;
    field binop -> BinOp of carrier;
  end;

  inherit AddMagma extends Magma where
    field carrier from carrier;
    field add from binop;
  end;
end;
```

目的:

- additive view と multiplicative view が構造を共有する library organization を、隠れた命名規約だけに頼らず表現する。
- 同じ carrier を異なる view で使う数学的習慣を保ちつつ、view translation を checked source として記録する。

### Frame 4.9 - Diamond Inheritance を診断の機会にする

```mizar
inherit DoubleLoopStr extends AddLoopStr;
inherit DoubleLoopStr extends MulLoopStr;
```

要点:

- 二つの inheritance path が一致するか、衝突するか、coherence proof が必要かを説明すべき。
- その説明は users、LSP、AI agents から使えるべき。
- 目標は multiple-view algebra を使いやすく保ちながら、隠れた inheritance order に意味を決めさせないこと。

### Frame 4.10 - Registrations And Clusters

現行 MML からの exact excerpt:

```mizar
registration
  let M be addMagma;
  cluster right_add-cancelable left_add-cancelable -> add-cancelable for
Element
    of M;
  coherence;
end;
```

Speaker note:

- Source: current MML `algstr_0.miz`, lines 104-109。
- attribute propagation を示す短い抜粋として使う。長い registration proof に slide を依存させない。

Evo 側の解釈:

- registration は数学的 mechanism として維持する。
- 実装は import-filtered で説明可能な registration graph を持つ。
- automatic attribute propagation は traceable であるべき。

設計理由:

- registrations と clusters は Mizar の強みなので削除しない。
- 変えるのは、自動 propagation が監査・cache・AI agent への局所提示に使える trace を残す点である。

### Frame 4.11 - Overload Resolution And `qua`

```mizar
consider p be Product(R qua MulStr) such that
  ...
```

要点:

- explicit disambiguation は inference の失敗ではない。
- 意図した mathematical view の記録である。
- AI agent にとっても小さく安全な repair target になる。

### Frame 4.12 - Proof Citation Repair

Before:

```mizar
thus thesis by A;
```

After:

```mizar
thus thesis by A, B, C;
```

現行 MML の proof-citation exact example:

```mizar
theorem
  for F being non degenerated ZeroOneStr holds 1.F in NonZero F
proof
  let F be non degenerated ZeroOneStr;
  not 1.F in {0.F} by TARSKI:def 1;
  hence thesis by XBOOLE_0:def 5;
end;
```

Speaker note:

- Source: current MML `struct_0.miz`, lines 637-643。
- citation repair は bounded local edit として説明する。agent は missing/more precise citation を提案できるが、受理は verifier と kernel evidence が決める。

要点:

- Green AI edit: source-local、verifier-checked、受理されれば theorem statement については意味保存的。
- 後続の refinement pass で、artifacts に記録された used axioms に基づき citation を最小化できる。
- 追加 dependency 自体は重要なので、artifact に記録し、後続の `mizar refine` 的な処理で noise を減らせるようにする。

### Frame 4.13 - Forbidden Repair

悪い AI repair:

```mizar
theorem
  for x be Nat holds x + 0 = x or x = 0;
```

要点:

- theorem statement を弱めるのは Red edit。
- proof を簡単にしても、意図した結果を壊す。
- ordinary AI agents には適用権限を与えてはいけない。できるとしても human unsafe-edit review が必要だと警告するだけに留める。

### Frame 4.14 - Annotations

```mizar
@show_type(total + x)

@show_resolution
Product[R](s);

@proof_hint(max_axioms: 32, solver: vampire)
thus result = unit(G) by group.identity_unique;
```

要点:

- annotations は humans、tools、AI のための structured context。
- informational annotations は logical meaning を変えるべきではない。
- search に影響する hint も proof-development metadata であり、accepted evidence ではない。

### Frame 4.14a - Templates

Evo 側の候補例:

```mizar
definition
  let T be type;
  struct MagmaStr[T] where
    field carrier -> T;
    field binop -> BinOp of T;
  end;
end;

definition
  let M be type extends commutative Magma;
  theorem PermProduct[M]:
    ...
end;
```

要点:

- templates は Mizar Evo の generics mechanism である。
- types、predicates、functors を parameter として definitions と theorem schemas を書ける。
- classical scheme-style reasoning を、別扱いの特殊構文ではなく、同じ template vocabulary に統合する。
- `of` / `over` forms は読みやすい shorthand として残し、bracket form が canonical parameterization を記録する。

設計理由:

- templates により、carrier、field、predicate instance ごとに同じ algebraic construction をコピーせずに済む。
- generic mathematical pattern が dependency fingerprints、theorem indexes、AI retrieval で扱えるほど明示的になる。
- `type extends commutative Magma` のような constraint は、ATP や proof search の前に parameter が何を提供すべきかを示す。

Speaker note:

- Source design vocabulary: `doc/spec/en/18.templates.md` は templates を parameterized definitions と theorem schemas のための generics と定義し、structures、attributes、modes、predicates、functors、schemes を含める。

### Frame 4.15 - Algorithm Verification

Evo 側の候補例:

```mizar
definition
  let x, y be Integer;

  algorithm max2(x, y) -> Integer
    ensures (result = x or result = y) & x <= result & y <= result
  do
    if x <= y do
      return y;
    end;
    return x;
  end;
end;
```

要点:

- algorithms は pure mathematics を超えた対象を扱えるようにする。
- contracts と generated VCs は、同じ verification pipeline に入る。

### Frame 4.16 - Compatibility Metadata

```mizar
@[origin("mizar:ALGSTR_0:addMagma")]
definition
  struct AddMagma where
    ...
  end;
end;
```

要点:

- migration は historical identity を保存すべき。
- origin metadata は Formalized Mathematics links、citation stability、review に効く。

### Frame 4.17 - Bialystok からの入力が必要な点

質問:

- どの legacy constructs は直接 syntax preservation が必要か。
- どの constructs はより明確な Evo forms へ翻訳すべきか。
- MML migration の origin identifiers として何が十分安定か。
- 現行 environment dependencies のうち、説明が最も難しいものは何か。

## Part 5. Architecture

### Frame 5.1 - Pipeline Overview

```text
Source
  -> TokenStream
  -> SurfaceAst
  -> ResolvedAst
  -> TypedAst
  -> CoreIr
  -> VcIr
  -> AtpProblem
  -> ProofCertificate
  -> VerifiedArtifact
```

要点:

- 各 layer は異なる責務を分離する。
- 分離の理由は diagnostics、caching、trust、reproducibility。
- これは一般的な compiler diagram ではない。各境界は、誰が fact を所有し、どの artifact が記録し、何が変わったときに再計算が必要かを示す。

### Frame 5.2 - Responsibility Split

| Layer | Responsibility |
|---|---|
| frontend | source、lexing、parsing、recovery |
| resolver | imports、names、labels、namespaces |
| checker | soft types、clusters、registrations、overloads |
| elaborator | core logical representation |
| VC generator | proof and algorithm obligations |
| ATP | evidence の探索 |
| kernel | replay/checking による受理 |
| artifact emitter | tools と dependents 向けの stable outputs |

### Frame 5.3 - Reasoning Boundary

```text
Mizar-side semantics
  -> well-typed, resolved obligations
ATP-side search
  -> candidate proof evidence
kernel-side checking
  -> accepted or rejected proof status
```

要点:

- ATP は name resolution、type inference、cluster expansion、overload decision をしない。
- kernel は proof search をしない。
- deterministic pre-ATP discharge も replayable evidence または明示的な policy status が必要であり、前段階が「完了」と言っただけでは受理されない。

設計理由:

- すべての reasoning を Mizar 内に置くと trusted verifier が大きくなる。
- すべてを ATP に委譲すると Mizar-specific semantics の制御が弱くなる。
- hybrid boundary は semantic processing を deterministic に保ち、ATP search を強力に使い、最終受理を独立に検査可能にする。

Status distinction:

| Status | Meaning |
|---|---|
| `kernel_verified` | replay/certificate-check 済み evidence を受理 |
| `discharged_builtin` | 同じ evidence discipline で deterministic discharge を受理 |
| `externally_attested` | backend/policy attestation の記録であり verified と同等ではない |
| `open` / policy status | 未完了または policy-controlled。silent promotion しない |

### Frame 5.4 - SAT-Based Small Kernel

説明:

- high-level proof search には ATP を使える。
- accepted evidence は certificate data に正規化される。
- kernel は imported facts、substitutions、alpha-conversion、clause well-formedness、resolution/SAT evidence を検査する。
- search の unsoundness が accepted result の unsoundness にならないようにする。
- SAT 部分は solver の success bit を信頼する話ではなく、proof evidence を検査する話である。backend-specific proof は受理前に replayable certificate data へ翻訳される必要がある。

### Frame 5.5 - Certificate Data

```text
Certificate
  target VC fingerprint
  kernel profile
  imported facts and hashes
  generated clauses
  substitutions
  resolution trace
  final goal
```

要点:

- certificate は proof を specific obligation と dependency slice に結びつける。
- deterministic で replayable である必要がある。
- そのため certificate は hash と kernel profile を持つ。ある dependency/policy context で受理された proof が、別 context に黙って移動してはいけない。

### Frame 5.6 - Kernel がしてはいけないこと

箇条書き:

- proof search しない。
- premise selection しない。
- name resolution しない。
- overload resolution しない。
- cluster expansion しない。
- hidden global state を参照しない。
- raw ATP success を信頼しない。

### Frame 5.7 - Cluster And Registration Architecture

要点:

- Mizar の registrations は強みだが、スケール時には traceability が必要。
- Evo は registration と cluster data を indexes に保存する。
- active module は import-filtered view を受け取る。
- explanation artifacts が、ある fact がなぜ導かれたか、または導かれなかったかを説明する。

### Frame 5.8 - Memory Model

```text
resident memory should scale with:
  active source
  imported public interfaces
  import-filtered indexes
  active module VCs

not with:
  imported proof bodies
  private lemmas outside the interface
  unused registration data outside the import closure
```

### Frame 5.9 - Incremental Verification

要点:

- source、dependency interfaces、generated VCs、proof witnesses を hash する。
- fingerprints と verifier policy が合う場合だけ reuse する。
- public statements と statuses が変わらない proof-body-only changes は、無関係な importers を rebuild しない。

### Frame 5.10 - Parallel Verification

要点:

- 独立した modules、obligations、ATP runs、kernel checks を parallelize する。
- diagnostics、artifacts、proof statuses は canonical order で公開する。
- runtime completion order が semantics を決めてはいけない。

### Frame 5.11 - Artifact Model

Artifacts の用途:

- downstream package verification。
- IDE hover、diagnostics、go-to-definition。
- AI context extraction と patch verification。
- documentation generation。
- Formalized Mathematics article links。
- reproducible build records。

### Frame 5.12 - LSP And MCP

要点:

- LSP は syntax と semantic artifacts から editor feedback を提供する。
- planned MCP-style resources/tools は bounded verifier context を AI agents に公開する。
- どちらも verifier を再実装するのではなく artifacts を消費する。
- wire protocol 自体が設計の中心ではない。中心は、agent が bounded、typed、auditable な context を読み、verification を迂回できないこと。
- 現時点で固定された policy は Green/Yellow/Red edit classification と authorization scopes であり、詳細な wire schema は roadmap material である。

### Frame 5.13 - Safe AI Edit Classes

| Class | Examples | Policy |
|---|---|---|
| Green | citation 追加、`qua` 挿入、情報 annotation 追加 | auto-proposed 可。ただし検証は必要。 |
| Yellow | import 追加、local lemma、registration、invariant | 提案可。検証成功後も human review。 |
| Red | axiom 追加、theorem 弱化、definition 変更、kernel 緩和 | ordinary AI agents には禁止。 |

### Frame 5.14 - Test Strategy

中心原則:

```text
Reject what must not pass
before
Accept everything that should pass
```

発表上の言い方:

- parser gaps は困る。
- soundness bugs は危険。
- kernel-adjacent tests は malformed/failing inputs を重視する。
- trust boundary 付近では test mix を意図的に非対称にし、invalid evidence をまず拒否し、その上で accepted-language coverage を広げる。

### Frame 5.15 - Architecture Questions

質問:

- SAT certificate story は Mizar-style proofs に対して説得力があるか。
- Mizar team が audit しやすい proof evidence format は何か。
- 実際の migration experiments のために最初に作るべき artifact はどれか。

## Part 6. MML Migration Roadmap

### Frame 6.1 - Roadmap Thesis

```text
MML migration is a research program, not a one-shot translation.
```

### Frame 6.2 - 2026年9月時点の目標

訪問後の成果物:

- migration benchmark articles の優先リスト。
- compatibility metadata needs への合意。
- language changes への review notes。
- first paper outline。
- objections and risks の共有リスト。

### Frame 6.3 - 2026年末 Alpha Target

可能な alpha scope:

- meaningful core subset の frontend/parser。
- import prelude と module resolution prototype。
- structured AST and diagnostics。
- basic type and registration scaffolding。
- test harness and corpus layout。
- parsed/resolved source の early artifact format。

Non-goals:

- full MML verification。
- final compatibility layer。
- stable AI protocol。
- complete Formalized Mathematics workflow。

### Frame 6.4 - 2027 Migration Laboratory

Focus:

- representative MML articles を 3-5 本選ぶ。
- 手作業と scripts で翻訳する。
- mismatch を classified issue として記録する。
- side-by-side reports を作る。

Issue classes の候補:

- syntax translation。
- environment-to-import mapping。
- structure inheritance。
- registration or cluster behavior。
- overload behavior。
- proof citation or ATP behavior。
- documentation and origin metadata。

### Frame 6.5 - 2027-2028 Library Expansion

候補順序:

1. foundational set and relation fragments。
2. functions and binary operations。
3. algebraic structures。
4. selected theorem-heavy articles。
5. 成功した fragments の broader dependency cones。

### Frame 6.6 - Migration Metrics

測るもの:

- translated LOC and article count。
- accepted parser subset。
- resolved imports versus unresolved dependencies。
- generated proof obligations。
- deterministic reasoning で閉じた obligations。
- ATP + kernel certificate で閉じた obligations。
- module ごとの memory and build time。
- human review を要した compatibility decisions の数。

### Frame 6.7 - Compatibility Policy

候補 policy:

- theorem identity は origin metadata で保存する。
- migration に役立つ場合は source-visible compatibility aliases を残す。
- clear module/artifact semantics を妨げる legacy forms は、無理に保存しない。
- old behavior との差分は、理由と test を添えて記録する。
- compatibility は、すべての surface form への郷愁ではなく、mathematical identity、review value、reproducibility から説明する。

### Frame 6.8 - Migration Risk Table

| Risk | Mitigation |
|---|---|
| syntax compatibility が project を食い尽くす | full automation ではなく representative slices から始める |
| registrations の挙動がずれる | trace and explanation artifacts を早期に作る |
| proof search が current verifier と違う | used facts と proof obligations を比較する |
| package layout が Formalized Mathematics links を壊す | origin metadata と article-to-library identifiers |
| AI edits が migration mistakes を隠す | Red edits を禁止し、verifier artifacts を必須にする |

### Frame 6.9 - Bialystok からの入力

質問:

- 小さいが構造的に代表性のある MML articles はどれか。
- registrations と structures に負荷をかける article families はどれか。
- technical convenience だけでなく文化的に重要な current Mizar idioms は何か。
- どんな migration result があれば、community は Evo を serious だと受け止めるか。

## Part 7. Formalized Mathematics

### Frame 7.1 - Publication Thesis

```text
The verified library and the published article should be linked but not forced
to have the same structure.
```

### Frame 7.2 - なぜ Library と Article を分けるか

Library modules が最適化するもの:

- reuse。
- dependency control。
- package versioning。
- stable machine-readable artifacts。

Formalized Mathematics articles が最適化するもの:

- exposition。
- narrative。
- citation。
- peer review。
- reader-facing mathematical structure。

### Frame 7.3 - Link Model

```text
Formalized Mathematics theorem
  -> library package
  -> module path
  -> theorem FQN
  -> statement fingerprint
  -> verified artifact hash
  -> rendered documentation page
```

### Frame 7.4 - Link Record の例

```text
article: Formalized Mathematics, future volume
section: Unique identity element
library_object: std.algebra.group.identity_unique
package: std_algebra
version: 0.4.0
statement_hash: sha256:...
artifact_hash: sha256:...
origin: mizar:GROUP_1:...
```

設計理由:

- これらの identity layer は意図的に分ける。
- article label は scholarly citation と narrative structure を支える。
- origin id は MML と既存 Formalized Mathematics の歴史的連続性を保つ。
- library FQN は現在の reusable theorem object を指す。
- statement fingerprint は semantic drift を検出する。
- artifact hash は exact build の reproducible verification を支える。

### Frame 7.5 - 利点

Readers にとって:

- article prose が主役であり続ける。
- formal source へ移動できる。
- proof status と dependency information を確認できる。

Maintainers にとって:

- library refactoring が article exposition を直ちに書き換えなくてよい。
- versioned links が reproducibility を保つ。
- origin metadata が historical continuity を保つ。

AI にとって:

- article prose が semantic search text になる。
- formal links が exact theorem identities を与える。
- agents は explanation と machine-checkable context の両方を取得できる。

### Frame 7.6 - Open Formalized Mathematics Questions

質問:

- stable identity は library FQN、article label、origin id、artifact hash のどれを中心にすべきか。
- user-facing citation で primary にする identity と、machine-facing reproducibility check に残す identity をどう分けるべきか。
- article revisions は library refactoring をどう追跡すべきか。
- Formalized Mathematics は generated HTML、PDF、または両方を publish すべきか。
- 旧 Formalized Mathematics articles は migrated library modules にどうリンクすべきか。

### Frame 7.7 - Possible Workflow

```text
library theorem verifies
  -> verified artifact emitted
  -> documentation generated
  -> article references formal objects
  -> publication build checks links and versions
  -> article published with formal links
```

## Part 8. Discussion And Requests

### Frame 8.1 - Bialystok チームにレビューしてほしいもの

箇条書き:

- language identity and compatibility。
- MML migration examples。
- registration and cluster semantics。
- proof evidence and kernel boundary。
- Formalized Mathematics link model。
- roadmap realism。

### Frame 8.2 - Concrete Next Steps

次の作業候補:

1. migration benchmark articles を選定する。
2. 正確な old/new code comparison slides を作成する。
3. 初期 MML migration report template を書く。
4. Bialystok review 後にこの詳細 Beamer deck を refine する。
5. 同じ main claims と review feedback で paper outline を開始する。

### Frame 8.3 - Closing

```text
Mizar Evo should be modern where scale demands it,
and conservative where Mizar's mathematical identity depends on it.
```

## Backup A. 準備済み exact examples

Beamer 化に向けて準備済みの短い exact excerpts:

| 目的 | Source | Lines | Migration point | Evo sketch |
|---|---|---:|---|---|
| Article environment | `algstr_0.miz` | 15-25 | article-level dependency classes を explicit import/package roles へ翻訳する。 | `import std.function.binop; import std.algebra.structure.sorted;` |
| Structure definition | `algstr_0.miz` | 37-40 | structures を保ちつつ、intrinsic fields、canonical properties、inheritance obligations を分けて見えるようにする。 | `field carrier`、`field add`、必要に応じて `property zero` / `property unit` |
| Registration/cluster propagation | `algstr_0.miz` | 104-109 | registrations は維持し、propagation を traceable / import-filtered にする。 | registration graph entry plus resolution trace artifact |
| Proof citation | `struct_0.miz` | 637-643 | citation repair を小さな verifier-checked edit として扱う。 | Green edit: `by` citations の追加・精密化。artifact に used facts を記録 |
| Publication link | `contents.html` | 516-517 | scholarly article metadata と reusable MML/library identifier を接続する。 | Formalized Mathematics article label + `origin: mizar:GROUP_1` + theorem FQN/fingerprint |

Source URLs:

- `ALGSTR_0`: <https://mizar.uwb.edu.pl/version/current/mml/algstr_0.miz>
- `STRUCT_0`: <https://mizar.uwb.edu.pl/version/current/mml/struct_0.miz>
- Formalized Mathematics contents:
  <https://mizar.uwb.edu.pl/fm/contents.html>

残る Beamer 準備:

- visible slide に raw source URL、短い citation label、または両方のどれを出すか決める。
- MML source licensing の attribution note を最後に追加する。
- slide の余白が厳しい場合、exact source URLs は speaker notes に移し、visible slide には article names と line numbers だけ残す。

## Backup B. 図の一覧

必要な図:

1. Current Mizar workflow: source -> Accommodator -> Verifier -> Exporter -> MML。
2. 三本柱: readability、AI-readiness、scalability。
3. Environment-to-import migration。
4. Structure inheritance and diamond coherence。
5. Full compiler/verifier pipeline。
6. Reasoning boundary: Mizar semantics / ATP search / kernel checking。
7. Certificate object and SAT/UNSAT checking。
8. Incremental dependency and fingerprint graph。
9. AI patch verification flow。
10. Formalized Mathematics article-to-library link model。
11. Roadmap timeline。

## Backup C. 論文アウトラインの種

論文タイトル候補:

```text
Mizar Evo: Readable, AI-Ready, and Scalable Formal Mathematics
```

章立て候補:

1. Introduction: なぜ今 Mizar の evolution が必要か。
2. Mizar as baseline: readability、MML、Formalized Mathematics。
3. Design principles。
4. Language evolution and compatibility。
5. Verifier architecture and small kernel。
6. AI-safe proof development。
7. Package-based library and publication workflow。
8. Migration plan and evaluation metrics。
9. Related work: current Mizar、Lean、Isabelle/Isar、ATP-integrated systems。
10. Conclusion and collaboration agenda。

## Backup D. レビューチェックリスト

Beamer 化前に確認すること:

- 現行 practice への批判が、その practice の有用性も認めているか。
- Lean comparison が dismissive に聞こえないか。
- architecture claim が既存または計画中の artifact に対応しているか。
- code example が exact、schematic、proposed のどれか明記されているか。
- structure slides が `field`、`property`、`attribute` を分ける理由を説明しているか。
- inheritance slides が `struct` と `inherit` を別宣言にした理由を説明しているか。
- template slides が generic definitions と theorem schemas に first-class mechanism が必要な理由を説明しているか。
- Red AI edits が明確に禁止されているか。
- MML migration claims が測定可能か。
- Formalized Mathematics section が publication の価値を保っており、library indexing だけの話になっていないか。
