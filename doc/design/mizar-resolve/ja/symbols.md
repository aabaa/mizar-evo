# Module: symbols

`validate_source_symbol_env` は既存の収集処理を再利用して局所記号環境全体の由来を照合する。

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

declaration-site `param_prefix` を持つ attribute pattern では、primary spelling は
`ranked` のような宣言された attribute suffix とし、optional notation spelling は
`2 - ranked` のような concrete pattern surface を保持する。resolver symbol
collection は prefix を checker `AttributeInput` argument へ lower しない。attribute の
exported lexical summary は suffix primary spelling を使う。imported attribute use は
attribute name を解決し、argument は後続の checker-owned payload extraction で運ぶためである。

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
metadata として記録する。exact ordinary-functor classifierはparser-backed
`SymbolKind::Functor` / `DefinitionKind::Functor` pair、または同等のexplicit test
projectionだけを受け入れる。namespace、primary spelling、normalized notation pattern、
normalized definition argument context、syntactic arityでgroup化し、recovered declarationと
別kind pairを除外する。

限定 ordinary-predicate 経路は一意で括弧なしの pattern loci を同じ block の先行する明示的な裸 set/object 仮引数に結び付ける。
private conflict key は namespace・kind・正準化した notation/locus 位置・順序付き引数型・arity を使い、
label・means 本体・束縛変数名に依存しない。公開名・同一性は維持し、未対応型・guard・括弧付き/template pattern・recovery は未対応とする。
predicate に return surface を作らず、実際の全 candidate をソース順に保持した SameSignatureDefinitionConflict を使う。

2件以上のcandidate groupでnormalized return surfaceがすべて同じ場合、resolverはinternal
`SameSignatureDefinitionConflict` diagnostic exactly 1件を出し、全candidate definitionへ
`DeclarationConflictClass::SameSignatureDefinitionConflict`を保存する。distinctなnormalized
return surfaceが複数あるgroupでは、既存`SameSignatureReturnConflict` diagnosticと
definition-conflict metadataがgroup全体で優先し、overlapするsame-return diagnosticは出さない。
両classは全candidateをcanonical source orderで保持し、canonical first candidateへ
diagnosticをanchorする。diagnostic Debug spellingはRust variant名
`SameSignatureDefinitionConflict`、new definition-conflict metadataのexplicit `SymbolEnv`
snapshot spellingはexact `same_signature_definition_conflict`である。

functor profile は syntactic surface check に留まる。predicate locus の正準化を一般の型等価性・import・template signature に広げず、
overload ranking と winner selection は checker が所有する。resolverはoverload candidateを選択、順位付け、書き換えしない。

## Checker Task 263R frozen selector-owner conflict partition

Chapter 5ではfield/propertyは特定structure内で宣言され、inherited memberはroot
declarationとinheritance path/viewの両方を保持する。したがって異なるstructure
declarationに属する同じselector spellingはmodule-level duplicateではない。同じ
structure内ではfield/propertyがmember namespaceを共有するため同名は引き続き
conflictする。

Task 263Rはnon-overloadable `SymbolKind::Selector`のconflict partitionだけを変更する。
keyは`(namespace, spelling, kind, nearest structure declaration owner)`であり、nearest
ancestor shellの`StructureDefinition`をownerとする。そのancestorがないselectorは
explicit missing-owner partitionを使い、従来の保守的name-level conflictを維持する。
non-selector keyはexact `(namespace, spelling, kind)`のまま。redefinition、overload
group、symbol id、definition、visibility、export、signature、contribution effect、
diagnostic orderは変更しない。

required evidenceはextractor-backed resolver 2 caseである。別structureがそれぞれ`field carrier`と
`property marker`を宣言しても`DuplicateDeclaration`を出さない。同一structure内で
field/propertyが同じspellingならexact canonical duplicate diagnostic 1件と両definition
のconflict metadataを維持する。projection order mutationで結果は変わらない。本taskは
type checking、inheritance validation、selector lookup、constructor generation、checker
intake、proof semanticsを行わない。

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

`SignatureProjectionExtractor::collect(module)` は opaque collection を再利用し、局所 functor の
synonym と一意に束縛された bare builtin loci に限り `SymbolDiagnosticClass::SynonymLociMismatch` を生成する。
両 pattern と、先行する完了済み definition block 内の functor 1個を namespace・notation shape・順序付き locus type で照合し、
binder 名・label・本体を constructor 同一性にしない。括弧付き引数列も対象だが、locus 重複・未束縛・
recovery・曖昧または未対応 target はこの診断を生成しない。診断は実際の alias shell/range、元 constructor の
SymbolId、source contribution を保持する。synonym の locus 数不一致は引き続き拒否する。plain binary functor synonym / predicate antonym で束縛 locus 集合が一致し、同じソース内で先に完結した definition block に属する同 namespace・対応 constructor kind の local target が一意な場合だけ既存 RelationMetadata(Synonym/Antonym, target) を生成でき、同数だけでは不足する。
これは元 constructor 同一性と antonym 反転であり証明承認ではない。checker は各 application で順序付き source loci を再照合する。source collection/replay が relation を認証し、opaque collection・数値 code・predicate synonym・alias chain・未対応 target の境界を維持する。antonym 不一致には synonym-loci 診断を与えない。

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

## Checker Task 263R implemented selector partition

frozen correctionはpublic symbol modelを変更せず実装された。各collected selectorは、
存在する場合だけnearest ancestor `StructureDefinition` shellをprivateに記録する。
duplicate collectionはselectorを`(namespace, spelling, kind, selector owner)`で
keyし、他の全symbol kindはowner coordinateを`None`に固定した既存
`(namespace, spelling, kind)` keyを使い続け、ownerless selectorも同じ`None`
owner partitionを共有する。
ownerはconflict-classification dataだけで、`SymbolId`、definition/signature shell、
module summary、lookup、provenance、overload/redefinition rule、recoveryを変更しない。

cross-structure extractor testはprojection orderの両方でdefinition symbol 6件、
conflict 0件を生成する。same-structure field/property testは両orderでstableな
`DuplicateDeclaration` 1件と`DuplicateSpelling` conflict mark 2件を生成する。
existing ownerless/non-selector duplicate matrixは不変。本実装はfrozen
`source_drift`とresolver `test_gap`を閉じるが、structure inheritance、constructor
selection、checker semanticsはTask 263へdeferする。

## Checker Task 264R context-only symbol boundary

`DeclarationShellKind::PropertyImplementation`はTask 264Rでcontext-onlyである。
`SignatureProjectionExtractor`はprojectionを返さず、normal symbol collectionは
property-owned `SymbolId`、`DefinitionId`、`RegistrationId`、overload/redefinition
relation、lexical summary、dependency、contribution effect、diagnosticを作らない。
existing one-per-source `LocalSource` contributionは保存し、anchorはproperty-
implementation shellを無視し、represented semantic shellがなければ`Point(0)`のまま。
context-only property shellはsemantic sibling ordinalに寄与せず、既存siblingの
structural path、`SymbolId`、origin、definition、contribution effectを変えない。
existing enum order/code/keyは不変とし、new variantは`VisibilityWrapper`の後ろに
append-onlyでcode `28`、key `property-implementation`とする。caller-fabricated
projectionはidentityを受けず、既存`ContextOnlyShell` guard/diagnosticを維持する。
`Selector`、`PropertyClause`、`PropertyRedefinition`、
`RegistrationKind::Property`として表現しない。既存siblingのidentity/orderは
exactに保存する。property reference/provenance、implementation domain、return type、
definiens payloadはresolver symbol tableではなく後続checker source producerがownする。

## Checker Task 264R implemented symbol boundary

property-implementation shellはprojectionをextractせず、symbol/definition/registration/
overload/dependency/lexical summary/diagnostic/property-owned contribution effectを作らない。
anchorはleading property shellをskipし、property-only sourceでは`Point(0)`。semantic
sibling ordinalから除外し、recovery theoremの`theorem#1`、`[2, 1]`、`304..334`、
`DefinitionId(1)`、contribution zeroを保存する。fabricated projectionは引き続き
`ContextOnlyShell`でrejectされる。

## Resolver Task 33R Exact Containing-Functor Owner Receipt

Canonical [Task 33R contract](../../task_contracts/ja/RESOLVE-FRAENKEL-CONTAINING-OWNER-33R.md)は
existing symbol/definition allocation boundaryへspecialized zero-semantic association 1件を追加する。
`SourceNestedFraenkelFunctorOwnerProducer`はsurface AST、module、complete structural resolver
arena、exact fingerprinted Fraenkel collectionだけを受け、declaration shells、parser-backed
projections、symbol resultをinternal recollectする。C4C8R common functor identityへ
`resolved_node_for(node_id)`でmapするunrecovered `FunctorDefinition` shell exact 1件だけを選ぶ。

Opaque immutable `SourceNestedFraenkelFunctorOwnerHandoff`はcomplete derivationをretainし、
source/module/fingerprint、definition-block/functor resolved node、declaration shell、final
`SymbolId`/`DefinitionId`、contribution、originのimmutable getterだけを公開する。Non-exhaustive
`SourceNestedFraenkelFunctorOwnerError`はcontract-frozen default-deny class 7件をownし、public
complete/exact-resolver oracleはfixed precedenceでrowをrederiveする。Exact return typeは
`SourceId`、`&ModuleId`、`&str`、`ResolvedNodeId` 2件、`DeclarationShellId`、`&SymbolId`、
`DefinitionId`、`SourceContributionId`、`&SemanticOrigin`、`String`。両validatorは
`Result<(), SourceNestedFraenkelFunctorOwnerError>`を返し、complete signatureとfieldless
error/display ABIはtask contractがownする。

Existing public `SymbolCollector::collect()`は不変でprivate no-target pathへdelegateする。
Specialized producerはauthenticated target shellをsame private pathへ渡し、exact
`CollectedProjection::new` symbol/origin/contributionとinsert中に返る`DefinitionId`をprivate
in-flight allocation rowへcaptureする。Rowはordinary symbol resultと共に返りcomplete oracleが
rederiveする。Finished env searchからassociationを再構成しない。

Associationはexisting `CollectedProjection::new` identity pathとfinal definition indexだけを使い、
display spelling、range、numeric-id reinterpretationは禁止。Token textはexact surface
fingerprint/debugに保持できるがassociation/admission keyではない。Missing/extra/duplicate/reordered/
stale/foreign/recovered/partial/mismatched/corrupt inputをsort/repair/inference/unchecked admission
なしでrejectする。Checker/Typed/Resolved/Core install、semantic result、parameter order、generated
origin、diagnostic、active route、coverage credit、Task277B readinessは追加しない。

Public-enum policy registrationには次も追加する:

- `SourceNestedFraenkelFunctorOwnerError`

## ソース字句宣言と公開シンボルの対応付け

`SymbolCollectionResult::pair_exported_lexical_declarations(locals, map_span)` は
`Option<Vec<(&SymbolEntry, &LocalUserSymbolDeclaration)>>` を返す。呼出側は
同一 frontend ソースの字句宣言を渡し、その前処理マップで字句座標の
`declared_at` を `Option<MappedSourceRange>` に変換する。`Exact` のロード済み
`primary` 範囲のみを使い、Composite/Degraded は全体を拒否する。これは対応関係の
検査であり、ソースの真正性検証・完全なモジュール要約・artifact identity の束縛ではない。

空でない変換済み範囲を包含するシンボル出自範囲は、可視性で絞る前に一意でなければ
ならない。出自と `ContributionKind::LocalSource { source_id }` contribution の
SourceId・モジュールの一致を検査し、
回復済み出自を拒否する。字句側モジュール名は既存 frontend と同様に resolver の
module path と一致させる。種別は predicate・functor・mode・attribute・structure の
同種対応のみ。`Visibility::Public` + `ExportStatus::Exported` は保持し、
`Visibility::Private` + `ExportStatus::LocalOnly` は除外する。
`ReExported` およびその他の組合せ・未対応または欠落した所有者・曖昧さ・変換失敗・不正な範囲・
collection 診断があれば全体を `None` とする。参照を返すため全フィールドと入力順を
保持し、複数の字句片が同一シンボルに対応してもよい。診断がなく入力が空なら空の
ベクトルを返す。resolver の notation から字句情報を推測しない。

別名・独立した演算子宣言・prepass 未対応の種別・完全性・ソースの真正性・artifact
origin の直列化は後続 producer の責務。本メソッドは `locals.user_symbols` のみを
対象とし、他の字句効果を省略した要約の publication を許可しない。

### Frontend 出力に結び付いた対応付け

`SymbolCollectionResult::pair_frontend_lexical_declarations(frontend)` は信頼された
変更のない frontend 実行の `FrontendOutput<SurfaceAst>` を受け取り、下位メソッドと
同じ借用ペアを返す。ソースの package/module、source id、保持された前処理・ローカル
宣言、実 AST 由来の resolver collection の一致を確認する。任意に構築された出力の
認証や意味的な受理は行わない。
AST 欠落、frontend 診断、source id/module 不一致、テキスト・hash・line map の
不整合、マップ登録失敗は拒否する。保持ソースを再前処理して保持結果との等価性を
要求し、字句テキストの再走査から収集した locals と保持宣言も比較する。
公開 raw 走査は frontend の非公開 parser 計画付き走査を再現しないため、
それに起因する不一致も未対応として拒否する。
AST から宣言 shell と signature を再収集し、この collection との等価性を要求する。
ロード、provider 呼出し、parser 再実行は行わない。復元 bridge の字句マッピングを
下位対応付けへ渡し、マッピング状態と既存の除外範囲を維持する。
統合テストは import や合成意味 projection を使わず、実 disk frontend/parser と
ソース由来 collection でコメントによる座標移動、公開・非公開宣言、メタデータ保持、
反復性、不一致・古い入力を検証する。テスト専用 provider は本番 provider ではない。

### ソース字句 contribution producer

`SymbolCollectionResult::export_frontend_lexical_contributions(frontend, identity)`
は frontend に結び付いた直接宣言ペアから `Option<Vec<LexicalContributionSummary>>`
を返す。build 所有の `ModuleSummaryIdentity` を呼出側が渡し、package/module path/
edition は保持ソースと一致する必要がある。identity の検証・JSON 正規化は artifact の
`ModuleSummaryIdentity::canonical_json` が所有し、version/lockfile の出所は呼出側が担う。
standalone operator 宣言、AST notation alias、identity 不一致、frontend 対応付け失敗、
A8 encode 失敗を拒否し、部分的な vector は返さない。各ペアは schema
`mizar-resolve/exported-lexical/v1` の kind `exported-symbol` contribution になる。
key は `entry.symbol().local().as_str()` であり、対応する exported symbol producer は
同じ `origin_id` を使う。spelling/rank/kind/arity/operator は保持し、module/symbol
identity は A10 の envelope 対応文字列に置換する。artifact の JSON と canonical string
writer を用い、末尾改行も保持する。ペア順序、非公開除外による rank の隙間、同一 origin
の複数表記を保持し、key による重複除去や notation からのメタデータ推測はしない。
これは contribution の生成であり、exported symbol 行、完全な module/lexical summary、
fingerprint、import provider、publication ではない。未対応 prepass family と export 方針は
既存所有者に残す。実 frontend の複数表記・非公開・既定 operator payload、identity の
変化と拒否、alias/operator 拒否、codec 上限、決定性をテストする。

## Constructor Actual Roles

将来の constructor-actual resolution は真正な mode/structure signature と順序付き type/value parameter role を消費し、bracket/of/over actual の declaration/scope と expression provenance を保持する。綴りから role を推測せず、型に基づく overload applicability を選ばない。不足/古い declaration input は意味 consumer を block する。

## Planned Algorithm Call Dependencies

既存 signature index は source body-call/definiens graph ではない。将来の真正な call target、完全な definition-block recursive group、source/range/edge provenance を Checker cycle/applicability check と Core lowering へ渡す。Resolver は typed overload winner を選ばず、graph reference を accepted call と扱わない。

## Source registration owner payloads

`source_registration_owner_payloads(ast: &SurfaceAst, source: &SurfaceResolvedArena, symbols: &SymbolEnv) -> Result<BTreeMap<SymbolId, Vec<u8>>, String>` は実 SymbolId を map key に保ち、名前付き owner の bytes を生成する。AST/arena/module の完全対応を先に検証し、既存の環境全体の再照合を行い、その error を保つ。同じ AST から既存 shell/projection を再収集し、caller 文字列、range だけの対応、debug text を owner 内容に代用しない。
対象は top-level DefinitionBlock 内の AttributeDefinition と top-level RegistrationBlock 内の ExistentialRegistration のみ。attribute pattern は optional ParameterPrefix の外に実 constructor token を一つだけ持ち、その綴りが実 projection の primary spelling と一致する必要があり、それ以外は payload を拒否する。各宣言に実 mandatory label、実 projection と一意の source SymbolEntry 対応を要求し、同じ owner kind 内の label 重複、symbol 対応重複、未対応 Attribute/Registration entry は拒否する。他 symbol kind は対象外。失敗は部分 map を返さず Err、対象がなければ空 map。任意の identity data を保留するだけで、名前解決、overload 選択、公開診断は変えない。
value は UTF-8 mizar-resolve/source-registration-owner/v1 と zero byte、kind byte 1(attribute) または 2(existential registration) で始まる。package、module path、実 label の順に checked big-endian u32 の UTF-8 byte length と bytes を並べ、attribute だけ実 constructor primary spelling を同じ framing で追加する。parser spelling を保ち case/Unicode normalization はしない。label 重複の拒否は新 language diagnostic ではない。SourceId/range/node/row/contribution id、structural ordinal、opaque signature、binder spelling、source body、debug provenance は含めない。
module identity と一意の宣言内容に対する限定 cross-edit payload であり、formatting/trivia/raw offset/binder spelling/宣言・block 順序では変わらず、module/kind/label/attribute constructor 変更で変わる。名前解決は引き続き SymbolId を使い、value は raw lookup 文字列でなく未 hash の owner 内容のみ。formula/context/definition body、完全 anchor/dependency、proof reuse、witness/Artifact publication は別 producer が必要で、currentness と実 source 実行由来は caller 責任に残る。
テストは実 Frontend/resolver の single/aggregate source、独立 framing、実 SymbolId 対応、実 allocation relocation・編集・並べ替え・名称/module 変更と、foreign AST/arena/env、recovery、欠落・重複・未対応 owner の拒否、旧結果維持を扱う。新 private setup helper は最大一つ・実三 family 共用。既存 replay、symbol identity、診断、fixture、proof/coverage expectation は変更しない。

## Arena registration owner payloads

`source_registration_owner_payloads_from_arena(source: &SurfaceResolvedArena, symbols: &SymbolEnv) -> Result<BTreeMap<SymbolId, Vec<u8>>, String>` は既存の構造再構築と環境全体の再照合を用い、[AST を受け取る owner producer](#source-registration-owner-payloads) へ委譲する。空結果も含め現在の SymbolId 対応と bytes をそのまま返し、再構築・環境・owner profile の失敗は既存 error とし、部分 map を返さない。raw source text、parser の再実行、caller identity map で不足を補わない。
再構築は保存された実 kind/token text、child 順、range、root/source/module と既存 SurfaceAstBuilder の検証を用いる構造射影であり、元の trivia、expression-root、Frontend storage 全体の復元ではない。owner map は既存 producer と同じ内容のみを表し、source currentness は caller 責任に残る。実行 receipt、formula/VC hash、完全 anchor/dependency、証明受理・reuse・publication は得られない。
テストは実 single/aggregate Frontend AST/arena/env、現在の key と独立 bytes、AST-bound producer との一致、実編集・並べ替え・allocation relocation と内容参加、foreign/augmented env、recovery と未対応 profile の拒否、既存検証と error 維持を扱う。

## Source registration syntax payloads

`source_registration_syntax_payloads_from_arena(source: &SurfaceResolvedArena, symbols: &SymbolEnv) -> Result<BTreeMap<SymbolId, Vec<u8>>, String>` は、現在の存在登録 SymbolId に宣言内の構文内容を対応付ける。既存の構造 AST／環境再構築、AST 側の所有者生成の順に検証し、上流エラーを保持する。同じ AST の shell を既存の `shell_origin`／`shell_context` 全体で一意の Registration に対応付け、実際の所有者マップのキーを要求する。全 Registration を過不足なく返し、他種は無視する。空／属性のみの入力も所有者検証後に空結果を返す。以後の失敗は `unsupported source registration syntax payload` とし、部分結果は返さない。
値は UTF-8 `mizar-resolve/source-registration-syntax/v1` とゼロ、既存の実所有者バイト列の big-endian u32 バイト長と内容で始まる。実際の shell 部分木を子の保存順に preorder で走査する。各ノードは Syntax 所有の明示的 `SyntaxKind` u16 を big endian で出力する。Token はさらにトークン種別 SyntaxKind u16、UTF-8 テキストの big-endian u32 バイト長と原文を出力する。続いて各ノードの big-endian u32 子数と、同じ形式の子を順に出力する。長さ／子数は checked 変換とする。ID・範囲・trivia・Debug／storage 文字列を含めず、部分木内の綴り・大文字小文字・束縛変数名は意味／alpha 正規化せず保持する。
対応する非 Token は unit 種のみ: ExistentialRegistration, TypeExpression, AttributeChain, AttributeRef, QualifiedSymbol, PathSegment, TypeHead, CorrectnessCondition, ProofBlock, TakeStatement, Witness, TermExpression, ChoiceTerm, ConclusionStatement, Proposition, FormulaExpression, IsAssertion, AttributeTestChain, JustificationClause, ReferenceList, Reference。Token は Identifier, ReservedWord, ReservedSymbol, UserSymbol のみ。追加フィールドを失わないよう、演算子・量化・定数・回復を含む他 variant は拒否する。任意の内容生成プロファイルであり、言語受理の制限ではない。木と語彙の責任は Syntax／Parser に残す。
選択した根は外側のブロック／束縛や参照先定義本体を含めず、それらは別の context／formula 生成が扱う。外部属性本体の編集ではローカル値は変わらない。整形・割当・宣言順によらず同じ所有者の値を保持し、package／module・所有者ラベル・部分木内のトークン／形状変更は値に含める。現在性は呼出側責任とし、完全な SourceShapeHash／anchor／依存関係、証明 identity・受理／再利用・公開は生成しない。
テストは実 Frontend／Resolver の単一／集約入力と、走査実装から独立して文法から構成する正確な期待バイト列を使う。集約 assertion の実 TypeExpression／TypeHead 形状、現 SymbolId 対応、中立／関与編集と再配置、空結果、実上流エラー／未対応部分木の拒否、入力／既存結果の不変性を扱う。架空の正常 AST・受理／キャッシュ状態を与えない。

## Source Public Theorem Signature Candidates
`source_public_theorem_signature_candidates(&SurfaceAst, &SurfaceResolvedArena, &SymbolEnv) -> Result<Option<BTreeMap<SymbolId, SignatureShell>>, String>` は、実際の公開 theorem/lemma の proof body を除いた surface-token statement shell を未公開で供給する。既存型を使い、元の env の signature は置換しない。
最初に[元の式対応](./labels.md#source-public-theorem-formula-candidates)を呼び、AST/arena/未拡張 env 全体の検証、source profile、error と呼出し全体の None を保持する。その実 SymbolId key 集合を過不足なく返す。受理された全 private 入力は Some(empty)。生の名前、割当の推測や架空の空値を不足 producer の代用にしない。
各元 formula の部分木だけを保存された子順で走査し、全 token を一度ずつ保持する。LIFO なら子を逆順に積む。Root の互換 token listing は走査しない。Identifier/ReservedWord/ReservedSymbol/Numeral/LexemeRun/UserSymbol の、空でなく UTF-8 byte 長を u32 で表せる text だけを対象とする。他 token、長さ overflow、選択された token-free formula は、対応済み owner を含む map 全体を None にする。token を捨てたり部分 map を返したりしない。
value は schema source-theorem-statement-tokens-v1 の SignatureShell::Opaque。String payload は各 token の record を順に連結する。record は token.kind.syntax_kind() as u16 の符号なし十進値、colon、checked u32 UTF-8 byte 長の十進値、colon、原文 text。数値に先頭ゼロを付けず、長さで区切るため text 内の区切り文字も保持できる。record 間の追加 separator、正規化、Debug encoding は使わない。
payload に含むのは元の token kind/綴り/束縛変数名/順序だけ。owner label/role/status、module identity、ID/range/trivia/comment、proof/justification body、非 token の node kind/variant field は含めない。対応する値は proof のみ、trivia、割当、owner 改名、宣言順の変更で不変とし、関与する token kind/text/order の変更を反映する。これは長さ付き surface-token 射影であり、AST codec、alpha 正規化／elaboration、意味的同値、semantic/interface/imported-statement fingerprint、proof/status authority ではない。
labels/tests.rs の既存 genuine Frontend helper を変更せず、source_public_theorem_signature_candidates_* の二 family だけを追加する。private/public、lemma、open/assumed、numeral の独立した完全 literal payload/schema と実 public SymbolId 対応、private 除外／全 private 空結果、再実行と入力不変を確認する。proof のみ／trivia/comment／再配置／改名／並べ替え、および束縛変数／演算子／numeral の関与編集を、producer traversal を oracle にせず比較する。spec02 の code ASCII 制限を保ち、Unicode comment は code token にならない。
未対応 token の負例は元の genuine AST から collector/arena を再構築し、対応する public owner を残して既存 formula association が Some であることを先に確認した上で map 全体の拒否を要求する。既存の検証／admission 行列を再利用し、foreign AST/env と未対応 owner／error 優先の代表例を含める。架空の正常 AST、owner、scope、受理状態、cache hit は作らない。
env signature 置換／合成と依存関係 join、完全 ModuleSummary/VerifiedExport と semantic fingerprint、元 source 取得／current-runtime 認証、公開診断、service 登録、phase15、型／証明受理、witness/footprint/reuse/cache/Doc、Steps6–8 完了は別作業。既存 collector、signature schema/caller、source text producer、保護された言語／test intent は変えない。

## Source Theorem Module Candidates
`source_theorem_module_candidates(&SurfaceAst, &SurfaceResolvedArena, &SymbolEnv) -> Result<Option<(ResolvedAst, SymbolEnv)>, String>` は既存型による未公開の source-profile metadata pair を合成する。呼出側は元の未拡張 env を保持する。合成 env は元の source replay receipt の代用でも、完全 ModuleResolver/Checker 入力契約でもない。
同じ元 AST/構造 arena/未拡張 env について source_theorem_resolved_ast、source_public_theorem_signature_candidates、source_theorem_label_reference_candidates の順に呼び、各実 lower error/None をその順序で保持する。実 semantic AST 全体を変更せず返し、実 label result の完全 index を使う。不足 producer を再構成／空値で代用しない。全 private／引用なしの入力も admission 後の実空 batch だけを使う。
追加の任意 aggregate profile は semantic AST の全 label-reference row が実 LabelResolution::Resolved である場合だけ。未解決・曖昧・他 outcome は pair 全体を None とし、旧 API の outcome／診断／lookup 方針を変えない。winner 選択・引用除去をせず、引用なしならこの条件は自明に成立する。
元の十二 index family を clone し、label index だけを実 result 全体で置換して、各実 label origin path を実 contribution の effect に記録する。元の symbol/definition/contribution identity、origin、visibility/export flag、他 index/field/counter を保持する。実所有対応を要求し、contribution mutator が存在しない ID を黙って無視する挙動に依存しない。
実 public signature map の value を対応する SymbolEntry と DefinitionEntry の両方へそのまま設定する。各 family で実 key 集合を過不足なく一度ずつ消費し、欠落・余剰・重複対応は pair 全体を拒否する。private signature と他 field は元のまま。SymbolIndex は重複 append せず再構築し、definition は元の数値 DefinitionId 順で全 optional field、shell-id vector、既存 dependency と割当 ID を保持して再構築する。SymbolId 順の iterator は割当順ではない。名前／counter から代用 ID を作らない。
各実 label-reference row の既存の checked hierarchical citation origin [theorem surface id, statement id, reference id] から引用元 theorem を取得する。実 declaration shell を既存 shell_origin/shell_context 全体と kind で一意の元 SymbolEntry に対応付け、range のみ・引用先 spelling・FQN の断片解析は使わない。実 resolved target と実 label index の対応を要求し、元 table ID/site/origin を保持する。
実 LabelRefId の iterator 順で、元 dependency index の clone に引用ごと一辺を実 allocation する。source は実引用元 owner の Symbol、target は実解決済み origin path の Label、kind は LabelCitation、anchor は実 reference-site range、contribution は実引用元 owner のもの。旧 edge/ID を保持し、新しい実 edge ID をその contribution effect と引用元 definition の dependency list へ記録する。引用の重複 occurrence は別辺とする。構文上の resolver metadata dependency であり、proof-obligation／accepted-used-fact dependency ではない。
同じ module の既存 SymbolEnv::new と、変更しない実 semantic AST を pair で返す。対応全体の検査に失敗すれば None、部分 pair は返さない。元 arena/env/AST と provenance を変更せず、合成 env を未拡張と偽らず、local SymbolId や架空 lower table/state を作らない。
labels/tests.rs の既存 genuine Frontend helper を変えず、source_theorem_module_candidates_* の二 family のみを追加する。元の実 key/ID、両 view の完全 literal public signature、private 保持、全 label row/contribution effect、各引用の source/target/kind/anchor/実 edge ID と元 definition への対応、全保持 definition field／未変更 index family、実 semantic AST 不変、再実行と入力不変を独立確認する。private/public、lemma、open/assumed、引用なし／numeral、全 private を含める。十二以上の theorem を実 Frontend へ渡し、DefinitionIndex iterator が割当順でないことを先に示して ID 保持を確認する。
負例は実 source_theorem_resolved_ast が実未解決引用を持つ Some を返すことを確認して aggregate None を要求する。既存 profile/error 行列を再利用し、未対応／foreign 入力と実 lower 優先順の代表例を扱う。semantic AST が Some のまま実 public-signature producer が None になる source-derived 負例でも pair 全体を拒否し、空 signature で代用しない。元の実 collector/arena を再構築し、架空の正常 AST/owner/label outcome/producer/evidence/受理・cache 状態は作らない。元 replay receipt は使用可能なまま、実 fixture の合成 env は元 receipt として通らないことを確認する。
この pair は限定 resolver metadata のみ。一般通常名／import、private VisibleItem の Checker 対応、完全 env/summary/VerifiedExport と semantic/interface/imported-statement fingerprint、公開診断、元 source/current-runtime 認証、storage codec/service/phase15、型／証明受理、完全 phase footprint/witness/reuse/cache/Doc と Steps6–8 は別作業。元 collector/schema/caller/診断/recovery/証明方針と保護 test intent は変えない。

## Source Theorem Module Original Inputs
`source_theorem_module_original_inputs(ast: &SurfaceAst, module: &ModuleId) -> Result<Option<(SurfaceResolvedArena, SymbolEnv)>, String>` は既存型の実元構造 arena と未拡張 collector env を返す。
DeclarationShellCollector::new(ast, module).collect() を実行し、SignatureProjectionExtractor::new(ast, &shells, NamespacePath::new(module.path().as_str())).collect(module) を実行する。実 collector 診断は SurfaceResolvedArena::lower(ast, module) より前に Ok(None) とし、lower error.to_string() を保持する。Some((実 arena, collected.into_env())) で既存の所有 env を clone/index/ID 再構成なしに移す。
Some はこの constructor の出力だけを表し、後段 theorem metadata profile の admission ではない。未対応／未解決引用や proof variable により実 pair producer は引き続き None/String を返しうる。空の代用品や拡張済み出力 env を元 receipt とせず、AST/module と保持 data を変えない。source/module provenance と currentness は呼出側が所有する。
二つの native module consumer と storage encoder/decoder はそれぞれ factory を直接呼び、返された両方の値を同じ [module-pair producer](#source-theorem-module-candidates) に渡す。native capture、typed-parent/出力全体一致、physical/publication の事前・再検査の順序と error を保持する。storage framing/cap は factory より前、nested decode は渡された実 SourceId/module を保持する。encoder/native の factory None/String はそのまま、decoder は実 factory/pair の拒否・error を None に変換する。
crates/mizar-resolve/src/labels/tests.rs に `source_theorem_module_original_inputs_match_real_original_context` と `source_theorem_module_original_inputs_preserve_constructor_and_admission_boundaries` だけを追加する。parsed_private_theorem_fixture/NoImportedSummaries、全既存 test/import/fixture/expectation は不変で、新 helper は作らない。
実 fixture AST/module と factory 前の独立した旧 collector/lowering を arena 全体・全 env field/ID/counter の oracle とし、別 package/dotted module も扱う。factory 前の旧 pair 全体を、返された実両値による結果と比較し、実拡張 env と区別する。private/public、全 private、lemma、open/assumed、numeral/混在値、反復引用、十二宣言の非割当順、BOM/CRLF/Unicode trivia、再実行と AST/module 不変を扱う。既存 signature/ID/citation/storage/native 行列を再利用する。
実未対応／未解決／foreign-qualified 引用・reserve・未解決 proof variable について、pair が None/String でも実 factory Some であることを示す。独立した旧 assembly/outcome と比較し実 variable error を unwrap する。変更しない helper は collector 診断を拒否するため、[重複 theorem source](../../../../tests/miz/fail/resolve/fail_resolve_duplicate_theorem_symbol_001.miz) だけは既存 Frontend setup を inline で使い、実診断を確認して factory Ok(None) を要求する。AST/module を保持する。架空の正常 AST/error/producer/state を作らず、valid parser AST の lower error 変換は静的・既存 lower coverage を保つ。
production 追加は計100行以内、factoryは32行以内、test追加は240行以内。production 変更は symbols.rs/frontend_adapter.rs の新関数と四 context-assembly block に限定し、他関数・guard・grammar・error・producer は byte 不変。新 type/field/adapter/codec/hash/profile/dependency、parser/filesystem/id allocator は追加しない。この元入力 seam は semantic stream/fingerprint、完全 env/ModuleResolver/summary/VerifiedExport、service/IR sealing、execution/currentness、証明/受理/cache/phase footprint や Steps6–8 完了を与えない。

## Source Theorem Module Semantic Payload
`source_theorem_module_semantic_payload(ast: &SurfaceAst, module: &ModuleId) -> Result<Option<(crate::resolved_ast::ResolvedAst, SymbolEnv, Vec<u8>)>, String>` は既存 module pair の実全体と同じ pair 由来の限定 semantic bytes を返す。外部 pair/env/bytes/hash/receipt は受け取らず、既存 production consumer もない。後続 publication 組立に欠ける operand の追加であり shared-helper 抽出ではない。
最初に magic、二つの u64 長、UTF-8 module package/path のサイズを checked arithmetic と 32 MiB cap で検査し、overflow/幅変換不能/超過なら構築・replay より前に Ok(None)。次に [original inputs](#source-theorem-module-original-inputs) を一度、返された実両値で [module candidates](#source-theorem-module-candidates) を一度呼ぶ。この順で実 None/String を保持し、実 Some pair 全体だけを inline Option::and_then serialization で処理して完全 bytes と未変更 pair を返す。profile/幅/サイズ拒否は None とし、部分出力や代替・再構成 ID/table を返さない。
private versioned stream は UTF-8 `mizar-resolve/source-theorem-module-semantics/v1` と zero、実 module package/path string の順。全整数 (tag/count/id/length/path element) は checked unsigned u64 little endian、string は UTF-8 byte 長と実 bytes、collection は count と records。各 append を checked addition と同じ 32 MiB cap で検査し、未検査長から確保しない。public decoder/dependency/generic codec/hash 実装、named helper/type/field/adapter/並列表現は追加しない。local scalar/string/origin/FQN framing closure は実 call site 三つ以上に限る。
以下 O は実 origin structural_path collection で、header module 一致、非 recovered、import_edge None を要求する。全 record の module は header と一致する。FQN は既存の全 string を直接読み、parse/split/再構成しない。実 local identity はその FQN と typed pair に残る。anchor/range/SourceId は source data で O に含めない。実 ID と既存 deterministic iterator を使い、sort/copy は追加しない: node/reference table は ID、symbol は SymbolId、label は origin/kind、definition は symbol/id、dependency は source/target/kind/id、contribution は ID 順。Root compatibility edge を含む child 順、保存 effect/dependency 順を維持する。

| section 順 | count 後の実 field (root は node count の前); 検査条件 |
|---|---|
| AST root/nodes | root.index、node count、各実 node ID、typed SyntaxKind numeric tag、token のみ token-kind SyntaxKind tag と実 text、child-ID collection、resolution tag (0 NotApplicable/1 Resolved)、ref tag (0 None/1 Name+ID/2 Label+ID)、O。recovery は Normal。 |
| Name references | 実 NameRefId、site.node ID、実 site spelling、実 BuiltinId string、実 BuiltinRef spelling、O。ResolvedBuiltin と Normal を要求。 |
| Label references | 実 LabelRefId、site.node ID、実 site spelling、実 resolved LabelOriginPath string、O。Resolved target kind Theorem と Normal を要求。 |
| Symbols | 実 FQN、kind (0 Theorem/1 Lemma)、visibility (0 Private/1 Public)、export (0 LocalOnly/1 Exported)、namespace、primary spelling、contribution ID、実 opaque signature schema/payload string、O。notation None、relations empty を検査。 |
| Labels | 実 origin-path string、上記 visibility/export tag、namespace、primary spelling、contribution ID、O。Theorem と Normal を要求。 |
| Definitions | 実 DefinitionId、実 symbol FQN、kind (0 Theorem/1 Lemma)、visibility tag、contribution ID、実 opaque signature schema/payload、O、保存 dependency-ID collection。parameters/binders empty、arity/notation/doc_attachment/conflict None を検査。 |
| Declaration dependencies | 実 edge ID、実 source Symbol FQN、実 target LabelOriginPath string、contribution ID。LabelCitation とこの endpoint class を要求し、反復 occurrence も別の実 record として維持する。anchor/range は source data。 |
| Contributions | 実 contribution ID、symbol-FQN effects、definition-ID effects、label-origin effects、dependency-ID effects の各 collection。header module と実 AST SourceId に一致する LocalSource を要求し、overload_groups/registrations/lexical_summaries/namespace_edges/imports/exports/diagnostics effects empty を検査。anchor/local SourceId は source data で zero 代用しない。 |

省略前に実 ResolvedImports imports/exports/canonical modules と env imports/exports/overloads/registrations/lexical_summaries/namespace_graph/module_summaries が empty であることを検査する。signature は全て実 Some(Opaque) とし、private parser-signature text を含む schema/payload を正確に保持し、実 structured-definition adornment の不在を検査する。既存 theorem constructor の lexical-kind None/unique-policy は検査対象の empty index を説明する事実であり、新 carried field や追加 getter/guard ではない。実 field/shape が違えば payload 全体を拒否し、filter/Debug/snapshot/storage encoding/opaque string 再解釈を行わない。
closed all-node form は Root/CompilationUnit/ItemList/VisibleItem/VisibilityMarker/JustificationClause/ReferenceList/Token/TheoremItem/LemmaItem/ProofBlock/LetStatement/ConclusionStatement/Proposition/FormulaExpression/QuantifiedFormula(Universal)/QuantifierVariableSegment/QualifiedVariableSegment/TypeExpression/TypeHead/BuiltinPredicateApplication/TermExpression/TermReference/NumeralTerm/Reference。tag55 は Universal を明示 guard、token は現 enum 十 variant の typed numeric tag のみ。未列挙・将来 node/token/reference variant や recovery state は拒否する。現 enum の Unknown と ErrorRecovery token variant は実旧 pair が Normal recovery で admit する場合だけ含め、syntax を拡張しない。
この conservative source-shaped internal phase projection は binder/owner/proof/numeral spelling、module、graph、declaration/citation 順、実 reference/key/signature/dependency、実 structural ordinal/table association を保持する。SourceId rebinding と layout/comment 編集は選択 association が同じ場合だけ bytes を保存する。builder next-ID counter、SourceId/range/anchor、trivia/expression-root side data、存在しない doc attachment は encode せず、typed pair 全体と既存 Syntax storage/replay に実 data を残す。alpha/logical equivalence、rich exported/imported-statement、dependency-facing proof-erased fingerprint ではない。semantic Some は独立 cap/domain の storage Some を意味しない。
既存 labels/tests.rs に `source_theorem_module_semantic_payload_matches_real_pair_and_exact_profile` と `source_theorem_module_semantic_payload_preserves_errors_and_participation` だけを追加する。旧 prefix/helper/import/fixture/expectation を全て維持し、未変更 parsed_private_theorem_fixture/NoImportedSummaries と既存 distinct_source_ids を使う。この二 family 外の新 helper/type/fixture/import は作らない。
新 API より前に実旧 original context と旧 pair 全体を取得し、返された pair 全体一致と input 不変を要求する。inline reader は新 encoder とは独立して完全 grammar を実旧 pair/literal field と比較し、literal magic/header/代表 numeric tag/string、全 count/record/ID/association、trailing bytes 不在を確認する。新 encoder に expected bytes を作らせず、writer/traversal を oracle として複製しない。既存 literal lower signature/association matrix を維持し、admit 時は実 storage/replay pair 全体も比較するが storage admission を主張しない。
private/public/全 private、lemma、open/assumed、mixed quantified/異なる numeral 値、反復 citation endpoint、非割当 iterator 順を先に示す十二 declaration、BOM/CRLF/内部 Unicode trivia、header/FQN association を変える実 other-package/dotted-module を扱う。既存 Syntax storage による別 allocator SourceId への実 rebinding は typed source data を変え bytes を保持する。comment/format は選択 association を保存し、module/owner/role/visibility/binder/proof/numeral/declaration 順/citation 順の編集は bytes と対応実 lower field を変える。alpha/proof 不変性は主張しない。
header domain 内で unresolved/qualified citation、reserve、unsupported unproved theorem、unresolved proof variable (実 unwrap_err) の実旧 None/String と比較する。duplicate AST は負例のみ: 既存 duplicate fixture の同幅で衝突しない Other variant を未変更 helper で parse し、既存 builder と実 node を使って owner token を Clash に戻し、collector を独立再生成して実 duplicate 診断の後に新 None を要求する。positive parser receipt にせず helper 弱化/new Frontend wrapper/架空診断を作らない。構造検査でこの derivative が成立しない場合、代替の実 inline Frontend setup に進む前に parent に戻す。
実巨大 typed module header (>32 MiB) は実 error source の replay 前に拒否させ、先に通常 module で同 source の実旧・新 error を比較する。幅不能、未強制の後段 body cap、future-profile guard は static/current-producer coverage の範囲を守る。架空の正常 AST/pair/Scope/summary/Hit/Accepted/Complete、偽巨大長・control state は作らない。
symbols.rs の新 public function 一つ (gross production 最大360行) と追加 test (最大500行) のみで、旧 production body は全て byte exact。source/test file、attribute、言語/診断/recovery/証明方針の変更、cache/reuse authority、架空 consumer は追加しない。完全 ModuleResolver、outgoing ModuleSummary/lexical summary、publication 組立/side table/診断/service、current runtime、完全 footprint/reuse、VerifiedExport、phase15/proof/acceptance/cache、Steps6–8 は open のまま。

## Source Theorem Module Storage
`source_theorem_module_candidates_canonical_bytes(&SurfaceAst, &ModuleId) -> Result<Option<Vec<u8>>, String>` は実 source AST と module から元 collector と [source_theorem_module_candidates](#source-theorem-module-candidates) が決定的に生成する pair のみを保存する。任意の ResolvedAst/SymbolEnv の汎用 serializer ではない。
最初に既存の完全な [SurfaceAst canonical storage](../../mizar-syntax/ja/ast.md#publication-storage) を取得し、storage 不正／範囲外なら None。aggregate は framing と module bytes を含め 32 MiB 以内で、checked arithmetic により overflow／超過を resolver replay 前に拒否する。言語受理の制限ではなく保存領域の制限である。
同じ AST/module の [元 module 入力](#source-theorem-module-original-inputs) を使う。collector 診断は None、lowering と実 pair error は元 String、実 pair None は None。実 pair 全体が Some の場合だけ保存し、独自 admission や代用 table は使わない。
正確な envelope は UTF-8 `mizar-resolve/source-theorem-module-storage/v1` とゼロで始まる。その後に literal UTF-8 module package、literal UTF-8 module path、変更しない完全 Syntax canonical AST bytes の三 field をこの順で置く。各 field は checked u64 little-endian byte length と実 bytes。source id、debug text、semantic table の複製、新 syntax vocabulary、source-body 再構成は保存しない。body/trivia/range は既存 AST 形式に含まれ、semantic/interface/imported-statement fingerprint ではない。
`source_theorem_module_candidates_from_canonical_bytes(&[u8], SourceId, &ModuleId) -> Option<(ResolvedAst, SymbolEnv)>` は呼出側の実 typed source id と期待 module を要求する。aggregate cap/domain/三 length/truncation/trailing bytes と期待 package/path の byte 完全一致を nested decode 前に検査する。保存 text/bits から module/source identity を作らない。既存 SurfaceAst::from_canonical_bytes に渡された source id で decode し、canonical/topology/trivia/resource 検査を継承する。
decode した AST と期待 module に同じ [元 module 入力](#source-theorem-module-original-inputs) と実 pair producer を適用し、実 pair 全体を返す。storage/collector/lowering/lower producer の拒否・error はすべて None。decoder は file load、source id 割当、source parse を行わず、source execution/currentness/証明/公開権限を与えない。成功は storage 構造と明示 module 一致のみを検証し、source/capture/provenance/current-runtime 検査は呼出側が所有する。
同じ source id なら semantic AST 全体と十二 env index family、元 id/field/counter、実 signature/label/citation edge/effect を含め pair 全体が一致する。再束縛は同じ保存 AST を渡された実 id で実 producer に通した結果と一致させる。解決済み pair の id 書換や合成 env を元 collector receipt と偽ることはしない。既存 producer/schema/lookup/error 方針は変更しない。
既存実 Frontend helper と codec/collector を変えず source_theorem_module_storage_* 二 family のみを追加する。private/public、全 private、lemma、open/assumed、重複・異なる引用 endpoint、異なる signature、十二 theorem の非割当 iterator 順、BOM/CRLF/内部 Unicode trivia を扱う。元の完全 pair と独立に再束縛した実 producer pair、固定 literal envelope prefix/nested AST bytes、再実行、入力不変を確認する。既存 session allocator で実際に異なる SourceId を割り当てる。Syntax decode は execution receipt ではない。lower 行列は再利用し重複しない。
実未対応・未解決引用・未解決 proof variable の lower outcome を負例とし、encoder は実 None/String を保持し decoder はその source-derived envelope を拒否する。全 strict truncation、誤 domain、length overflow、余剰 field/trailing data、不正・非 canonical・未対応 nested Syntax bytes、期待 module 不一致、外側 size cap を検査する。source 不整合 AST 改変は負例のみ。架空の正常 AST/owner/frame/producer/受理・cache 状態を作らない。production に filesystem/parser/id allocator を導入しない。
この source-derived 保存前提は IR seal や service 登録を行わない。一般名/import と任意 resolver 値、完全 ModuleResolver/Checker、完全 summary/VerifiedExport、semantic fingerprint/公開診断/live runtime、phase15/型/証明受理、完全 footprint/witness/reuse/cache/Doc と Steps6–8 は別作業である。

## Source Theorem Syntax Payloads

`source_theorem_syntax_payloads_from_arena(source: &SurfaceResolvedArena, symbols: &SymbolEnv) -> Result<BTreeMap<SymbolId, Vec<u8>>, String>` は限定 profile の宣言内 raw syntax を供給する。最初に既存 `validated_source_symbol_ast(source, symbols)?` を呼び、実 AST 全体の再構築、環境全体の collector replay、上流エラーを保持し、その実 AST を使う。新たな named-owner producer は不要。
その AST と module から実 declaration shell を再収集する。parent のない `DeclarationShellKind::Theorem` のみ選択し、collector が外側 VisibleItem を shell parent なしで unwrap することを使う。既存の完全な `shell_origin(ast.source_id, source.module(), shell, &shell_context(...))` により一意の実 `SymbolKind::Theorem` entry と対応させ、その実部分木のバイト列を現在の SymbolId に対応付ける。名前／id fallback は禁止。Lemma など他種は無視し、空／他種のみも検証後に空結果を返す。
各値は UTF-8 `mizar-resolve/source-theorem-syntax/v1` とゼロで始まる。実ノードを保存された子順の preorder で訪問し、各 `syntax_kind()` を big-endian u16 で出力する。Token はさらに token-kind SyntaxKind の big-endian u16、checked big-endian u32 UTF-8 byte 長、原文 text を出力する。全ノードで続いて checked big-endian u32 子数と各子を再帰的に出力し、LIFO なら子を逆順に積む。owner bytes、id、range、structural path／order index、trivia、Debug／storage 文字列、module／visibility metadata は含めない。
対応する unit 非 Token 種は TheoremItem, FormulaExpression, QuantifierVariableSegment, TypeExpression, TypeHead, BuiltinPredicateApplication, TermExpression, TermReference, ProofBlock, LetStatement, QualifiedVariableSegment, ConclusionStatement, Proposition のみ。追加で `QuantifiedFormula(SurfaceQuantifierKind::Universal)` のみ許可し、その parameter は本 versioned profile で定数なので固定 tag で一意となる。Token は空でない Identifier, ReservedWord, ReservedSymbol のみ。他の node／token／recovery／unknown／parameter 値は全て拒否する。量化種の拡張には別 version／review を要求し、可変 payload の省略は禁止。
未対応の選択／部分木、子／node 不足、長さ／子数 overflow、key 重複、theorem row 不足は呼出し全体の `unsupported source theorem syntax payload` とし部分結果を返さない。実 Theorem entry ごとに結果 row をちょうど一つ要求する。先行する再構築／環境エラーの優先を保持し、この限定 profile は Parser／Checker／Core の受理や診断を変えない。
根は外側 VisibleItem／private marker、他宣言、参照先定義を含めない。named package／module／namespace／visibility／role は別の genuine checked-owner 要素。同じ表現の宣言は module／private wrapper が異なっても値が等しく、実 key／owner fact は各自のものとする。現在の SymbolId は対応付けのみ。local label／binder／proof-token の綴りと topology は case／alpha／意味正規化せず反映し、整形／comment／allocation／source id／宣言位置は反映しない。
この値は意味同値、source freshness、完全 SourceShapeHash／ObligationAnchor／依存関係、proof identity／受理、witness／reuse／cache／publication、Steps6–8 完了を意味しない。既存の [registration syntax](#source-registration-syntax-payloads) と [public theorem signature](#source-public-theorem-signature-candidates) profile は別のまま変更しない。
`crates/mizar-test/src/runner/tests.rs` に `source_theorem_syntax_payloads_exact_current_keys_and_empty`、`source_theorem_syntax_payloads_neutral_relocation_order_and_local_edits`、`source_theorem_syntax_payloads_preserve_upstream_and_refuse_unsupported_subtrees` の三 family のみ追加し、既存 `source_registration_owner_inputs` を変更せず使う。第一は genuine 単一／集約 T/U、正確な実 current key、再現性、入力／既存結果不変、検証済み空／lemma-only を扱う。実装前に旧 parser／Syntax の観察から `theorem T: for X being set holds X = X proof let X be set; thus X = X; end;` の完全 literal 期待 bytes を独立導出し、U は明示した一つの framed identifier 置換のみ可。producer traversal／output は oracle にしない。
第二は genuine whitespace／Unicode-comment、allocation relocation、宣言並替え、package／module、private wrapper を使い、各実 named entry ごとに対応する値を比較する。local label／binder／proof-local token 編集は bytes を変える。渡した source／env と既存結果を保存し、container relabel や渡していない無関係な Frontend state の assertion は禁止。
第三は genuine foreign／augmented／recovered source／env の API エラーを既存 `validate_source_symbol_env` と比較し、既存行列を再利用する。実 existential-quantifier theorem は旧 whole-source 検証成功後に本 profile を拒否し、その existential theorem と対応済み T の混在で map 全体の拒否を示す。実 parser-base AST を Existential に変える場合は実再構築と旧検証成功後の malformed 負例だけとする。全 assertion は API に到達し、架空の正常 AST／owner／受理／cache state や上流結果を作らない。
実装は既存 `symbols.rs` の inline public 関数一つ（production 追加 130 行以下）と三 family の末尾追加（test 追加 240 行以下）だけとし、旧 Rust／import／helper bytes を全て保存する。新 file／type／field／helper／adapter／parser／表現／dependency／hash／codec、owner producer、他 Rust／spec／.miz／expectation／trace／ledger／lint／Cargo の変更、corpus activation は禁止。

## Source Public Theorem Export Candidates
### API and invariants
`source_public_theorem_export_candidates(ast: &SurfaceAst, source: &SurfaceResolvedArena, symbols: &SymbolEnv, identity: &mizar_artifact::module_summary::ModuleSummaryIdentity) -> Result<Option<Vec<mizar_artifact::module_summary::ExportedSymbolSummary>>, String>` は入力を変更せず未公開 source metadata を返す。
最初に元 AST／arena／全体 unaugmented env で [source_public_theorem_formula_candidates](./labels.md#source-public-theorem-formula-candidates) を呼び、実 None／String と admission を保つ。augmented pair、source 再読込、caller の formula／name／status／row／hash で不足を補わない。
次に既存 canonical_json で supplied identity を検証し、空行の成功より前にも package_id／module_path と source.module() の完全一致を要求する。不一致は String error。version／lockfile／edition は検証済み caller metadata であり、SourceId 由来の build 認証ではない。
返された実 SymbolId／formula 対応を各一回使う。同じ AST から既存 shell を再収集し、元 entry の SemanticOrigin と完全な shell_context／shell_origin および Theorem／Lemma kind が一致する shell を一つ要求する。実 node_id を使い、range／spelling だけの対応や owner 再構築はしない。
その item の direct child を順に実 formula node まで含めて描画し、formula は direct child として正確に一回到達する必要がある。選択 subtree の token を保存順 preorder で歩き、実 text を ASCII space 一つで結ぶ。written role／status／label、spelling／case／Unicode を保ち、terminator／justification／proof-body child を除く。
recovered selected node、空 token text、Identifier／ReservedWord／ReservedSymbol／Numeral／LexemeRun／UserSymbol 以外を拒否する。この決定的 displayed surface-token string は再 parse／正規化 syntax や kernel／imported-statement canonicalization ではなく、SignatureShell と既存 framing は変更しない。
### Row profile and boundaries
origin_id と fully_qualified_name は実元 owner の FQN から real theorem-label と同じ identity spelling をコピーする。実 namespace は dot で分割し、trim／case／Unicode 正規化をしない。admitted Public／Exported owner の visibility は public、kind は実 theorem／lemma とする。
source_range は検査済み実 shell／declaration 全体の range。proof_status は必ず None で projected proof metadata 不在を表す。written status から Accepted／NotAccepted／NotRequired を推論せず、import／proof acceptance を認可しない。
実 row の interface_fingerprint は current_schema_version() と supplied identity で既存 [ExportedSymbolSummary::compute_interface_fingerprint](../../mizar-artifact/en/module_summary.md#exported-element-fingerprint) を使い、検証 String を伝播する。必須の局所初期値は return 前に上書きする。全 field の寄与と range／stored-fingerprint 除外は同 helper に従い、新 hasher／表現を作らない。
FQN、origin_id 順に sort する。unsupported／missing／duplicate 対応は部分行なしで全体 None、上流／Artifact／conversion error は String。genuine all-private source は継承 source 検証と identity 検証の後だけ空行を返せる。
trusted actual AST 取得、currentness、完全 build-identity binding は caller の責務。完全 ModuleSummary／lexical／dependency／reexport accounting、VerifiedExport、service／execution attestation、rich formula／statement fingerprint、proof acceptance、phase footprint、cache／reuse／publication authority は供給しない。既存 lower profile／diagnostic／proof policy／Steps6–8 gate を保つ。
### Test design
crates/mizar-resolve/src/labels/tests.rs の末尾に source_public_theorem_export_candidates_preserve_real_surface_and_identity と source_public_theorem_export_candidates_preserve_refusals_and_boundaries のみ追加する。変更しない parsed_private_theorem_fixture／NoImportedSummaries を使って setup を inline 化し、helper／import／旧 test は変更しない。
real Frontend AST／元 arena／unaugmented env を使い、実 lower formula map／元 owner を先に独立取得する。完全 literal row、正確な FQN／origin／kind／namespace／whole range、順序、再現性、入力不変を検査する。新 traversal や synthetic positive AST／symbol／status を oracle にしない。
private／open-assumed fixture と inline all-private／lemma／mixed visibility／repeated citation／distinct numeral source を使い、正確な role／status／label／formula header、body／terminator 除外、None status を検査する。public formula／written-status 編集は signature／hash を変え、proof-only 編集と実 range／allocation／trivia relocation は除外 range 以外の対応 body-free row／hash を保つ。
explicit literal row field／full identity から既存 canonical／hash-domain helper で独立に Artifact element JSON／framing を作る。Artifact field matrix を再利用し、version／lock／edition 感度と、実再収集した別 package／dotted module による current package／path binding を加える。generic publisher output や数値 metadata による proof-validity 主張は禁止。
representative unsupported／recovered／foreign／augmented profile と到達可能な source error は genuine lower None／String を先に正確に比較し、優先順を保つ。実空行でも invalid identity と wrong package／path を検査する。optional-field／namespace／range 負例は real または負例専用 derived data に限り、架空 width／coherence／race、Accepted／Hit／Complete／summary fixture を使わない。

## Source Theorem Module Summary Candidate
### Summary candidate API
`SymbolCollectionResult::source_theorem_module_summary_candidate(&self, frontend: &mizar_frontend::orchestration::FrontendOutput<SurfaceAst>, identity: &mizar_artifact::module_summary::ModuleSummaryIdentity) -> Result<Option<mizar_artifact::module_summary::ModuleSummary>, String>` は入力を変更せず未公開 candidate を返す。
collection diagnostics が非空なら拒否し、最初に [export_frontend_lexical_contributions](#ソース字句-contribution-producer) を呼ぶ。全体 None、identity/source/edition 検査、実 frontend/collection 対応を保つ。実際の変更されていない Frontend run の取得、source currentness、完全な build identity provenance は caller が負い、本 method は取得自体を認証しない。
実 retained user_symbols、operator_declarations、preprocessed import_stubs がすべて空であることを要求する。同じ frontend AST を self.env.module_id() の元 SurfaceResolvedArena に lower し String を保ち、元 self.env と既存 [source_theorem_resolved_ast](./resolved_ast.md#source-theorem-semantic-ast) を呼び、実 None/String と閉じた全 AST admission を保つ。
resolved imports/exports/canonical_import_modules と env imports/exports/overloads/registrations/lexical_summaries/namespace_graph/module_summaries がすべて空であることを要求する。既存 closed admission は private も含む未対応宣言、import/export/alias/operator/definition を除外し、公開 vector が空でも情報不在の代用にしない。augmented env、重複 allowlist、使わない serialized semantic payload、parser/provider/source replay は加えない。
### Summary candidate projection and boundaries
同じ AST/元 arena/env で既存 [source_public_theorem_export_candidates](#source-public-theorem-export-candidates)、[source_public_theorem_label_candidates](./labels.md#source-public-theorem-label-candidates) の順に呼び、それぞれの実 None/String、実全行、未設定 proof_status を再解釈せず保つ。
既存 Artifact ModuleSummary を current_schema_version()、supplied identity の完全 clone、実 frontend.source.source_hash、実 symbol/label 行、実 lexical contributions から構成する。lexical schema は既存 mizar-resolve/exported-lexical/v1、fingerprint は None（producer fingerprint なし）で、前段検査により contributions は実際に空。reexports/dependency_interfaces の空は admitted source と元 index の不在確認だけを根拠とし、省略 metadata から推論しない。package dependency 宣言は affected module-interface input ではない。
必須の局所初期 interface_hash は return 前に既存 refresh_interface_hash で上書きし、既存 module_summary_json で全 shape/stored-hash を検証する。Artifact error は String で伝播する。新 constructor/hasher/codec や暫定 hash の返却はなく、source hash と source cache key/interface hash を分離する。
candidate は proof acceptance/import authorization、rich semantic fingerprint、完全 phase footprint、cache/reuse authority、native identity/currentness、service/runtime execution、Artifact publication、Steps6–8 完了を与えない。既存下位 profile、diagnostics、recovery、proof policy は変更せず、未対応 source は部分 summary なしの全体 None。
### Summary candidate test design
labels/tests.rs に source_theorem_module_summary_candidate_preserves_real_rows_and_hashes と source_theorem_module_summary_candidate_refuses_missing_or_unsupported_inputs だけを追記する。既存 NoImportedSummaries と実 collector を用いる実 disk Frontend/parser run を inline 化し、local import は可、helper/top-level import/既存 test は変更しない。private/public、open/assumed の既存 .miz fixture は不変。
実 supported theorem、private/public 混在、all-private、lemma、open/assumed、実 empty source で先に既存 lexical/semantic/symbol/label の実結果を得る。candidate 全 field/全 row、実 loaded source hash、lexical schema/None、実不在、反復、渡した元入力の不変性、完全 expected identity で writer/reader round trip を検査する。単純な実 theorem の canonical interface projection は literal expected field と既存 Artifact framing/hash helper から独立に組み立て、producer 出力を oracle にしない。
実 trivia/source relocation と proof-only 編集では正確な source hash と不変 body-free interface を区別する。公開 statement/label/role、有効 version/lock metadata の編集は interface を変える。実 other-package/dotted-module を再収集し、空 export でも package/path/edition および不正必須/任意 field の binding を検査する。架空 build provenance/proof status は使わない。
負例では実 import/reexport/definition/local lexical/operator/notation、未対応 formula/proof、duplicate declaration、recovered frontend、実 foreign collection の既存下位結果を先に確認する。augmented env は既存下位 API の実拒否だけを検査する。SymbolCollectionResult には呼出可能な augmented-env constructor がないため捏造せず、wrapper の動的 coverage と主張しない。実 retained input の missing AST/diagnostics/stale source/map/locals/preprocessing/省略 import stub 改変は負例専用で拒否を要求する。supported/unsupported 混在の全体拒否と private lexical 省略が不在根拠にならないことを検査する。正例 AST/env/row/summary/Accepted/Hit/Complete を捏造せず、到達不能 guard の動的検査を主張しない。

## Original source proof occurrences validation
`validate_source_proof_occurrences(&SurfaceResolvedArena, &SymbolEnv, &ResolvedVariableScope) -> Result<(), String>` は元ソース・環境と証明出現の全対象行を認証し、Syntax を呼出側へ公開しない。
変更しない validated_source_symbol_ast を先に呼び、その既存 String をそのまま返す。同じ非公開 AST・module・元環境から実際の proof occurrences を解決し、再解決失敗または source/module/順序付き束縛・参照全行の不一致には内部 String `source proof occurrences do not match the original source` を返す。既存行の等値で全フィールドと未使用宣言を比較し、行の選別・代替生成をしない。
認証はこの対象行だけであり、legacy thesis/statements、過去の実行・解析取得、currentness、完全フェーズ、証明・型・受理は扱わない。既存 resolver profile、エラー、生成器は変更しない。
### Validation tests
labels/tests.rs に source_proof_occurrence_validation_preserves_real_rows_and_inputs と source_proof_occurrence_validation_rejects_foreign_rows_and_preserves_errors だけ追加し、既存 Frontend 補助を再利用する。新 helper と旧 Rust 変更は禁止。実際の private/public、grouped/nested/未使用、Given/Consider、対象行が等しい legacy 経路、再実行・不変性、同一ソースの参照だけ・未使用束縛だけの改変、別 source/module、元環境と拡張環境、実際の parser recovery、未束縛使用の再解決失敗を確認する。正しい実入力と分離した行差を先に示し、scope も不正な場合に元環境のエラーが優先することを確認する。空行での source-id 不一致は実際に割当・全面再束縛したソースを用い、範囲の書換えや正例レシートの捏造で代用しない。
