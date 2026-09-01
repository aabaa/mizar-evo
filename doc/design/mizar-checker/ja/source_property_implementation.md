# Source property-implementation transport

> Canonical document: English
> [../en/source_property_implementation.md](../en/source_property_implementation.md).
> 本文は同じTask 264 contractの日本語companionです。

## Task 264 scope / authority

Task 264はstruct property implementationとinitial correctness obligationの
syntax-free immutable intakeだけを所有します。authorityはChapter 5のvirtual
property/value-source境界、Chapter 7 §§7.4.1/7.8.2/7.10、Chapter 13
§§13.1.2/13.8.2、Chapter 16 §§16.6.1/16.6.2/16.7.2、Parser Task 48
pass/recovery、inactive overlap seed、implemented Tasks 248--255、259、263、
264R、248Pです。mixed predicate/functor gapはread-only isolation oracleです。

本taskはbounded profileの`source_drift`、`design_drift`、canonical由来
`test_gap`だけを閉じます。existence/uniqueness/coherenceのproof、acceptance、
property value/fact/axiom、FOL goal/guard、discharge、Core/CFG/VCは作りません。
Task-264R shellをsemantic identityにすることやjustification syntaxからproof
semanticsを導くことは`boundary_violation`です。blocking `spec_gap`はありません。

## Exact sources / lower ownership

Means sourceはfinal LF込み263 bytes/13 lines、SHA-256
`cc90659f10cae4ef68890624df9b8b9d3f0e830dae5e20cc195dc8b263c5fa2b`です。

```mizar
definition
  struct Task264Carrier where
    field carrier -> set;
    property marker -> set;
  end;
end;

definition
  let M be Task264Carrier;
  property M.marker means it = it;
  existence by computation(steps: 1);
  uniqueness by computation(steps: 1);
end;
```

Equals sourceは189 bytes/11 lines、SHA-256
`175135aaf40b9eab1a28e73ca1aae9f250e66278410d50575cdd279f6d7a2784`です。

```mizar
definition
  struct Task264Carrier where
    field carrier -> set;
    property marker -> set;
  end;
end;

definition
  let M be Task264Carrier;
  property M.marker equals M.carrier;
end;
```

各sourceはone structure、field `carrier -> set`、virtual property
`marker -> set`、one top-level implementation、one parameterだけです。
Meansは`it = it`とexplicit existence/uniqueness、EqualsはTask-254 selector
termでcorrectness zeroです。`assume`、coherence、case/otherwise、proof block、
recovery、property-owned return declarationはありません。domainはparameter
type、return typeはreferenced `marker` declarationからlookupします。

Means ASTは85 rows/root 84/range `0..262`です。structure/type/member ownerは
53--61、parameter path/type/rowは62--65、two `it`は66/68、equality/formula/
definiensは70/71/72、existence/uniqueness ownerは76/80、implementationは81
です。Equals ASTは56 rows/root 55/range `0..188`で、common structureは35--43、
parameterは44--47、`M`/selector/definiensは48--51、implementationは52です。
private selectorは全byte/final LF/row kind/range/recovery/children/root/subtreeを
authenticateします。

Resolverは各source exactly shells/symbols/definitions/contributions
`5/3/3/1`、diagnostics zeroです。shell 4だけがparentless context-only
PropertyImplementationです。`marker`はdefinition 2、contribution 0、origin
`71..94/[4,0,11,0,19,1]`のlocal exported Selectorです。shell 4にはsymbol/
definition/signature projection/semantic originがありません。definition 0とその
symbolはnormal local structure origin `13..101/[4,0,11,0]`を共有し、Task-249PI
parameter headはtask-local fixed FQNではなくこのexact resolver identityを参照します。
`carrier` symbol/definitionはnormal local sibling origin
`45..66/[4,0,11,0,18,0]`を共有します。

Lower bundleはMeansがTask-248P Profile C、Task-249PI `1/3/2`、Task-252
`2/0/0`、Task-256 `1/.../2/2`で、EqualsはProfile C、Task-249PI、Task-252
`1/1/0`、Task-254 selector `1/0/0/1/0/1/3`です。Task253/255とTasks259/263
handoffはabsentです。required fingerprintsはcomplete Task-248P/249PI/252
debug、optional Task253/254/255/256はMeans `None/None/None/Some`、Equals
`None/Some/None/None`です。

Exact Profile Cはmodule site/root 84または55、item shell/ordinal `4/4`、item
site 81/52、range `108..262`/`108..188`、local scope `[4]`、binding/local
context `1/1`、module context/link `0/0`です。Binding 0はnormal definition
parameter `M`、transaction/visible-after ordinal zero、declaration
`125..126`、written type `130..144`、site 65/47、shadow/predecessor noneです。
Local assumptions/factsはempty、cardinalityは`1/1/1/2/2/2/0`です。

Current Task-249Sはstandalone four-row profileだけでparameter applicationと
member returnをcomposeできません。これはlower `source_drift`です。Task264
docs commit後、separate Task 249PIがapplication 0 (`Task264Carrier`)とmember
rows 0/1 (`carrier`/`marker`)のcombined profileをdocs/implementation別commitで
追加します。Task264は`SourceTypeStructureMemberId(1)`だけをreturn rowとして
consumeし、lower dataをfabricateしません。Exact expression/head site pairはMeansが
parameter `63/64`、field `55/54`、property `58/57`、Equalsがparameter
`45/46`、field `37/36`、property `40/39`です。各transactionの3 expressionsは
normal、argument-free、current-module source typeです。

### Carrier identity transport

Representation-only
[Task264C contract](../../task_contracts/ja/CHECKER-SOURCE-PROPERTY-CARRIER-IDENTITY-264C.md)
はexisting handoffにimmutable `SourcePropertyCarrierIdentity`を1個追加する。
Private fieldはexact structure/field/property resolver tupleを保持し、12個の
role-specific getterがwhole symbol/definition/contribution/semantic originを公開し、
`SourcePropertyImplementationHandoff::carrier_identity()`がaggregateを公開する。
Producerはexisting `SymbolEnv`からderiveし、signatureとTyped/Resolved installation
APIは不変である。

Constructionは3 resolver rowとsole contribution effectsをexact validateする。Replayは
retained normal origin、common module/contribution、parameter type head、retained propertyと
target row 0の一致をvalidateし、failureはexisting
`InvalidResolverTarget { index: 0 }`を使う。このaggregateはauthenticated transport
だけであり、property-implementation shell identity、Core item、property value、accepted
semantic factではない。Deterministic debugは
`source-property-implementation-debug-v2`で、existing payload row前にstructure/field/
property identity rowを出す。

### Equals selector identity association

Representation-only
[Task264D contract](../../task_contracts/ja/CHECKER-SOURCE-PROPERTY-EQUALS-SELECTOR-IDENTITY-264D.md)
はexisting Task264 ownerをwidenせずseparate branded handoffを追加する。Complete
property/primary-term/structure handoffをby-value consumeし、constructionでexplicit
`SymbolEnv`を使い、implementation/definiens 0、
structure term/member/member-identity request 0、base term/reference/binding 0、whole
authenticated `carrier` field symbolまでのexact equals-only chainをpublishする。

Producerはmeans、mixed profile、foreign、fingerprint mismatch、malformed associationを
publish前にrejectする。Task254 requestをgeneric resolveせず、environment/whole resolver
identity/normal originでjoint authenticationし、spelling aloneからsymbolをinferしない。
Existing Task264 handoffとv2 debug bytesは不変で、新handoffだけが独自v1
debugを持つ。これはCore35 prerequisiteであり、term/property value/semantic resultではない。

## Public contract / rows

New moduleはdense IDs
`SourcePropertyImplementationId`、`SourcePropertyParameterId`、
`SourcePropertyTargetId`、`SourcePropertyDefiniensId`、
`SourcePropertyCorrectnessId`と同名5 tablesを公開します。Input tablesは
implementations/parameters/targets/definientia/correctnessです。

Immutable row typeとstored-field順はexactly次です。

| Row | Stored fields（API順） |
| --- | --- |
| `SourcePropertyImplementation` | `id`, `shell`, `site`, `source_range`, `source_ordinal`, `context`, `recovery`, `spelling`, `style`, `parameter`, `target`, `definiens` |
| `SourcePropertyParameter` | `id`, `owner`, `ordinal`, `binding`, `written_type`, `site`, `source_range`, `declaration_range`, `context`, `recovery`, `spelling` |
| `SourcePropertyTarget` | `id`, `owner`, `ordinal`, `subject`, `symbol`, `definition`, `contribution`, `site`, `source_range`, `subject_range`, `name_range`, `spelling`, `return_type`, derived `origin` |
| `SourcePropertyDefiniens` | `id`, `owner`, `ordinal`, `target`, `site`, `source_range`, `context`, `recovery`, `spelling` |
| `SourcePropertyCorrectness` | `id`, `owner`, `ordinal`, `kind`, `site`, `source_range`, `justification`, `recovery`, `spelling`, derived `obligation` |

各fieldには同名read-only getterが1個だけあります。`spelling() -> &str`だけが
`pub fn`、他は全て`pub const fn`です。Exact return signaturesは次です。

- implementation: `id() -> SourcePropertyImplementationId`,
  `shell() -> DeclarationShellId`, `site() -> &TypedSiteRef`,
  `source_range() -> SourceRange`, `source_ordinal() -> usize`,
  `context() -> BindingContextId`,
  `recovery() -> SourcePropertyImplementationRecovery`,
  `style() -> SourcePropertyImplementationStyle`,
  `parameter() -> SourcePropertyParameterId`,
  `target() -> SourcePropertyTargetId`,
  `definiens() -> SourcePropertyDefiniensId`;
- parameter: `id() -> SourcePropertyParameterId`,
  `owner() -> SourcePropertyImplementationId`, `ordinal() -> usize`,
  `binding() -> BindingId`, `written_type() -> SourceTypeApplicationId`,
  `site() -> &TypedSiteRef`, `source_range() -> SourceRange`,
  `declaration_range() -> SourceRange`, `context() -> BindingContextId`,
  `recovery() -> SourcePropertyImplementationRecovery`;
- target: `id() -> SourcePropertyTargetId`,
  `owner() -> SourcePropertyImplementationId`, `ordinal() -> usize`,
  `subject() -> BindingId`, `symbol() -> &SymbolId`,
  `definition() -> DefinitionId`, `contribution() -> SourceContributionId`,
  `site() -> &TypedSiteRef`, `source_range() -> SourceRange`,
  `subject_range() -> SourceRange`, `name_range() -> SourceRange`,
  `return_type() -> SourceTypeStructureMemberId`,
  `origin() -> &SemanticOrigin`;
- definiens: `id() -> SourcePropertyDefiniensId`,
  `owner() -> SourcePropertyImplementationId`, `ordinal() -> usize`,
  `target() -> SourcePropertyDefiniensTarget`, `site() -> &TypedSiteRef`,
  `source_range() -> SourceRange`, `context() -> BindingContextId`,
  `recovery() -> SourcePropertyImplementationRecovery`;
- correctness: `id() -> SourcePropertyCorrectnessId`,
  `owner() -> SourcePropertyImplementationId`, `ordinal() -> usize`,
  `kind() -> SourcePropertyCorrectnessKind`, `site() -> &TypedSiteRef`,
  `source_range() -> SourceRange`, `justification() -> &SourceAnchor`,
  `recovery() -> SourcePropertyImplementationRecovery`,
  `obligation() -> InitialObligationId`。

Table名はexactly `SourcePropertyImplementationTable`、
`SourcePropertyParameterTable`、`SourcePropertyTargetTable`、
`SourcePropertyDefiniensTable`、`SourcePropertyCorrectnessTable`です。各tableは
`get(id) -> Option<&Row>`、dense source順
`iter() -> impl Iterator<Item = (Id, &Row)>`、`const len() -> usize`、
`const is_empty() -> bool`だけを公開します。

Implementation rowはshell/site/range/ordinal/context/recovery/spelling/styleと
parameter/target/definiens IDs、parameterはowner/binding/written type/ranges、
targetはowner/subject binding/resolver symbol+definition+contribution/ranges/
return member row、definiensはlower target、correctnessはkind/site/range/
justificationを保持します。Target siteはimplementation node上のtyped roleで、
header range `157..165`、subject/name `157..158`/`159..165`です。Exact roleは
`TypeRole::new("source.property-implementation.target")`で、Meansはowner 81、
Equalsはowner 52です。

Public non-exhaustive enumsはstyle `Equals/Means`、definiens target
`Primary/Application/Structure/SetTerm/AtomicFormula`、correctness
`Existence/Uniqueness`、recovery `Normal/Degraded`、category-specific errorです。
ProducerはSymbolEnv、source context/type/term、optional application/structure/
set/atomic、base obligations、arenaをexplicitにconsumeし、atomic projectionを
返します。Mutable getter/raw syntax/public constructor/replacement/property valueは
ありません。Handoff getter surfaceはexactly次です。

```rust
pub struct SourcePropertyCarrierIdentity { /* private fields */ }

impl SourcePropertyCarrierIdentity {
    pub fn structure_symbol(&self) -> &SymbolId;
    pub const fn structure_definition(&self) -> DefinitionId;
    pub const fn structure_contribution(&self) -> SourceContributionId;
    pub const fn structure_origin(&self) -> &SemanticOrigin;
    pub fn field_symbol(&self) -> &SymbolId;
    pub const fn field_definition(&self) -> DefinitionId;
    pub const fn field_contribution(&self) -> SourceContributionId;
    pub const fn field_origin(&self) -> &SemanticOrigin;
    pub fn property_symbol(&self) -> &SymbolId;
    pub const fn property_definition(&self) -> DefinitionId;
    pub const fn property_contribution(&self) -> SourceContributionId;
    pub const fn property_origin(&self) -> &SemanticOrigin;
}

impl SourcePropertyImplementationHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub const fn carrier_identity(&self) -> &SourcePropertyCarrierIdentity;
    pub fn source_context_fingerprint(&self) -> &str;
    pub fn source_type_fingerprint(&self) -> &str;
    pub fn source_term_fingerprint(&self) -> &str;
    pub fn source_functor_application_fingerprint(&self) -> Option<&str>;
    pub fn source_structure_fingerprint(&self) -> Option<&str>;
    pub fn source_set_term_fingerprint(&self) -> Option<&str>;
    pub fn source_atomic_formula_fingerprint(&self) -> Option<&str>;
    pub const fn implementations(&self)
        -> &SourcePropertyImplementationTable;
    pub const fn parameters(&self) -> &SourcePropertyParameterTable;
    pub const fn targets(&self) -> &SourcePropertyTargetTable;
    pub const fn definientia(&self) -> &SourcePropertyDefiniensTable;
    pub const fn correctness(&self) -> &SourcePropertyCorrectnessTable;
    pub fn debug_text(&self) -> String;
}
```

Required three fingerprintsはcomplete lower debug、optional fourはabsent時
`None`、present時`Some(&str)`で、caller suppliedではありません。Exact debug
grammarは次の順で、blank lineなし、final LF exactly oneです。`Rust-debug`は
standard escaped `{:?}`、optional absentはunquoted `none`、presentは
`some(<Rust-debug String>)`です。`<definiens-target>`は
`primary#<id>`/`application#<id>`/`structure#<id>`/`set-term#<id>`/
`atomic-formula#<id>`のexactly oneです。

```text
source-property-implementation-debug-v2
module: <ModuleId.path>
source-context-fingerprint: <Rust-debug String>
source-type-fingerprint: <Rust-debug String>
source-term-fingerprint: <Rust-debug String>
source-functor-application-fingerprint: <none|some(Rust-debug String)>
source-structure-fingerprint: <none|some(Rust-debug String)>
source-set-term-fingerprint: <none|some(Rust-debug String)>
source-atomic-formula-fingerprint: <none|some(Rust-debug String)>
carrier-identity#0 role=structure symbol=<Rust-debug FQN string> definition=<id> contribution=<id> origin_range=<start>..<end> origin_path=<Rust-debug [u32]>
carrier-identity#1 role=field symbol=<Rust-debug FQN string> definition=<id> contribution=<id> origin_range=<start>..<end> origin_path=<Rust-debug [u32]>
carrier-identity#2 role=property symbol=<Rust-debug FQN string> definition=<id> contribution=<id> origin_range=<start>..<end> origin_path=<Rust-debug [u32]>
implementation#<id> shell=<id> range=<start>..<end> site=node#<id> ordinal=<n> context=<id> recovery=<normal|degraded> spelling=<Rust-debug String> style=<equals|means> parameter=<id> target=<id> definiens=<id>
parameter#<id> owner=<id> ordinal=<n> binding=<id> written_type=<id> range=<start>..<end> declaration_range=<start>..<end> site=node#<id> context=<id> recovery=<normal|degraded> spelling=<Rust-debug String>
target#<id> owner=<id> ordinal=<n> subject=<id> symbol=<Rust-debug FQN string> definition=<id> contribution=<id> range=<start>..<end> subject_range=<start>..<end> name_range=<start>..<end> site=role#<node>:source.property-implementation.target spelling=<Rust-debug String> return_type=<id> origin_range=<start>..<end> origin_path=<Rust-debug [u32]>
definiens#<id> owner=<id> ordinal=<n> target=<definiens-target> range=<start>..<end> site=node#<id> context=<id> recovery=<normal|degraded> spelling=<Rust-debug String>
correctness#<id> owner=<id> ordinal=<n> kind=<existence|uniqueness> range=<start>..<end> site=node#<id> justification=range:<start>..<end> recovery=<normal|degraded> spelling=<Rust-debug String> obligation=<id>
```

Active cardinalityはMeans `1/1/1/1/2`、Equals `1/1/1/1/0`です。Parameterは
binding/application 0、range `121..145`、declaration `125..126`、context 1。
Targetはmarker definition 2、return member 1です。Means definiensは
`AtomicFormula(0)` site/range `72/172..179`、Equalsは`Structure(0)`
`51/173..182`です。Correctness sites/rangesは`76/183..218`と
`80/221..257`、anchors `193..217`/`232..256`です。

Means implementation spellingはfinal LFなしでexactly
`definition\n  let M be Task264Carrier;\n  property M.marker means it = it;\n  existence by computation(steps: 1);\n  uniqueness by computation(steps: 1);\nend;`、
Equalsは
`definition\n  let M be Task264Carrier;\n  property M.marker equals M.carrier;\nend;`
です。全input/output rowは`Normal`で、`Degraded`はrow-familyの
`Invalid* { index }`になります。

Task264-owned/direct lower typed kindはMeansで54/57/64
`source.type.head`、55/58/63 `source.type.expression`、56/59
`source.definition.structure.member`、60 `source.definition.structure`、65
`source.definition.property-implementation.parameter`、66/68
`source.term.it`、70 `source.formula.atomic.equality`、72
`source.definition.property-implementation.definiens`、76/80
`source.definition.property-implementation.correctness`、81
`source.definition.property-implementation`、84 `source.module`です。Equalsは
36/39/46 `source.type.head`、37/40/45 `source.type.expression`、38/41
`source.definition.structure.member`、42 `source.definition.structure`、47
`source.definition.property-implementation.parameter`です。さらにEquals node
31は`source.term.structure.member.selector`、48は
`source.term.variable-reference`、49 `source.term.structure.selector`、51
`source.definition.property-implementation.definiens`、52
`source.definition.property-implementation`、55 `source.module`です。Other nodesは
`source.surface.unowned`、wrong kindはpublication前
`InvalidArenaOwnership`です。Parameter ownership node 65/47だけはfrozen
Task248P context contractに従いdeclaration anchor `125..126`を使い、parameter
payload row自体はsource range `121..145`、declaration range `125..126`を保持します。
Other owned nodesはexact Surface range、全nodeは`TypingState::Unknown`、
`NodeRecoveryState::Normal`、lower-authenticated context linkです。

Meansだけがformula targetとdefiniens内exact two
`It/CurrentDefinitionResult`をadmitします。One/three/zero `it`、means外
relocation、role/spelling driftはexactly
`InvalidDefiniens { index: 0 }`です。Equalsはnon-formula term、correctness zero、
`it` zeroで、Equalsへの`it`注入またはEquals+formulaもfail closedです。Guard
table/input/fingerprintは存在せず、mode typeからFOL guardをinventしません。

## Obligations / Typed / Resolved

Initial kindsはexactly `PropertyImplementationExistence` /
`PropertyImplementationUniqueness`をappendし、serializer bytesは
`property_implementation_existence` / `property_implementation_uniqueness`です。
Meansはbaseline `b/b+1`へpending two rows、Equalsはzeroです。Assumptionsは
emptyで、opaque goal/provenanceはimplementation 0/correctness 0/1をstableに
encodeします。これはunguarded semantic claimではなく、goal/guard/FOL
compositionをdeferする境界です。

One-shot `TypedAst::with_source_property_implementation`とtyped/final getters、
`InvalidSourcePropertyImplementation` errorsを追加します。TypedAstPartsと
ResolvedTypedAstInputsにpublic construction fieldは追加しません。Task259
handoff/`PredicatePropertyCorrectness`、functor kinds、orphan/extra/mismatched
property kindsはtyped/finalでrejectします。Task259 codeとmixed gap expectationは
unchangedです。

## Tests / impact / deferrals

New pass pairsはmeans/equals各1件で、one covered trace requirement
`spec.en.checker.type_elaboration.source_property_implementation_payload`を
reciprocalに参照します。Checker tests five、runner tests fourです。Checkerは
exact rows/debug/obligations、independent corruption、resolver/return/lower/
obligation corruption、transaction、final/Task259 isolationをcoverします。
Runnerはboth exact sources、all owner mutations、two-case trace selection、
inactive coherence/mixed isolation、no semantic publicationをcoverします。
Frozen impossible-state `it` validationをexerciseするため、`source_term.rs`には
generic `#[cfg(test)]` raw-term corruption seamをexactly 1件だけ許可します。この
seamはproduction behavior/public APIを追加せず、他のunrelated lower producer変更は
禁止です。

Task249PIはchecker-local lower tests exactly four、runner zeroを追加して
`469 -> 473`へrebaselineします。Task264 projected countsはchecker
`473 -> 478` (+5)、runner `528 -> 532` (+4)、resolver/syntax
`148/59`、corpus/requirements `426/394 -> 428/395`、pass/fail
`233/193 -> 235/193`、active type `203 -> 205`、type requirements
`258 = 246+12 -> 259 = 247+12`、warnings/errors `23/0`です。Checker/runner
production pathsは`28 -> 29`/`35 -> 36`で、line/content/path hashesは実装後
remeasureします。Docs prerequisiteはexecutable/fixture/trace/count/hashを
一切変更しません。

Deferredは禁止事項はgoal/guard/return/definiens composition、`it` substitution、
FOL existence/uniqueness/coherence、overlap/coherence seed activation、proof/
discharge/acceptance/facts/axioms、use-site property lookup、selector result typing、
case/otherwise、import/multiple/inheritance/redefinition、Core/CFG/VCです。

Docs exitはEN/JA sync、repeated **NO FINDINGS**、all nine gates PASS、uncapped
90+、docs-only stage/commit、clean/stash invariant、fresh Task249PI selectionです。
Task249PI docs/implementation後Task264へ自動復帰し、Task264も同じreviews/gates、
exact count/hash、one logical commitを満たします。

## Public Enum Policy

| Public enum | compatibility policy |
| --- | --- |
| `SourcePropertyImplementationStyle` | `#[non_exhaustive]`。callerはlater explicitly-frozen implementation styleを許容する。 |
| `SourcePropertyDefiniensTarget` | `#[non_exhaustive]`。callerはlater explicitly-frozen lower-root targetを許容する。 |
| `SourcePropertyCorrectnessKind` | `#[non_exhaustive]`。callerはlater explicitly-frozen correctness kindを許容する。 |
| `SourcePropertyImplementationRecovery` | `#[non_exhaustive]`。callerはlater recovery classを許容する。 |
| `SourcePropertyImplementationError` | `#[non_exhaustive]`。callerはvalidation failureをexhaustive matchしない。 |
| `SourcePropertyEqualsSelectorIdentityError` | `#[non_exhaustive]`。callerはlater fail-closed association failureを許容する。 |

この module が所有する exhaustive public enum exception はない。

## Implemented Task 264 result

frozen contractをcanonical spec、既存fixture、既存expectationを変更せず実装した。
checker productionは`29` path / `162347` line、path/content hashは
`37b91c2c419b83fa63150fe65d09b56c474dfa3d61134ba84056009dcdb923c1` /
`450abc3b7407f206c27b04613737716cf2192fb46c8960c8e167fcf0900fa143`。
checker library testは`478`、raw/normalized hashは
`b3d0b2e398899adac6b94c7bbaba93d89fdc2067452e6b3c16efb60783401b8d` /
`4d9c7f9821182f08aa37686c7fecc1374d3857fdb7fdd64c83520dd05988d500`。

runner productionは`36` path / `69417` line、path/content hashは
`38a20909d1f89aa2a4c325fb47126cc911bb943b7fe1190dc668713f64ad49e2` /
`72cc9036654639dff5933dced07e79ec6132696b5f92eca5e0149085f4651d91`。
runner library testは`532`、raw/normalized hashは
`8122a53fddb8ee98cf1225f43c4f6966f3f7b5718673f55218601ca3464ca293` /
`fbd9e691357c14cd413df7ffd46677e34914bbacffca4dc2fe25a856d3b9434a`。

source/sidecar hashはequals
`175135aaf40b9eab1a28e73ca1aae9f250e66278410d50575cdd279f6d7a2784` /
`c491d7ea65e1c096d869af4666a06a053a5a0b213d9e79483d13e5ec91b75b6e`、
means
`cc90659f10cae4ef68890624df9b8b9d3f0e830dae5e20cc195dc8b263c5fa2b` /
`bced77302602f43f3237424aa2963e5522c1458e879e606c68d1a516cd737c3a`。
trace hashは`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`。
metadataは`428/395`、pass/fail `235/193`、active stage `101/7/205/1`、type
coverage `259 = 247 + 12`、warnings/errors `23/0`。plan/parse/declaration/type/proof
stdout hashは
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`、
`a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`、
`71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`、
`4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`、
`ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`。
