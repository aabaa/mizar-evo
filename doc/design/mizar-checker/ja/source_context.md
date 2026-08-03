# mizar-checker: Source / Binding Context Projection

> Canonical language: English. English canonical:
> [../en/source_context.md](../en/source_context.md).

## 目的と authority

`source_context` は [`00.crate_plan.md`](./00.crate_plan.md) で固定した
Task 248 source/binding-context producer を実装する。Profile Aのlanguage
authorityはChapters 04 §4.3/§4.6、11 §11.2、12 §12.3/§12.7、15 §15.10に
限定する。Profile BはさらにChapters 04 §§4.2/4.6、09
§§9.1/9.3--9.5/9.9.3--9.9.5、18 §§18.2.1/18.2.6/18.6、Appendix Aを使う。
moduleはsource-item order、resolver shell provenance、reserve/definition
parameterのdistinct identity、local shadow、checker context linkを保持する。

## Boundary

module は syntax-free projection のみ受け取る。opaque `DeclarationShellId` は
resolver の実 `DeclarationShellSet` から来なければならず、checker は shell
identity を生成せず `mizar-syntax` を import しない。`mizar-test` が bounded
`SurfaceAst` walk を所有し、source range、typed site、lexical scope、source
order、resolver-shaped `LocalTermBinding` provenance を供給する。

Task 248 が受理するのは本書で凍結した2つの named real-consumer profileだけで
ある。Profile A は実装済みの module-level `reserve x for set;` と、それに続く
`x` という `set` local parameter 1件を持つ `definition` blockである。Profile B
は下記で別途凍結するone-normal-definition-block/two-parameter extensionである。
Vec-based input/table shapeはorderを保持するが、他のcardinality/role combination
は受理しない。canonicalなdistinct-name multiple-reserve inputを含むadditional
reserve itemはvalid language shapeだが、このexact profile外なので
`UnsupportedTaskShape`として拒否する。引用canonical specで未定義なのはsame
identifier再reserve時のreplacement/duplicate ruleだけであり、このnonblocking
`spec_gap`はhuman-reviewed authorityなしに意味を与えない。

type normalization、use-site resolution、RHS evaluation、fact/obligation 構築、
formula/proof verification、Tasks 249+/269+ は所有しない。Steps 6/7 は deferred
のままである。

## Projection model

- `SourceBindingContextInput` は source/module identity、module typed site、ordered
  item shell、ordered binding site を運ぶ。
- complete construction は checker-owned source-item/declaration table、1つの
  `BindingEnv`、exact binding/local-context link、local-context table を所有する
  immutable `SourceBindingContextHandoff` を生成する。
- `TypedAst` は source/module identity、local-context table 全体、item/declaration
  site、context link、module root owner が一致する場合だけ handoff を install する。
  `ResolvedTypedAst` は install 済み handoff を clone する経路しか持たない。
- reserve と local parameter は distinct checker id を保持し、local row は module
  reserve を structural shadow predecessor として記録する。

## Validation、recovery、atomicity

missing/duplicate/reordered row、stale ordinal、source/module/range mismatch、invalid
parent/context/site link、unsupported visibility、stale local provenance、wrong role、
duplicate local binder、partial payload は complete handoff 公開前に拒否する。
実装済みProfile Aでは両itemをtop levelに固定し、definition parameterにreserveと
同じspellingを要求するため、structural shadow linkは欠落しない。凍結Profile Bは
one top-level definition item、two distinct same-scope parameter spelling、no
shadow linkを要求する。

recovered definition shell は binding をclaimしない場合だけ supported である。この
場合 producer は empty recovered context と deterministic internal diagnostic 1件を
持つ `SourceBindingContextIncomplete` を返す。binding を持つ recovered shell は
拒否する。incomplete/inconsistent data は `TypedAst` / `ResolvedTypedAst` に table
を一切 install しない。

このrecovery ruleはProfile Aだけに属する。Profile Bはnormal-onlyであり、
recovered definition item、いずれかのrecovered parameter、partial
two-parameter payloadは拒否され、incomplete Profile-B handoffを公開しない。

## Determinism と coverage

dense id は validated source order に従う。同一 input は equal table と byte-identical
な nonempty debug text を生成し、reordered input は sort せず拒否する。
source-context handoff を持たない legacy `TypedAst` path は exact full-string debug
oracle を維持する。

implemented Profile-A fixture
`pass_type_elaboration_source_binding_context_shadowing_001.miz` は frontend、
resolver shell、producer、`TypedAst`、`ResolvedTypedAst` を通る。runner test はその
実 opaque shell id からだけ corruption input を再構築し、frozen
corruption/recovery/atomicity matrix をcoverする。later type/fact/obligation/formula/
statement/proof payload はすべて empty のままである。

## Public Enum Policy

| Public enum | Compatibility policy |
|---|---|
| `SourceItemRole` | `#[non_exhaustive]`; 後続 source-item role を許容する。 |
| `SourceItemVisibility` | `#[non_exhaustive]`; Task 248 は `Unspecified` だけ受理する。 |
| `SourceItemRecovery` | `#[non_exhaustive]`; 後続 recovery state を許容する。 |
| `SourceBindingContextOwner` | `#[non_exhaustive]`; 後続 owner form を許容する。 |
| `SourceBindingSiteRole` | `#[non_exhaustive]`; 後続 binding role を許容する。 |
| `SourceBindingContextBuild` | `#[non_exhaustive]`; complete/incomplete result を区別する。 |
| `SourceContextError` | `#[non_exhaustive]`; validation failure を exhaustive match しない。 |

この module が所有する exhaustive public enum exception はない。

## Task 248 classification

| Class | Result |
|---|---|
| `test_gap` | Profile Aはclosed。frozen Profile-B focused Rust matrixはseparate implementationまでopen。broader canonical shapeはMC-G011/MC-G016に残る。 |
| `source_drift` | Profile Aはrepaired。closed profile gate/missing private Profile-B extractorはseparate implementationまでbounded。 |
| `design_drift` | Profile-B contractを本書、paired audit、plan、todo、harness recordで同期。 |
| `boundary_violation` | current violation なし。shell fabrication と syntax import は禁止。 |
| `spec_gap` | same-identifier re-reservationのreplacement/duplicate semanticsだけが未定義。このnonblocking gapは実装authorityを与えない。 |
| `repo_metadata_conflict` | 未検出。 |

## Task 258A downstream exclusion

Task 258AはTask-248 binding/context modelをauthorityとして再利用するが、現行
exact `SourceBindingContextHandoff` profileはreserve-plus-theorem source
transactionを受理しない。従ってstatement producerはTask-48由来
`BindingEnv`をdirectに受け、Task-248 handoffをfabricate/extend/installしては
ならない。Task 258Aのtheorem visibility row 1件は`source_statement`がownし、
本moduleのexisting table/profile/API/test/count/hashは不変。

later typed ownerはexclusive。productionはTask 248を先にconstructするだけで、
その後のTask 258Aは`TypedAstError::InvalidSourceStatement`。Task 248に
post-construction installerはなく本taskも追加しない。reverse logical attemptは
checker-test-only `with_source_context_for_test`でsame validationを実行し、
`TypedAstError::InvalidSourceContext`。`inject_source_statement_for_test`
だけでprepareしたcoexistenceのfinal assemblyは
`ResolvedTypedAstError::InvalidSourceStatement`。testsはproduction direction、
named reverse test seam、final rejection、byte-identical rollback、valid
single-owner replayをcoverする。

## Task 248 Two-Parameter Profile-Extension Frozen Contract

### Authority と dependency purpose

本sectionはChecker Task 259が必要とするlower-stage extensionのdocumentation
prerequisiteである。canonical authorityは次である。

- Chapter 4 §§4.2/4.6: declarationはbinding identityを作り、same-scope
  redeclarationはrejectされ、outer bindingをshadowできるのはinner bindingだけ。
- Chapter 9 §§9.1/9.3--9.5: predicateはordered typed parameter、
  definition-local assumption、definiens、correctness propertyを持つ。
- Chapter 9 §§9.9.3--9.9.5: parameter type/guardは後続logical meaningを
  constrainするが、本lower producerへproof semanticsを与えない。
- Chapter 18 §§18.2.1/18.2.6/18.6: leading `let` declarationはordered
  definition-block parameterであり、そのscopeはblock全体で共有される。
- Appendix Aおよび既存parser/resolver fixture: concrete
  `DefinitionParameter`/`DefinitionBlock` shapeとopaque real declaration-shell
  identity。

このauthorityはone scope内のtwo ordered/separately written
definition-parameter identityを保持するのに十分である。predicate meaning、guard
composition、property proof、type normalization、use-site resolution、reject以外の
same-scope duplicate behaviorを認可しない。このbounded transportに不可欠な
human semantic decisionはない。

fresh inventoryではmissing contractを`design_drift`、closed current profile gateと
missing private extractorをbounded `source_drift`、focused Rust matrix欠落を
`test_gap`と分類する。blocking `spec_gap`、
`source_undocumented_behavior`、`test_expectation_drift`、current
`boundary_violation`、`repo_metadata_conflict`はない。Task-259-private
`BindingEnv`/`BindingId` reconstructionは`boundary_violation`なので、このextension
はseparate Task-248 documentation taskと後続separate Task-248 implementation
taskに分離する。

### Exact Task-259 consumer

original Profile-B consumerは
[`source_predicate_definition.md`](./source_predicate_definition.md)で凍結した
165-byte/final-LF future Task-259 sourceであり、SHA-256は
`91bdb5f51c0ea5f07bdd831700cb9803f2aa57e005921c7e4e1798ecbbf2bd9f`。
Task 259がexact-source selectionとfull 71-row/root-70 normal ASTを所有する。
Task 248は次のdirect lower sliceだけをconsumeする。

| Surface row | Range | Task-248 meaning |
| ---: | --- | --- |
| `DefinitionBlockItem` 67 | `0..164` | sole top-level source item / real resolver shell 0 |
| `DefinitionParameter` 41 | `13..26` | parameter 0、`let x be set;` |
| identifier `x` | `17..18` | binding 0 declaration range |
| type expression `set` | `22..25` | binding 0 written type range |
| `DefinitionParameter` 45 | `29..42` | parameter 1、`let y be set;` |
| identifier `y` | `33..34` | binding 1 declaration range |
| type expression `set` | `38..41` | binding 1 written type range |

real `DeclarationShellSet`はshell 0、ordinal 0、
`DeclarationShellKind::DefinitionBlock`、node 67、module/source/range、normal
recovery、no parent、unspecified visibilityを供給する。resolverにはparameter shellが
なく、predicate projectionのparameter/binder collectionとsyntactic arityはempty
である。従ってprivate runnerはexact direct parameter syntax、shell-derived
scope `[0]`、source ordinal `0/1`、declaration rangeだけから
`LocalTermBinding`をderiveする。resolver predicate projectionがparameterを
供給したとはclaimしない。

Task 260はsecond exact Profile-B consumerである。262-byte/final-LF source、
108-row/root-107 Surface profile、node/range `104/0..261`のdefinition-block
shell 0は[`source_functor_definition.md`](./source_functor_definition.md)で
freezeする。同じpublic Task-248 producer contractとparameter slice
`65/13..26`、`69/29..42`をreuseするが、private Task-259 runner helperはgeneral
Profile-B factoryではなく165-byte Task-259 sourceと`0..164` ownerを
authenticateする。従ってnew Task-260 routeはauthenticated Task-260 Surface rowと
resolver shellから独自のexact `SourceBindingContextInput`をconstruct/validateする。
Task 260はexisting Task-259 helperをmodify/generalizeせず、このaddendumはTask-248
public ABI/debug byte/checker test/source-context test countを変更しない。

### Profile preservation と closed admission

Profile Aのbehaviorはbyte-for-byte不変である。

- exactly two top-level items（reserve、definition block順）。
- one reserve binding、およびnormal definitionではsame-spelling local
  definition parameter 1件とexisting structural shadow link。
- existing recovered-definition-with-zero-bindings
  `SourceBindingContextIncomplete` result/diagnostic。
- current real fixture、active route、Task-249 co-installation、public error、
  debug header、table、count、semantics。

Profile Bが受理するのはexactly次である。

- one normal top-level `DefinitionBlock` item、shell ordinal 0、no parent、
  unspecified visibility、lexical scope `[0]`、no reserve item。
- 同じshellがownするtwo normal ordered `DefinitionParameter` row。
  source ordinal/resolver-local visible-after ordinalは`0/1`。
- syntax-free checker input上のtwo nonempty distinct spelling、distinct
  declaration/type range/typed site、same module/source/shell/scope、no
  same-scope duplicate。
- no recovered item/binding、no third parameter、no additional source item、
  no reserve/definition hybrid、no partial payload。

public checkerはliteral source text/builtin type nameを知らず、identity/structural
consistencyだけをvalidateする。private real-source extractorはさらにliteral
`x` then `y`、上記exact rangeのtwo bare/unattributed builtin `set` type expressionを
要求する。unsupported cardinality/role combinationは引き続き
`UnsupportedTaskShape`でfailし、generic corruptionが先に検出される場合はexisting
more-specific validation errorが優先する。`MissingRequiredShadow`はnormal
Profile Aだけへ適用し、Profile Bはshadowをfabricateしない。

### Existing syntax-free ABI と exact table

implementationはpublic type、enum/error variant、method、field、trait
implementation、crate dependencyを追加しない。existing syntax-free ABIを保持した
まま`SourceBindingContextProducer::build`のclosed profile discriminatorだけを
broadenする。

```rust
SourceBindingContextInput
SourceItemInput
SourceBindingSiteInput
SourceBindingContextBuild
SourceBindingContextProjection
SourceBindingContextHandoff
```

Profile Bのsource orderはexactly次を生成する。

- definition shell 0の`SourceItemId(0)` 1件。binding/local context 1、
  predecessor `None`、caller-supplied definition site。
- `x`,`y` orderのtwo `SourceDeclarationId` / two `BindingId`。
- declaration predecessorは`None`、declaration 0。両rowの
  `shadowed_binding = None`。
- two active `BindingKind::DefinitionParameter` row。
  `BinderIdentity::ResolverLocal { scope: [0], ordinal: 0/1,
  declaration_range: 17..18/33..34 }`、
  `BindingTypeSite::Source(22..25/38..41)`、empty capture、owner context 1。
- binding context 0はmodule owner/no bindings。context 1はdefinition shell 0
  owner、parent 0、bindings/visible bindings `[0,1]`、lexical scope `[0]`、
  normal recovery。
- local context 0はcaller-supplied module site ownerでempty。local context 1は
  caller-supplied definition site owner、parent 0、two caller-supplied parameter
  site、no fact/assumption。
- two context link。module context 0 -> local context 0/item `None`、definition
  context 1 -> local context 1/item 0。

従ってitem/declaration/binding/binding-context/local-context/context-link/
diagnostic cardinalityはexactly `1/2/2/2/2/2/0`。
`source-binding-context-debug-v1` header/row grammarは不変で、equal inputはequal
handoffとbyte-identical debug outputを生成する。

### Private runner extractor

matching implementationは次のrunner-private dormant lower helperを追加する。

```rust
pub(in crate::runner) struct SourceTwoParameterDefinitionContextSites {
    pub module: TypedSiteRef,
    pub definition: TypedSiteRef,
    pub parameters: [TypedSiteRef; 2],
}

pub(in crate::runner) fn source_two_parameter_definition_context_projection(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    definition_node: SurfaceNodeId,
    nodes: &TypedArena,
    sites: SourceTwoParameterDefinitionContextSites,
) -> Result<SourceBindingContextProjection, String>;
```

Task-259 callerがexact source selection、full AST/resolver authentication、
typed-arena allocation、single shared arena由来four distinct site供給をownする。
helperはone real top-level definition shell、two leading direct normal parameter
subtree、exact token/range/type payload、shell-derived local scope、Task-248
projectionをauthenticateする。publish前に`nodes`上で全four siteをresolveし、module
siteをarena rootに固定し、各normal nodeのexact range/local-context link
（moduleは`0`、definition/parametersは`1`）をauthenticateする。existing
projectionだけをreturnし、`TypedAst`、`ResolvedTypedAst`、Task-249 handoff、
active runner result、diagnostic detail key、corpus selectorを作らない。

two leading parameter以後のguard、predicate definition、property、
justification、term、formula、proof、token wrapper、descendant subtreeはすべて
excludeする。helperはそれらへdescendせずrowもpublishしない。additional/non-leading
`DefinitionParameter`をrejectするためdirect child kindだけscanしてよい。
Task 259/later ownerがこれらsubtreeを保持する。existing active
`source_binding_context_output`はunchangedでfuture sourceをselectできない。

### Frozen tests と write scope

later implementationはexactly four runner-library testsを追加する。

1. `task248_two_parameter_definition_profile_publishes_dense_context`;
2. `task248_two_parameter_definition_profile_rejects_corruption`;
3. `task248_two_parameter_definition_extractor_is_default_deny`;
4. `task248_two_parameter_definition_installation_is_transactional_and_deterministic`.

profile testはrejectionだけでなくchecker validation precedenceをfreezeする。全
mutationはexact `SourceContextError` variantと、存在する場合は`index`をassertする。

| Mutation | Exact checker result |
| --- | --- |
| no item | `MissingItems` |
| one normal definition item / zero bindings | `PartialItem { index: 0 }` |
| one normal definition item / one otherwise-valid binding | `UnsupportedTaskShape` |
| one normal definition item / three otherwise-valid bindings | `UnsupportedTaskShape` |
| recovered definition / zero bindings | `UnsupportedTaskShape` |
| recovered definition claiming binding 0 | `RecoveredItemClaimsBinding { index: 0 }` |
| recovered binding row `i` | `RecoveredBinding { index: i }` |
| row 1 duplicate same-scope spelling | `DuplicateSameScopeBinding { index: 1 }` |
| row `i` stale source/local ordinal | corrupted fieldに応じ`StaleBindingOrdinal { index: i }`または`StaleLocalIdentity { index: i }` |
| row 1 reordered declaration range | `ReorderedBindings { index: 1 }` |
| row `i` wrong shell/context owner | `UnknownBindingShell { index: i }`または`RoleMismatch { index: i }` |
| row `i` empty spelling | `EmptyBindingSpelling { index: i }` |
| item module/source mismatch | `ModuleMismatch { index: 0 }` / `ItemSourceMismatch { index: 0 }` |
| row `i` binding source/out-of-item range | `BindingSourceMismatch { index: i }` / `BindingRangeMismatch { index: i }` |
| stale shell ordinal / invalid parent/context/visibility | `StaleShellOrdinal { index: 0 }`、`InvalidParent { index: 0 }`、`InvalidItemContext { index: 0 }`、`UnsupportedVisibility { index: 0 }` |
| duplicate shell / any duplicate typed site | `DuplicateShell { index: 1 }` / `DuplicateTypedSite` |
| coherent reserve/definition hybrid、extra item、unsupported role/cardinality | `UnsupportedTaskShape` |

valid Profile Bとsyntax-free distinct-name substitutionはcompleteする。literal
`x`/`y`はrunner authenticationでchecker syntax knowledgeではない。duplicate
spellingは`DuplicateSameScopeBinding`でfailし、valid/corruptを問わずProfile-B
inputは`MissingRequiredShadow`を返さない。このerrorはpreserved Profile-A matrix
だけでassertする。generic field errorはcurrent behaviorと同じくclosed profile
discriminatorより先に返る。

extractor/default-deny testは全private authentication predicateを独立にmutateする。

- cross-wired AST/shell/module input、およびmissing/duplicate/wrong-kind/
  recovered/wrong-node/wrong-range/wrong-parent/wrong-ordinal/wrong-visibility
  definition shell。
- non-direct/nested/non-leading/reordered/missing/duplicated/third
  `DefinitionParameter` child。
- 全`let`/`;`/identifier/`be` token、literal `x`/`y` order、one-segment
  topology、type-node kind、bare form、builtin `set` head/spelling、attribute、
  declaration/type range、scope、local identity、ordinal。
- missing arena node、role/non-root/wrong module site、cross-wired site、
  module/definition/parameter各nodeのwrong anchor/context、recovered/degraded
  node、全duplicate-site pairing。
- equal rangeのnormal excluded guard/predicate/property/justification descendant
  tokenだけを変え、Task-248 projection/debug bytes不変を示す
  acceptance-invariance pair。

各negativeはpublication前にprivate helperでfailする。testはreal
parser/resolver outputと`TypedArenaBuilder`を使い、synthetic shell idやunchecked
opaque siteだけではevidence不足である。

dormancyも明示的にassertする。exact future sourceに対しunchanged
`source_binding_context_output`/`source_binding_context_detail_keys`は両方`None`。
expectation field/metadataはProfile Bをselectしない。returned projectionをexisting
typed/final pathでinstallしてもsource type、attribute、evidence、term、
application、structure、set-term、atomic/composite/composition、statement、
Task-259 handoffはabsent。types、facts、coercions、obligations、diagnostics、
checked formulas、statement semantics、proofs、terminal goals、全resolved semantic
tableはemptyである。

positive testはtest module path `task248.two_parameter_profile`を使い、次のexact
handoff debug stringをfreezeする。

```text
source-binding-context-debug-v1
module: task248.two_parameter_profile
item#0 shell=0 ordinal=0 role=definition-block range=0..164 parent=none context=1 local_context=1 predecessor=none
declaration#0 item=0 binding=0 ordinal=0 role=definition-parameter range=17..18 type_range=22..25 context=1 local_context=1 shadowed=none predecessor=none
declaration#1 item=0 binding=1 ordinal=1 role=definition-parameter range=33..34 type_range=38..41 context=1 local_context=1 shadowed=none predecessor=0
context-link#0 binding_context=0 local_context=0 item=module
context-link#1 binding_context=1 local_context=1 item=0
```

typed debugはそのliteral blockを`typed-ast-debug-v1` module/root/resolved-root
prelude直後にexactly once含み、final debugもunchanged source-context positionに
identical blockをexactly once含む。full typed/final stringをreplay間でcompareし、
Profile-A existing full-string/conditional debug oracleは不変。

従ってfour testsはexact real AST/shell/range/type、全table/identity field、全
profile discriminator、complete corruption/exclusion matrix、transactional typed
installation、final clone、deterministic serializationをcoverする。このlower
helperはTask-259 real consumer implementationまでdormantなので`.miz` fixture、
sidecar、expectation、trace rowを追加しない。

exact later Rust write scopeは次である。

- `crates/mizar-checker/src/source_context.rs`;
- `crates/mizar-test/src/runner/type_elaboration/source_context.rs`;
- `crates/mizar-test/src/runner/type_elaboration.rs`;
- `crates/mizar-test/src/runner/tests/support.rs`;
- `crates/mizar-test/src/runner/tests/type_elaboration/source_context.rs`.

production path countはchecker 23/runner 30のまま。checker library test listは430
のままで、four named testsによりrunner libraryは504から508へprojectする。
resolver 144/syntax 59は不変。implementationではprojected deltaをevidenceとせず
全line count/hashをfresh-measureする。

### Documentation baseline、audit impact、exit

本documentation prerequisiteはsynchronized derived design recordだけを変更する。
production/test source、specification、existing `.miz` fixture、sidecar、
expectation、`tests/coverage/spec_trace.toml`、trace status/mapping/backlink/owner、
active outcome、coverage credit、count、CLI hashは不変である。frozen current
metadataはcases/requirements `421/389`、pass/fail `228/193`、active
parse/declaration/type/proof `101/7/198/1`、declaration requirements
`12 = 7 covered + 5 partial`、type requirements
`253 = 241 covered + 12 deferred`、warnings/errors `23/0`。

`doc/design/spec_coverage_audit.md`にはnarrative dependency ownershipだけを追加し、
trace fileは意図的なbyte-level no-opとする。exitにはsynchronized EN/JA design、
findings-free specification review、docs-only scope/verification、全9 protocol hard
gate、valid independent quality score 90/100以上、exact task-only staging、one
dedicated documentation commit、clean post-commit inventory、protected stash不変を
要求する。fresh inventoryはTask 259ではなく、separate five-Rust-file Task-248
implementationを選択する。
