# Module: symbols

> 正本は英語です。英語版: [../en/symbols.md](../en/symbols.md)。

Status: task R-019〜R-021 は resolver-owned な signature collection 経路を仕様化し
実装する。R-020 は explicit declaration projection 上の collection、
duplicate/conflict detection、registration indexing、overload grouping を実装した。
R-021 は parser-backed な kind ごとの projection extraction、parser-owned な
opaque signature payload、exported lexer-visible spelling 用の resolver-local
module lexical summary index を追加した。semantic `.miz` corpus coverage と
traceability metadata は task R-023 に残る。

## 参照

この設計は、resolver-owned な symbol と signature の契約を以下から導く:

- architecture 03 Step 5「Collect Signature Environment」。
- architecture 03「Signature Collection Is a Declaration Pass, Not Type
  Checking」。
- architecture 01 の `SymbolEnv` / `ResolvedAst` 境界。
- spec chapter 5、6、7、9、10 の structure、attribute、mode、predicate、functor。
- spec chapter 11 の synonym、antonym、visibility、import、conflict behavior。
- spec chapter 12 の module interface、export visibility、public algorithm
  signature。
- spec chapter 16、18 の theorem / lemma、scheme、template declaration surface。
- spec chapter 17 の registration label と registration declaration surface。
- spec chapter 19 の overload candidate construction と checker-owned overload
  winner selection。
- spec chapter 20 の algorithm signature、contract、signature/body visibility split。
- spec chapter 22 の diagnostic payload requirements と現在の resolver-code
  `spec_gap`。
- resolver-local な `env.md`、`resolved_ast.md`、`declarations.md`、
  `imports.md`、`names.md`、`labels.md`。

## 目的

symbols phase は、import、declaration shell、name lookup、label lookup が
source-shaped な resolver fact を生成した後で、resolver-visible な declaration
signature を収集する。checker と downstream module resolution が消費する
`SymbolEnv` index を構築し、未解決、曖昧、malformed、recovered な declaration
fact を明示的に保持する。

入力:

- 現在の module の `ResolvedAst`。
- `declarations.md` 由来の local `DeclarationShellSet` と
  `ExportProjectionShell` record。
- 先行 resolver stage 由来の namespace、symbol-name、import、export、label
  projection。
- source-backed fixture または in-memory summary 由来の dependency module projection。
- `mizar-syntax` が所有する syntax recovery marker と source range。

出力:

- stable `SymbolId` を持つ local / visible `SymbolIndex` entry。
- symbol identity を key にした `DefinitionIndex` record。
- resolver-visible overload family の `OverloadIndex` group。
- checker activation 前の `RegistrationIndex` entry。
- resolver-owned な synonym、antonym、redefinition relation record。
- type checking なしで発見された `DeclarationDependencyIndex` edge。
- 必要な in-memory summary shape が利用可能な場合の dependency-facing
  `ModuleSummary` projection。
- downstream active lexical environment 用の module lexical summary contribution。

## 境界

signature collection は declaration pass である。実行してよいこと:

- 表現可能な semantic declaration に stable symbol identity を割り当てる。
- 言語仕様の public/private default を適用する。
- source spelling、notation spelling、syntactic arity、declaration parameter、
  contract、source-level dependency mention を登録する。
- overload set を形成できない duplicate declaration を拒否する。
- illegal overload group と name-level malformed relation target を記録する。
- public symbol、label、lexical spelling の deterministic exported projection を構築する。

実行してはならないこと:

- expression、term、type の正しさを推論または検証する。
- overload winner を選択したり、inferred type で candidate を順位付けしたり、
  coercion を挿入する。
- registration を発火し、cluster closure を計算し、具体的な term への
  applicability を決める。
- structure type によって selector access を検証する。
- definition、registration、algorithm、theorem、lemma、scheme を証明する。
- algorithm body を lower し、verification condition を計算し、algorithm
  termination を判定する。
- parser syntax、build-owned module discovery、driver orchestration、artifact
  schema を創作する。
- R-G001 が未解決の間に public user-facing resolver diagnostic code を創作する。

## Symbol-bearing Shell

R-011 は semantic declaration と context-only shell の両方を収集する。R-019 は
R-020 が id を割り当てる前に、symbol-bearing shell を分類する。

symbol-bearing shell には、module semantic declaration を提供する表現済み
declaration が含まれる: structure、mode、attribute、predicate、functor、algorithm、
theorem / lemma result、scheme/template、registration、synonym、antonym、
redefinition、structure member。

proof-local inline `deffunc` / `defpred` abbreviation は local resolver binding
shell であり、module symbol ではない。proof-scope lookup のために表現してよいが、
後続の human-reviewed specification が別の local-binding identity family を定義しない限り、
exported または module `SymbolId` を受け取らず、module `SymbolIndex` を構築せず、
module lexical summary を seed しない。

context-only shell はそれ自体では `SymbolId` を受け取らない:

- definition、registration、claim、proof の grouping block。
- visibility wrapper、annotation wrapper、recovered wrapper。
- placeholder、reserve、import、export container item。
- raw parameter、body、statement、expression、recovery node。

context-only shell は symbol-bearing child に structural path、parameter context、
visibility marker、export projection、recovery state を提供してよい。後続の
human-reviewed specification が明示的に semantic declaration としない限り、resolver は
grouping shell の symbol identity を創作してはならない。

## 収集順序

collection は決定的で、declaration point を意識する:

1. local declaration より前に import と dependency projection を読み込む。
2. definition-block / registration-block の structural path を含め、
   local declaration shell を source order で走査する。
3. relation と dependency edge を確定する前に、表現可能な各 declaration へ
   provisional signature shell と stable `SymbolId` を与える。
4. duplicate、conflict、overload-group check は、現在の module の完全な
   source-order declaration inventory 上で実行する。
5. export projection、dependency-facing summary、lexical summary は、local conflict を
   記録した後の public visible surface から構築する。
6. debug rendering、cache-key input、module-summary projection の前に、index を
   canonical order へ整列する。

signature collection は later declaration への forward reference を導入しない。
syntactic target への dependency mention は記録してよいが、name-use visibility は
`names.md` に従う。

## Stable Symbol Identity と Origin

`SymbolId` は `names.md` の preliminary identity projection を完全な declaration
identity へ拡張する。正規化された symbol origin は以下を含む:

- canonical `ModuleId`。
- declaration kind family。
- resolver spelling rule で正規化された primary spelling または notation slot。
- definition-block または registration-block の structural path。
- 対応する structural owner 内の declaration ordinal。
- same-spelling declaration または declaration-owned subitem が合法的に共存できる場合の
  overload、relation、member、contract slot。
- invalidation と provenance grouping のための source contribution id。

canonical identity は formatting change、trivia edit、structural owner 外の無関係な
局所編集で安定しなければならない。source range、`SurfaceNodeId`、session-local
allocation counter は diagnostic provenance にすぎず、それ自体では semantic identity
として十分ではない。

## Signature Shell

収集された各 declaration は `DefinitionIndex` に resolver-level signature shell を保存する。
共通 shell は以下を記録する:

- `SymbolId`、declaration kind、visibility、export status、recovery state。
- primary spelling と optional notation spelling。
- syntactic arity と表現済み parameter/locus slot。
- source structural path と normalized semantic origin。
- syntax trivia から得られる場合の doc/comment attachment id。
- declaration が synonym、antonym、redefinition の場合の relation role。
- syntactic dependency mention と unresolved target placeholder。

signature shell は意図的に浅い。checker/type/proof phase が必要とする syntax は保持するが、
signature が well-typed、semantically compatible、terminating、executable、
proof-valid であるとは主張しない。

## Kind ごとの Signature Shape

| Declaration family | Resolver-owned signature payload |
|---|---|
| structure | constructor spelling、表現済み parent/inheritance mention、field/member name、property name、selector/member structural path、recovery state。field typing と inheritance validity は checker-owned。 |
| mode | mode constructor spelling、存在する場合の label、locus/template slot、表現済み type-parameter surface、radix または parent mode mention、source arity。type expression validity は checker-owned。 |
| attribute | attribute constructor spelling、存在する場合の label、optional parameter prefix、locus/template slot、存在する場合の antonym/synonym role、syntactic target mention。attribute consistency は checker-owned。 |
| predicate | 存在する場合の predicate label、predicate pattern または notation spelling、locus/template slot、syntactic arity、表現済み definiens/proposition anchor。formula typing と logical equivalence は checker/proof-owned。 |
| functor | 存在する場合の functor label、functor pattern または notation spelling、locus/template slot、syntactic arity、optional result-mode mention、表現済み definiens anchor。result typing と coherence は checker-owned。 |
| algorithm | algorithm identifier、terminating modifier、schema/template parameter slot、formal parameter name、optional result type mention、`requires` / `ensures` / `decreasing` contract anchor、body anchor。body lowering、execution、VC generation、termination proof は checker/algorithm/proof-owned。public summary は signature と contract のみを公開し、body は公開しない。algorithm は operator-like lexical spelling を seed しない。 |
| theorem / lemma | 存在する場合の label origin、statement shell anchor、表現済み template/parameter slot、label/name phase で解決済みの cited dependency mention。proof validity と obligation generation は proof-owned。 |
| scheme / template | declared name または label、template parameter slot、formula/term parameter shell、statement shell anchor、dependency mention。instantiation validity は checker/proof-owned。 |
| registration | registration label、registration kind、syntactic target shell、parameter slot、dependency mention、適用可能な visibility/export metadata、recovery state。registration activation と cluster closure は checker-owned。 |
| synonym / antonym | alternate pattern shell、original target mention、relation polarity、arity/notation surface、unresolved または ambiguous target payload。semantic equivalence または negation proof は checker/proof-owned。 |
| redefinition | redefined target mention、replacement signature shell、relation ordinal、compatibility placeholder。compatibility checking は checker-owned。 |
| inline deffunc / defpred | resolver input が表現する場合のみ local proof abbreviation shell。これは proof-scope binding であり、module `SymbolEnv` symbol ではない。exported または module `SymbolId` を受け取らず、module lexical summary を seed せず、exported module symbol にならない。 |

R-020 は explicit declaration projection を使い、R-021 が parser-backed extraction
を行うまで、source shell subitem を最も近い resolver-owned symbol family へ明示的に
畳み込んでよい。`StructureField` は `selector` を使う。property clause と
structure property は extractor が選ぶ projected `attribute` または `selector`
family を使う。predicate/functor/attribute/field/property redefinition は
`redefinition` を使う。表現済み inheritance は extractor が `structure`
projection を提供する場合だけ寄与する。この collapse は `SymbolDeclarationProjection`
内で明示されるため、collector が context shell だけから family を推論することはない。

parser coverage がまだ concrete source role を公開していない場合、R-021 は source
structure を創作せず、payload を opaque のまま pending extraction として記録する。

R-021 は theorem / lemma label、attribute / predicate / functor / mode /
structure pattern、algorithm、notation alias、redefinition、property clause、
structure selector、label 付き registration の、表現済み parser-backed source role
を抽出する。direct declaration child である template parameter は named payload role
として保持し、parser pattern node の下に nest する template loci は、syntax が専用の
declaration-owned role を公開するまで flattened parser-owned signature surface 内で
保持する。現在の parser/syntax 境界には module-level scheme declaration shell がないため、
resolver extraction は scheme declaration を external source-role dependency gap として扱い、
scheme/template module symbol を創作しない。

## Duplicate、Conflict、Overload

duplicate detection は name-level かつ kind-family specific である:

- overload 不可能な declaration が同じ namespace、kind family、spelling を持つ場合、
  duplicate/conflict record になる。
- private と public の declaration も、言語上共存できなければ defining module 内で
  conflict する。
- same-spelling import は、qualification または aliasing で conflict が解けるかどうかを
  semantic lookup が判断できるまで candidate として visible に残る。
- recovered declaration は spelling と kind family が表現されていれば degraded
  symbol / definition fact と `RecoveredShell` metadata を保持してよいが、duplicate
  または illegal-overload diagnostic には参加しない。

overload group は、その family が overloadable で、利用可能な syntax から checker なしで
互換な resolver-owned grouping key が得られる場合にだけ形成できる:

- namespace または module visibility context。
- surface spelling または symbolic notation。
- kind family。
- type checking なしで利用可能な syntactic arity または notation shape。

illegal overload group は crate-local/internal diagnostic と `OverloadIndex` failure
metadata として記録する。resolver は overload candidate を選択、順位付け、書き換えしない。

## Visibility、Export、Summary、Lexical Contribution

symbols phase は `DeclarationShellVisibility` が unspecified の場合、該当する言語仕様章の
default visibility を適用する。spec chapter 11 は ordinary symbol default を定義し、
chapter 12 は definition と algorithm を module interface に配置し、chapter 17 は
registration label と interface behavior を定義し、chapter 20 は algorithm の
signature/body split を定義する。明示的な private declaration は宣言点以降の defining
module 内では利用可能だが、dependency-facing public summary からは除外される。

export projection は name-level interface projection である:

- public local symbol と合法な re-export は exported surface に現れる。
- private declaration は export または re-export されない。
- private、unresolved、ambiguous、malformed な symbol を対象にした export は
  resolver-owned failure metadata を記録する。
- dependency summary は exported symbol / label projection のみを公開する。

この phase が生成する dependency-facing `ModuleSummary` shape は resolver-owned な
interface data だけを含む:

- canonical module identity と in-memory seam 用 summary version。
- exported symbol entry、label projection、relation link、overload group。
- importer と invalidation に必要な declaration dependency edge。
- public symbolic spelling 用の module lexical summary entry。

module lexical summary は downstream active lexical environment に public predicate /
functor notation とその他の lexer-visible user symbol を供給する。private declaration、
algorithm identifier、theorem label、inline proof abbreviation は含めない。
R-024 は artifact-backed summary reuse を独立した adapter として追加し、canonical な
`mizar-artifact` `ModuleSummary` record を消費して、検証済み public surface をこれらの
summary contribution index へ project する。resolver-local artifact schema は定義しない。

R-020 は専用の lexical-summary または artifact-summary data shape を追加しない。
R-021 は resolver-local `ModuleLexicalSummaryIndex` を追加する。parser-backed extractor
が lexer-visible notation token（`UserSymbol` / `LexemeRun`）を eligible として marker し、
collection は export-visible かつ non-recovered な projection だけを seed する。property
keyword、algorithm、theorem label、structure constructor、selector は R-021 の active
lexical summary を seed しない。R-024 は検証済み canonical summary に含まれる exported
lexical entry だけを再利用する。

## Dependency Edge と Relation

`DeclarationDependencyIndex` は signature collection 中に発見された resolver-visible edge を
記録する:

- ある declaration shell から別の symbol、label、unresolved target key への
  signature mention。
- synonym、antonym、redefinition target reference。
- registration target と prerequisite mention。
- body lowering なしで見える algorithm contract mention。
- proof checking なしで見える theorem/lemma statement mention。
- invalidation と diagnostics に必要な export / re-export edge。

edge は dependency kind、source contribution id、use-site range または recovered anchor、
deterministic source/target key を保持する。type-derived dependency、selected overload
winner、cluster firing trace、algorithm execution trace、proof-obligation dependency を
encode してはならない。

## Recovery と Diagnostics

recovered または malformed declaration syntax は、周辺 source shape が表現されている限り保持する:

- usable spelling を持つ recovered declaration は recovered signature shell と
  `RecoveredShell` conflict metadata を受け取ってよい。
- identity data が不足する recovered declaration は shell-only unresolved declaration fact に残る。
- malformed relation target は link を創作せず、unresolved または ambiguous target payload を記録する。
- duplicate / illegal-overload diagnostics は recovered declaration を無視し、
  parser recovery が semantic conflict report へ連鎖しないようにする。

diagnostic record は R-G001 が未解決の間 crate-local/internal に留める。source range、
declaration origin、conflict candidate、relation target、recovery state は保持するが、
public numeric resolver code は割り当てない。

## Determinism

signature collection は同等入力に対して byte-stable でなければならない:

- declaration traversal は source order と structural path に従う。
- symbol id、overload slot、relation ordinal、member slot は canonical grouping key から
  割り当てる。
- candidate list と diagnostic list は normalized semantic origin、kind family、
  source range、declaration ordinal で整列する。
- map は raw hash iteration ではなく sorted projection を通じて render する。
- debug snapshot には session-local address や allocation id ではなく normalized origin と
  stable spelling を含める。

## 公開 enum の前方互換性

task R-026 は frontend task 25 の public-enum decision procedure をこの module に適用する。
`symbols` が所有する公開 resolver enum はすべて forward-compatible API surface であり、
`#[non_exhaustive]` を維持しなければならない:

- `SymbolOverloadPolicy`
- `SymbolDiagnosticClass`

この module は exhaustive な公開 enum 例外を所有しない。下流 consumer は wildcard
または fallback arm を持たなければならない。resolver 内部の match は、仕様化済みの
挙動を実装する範囲で、現在表現されている variant に対して exhaustive でよい。

## Test Obligation

R-019 は documentation-only であり、executable test は追加しない。R-020 は以下の
resolver unit test を追加する:

- opaque declaration signature の `SymbolEnv` への登録。
- 表現済み kind family ごとの duplicate/conflict detection。
- legal / illegal overload grouping。
- symbol / definition / registration index への registration insertion。
- recovered と context-only shell の policy。
- symbol、definition、overload、diagnostic、contribution ordering の決定性。

R-021 は表現済み source role に対する parser-backed な kind-specific signature
extraction、opaque payload 内の template-role preservation、syntax が対応した
parser-backed lexical-summary spelling fixture の resolver unit test を追加する。
R-023 は `declaration_symbol` stage 用の semantic `.miz` corpus case と traceability
metadata を追加する。既存 `.miz` case と expectation は resolver implementation
behavior に合わせる目的で rebaseline してはならない。
