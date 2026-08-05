# mizar-checker: source type application projection

> canonical languageはEnglish。English canonical:
> [../en/source_type.md](../en/source_type.md)。

## Purpose And Authority

`source_type`は[`00.crate_plan.md`](./00.crate_plan.md)でfreezeしたTask 249
type head/application/argument producerを実装する。canonical authorityは
Chapters 03 §§3.2-3.3、05 §§5.2/5.6、07 §§7.2-7.3/7.6、08 §§8.1/8.3、
12 §§12.3/12.5/12.6.1/12.7、18 §§18.1/18.2.2、Appendix Aである。
bounded audit ownerはMC-G014、MC-G016、MC-G020である。

## Boundary And Model

moduleは`SurfaceAst`、`SurfaceNodeId`、`SyntaxKind`を受け取らない。
syntax-free `SourceTypeHandoffInput`はdense outer-application、
expression/head、ordered argument vectorを持つ。applicationはauthenticated
reserve/definition bindingをroot expressionへlinkする。expressionはwritten/
head site、range、spelling、recovery、form、builtinまたはresolver-authenticated
mode/structure headを保持する。argumentは`TermSite`、recursive `TypeSite`、
`QuaSite`だけであり、term/`qua` siteは`SemanticOrigin`を持つがselected
`BindingId`を持たない。

`SourceTypeProducer`はactual `BindingEnv`、`SymbolEnv`、`TypedArena`に対して
inputをauthenticateしてから`SourceTypeApplicationHandoff`をpublishする。
legacy reserve bridgeの`prepare_binding_env`はinput-only pathであり、symbol
headをvalidateしてreal binding environmentを構築するが、declaration checkや
type normalizationは実行しない。definition-parameter applicationはactual
resolver `DeclarationShell` ownerを要求し、generated contextをdeclarationとして
authenticateしない。

## Validation And Atomicity

cross-source/module、stale binding identity/order/type site、unsupported head
kind、stale contribution provenance、siteより後のlocal head、invisible imported
head、missing/out-of-closure import edge/target、invalid/duplicate typed site、
empty spelling、range/recovery mismatch、invalid `SemanticOrigin`をrejectする。
Term/`qua` provenanceはexact identifier range、current source/module、import
edgeなし、matching recovery、deterministic
`[parent-expression, argument-ordinal]` structural pathを要求する。

flat graphはdangling、cycle、multiple parent、forward parent、duplicate child、
wrong form、unreachable、non-contained、overlapping sibling/top-level relationを
rejectする。cycle/reachability checkはiterative worklistを使い、public flat
inputでcall stackを消費しない。inputをsort/repairせず、failure時はpartial
handoffをpublishしない。

全expression/head/term/`qua` siteはproducer時と`TypedAst` install時の両方で
actual typed-arena nodeへ照合する。owning nodeはsame-source rangeでnarrower row
rangeをcontainし、recoveryが完全一致しなければならない。これにより既存Task-248
item node上のdistinct role siteをarena変更なしで利用できる。

## Ownership, Consumers, And Exclusions

optional immutable handoffは`TypedAst`が所有する。`ResolvedTypedAst`はtyped AST
からcloneするだけで、separately replaceable resolved inputを持たない。handoff
absent時はconditional debug renderingによりlegacy byteを維持する。

broad real consumerはexact ten reserve written typeをtraverseし、application
10、expression/head 13、argument 6をpublishする。Task-248 routeは別にactual
checker-owned binding environmentを使い、`Bare`/builtin-`set` 2 rowとargument
0をco-installする。expansion、normalization、inhabitation、subtyping、
evidence、term/`qua` binding selection、fact、declaration/proof acceptance、
Core/CFG/VCはTask 249外である。

## Public Enum Policy

| Public enum | Compatibility policy |
|---|---|
| `SourceTypeApplicationForm` | `#[non_exhaustive]`。callerはlater source-written formを許容する。 |
| `SourceTypeHead` | `#[non_exhaustive]`。callerはlater authenticated head kindを許容する。 |
| `SourceTypeArgument` | `#[non_exhaustive]`。callerはlater syntax-free argument shapeを許容する。 |
| `SourceTypeError` | `#[non_exhaustive]`。callerはvalidation failureをexhaustive matchしない。 |
| `SourceProofLocalLetTypeError` | `#[non_exhaustive]`。callerはTask-269CT dependency、overlay、source-type、installation failureをexhaustive matchしない。 |
| `SourceProofLocalGivenTypeError` | `#[non_exhaustive]`。callerはTask-269GT dependency、overlay、source-type、installation failureをexhaustive matchしない。 |
| `SourceProofLocalGivenUseTypeError` | `#[non_exhaustive]`。callerはTask-269GUPT dependency、copied overlay、source-type、installation failureをexhaustive matchしない。 |
| `SourceProofLocalGivenConditionTypeError` | `#[non_exhaustive]`。callerはTask-269GCT dependency、upgraded overlay、source-type、installation failureをexhaustive matchしない。 |
| `SourceProofLocalGivenDescendantTypeError` | `#[non_exhaustive]`。callerはTask-269SDT dependency、upgraded overlay、source-type、installation failureをexhaustive matchしない。 |

この module が所有する exhaustive public enum exception はない。

## Task 249 Classification

| Class | Result |
|---|---|
| `test_gap` | exact Task-249 handoffとTask-248 dependency consumerについてのみclose。 |
| `source_drift` | complete type-head/application/argument、final-handoff transport、import-closure authentication、real `DeclarationShell` ownershipをrepair。 |
| `design_drift` | paired component/plan/todo/audit/runner docsでrepair。 |
| `boundary_violation` | implementation reviewで検出したrecursive public-input graph traversalをiterative worklistへ置換。syntaxはrunner-owned、semantic result fabricationは禁止。 |
| `spec_gap` | bounded input-handoff sliceにはなし。 |
| `repo_metadata_conflict` | 観測なし。 |

## Task 251 evidence-association addendum

`SourceTypeApplicationHandoff`は全Task-251 requestのauthenticated parent
inputである。unattributed requestはroot expression、owner/head site、
expression range/recovery、application source ordinalを保持する。attributed
requestは同じapplication/expression identityを保持し、independent Task-250
chainがrequest site/range/recoveryを供給するが、request ordinalはTask-249
application ordinalのままである。resolver-authenticated mode/structure headが
unattributed request kind 2件をselectし、builtin headはrequestをemitしない。
Task 251はsource-type tableを変更せず、expansion/inhabitation/normalization/
acceptanceをinferしない。

## Task 249R frozen definition-return extension

### Selection / authority / classification

Task-260 implementationのfresh preflightで、completed Task-249 producerは
authenticated `BindingId`ごとにexactly one `SourceTypeApplicationId`を要求する
ことを確認した。Task 248 Profile Bのbindingは2件なので、parameter type 2件と
functor return type 2件をTask-260 `4/4/0`として表すにはbinding 2件を捏造する
必要があり、これは`boundary_violation`である。frozen Task-260 docsは
nonblocking `design_drift`、independent return transportの欠落は`source_drift`
である。blocking `spec_gap`はない。Chapter 10 §10.1は各`func`の`->`後に
independent written typeを要求し、§10.5はinput-dependent return typeを許可する。

Checker Task 249RはTask 260のdependency-ready lower prerequisiteである。
authorityはChapter 10 §§10.1/10.5とfrozen Task-260 exact source/Surface profile、
consumerはTask 260だけである。language semantics、canonical spec、existing
`.miz`/sidecar/expectation/trace、runner/resolver/Cargo metadataは変更しない。

### Exact additive public ABI

Task 249Rはexisting immutable `SourceTypeApplicationHandoff`をextendし、
binding-linked application tableを緩和・overloadしない。exact public typesは
canonical ENの`SourceTypeDefinitionReturnId`、
`SourceTypeDefinitionReturnExtensionInput`、
`SourceTypeDefinitionReturnInput`、`SourceTypeDefinitionReturnTable`、
`SourceTypeDefinitionReturn`、`SourceTypeDefinitionReturnProducer`であり、field
names/order/typesもEN code blockどおりである。

dense ID/table/row/handoff getterのfull signature、return type、iterator item
shape、constnessとderiveはcanonical EN code blockどおりである。producerは
borrowed base、extension input、`TypedArena`を取りnew handoffまたは
`SourceTypeError`を返すexact `extend` signatureを持つ。

base producerはempty return tableを初期化し、extensionはone-shot、failure時に
borrowed baseを変更しない。error追加は`EmptyDefinitionReturns`、
`DefinitionReturnCardinalityMismatch`、`DefinitionReturnsAlreadyPresent`、
`InvalidDefinitionReturnBase`、
`InvalidDefinitionReturn`、
`InvalidDefinitionReturnSite`、`UnsupportedDefinitionReturn`、
`OverlappingDefinitionReturns`で、ID fieldはcanonical ENどおりである。

### Exact Task-260 profile / validation

base applications/expressions/argumentsはTask-248 Profile Bの`2/2/0`。return
row/expressionを2件追加してapplications/expressions/arguments/definition
returnsは`2/4/0/2`となる。row 0はdefinition node 84/range `61..118`/ordinal 0、
expression/head nodes 80/79/range `105..108`/root 2。row 1はnode 95/
`121..179`/ordinal 1、nodes 87/86/`163..166`/root 3。exactly two rowsだけを
admitし、両方ともnormal、argument-free、`Bare` builtin `set`である。

exact base applicationは`(binding 0, ordinal 0, root 0)` / `(binding 1,
ordinal 1, root 1)`。base expression 0/1はnode/head 63/62と67/66、range
`22..25`/`38..41`、`Bare` builtin `set`、normal、spelling/head spelling `set`、
argument/definition-return emptyである。他のbase shapeは
`InvalidDefinitionReturnBase`となる。

source/moduleはbaseと一致し、return ordinal/IDはvector order。definition owner
rangeはactual same-source arena nodeのexact rangeで、ordered/nonempty/
nonoverlappingかつreturn expressionをcontainする。expression/head site/range/
recoveryをactual arenaへ再照合し、syntax-free input spelling/head spellingは
それぞれ`set`と一致しなければならない。definition/expression/head siteはexact
`TypedSiteRef::Node`だけでrole siteをrejectする。uniqueness scopeはcombined
source-type handoff内、すなわちbase expression/head siteとnew definition/
expression/head triple 2件だけで、cross-family arena-site reuseは不変である。
new expression IDはprevious expression lengthからappendする。baseは
extension前に再validateし、`TypedAst` installationがreturn rowとall four
expressionを再validateする。`TypedAst`がsole ownerで、final assemblyはalready
validated immutable valueをtrustし、`ResolvedTypedAst`は同じhandoffをcloneする
だけである。second field/install pathは追加しない。

return table empty時のdebug byteは完全に不変。present時はapplication row後、
expression row前にcanonical ENの`definition-return#...` lineをemitする。
combined debug全体がTask 260 required source-type fingerprintで、Task 260は
return row 0/1を`SourceTypeDefinitionReturnId`で参照し、
`SourceTypeApplicationId`を使わない。

### Tests / exclusions / audit / exit

implementationはcanonical ENで命名したchecker test exactly 4件を追加する。
それぞれexact API/debug/legacy stability、independent corruption/atomic failure、
one-shot/base/environment/arena fail-closed、TypedAst/ResolvedTypedAst clone/
replay/no semantic outputを所有する。
checker `435 -> 439`、runner/resolver/syntax `512/144/59` unchanged。Task 260は
その後checker `439 -> 444`、runner `512 -> 516`。両Task-249R commitでcorpus/
metadata/CLI/fixture/sidecar/expectation/trace count/hashは不変である。

artificial `BindingId`、general composite/attributed/dependent return graph、exact
owner rowを越えるassociation、expansion/normalization/inhabitation/subtype/
evidence、goal/guard composition、proof/discharge/acceptance/fact/axiom/Core/CFG/
VC、public diagnostic、Task-260 producer/runner workはforbidden/deferred。
docs prerequisiteはEN/JA sync、review-only **NO FINDINGS**、unchanged executable/
count/hash、all nine gates、quality 90+、dedicated docs commit、clean/stash
invariantでexitする。separate implementationはfour tests、exact `2/4/0/2`、
full verification/reviews/gates、dedicated commit後、Task 260へ自動復帰する。

### Task 249R implementation closure

checker implementationはfrozen additive ABIを`source_type.rs`に実装した。
`SourceTypeProducer::build`はlegacy empty-table byteを保ち、
`SourceTypeDefinitionReturnProducer::extend`はexact Task-249/Profile-B baseと
exact two-row Task-260 return profileだけをacceptする。installationはowner/
expression/headの全arena fieldを再validateし、final ownerは同じimmutable
handoffをclone-preserveする。追加testはfrozen checker 4件だけであり、runner/
resolver/syntax code、corpus artifact、trace row、diagnostic、fact、proof、
acceptance、VC behaviorは追加しない。

fresh executable inventoryはapplications/expressions/arguments/returns
`2/4/0/2`、checker `439`、runner/resolver/syntax unchanged `512/144/59`。
`source_type.rs`は`4407` lines、checker production manifestは`24/148143`で
ある。metadata CLI 5本のoutput/hashは全て不変。Task 260がsole next consumerで
あり、上記semantic deferralはすべて継続する。

## Task 249M frozen standalone mode-RHS extension

### selection、authority、classification

fresh Task-262 preflightはexisting `SourceTypeApplicationInput`が各
`BindingId`へone-to-one linkすることを再確認した。exact mode definitionは
parameter binding 2個に対しwritten type expression 3個、すなわちparameter
type root 0/1とindependently written mode RHS root 2を持つ。root 2をthird
applicationとして扱うことはbinding fabricationで`boundary_violation`になる。
missing independent RHS ownerは`source_drift`で、committed Task-262 upper
contractが対応する`design_drift`をrepair済みである。Chapter 7 §§7.1--7.3/
7.6--7.8がparameter tupleとmode RHS/expansionを区別し後者のinhabitationを
要求するためblocking `spec_gap`はない。

Checker Task 249MはTask 262のmandatory lower-stage prerequisiteである。
authority/consumerはexact Chapter-7 RHSとfrozen 141-byte Task-262 source/
54-row Surface oracleだけである。language semantics、canonical spec、existing
`.miz`/sidecar/expectation/trace row/status/count、runner/resolver/parser、public
diagnostic、Cargo metadataを変更しない。Task 262 production/corpus/trace
activationはlater logical taskのままである。

### exact additive public ABI

Task 249Mはbinding-linked application tableを弱めずTask 249R return semanticsを
reuseせず、immutable `SourceTypeApplicationHandoff`をextendする。exact new
public typesは次のとおりである。

```rust
pub struct SourceTypeModeRhsId(usize);

pub struct SourceTypeModeRhsExtensionInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub rhs: Vec<SourceTypeModeRhsInput>,
}

pub struct SourceTypeModeRhsInput {
    pub definition_site: TypedSiteRef,
    pub definition_range: SourceRange,
    pub source_ordinal: usize,
    pub expression: SourceTypeExpressionInput,
}

pub struct SourceTypeModeRhsTable { /* private entries */ }

pub struct SourceTypeModeRhs {
    /* private id, definition_site, definition_range, source_ordinal, root */
}

pub struct SourceTypeModeRhsProducer;
```

IDは`Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash`、input
struct 2個/immutable rowは`Debug + Clone + PartialEq + Eq`、tableはそれらに
`Default`を加えてderiveする。exact read-only method/constnessは次のとおり。

```rust
impl SourceTypeModeRhsId {
    pub const fn new(index: usize) -> Self;
    pub const fn index(self) -> usize;
}

impl SourceTypeModeRhsTable {
    pub fn get(&self, id: SourceTypeModeRhsId) -> Option<&SourceTypeModeRhs>;
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (SourceTypeModeRhsId, &SourceTypeModeRhs)>;
    pub const fn len(&self) -> usize;
    pub const fn is_empty(&self) -> bool;
}

impl SourceTypeModeRhs {
    pub const fn id(&self) -> SourceTypeModeRhsId;
    pub const fn definition_site(&self) -> &TypedSiteRef;
    pub const fn definition_range(&self) -> SourceRange;
    pub const fn source_ordinal(&self) -> usize;
    pub const fn root(&self) -> SourceTypeExpressionId;
}

impl SourceTypeApplicationHandoff {
    pub const fn mode_rhs(&self) -> &SourceTypeModeRhsTable;
}
```

producer surfaceはexactに次である。

```rust
impl SourceTypeModeRhsProducer {
    pub fn extend(
        base: &SourceTypeApplicationHandoff,
        input: SourceTypeModeRhsExtensionInput,
        arena: &TypedArena,
    ) -> Result<SourceTypeApplicationHandoff, SourceTypeError>;
}
```

`SourceTypeProducer::build`はdefinition-return/mode-RHS tableをempty initialize
する。`extend`はone-shotでfailure時もborrowed baseを変更せず、success時にnew
immutable handoffを返す。`SourceTypeError`はnon-exhaustive variant
`EmptyModeRhs`、`ModeRhsCardinalityMismatch`、`ModeRhsAlreadyPresent`、
`InvalidModeRhsBase`、`InvalidModeRhs { mode_rhs: SourceTypeModeRhsId }`、
`InvalidModeRhsSite { mode_rhs: SourceTypeModeRhsId }`、
`UnsupportedModeRhs { mode_rhs: SourceTypeModeRhsId }`を追加する。error
precedenceはalready-present、empty、non-singleton cardinality、environment、
base、row、owner/site、unsupported expressionである。

### exact Task-262 lower profile と validation

required baseはTask-262 Task-248/Profile-B Task-249 profileのapplications/
expressions/arguments/definition-returns/mode-RHS `2/2/0/0/0`である。Task
249MはRHS row/expression各1個をappendし`2/3/0/0/1`にする。

| Row | Definition owner | RHS expression/head | Output root |
| ---: | --- | --- | ---: |
| 0 | node 49, `45..135`, ordinal 0 | nodes 44/43, `95..98`, `Bare`, builtin `set`, normal | 2 |

exact base applicationは`(binding 0, ordinal 0, root 0)`と`(binding 1,
ordinal 1, root 1)`である。base expression 0/1はexpression/head site 35/34、
39/38、range `22..25`/`38..41`、`Bare`、builtin `set`、normal、両spelling
`set`である。argument/definition return/mode RHSはempty。Task-249R handoffを
含む他shapeは`InvalidModeRhsBase`を返す。

normal/argument-free/bare builtin-`set` RHS exactly 1個だけをadmitする。
source/module identityはbaseと一致し、dense row ID/source ordinalは0。
definition rangeはsame-source arena node 49のexact rangeで、RHS expressionを
containしnonemptyである。definition/expression/head siteはexact
`TypedSiteRef::Node`でrole siteをrejectする。expression/head site/range/
recoveryはactual arenaへrevalidateし、両syntax-free spellingはexact `set`。
new siteはbase expression/head/definition siteとduplicateできない。new
expression IDはprior expression lengthへroot 2としてappendする。

extension前にbase全体をrevalidateし、installationはmode-RHS rowとexpression
3個をrevalidateする。definition-return/mode-RHS extensionは両orderでmutually
exclusive、malformed combined stateはfail closed。`TypedAst`がsole ownerで、
`ResolvedTypedAst`は同handoffをclone-preserveするだけである。second field/
installer/parts field/replaceable final inputは追加しない。

mode-RHS table empty時はexisting debug prefixとlegacy/Task-249R byteをすべて
維持する。Task 249M present時はdefinition-return row後、expression row前に
次をrenderする。

```text
mode-rhs#<id> ordinal=<n> definition_range=<start>..<end> definition_site=node#<id> root=<expression-id>
```

active lower profileのcomplete suffixはrow 0とdense expression 0--2で、argument
rowはexisting final positionを維持する。complete combined debug textがfuture
Task-262 source-type fingerprintである。Task 262はrow 0を
`SourceTypeModeRhsId`で参照し、`SourceTypeApplicationId`/
`SourceTypeDefinitionReturnId`を使わない。

### tests、exclusions、audit impact、exit

implementationはchecker library test exact 4個を追加する。

1. `task_249m_exact_mode_rhs_extension_and_legacy_debug`
2. `task_249m_mode_rhs_corruption_fails_atomically`
3. `task_249m_one_shot_base_and_arena_drift_fail_closed`
4. `task_249m_typed_final_clone_replay_and_task_249r_isolation`

各testはexact extension/API/debugとlegacy/Task-249R byte stability、empty/
multiple/environment/owner/expression/site/spelling/recovery corruptionと
borrowed-base atomicity、exact-base/one-shot/arena/installation drift、Typed-to-
Resolved clone/replay/two-way Task-249R isolation/no semantic outputをownする。
全field/error classをindependently mutateする。test 2と3はalready-present over
empty、empty over non-singleton cardinality、cardinality over environment
mismatch、environment mismatch over invalid base、invalid base over invalid
row、invalid row over invalid owner/site、invalid owner/site over unsupported
expressionという全adjacent precedence boundaryのcompound mutationもownする。

checker baselineは`449 -> 453`、runner/resolver/syntaxは`520/144/59`。
metadataはcases/requirements `424/392`、pass/fail `231/193`、active parse/
declaration/type/proof `101/7/201/1`、type requirements `256/244`、warnings/
errors `23/0`を維持する。Task 262がlater checker/runner `458/524`とsole
corpus/trace deltaをownする。production/test-list/CLI/manifest hashは
implementationでfresh-measureする。

artificial `BindingId`/application row、definition-return rowのreuse/rename、
generalized/attributed/argument-bearing/resolver-symbol/structure/recovered mode
RHS、request/inhabitation response、expansion/normalization/acceptance、sethood
goal/guard/proof/discharge/fact、public diagnostic、Core/CFG/VC、全Task-262
checker/runner/corpus/trace changeは禁止/deferredである。

本documentation prerequisiteはsynchronized design recordだけを変更し、repeated
review-only **NO FINDINGS**、unchanged executable/count/hash gate、hard gate 9件、
quality 90以上、dedicated docs commit、clean/stash-invariant fresh inventoryで
exitする。separate implementation write scopeは
`crates/mizar-checker/src/source_type.rs`とsynchronized design recordだけで、
exact test 4件、`2/3/0/0/1` profile、full review/verification/gate、dedicated
commit後にTask 262 implementationへautomatic fresh-inventory returnする。

## Task 249M active implementation result

先行のfuture/prerequisite記述は歴史である。frozen APIは
`crates/mizar-checker/src/source_type.rs`に実装済みで、dense mode-RHS ID、
extension/row input、immutable row/table、borrowed getter、unit producer、frozen
precedenceのerror 7件、exact base/arena revalidation、one-shot atomic extension、
installation validation、deterministic debug order、bidirectional Task-249R exclusionを
publishする。

exact named test 4件は全field/error class、全adjacent compound-precedence
boundary、legacy/Task-249R byte、arena/installation drift、Typed/Resolved clone/
replay、empty semantic outputをcoverする。lower oracleはexact `2/3/0/0/1`。
checker libraryは`453`、raw/normalized test-list hashは
`34f63b3b9fb1ae2f3b43d769184be2b0c23cc3ada13b5a8b45a933aed629fe25` /
`ee25ffd88d06e34491ced5c0499acc4198c1e8690ed40c3fb79fb276e3852db4`。
productionは`26/153116`、path/content hashは
`e290d082e428124d3fd21919e76b88458daabfa44b7009a8cb1b3d8c430fec53` /
`3c85673ebb527cb33bb4b042b1b1194bda34a5348b4b6b20142617db47bde2f2`。
runner/resolver/syntax/corpus/trace/CLI/metadataはfrozen baseline不変。Task 262は
sole next consumerかつseparate logical taskである。
## Task 249S standalone structure-member type intake frozen contract

### 選択、authority、分類

Checker Task 263のfresh preflightで、exact 320-byte structure sourceはparameter
binding 0件に対しindependently written member type expressionを4件持つことを
確認した。`SourceTypeProducer`はnonempty `SourceTypeApplicationInput`と
authenticated `BindingId`の一対一対応を意図的に要求するため、binding 4件を
捏造することは`boundary_violation`である。Task-249R definition-return rowや
Task-249M mode-RHS rowのreuseもowner semanticsが異なる。standalone member-type
ownerの欠落は`source_drift`、このlower contractの欠落は`design_drift`である。
blocking `spec_gap`はない。canonical Chapter 5 §§5.1--5.3は各field/propertyに
written typeを明示し、§5.2はproperty valueをconstructor argumentから除外する。

Task 249SはTask 263のmandatory checker-only lower prerequisiteである。canonical
sourceはfinal LF込みexact 320 bytes、SHA-256
`078eaee4b17341c9d8ebeb8a1f631ca984873bd07eb4e5d9c1a9486b39ac6671`であり、
canonical EN sectionのcode blockと同一である。このlower boundaryをcrossするのは
declaration member type expression 4件だけである。structure-definition node、
structure/selector symbol、member kind、parent/root/path/view identity、inheritance
target/redefinition、field coverage、constructor/selector declaration、coherence、
obligationはTask 263に残す。

### exact additive public ABI

Task 249Sはexisting syntax-free moduleへcanonical ENで定義した
`SourceTypeStructureMemberId`、`SourceTypeStructureMemberHandoffInput`、
`SourceTypeStructureMemberInput`、immutable table/row、handoff getter、
`SourceTypeStructureMemberProducer`をexact field/name/order/type/signatureで追加する。
new public enum typeは追加せず、existing non-exhaustive public
`SourceTypeError`へ下記variant 5件をappendする。derive、dense ID/table/row getterのconstness、iterator
item shape、producer `build(input, arena) -> Result<SourceTypeApplicationHandoff,
SourceTypeError>`はcanonical EN code blockどおりである。

`SourceTypeProducer::build`はnew tableをempty initializeし、Task-249R/249M
producerはempty tableを保持する。standalone producerはbaseを受けず捏造せず、
transactionalにnew immutable handoffを返す。non-exhaustive errorは
`EmptyStructureMembers`、`StructureMemberCardinalityMismatch`、
`InvalidStructureMember`、`InvalidStructureMemberSite`、
`UnsupportedStructureMember`をcanonical ENのID fieldで追加する。

### exact Task-263 lower profile / validation

successful profileはapplications/expressions/arguments/definition returns/mode
RHS/structure members `0/4/0/0/0/4`である。

| Row | member owner | type expression/head | Root |
| ---: | --- | --- | ---: |
| 0 | node 53、`42..63`、ordinal 0 | nodes 52/51、`59..62`、`Bare` builtin `set` normal | 0 |
| 1 | node 56、`68..91`、ordinal 1 | nodes 55/54、`87..90`、同上 | 1 |
| 2 | node 61、`134..155`、ordinal 2 | nodes 60/59、`151..154`、同上 | 2 |
| 3 | node 64、`160..183`、ordinal 3 | nodes 63/62、`179..182`、同上 | 3 |

exactly 4 rowsだけをadmitし、dense ID/source ordinal/rootはvector order。
row/expressionのsource/module identity、nonempty same-source exact normal arena
member range、expression containmentをvalidateする。member/expression/head siteは
相互にdistinctなexact `TypedSiteRef::Node`で、role/duplicateをrejectする。
expression/head range/recoveryをarenaへ再照合し、全expressionはargument-free、
bare、normal builtin `set`、両spelling exact `set`である。failure precedenceは
empty、non-four cardinality、row/environment/range、site/arena、unsupported shape。

handoff validationはこのprofileを認識し、binding application/argument/
definition-return/mode-RHSとmutually exclusiveにする。`TypedAst`はexisting
optional `source_type` field/install pathだけでsole ownerとなり、
`ResolvedTypedAst`はclone-preserveだけを行う。second owner/installerは追加しない。

member table empty時はexisting debug byteが完全に不変。present時はmode-RHS row後、
expression row前にcanonical ENの`structure-member#...` lineを出力する。complete
debug textがTask-263 lower fingerprintとなり、Task 263はfabricated application ID
ではなく`SourceTypeStructureMemberId`を参照する。

### test、禁止範囲、audit impact、exit

implementationはcanonical ENで命名したchecker test exactly 4件を追加し、exact
API/profile/debug/legacy stability、corruption/error precedence、arena/install
revalidation、replay/Typed-final clone/sibling isolationを所有する。checkerは
`458 -> 462`、runner/resolver/syntaxは`524/146/59` unchanged。runner/corpus/
sidecar/expectation/trace row/status/count/diagnostic/obligation/metadata caseは
追加しない。

artificial binding/application、generalized member type graph、parameter/context、
field/property classification、structure/member/resolver identity association、
inheritance parent/root/path/view/coverage/constructor/selector/redefinition、type
equality/subtyping/inhabitation、coherence、goal/guard composition、proof/discharge/
acceptance/fact/axiom/Core/CFG/VC、public diagnostic、Task-263 producer/runner/corpus
workはforbidden/deferredである。

docs prerequisiteはproduction/fixture/sidecar/expectation/trace/test-list/CLI/
manifest/executable hashを変更せず、EN/JA sync、repeated review-only **NO
FINDINGS**、all nine gates、uncapped quality 90+、dedicated docs commit、clean
inventory、origin classification/stash invarianceでexitする。separate
implementationはexact four tests、`0/4/0/0/0/4`、full reviews/verification、
dedicated commit後にTask 263へ自動復帰する。

## Task 249S active implementation result

frozen APIとstandalone `0/4/0/0/0/4` profileをcontract変更なしで実装した。
validationはfrozen orderでglobal passを行う: cardinality、全row/environment/
range identity、全site/arena identity、全expression shape。これによりearlier
rowのsite/shape faultがlater rowの高優先度faultをmaskしない。application、
argument、definition-return、mode-RHSとのmixed tableはsibling validatorより
前にfail closedする。

exact test 4件は全てPASSし、全owner/expression/head arena nodeのrecovered/
normal-wrong-range drift、mixed-table 4種、cross-row compound precedence、
deterministic replay、Typed/final ownershipをcoverする。legacy empty-member
debug byteは不変。`source_type.rs`は`6244` lines、checkerは`462` tests、
raw/normalized hashは
`5f18c633183db679ecacb2781c9133dad5b4c48fdb00e33435dd4c1329105fd2` /
`e0da07dbaf28c659f9e3ac682ae5cf694e7ddd5cdb987abe5d2598ebbfc68d7d`。
Task 263と全semantic deferralは分離されたままである。

## Task 263 test-only lower replay seam

Task 263はstored structure-member rootをcorruptする`cfg(test)`-only crate-private
mutator 1件を追加し、later lower-relation categoryをmapping/coherence faultとpairにする。
production Task-249S validation、public API、fingerprint grammar、accepted
`0/4/0/0/0/4` behaviorは不変で、`source_type.rs`は6,253 linesである。

## Task 249PI frozen property-implementation composition

### selection、authority、one-task scope

Task-264 documentation prerequisite後のfresh inventoryはchecker Task 249PIをsole
dependency-ready taskとしてselectする。canonical Chapter 5 §§5.1--5.2は
`carrier`とvirtual `marker`のwritten `set` returnを要求し、Chapter 7
§§7.4.1、7.8.2、7.10はimplementation parameter `M: Task264Carrier`を要求する。
exact Task-264 means/equals source、parser row、resolver local structure symbol、
Task-248P binding 0、frozen Task-264 lower bundleから下記compositionを導出できる。
blocking `spec_gap`はない。

current Task 249はbinding-linked structure applicationをauthenticateでき、Task-249Sは
structure-member return typeをindependentにownできるが、Task-249Sは意図的にTask-263
standalone four-member profileだけをadmitする。同一immutable handoffで共存できない
ことはlower `source_drift`とpaired `design_drift`であり、canonical-derived checker
regression 4件が`test_gap`を閉じる。Task 264でmember returnをfabricateすること、
member用parameter applicationをfabricateすること、definition-return/mode-RHS rowを
reuseすることは`boundary_violation`である。

Task 249PIがownするのはTask 264に必要なexact source-type compositionだけである。
property identity/member kind、implementation target、`marker`からmember row 1への
lookup、parameter/binding context、`equals`/`means`、definiens term/formula、`it`、
correctness/initial obligation、coherence、goal/guard、proof/discharge/acceptance、
fact/axiom、diagnostic、runner selection、Core/CFG/VC、Task-259 dataはownしない。

### exact additive API / error

new public input/row/table/ID/enum/handoff/owner/debug familyは追加しない。existing
producerへcanonical ENのexact signatureで
`SourceTypeStructureMemberProducer::extend_property_implementation(base, input,
arena)`をappendする。already-authenticated Task-249 baseをborrowし、全base prerequisite
PASS後だけcloneし、existing member row 2件とexpression rootをappendし、complete resultを
validateしてnew immutable handoffを返す。input source/moduleはbaseと一致しなければならず、
baseはsuccess時にもmutationしない。standalone `build(input, arena)`はexactly unchanged。

existing non-exhaustive `SourceTypeError`へ
`StructureMembersAlreadyPresent`、
`StructureMemberExtensionCardinalityMismatch`、
`InvalidStructureMemberBase`をexactly appendする。display textはcanonical EN記載どおり。
existing row/site/shape errorのfield/textは不変。failure precedenceはalready-present、
empty、non-two extension cardinality、source/module mismatch、invalid base、全row/
environment/range identityのordinal order、全site/arena identityのordinal order、全
expression shapeのordinal orderである。

### frozen means / equals profile

successful handoffはいずれもapplications/expressions/arguments/definition returns/
mode RHS/structure members `1/3/0/0/0/2`。application 0はbinding 0、source ordinal
0、root expression 0。argument/definition return/mode RHSはempty。member ID/source
ordinalは0/1、rootはappended expression 1/2。全stored IDはvector orderと一致する。

parameter rootはnormal/argument-free `Bare` symbol applicationで、source/head
spelling exact `Task264Carrier`、source/head range `130..144`。`SourceTypeProducer`が
`SourceTypeHead::Symbol`をresolverに対してcurrent-module local structure / local-source
contribution 0として既にauthenticateしている。Task249PIはそのexact `SymbolId`/
contributionをpreserveし、spellingからFQNをreconstructせず、resolver-generated FQNを
simplified `<module>::Task264Carrier`へ置換しない。means expression/head nodeは63/64、
equalsは45/46。

appended rowはexactly次のとおり。

| Profile | Member | owner node/range | expression/head node / range | Root |
| --- | ---: | --- | --- | ---: |
| means | 0 | 56 / `45..66` | 55/54 / `62..65` | 1 |
| means | 1 | 59 / `71..94` | 58/57 / `90..93` | 2 |
| equals | 0 | 38 / `45..66` | 37/36 / `62..65` | 1 |
| equals | 1 | 41 / `71..94` | 40/39 / `90..93` | 2 |

member expressionはnormal/argument-free `Bare` builtin `set`で、source/head spelling
はexact `set`。member rangeはnonempty same-source exact normal owner-arena rangeで
expressionをcontainする。parameter/member/expression/head siteは表のexact distinct
`TypedSiteRef::Node`で、role/duplicate/missing/recovered/wrong-rangeをfail closedする。
全expression/head rangeをarenaへ再照合する。

complete validatorがrecognizeするstructure-member profileはmutually exclusiveな2種、
legacy standalone Task-249S `0/4/0/0/0/4` byte-for-byte unchangedとTask-249PI
`1/3/0/0/0/2`だけである。Task-249R returnやTask-249M RHSを含むそれ以外のmixed
application/member shapeはfail closedする。empty-member legacy Task-249 application
handoffはvalidかつbyte-identicalのまま。

### debug fingerprint / ownership

existing `source-type-application-debug-v1` grammar/orderは不変。Task-249PIはversion
line、module line、application 0、member row 0/1、expression 0/1/2の順にemitする。
canonical EN templateの`<module-path>`はexact `ModuleId.path`、`<resolver-fqn>`は
already-authenticated resolver `SymbolId`にstoredされたcomplete FQN、各node placeholderは
means/equals値を一貫して選ぶ。全placeholderのconcrete substitution、final expression
line後exactly one LF、extra blank lineなしがcomplete fingerprintである。callerは
fingerprint textを供給できない。

`TypedAst`はexisting optional `source_type` fieldとone-shot installationでsole owner。
installationはcomplete profile/arena identityを再validateする。`ResolvedTypedAst`はsame
handoff/exact fingerprintをclone-preserveするだけ。Task 249PIはどちらにもfield/
installerを追加しない。Task 264はcomplete fingerprintとmember ID 1をconsumeし、lower
rowをinferしない。

### test、count/hash impact、exit

implementationは`crates/mizar-checker/src/source_type.rs`とsynchronized derived design
recordだけを変更し、canonical ENで命名したchecker test exactly 4件を追加する。両exact
profile/fingerprintとlegacy Task-249S byte、全base/row/range/site/shape/precedence/
one-shot failure、arena/install drift、replay、Typed/final clone ownership、Task-249R/
249M/249S/259 isolationをownする。checker libraryは`469 -> 473`、runner/resolver/
syntaxは`528/148/59`。runner production、fixture、sidecar、expectation、trace row/
backlink/status/count、metadata、diagnostic、CLI、executable coverage deltaはない。

docs prerequisiteはEN/JA sync、repeated review-only **NO FINDINGS**、score capなしの
hard gate 9件PASS / quality 90+、exact docs-only staging/commit、clean fresh inventory、
report-only origin divergence、protected-stash invarianceでexitする。separate
implementationはexact test 4件、review、focused/full verification、count/hash、one-file
task-only staging、dedicated commit、clean fresh inventory後にTask 264へautomatic returnする。

## Task 249PI documentation-prerequisite verification

resolver-FQN/debug template修正とstale Task264 checklist closure後、repeated spec reviewと
boundary/source-doc reviewは**NO FINDINGS**。independent final qualityも**NO FINDINGS**、
hard gate 9件はscore capなしで全PASS、valid `100/100`
(`20/20/15/15/10/10/5/5`)。deltaはexactly synchronized design 32 filesで、canonical
spec/production/test/fixture/sidecar/expectation/trace/Cargo/metadata changeはzero。

focused parser/resolver/Profile-C checkerは`1/2/2`、checker/runner lintは`15/14`、
metadataは`137/137`、fmt、warnings-denied Clippy、full workspace test、Cargo metadata、
all five CLI、`git diff --check`はPASS。CLIはcases/requirements `426/394`、pass/fail
`233/193`、active `101/7/203/1`、type `258=246+12`、warnings/errors `23/0`を再現する。

checker/runner/resolver/syntax listは`469/528/148/59`で、raw/normalized hashはcanonical
EN sectionのexact 8値から不変。checker productionは`28/158478`、path/content
`6e4bc96ef04cb5f011d53c651bb93549992e3c7fd0e7595b851d7181c8a65dcd` /
`19a0dd0472f0e3b40c486ab9451322be03aab4322c53d30cff03ef5e6f8c8490`、runnerは
`35/67939`、`4218936ff3ee3baaceb7c0723307ad266d722d0a2473e8b7f82e11c75aeb2b6e` /
`a543608c5075ffed97141626ebbf8d952a847051a34d6782097329b44aa1d09e`。
traceは`cf0ef6d28a132bcbafc8aa1214ded935a715fdffdb3421c37d66c35954f2a06c`、
Task48/mixed-gap source/sidecar hashも不変。exact stage/commit/post-commit inventoryは
parent-owned gateとして残る。

## Task 249PI implementation verification

frozen method、error 3件、exact profile 2件、debug byte、one-shot Typed/final ownership、
semantic exclusionは`source_type.rs`だけへ実装した。named test 4件はPASSし、checkerは
`473`、raw/normalized hashは
`5481b3b20fb75e4d2bab93ce575660f0941aaef01210b06544c9910ecace97cd` /
`db822929f96290beda1209837b0f517ee555f6e01e38b3f13a59918423bb327d`。
ownerは`7423`行、SHA-256
`ef6ec1978ab1b25d01f9ee6fb78538f4a1fb6c97c3a32ba3af618c981d0f4c86`、checker
productionは`28/159648`、path/contentは
`6e4bc96ef04cb5f011d53c651bb93549992e3c7fd0e7595b851d7181c8a65dcd` /
`7d38e5c9fbc3ee2cb09d0d5d1187c4d29d1086c56f0b2dcd7f07cd0b60be283c`。

test reviewがadjacent precedence/member-1 corruption gapを、implementation reviewが
orphan-member installation shapeを検出したが、双方を修正し再reviewは**NO FINDINGS**。
Task-249 siblings、runner `528`、resolver `148`、syntax `59`、runner production
`35/67939`、corpus/metadata/CLI、fixture/expectation/trace hashはexact不変。fmt、
warnings-denied workspace Clippy、full workspace test、CLI 5件、`git diff --check`はPASS。
source-doc consistencyも**NO FINDINGS**、independent qualityはscore capなしでhard gate
9件をPASSし`100/100`。stage/commit/fresh Task264 inventoryがfinal parent-owned gateである。

## Task 269CT frozen proof-`let` source-type composition

### Selection、authority、classification

Task 269C commit `399dc44b2a4400f9eeb1b651d1ddd0bbc7a09f6a`後のfresh inventoryは、
exact dormant `FormulaStatementLetSmoke` proofのseparate source-type prerequisiteとして
Task 269CTだけをselectする。authorityはcanonical Chapter 4 §§4.1--4.2、Chapter 8
§§8.1/8.3、Chapter 15 §§15.2.1/15.10/15.11.1、Chapter 16
§§16.3.1/16.3.3/16.4.1--16.4.2。proof-local `let y be set;`はwritten typeを持つ
scoped arbitrary valueとtype assumptionを導入するが、このprerequisiteはassumption構築、
goal/`thesis`更新、proof-skeleton分解、obligation、discharge、fact、proof acceptanceを行わない。

blocking `spec_gap`はない。Task 269CPがexact source/Surface/resolver type site、Task 269Cが
definition-site binding、Task 249がsyntax-free type modelをauthenticate済み。absent contract /
producer / test 8件は`design_drift` / `source_drift` / `test_gap`。Task 269Cの
`BindingTypeSite::Missing`はimmutable prerequisite snapshotでdriftではない。
`source_undocumented_behavior`、`test_expectation_drift`、current `boundary_violation`はない。
origin差はreport-only `repo_metadata_conflict`でrepairしない。

### Exact dependency / lower profile

sourceはfinal LFを含む100 byte、SHA-256
`7860a3fe5af89063ac6a2b9a4465cac36d26f6d64e892ba6e2c89bcbaaf9763a`。
Task 269CPだけが51 normal unrecovered node/root 50、Surface SHA-256
`1fc35ec18db82efc0968b2f42b08cfaae678184983210cd26f060d45354c7f68`、reserve
type/head `14..17`、theorem/proof/let/segment/name
`19..99`/`59..98`/`67..80`/`71..79`/`71..72`、proof-`let` type/head
`76..79`、ordinal 1、scope `[0]`、local `y`、shell 2件とtheorem provenanceをownする。
269CTはTask-269C handoffとopaque debug fingerprintをconsumeし、syntax rescan、resolver identity
再構築、lower変更をしない。

dependencyはexact base/final binding env `1/1/0 -> 2/2/0`をpreserveする。binding 0はsource
type site `14..17`のreserved `x`。binding 1はproof-context `LetBinding` `y`、context 1、
scope `[0]`、declaration `71..72`、visible-after/source ordinal 1、normal、uncaptured、
diagnostic-freeで、immutable Task-269C snapshot内では`Missing`のまま。

### Exact additive public API / atomic model

`source_type.rs`に次を追加する。

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalLetTypeHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    dependency: SourceProofLocalLetBindingHandoff,
    dependency_fingerprint: String,
    binding_env: BindingEnv,
    binding_fingerprint: String,
    source_type: SourceTypeApplicationHandoff,
    source_type_fingerprint: String,
}
pub struct SourceProofLocalLetTypeProducer;

impl SourceProofLocalLetTypeProducer {
    pub fn build(
        dependency: SourceProofLocalLetBindingHandoff,
        input: SourceTypeHandoffInput,
        symbols: &SymbolEnv,
        arena: &TypedArena,
    ) -> Result<SourceProofLocalLetTypeHandoff, SourceProofLocalLetTypeError>;
}

impl SourceProofLocalLetTypeHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub const fn dependency(&self) -> &SourceProofLocalLetBindingHandoff;
    pub const fn binding_env(&self) -> &BindingEnv;
    pub fn dependency_fingerprint(&self) -> &str;
    pub fn binding_fingerprint(&self) -> &str;
    pub const fn source_type(&self) -> &SourceTypeApplicationHandoff;
    pub fn source_type_fingerprint(&self) -> &str;
    pub fn debug_text(&self) -> String;

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        arena: &TypedArena,
    ) -> Result<(), SourceProofLocalLetTypeError>;
    pub(crate) fn validate_complete_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        arena: &TypedArena,
        installation_available: bool,
    ) -> Result<(), SourceProofLocalLetTypeError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceProofLocalLetTypeError {
    InvalidDependency,
    InvalidBindingEnvironment,
    InvalidSourceType,
    InvalidInstallation,
}
```

exact displayは順に`source proof-local let type dependency is invalid`、
`source proof-local let typed binding environment is invalid`、
`source proof-local let source type is invalid`、
`source proof-local let type installation is invalid`。precedenceはdependency、upgraded env、
source-type input/arena/symbols、one-shot installationで、partial handoffをpublishしない。
earlier source-backed Public Enum Policy tableはdocs-only prerequisiteではcurrent sourceを
describeするためdeliberately不変とし、implementation commitがnew non-exhaustive error rowと
enumを同時追加する。

producerはTask 269Cをby-value consumeして`dependency()`として不変保存する。sort/repairせず、
context/binding/identity/lookup/diagnosticとnon-type fieldが同じtyped binding envを構築し、binding
1だけ`Missing -> Source(76..79)`、binding 0は`Source(14..17)`、cardinality `2/2/0`。
Task-269C fingerprintは不変、upgraded envは独立exact fingerprintを持つ。

embedded source-typeはapplication/expression/argument/definition-return/mode-RHS/
structure-member `2/2/0/0/0/0`。applicationはbinding/root/ordinal `0/0/0`と`1/1/1`。
expression 2件はnormal、argument-free、`Bare` builtin `set`、spelling `set`、range `14..17` /
`76..79`。resolver head、argument、attribute、type result、normalization、inhabitation、subtyping、
coercion、evidence、semantic assumptionはinferしない。generic `SourceTypeProducer::build`は不変で、
exact LetBindingをadmitするのは新producerだけ。

### Arena、fingerprint、Typed/final ownership

exact checker `TypedArena`はnormal/untyped/unlinked 3 node/root 2。node 0は
`source.proof-local.let.reserve-type` `14..17`、node 1は
`source.proof-local.let.type` `76..79`、node 2は
`source.proof-local.let.type-root` `0..99`、children `[0,1]`。resolved linkはabsent。
expression/headは対応node上のdistinct role `source.type.expression` / `source.type.head`。
producer/install双方がexact profileをauthenticateする。

fingerprintはdependency debug、upgraded binding debug、embedded source-type debugの3件。
debug textはterminal LF 1件、extra lineなしでexact次のgrammar。各`{:?}`はcomplete
fingerprint stringのRust debug formatting。

```text
source-proof-local-let-type-debug-v1
module: {package}::{module_path}
dependency-fingerprint: {dependency_fingerprint:?}
binding-fingerprint: {binding_fingerprint:?}
source-type-fingerprint: {source_type_fingerprint:?}
```

private field orderは上のdeclaration exact。unit producerにderive requirementはない。
`validate_installation`はdependency、upgraded env、fingerprint 3件、embedded type handoff、arenaを
authenticateし、`validate_complete_installation`はそれを先に呼び
`installation_available`を最後にcheckする。Typed install/final assemblyはexact typed arenaを
渡し、independent reconstructed final arenaをinputにしない。existing empty/Task-249/269C
byteは不変。

`TypedAst`/`ResolvedTypedAst`はboxed optional `source_proof_local_let_type`とexact public methodを
追加する。

```rust
impl TypedAst {
    pub const fn source_proof_local_let_type(
        &self,
    ) -> Option<&SourceProofLocalLetTypeHandoff>;
    pub fn with_source_proof_local_let_type(
        self,
        handoff: SourceProofLocalLetTypeHandoff,
    ) -> Result<Self, TypedAstError>;
}

impl ResolvedTypedAst {
    pub const fn source_proof_local_let_type(
        &self,
    ) -> Option<&SourceProofLocalLetTypeHandoff>;
}
```

`TypedAstError` / `ResolvedTypedAstError`は`InvalidSourceProofLocalLetType`をappendし、displayは
`source proof-local let type handoff is invalid` / `resolved typed AST source proof-local let type handoff is invalid`。
このcompositeと3-node arenaだけをownし、legacy direct `source_type` /
`source_proof_local_let_binding`はempty。finalはcompositeをcloneし3 nodeをrole
`source.proof-local.let.type`でone-for-one map。semantic tableは全empty。duplicate、wrong
dependency/fingerprint/arena/site/root、occupied sibling、nonempty semanticsはatomic reject。

### Runner、test、exclusion、exit

dormant runnerはunchanged Task 269Cを呼び、そのhandoffとTask-269CP type rangeだけでarena/inputを
構築しpublic dispatchへ入らない。implementationはexisting Rust 7 file: checker
`source_type.rs` / `typed_ast.rs` / `resolved_typed_ast.rs`、runner existing proof-local leaf、
facade 2件、test leaf。parser/resolver、fixture/sidecar/expectation/trace/metadata/Cargo/diagnostic/
dispatchは不変。

checker 4件+runner 4件はexact transaction/fingerprint、corruption/precedence、Typed/final
atomicity、near miss/Task269C不変/generic isolation/empty semanticsをcover。library
`486/544 -> 490/548`、resolver/syntax `148/59`、production path `30/37`。line/hashは再測定。
corpus/requirements `428/395`、pass/fail `235/193`、active `101/7/205/1`、type
`259=247+12`、warnings/errors `23/0`、trace
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`、CLI hashは不変。

use/capture、assumption/guard、goal/`thesis`、proof skeleton、initial obligation、proof/discharge/
acceptance、fact、overload/normalization、Core/CFG/VC、他local form、active creditをexcludeする。
exitは全review NO FINDINGS、score capなしhard gate 9件/90点以上、full verification/count/hash、
docs-only commit、fresh preflight、exact implementation commit、後続fresh selectionである。

### Documentation prerequisite verification

initial specification reviewのfindings 3件はcomplete debug grammar、exact private
validation contract、current global task statusをfreezeして修正した。repeated reviewと
independent specification reviewはいずれも**NO FINDINGS**。preflightはunchanged
Task-269CおよびTask-269CP/C lower transaction、library baseline `486/544`、CLI 5件の
hash、trace hashをauthenticateした。format、repository lint policy 2件、metadata、
Cargo metadata、warnings-denied workspace Clippy、full workspace tests、whitespaceは、
executable/canonical artifactを変更せずPASS。source/docs reviewとfinal quality reviewは
**NO FINDINGS**、hard gate 9件はscore capなしで全PASS、valid `100/100`。task-only
staging/commit、fresh implementation preflightがremainingである。

### Task 269CT implementation verification

frozen APIをexact実装した。producerはTask 269Cをby-value consumeし、binding 1の
`Source(76..79)` overlayだけをreconstructし、generic Task 249をbroadenせずexact proof-local
bare builtin-`set` row 2件をadmitし、complete 3-node arenaをauthenticateしてfingerprint 3件を
含むimmutable composite 1件をpublishする。validation precedenceはdependency、binding
environment、source type/arena、availability。

checker/runner test exactly 4/4は全overlay field、application/expression 2 row、全node
anchor/children/link/typing/recovery、actual payload corruption、complete precedence、one-shot/
cross-family final failure、generic isolation、near miss、semantic emptiness全件をcoverする。
implementation reviewはshared statement node hintを許した`boundary_violation` 1件を検出。
Task-specific final predicateをempty hint必須へ修正し、repeated test/implementation reviewは
**NO FINDINGS**。

libraryは`490/548`、productionは`30/168322` / `37/71647`、content hashは
`4d0c793a47dac672e5f395c9c2b9e7c9274b5d776b54870888ba5c918f751dc2` /
`0f8f5926b9bee23c92d1f05e9cc9e85b4c0561b543e9e0a1e4c825f43b6c5798`。
path hashはfrozen値不変。checker raw/normalized test-list hashは
`10e1f56783a472b63a0473893196d68b54a7a7aa3a3aff4f66e74ac42b4a2ad2` /
`21d65f467319e2e7ac463344902b10dfce5716a96c41a87e879326c293ff36e0`、runnerは
`cd47be81d6e0987a4461191b700c442c3182fb9f35fe6ab6e2d216ba122fd841` /
`e24bc08e3c8207ba96b6df3de995a3b489e333f8599233c1eded9f81fe696a77`。
canonical fixture/expectation/trace/metadata/count/dispatchと全semantic deferralはunchanged。

focused/crate/workspace test、lint `15/14`、metadata `137`、format、warnings-denied
Clippy、Cargo metadata、CLI hash 5件、exact count/list/production/fixture/trace hash、
whitespaceはPASS。

repeated source/docs consistency/final quality reviewは**NO FINDINGS**。hard gate 9件は
score capなし`100/100`で全PASS。

## Task 269GP no-type boundary

private lower rowはwritten bare builtin-`set` spelling/range `84..87`だけをrecordし、
`SourceTypeProducer`、type id/node、Task-269CTを変更しない。canonical `given` scope
矛盾はbinding-only 269Gとtype admission 269GTをblockする。type assumption/guardは
semantic deferral。

implemented syntax rowはspelling/rangeだけをrecordし、Task-269CT replay testはgreen、
checker source-type byte変更なし。

## Task 269GS type deferral after scope resolution

canonical witness lifetimeはfixedだがTask269GSはsource typeをadmitしない。Task269Gがscoped
bindingを先に確立し、Task269GTがwritten `set` type site/block-consistent type replayをowner。
type assumption、guard、proof obligationはdefer。

## Task 269G missing-type boundary

new `GivenWitness` rowは`BindingTypeSite::Missing`を保持。lowerの`set@84..87`を独立admit/
infer/guard化しない。Task269GTだけがimmutable binding handoffをby-value consumeして
source-type ownerを追加できる。

Task269G implementationは`BindingTypeSite::Missing`を意図通り保持し、authenticated lower
`set@84..87`をchecker source-type ownershipへconsumeしない。Task269GTはdependency-readyな
separate scopeのまま。

## Task 269GT frozen proof-`given` source-type composition

### selection、authority、classification

Task269G commit `4f65bc4d50ab950c6976a4b3f3cb4bc0948b27c1`後のfresh inventoryは
dormant `FormulaStatementGivenSmoke`のsource-type prerequisite Task269GTだけをselect。
canonical authorityはChapter 4 §§4.1--4.2/4.6、Chapter 8 §§8.1/8.3、Chapter 15
§§15.3.3/15.10/15.11.4、Chapter 16 §§16.3.3/16.4.2。`given y being set`がwritten
typeを持つblock-local witnessを導入することは定めるが、`such that` condition、type guard/
assumption、Skolem/existential fact、goal、initial obligation、proof/discharge/acceptance、IRを
本prerequisiteがpublishするauthorityではない。

blocking `spec_gap`なし。Task269GPはexact written type site/source/resolver、Task269Gは
lexical `GivenWitness` binding、Task249とimplemented Task269CTはsyntax-free type model/
proof-local composition patternをauthenticate済み。missing Given-specific contractは
`design_drift`、absent producer/testはimplementationの`source_drift`/`test_gap`。
Task269G `BindingTypeSite::Missing`はimmutable prerequisite snapshot。
`source_undocumented_behavior`、`test_expectation_drift`、current `boundary_violation`なし。
`origin/main...HEAD=0/3`はreport-only `repo_metadata_conflict`で修復しない。

### exact dependency/lower profile

sourceは129 bytes、final LF 1件、SHA-256
`04e54b8ada9af54fde9f937e1bb0f96bd8cf85002b2b57f4d348b11c8eb72a2f`。
Task269GPがsole source/Surface/resolver lower owner: normal unrecovered 48 nodes、root 47、
Surface SHA-256
`58ac16a3c75860180a8bec5dc8e87ec8b269fe75715a6d8363f7ef064e3deea8`、reserve
type/head `14..17`、theorem/proof/given/segment/name
`19..128`/`62..127`/`70..108`/`76..87`/`76..77`、proof-`given` type/head
`84..87`、source ordinal 1、scope `[0]`、local `y`。shell 2件、theorem symbol/
definition/contribution `0/0`はexact。Task269GTはTask269G handoff/lower fingerprintをconsumeし、
syntax rescan/resolver reconstruction/lower modificationなし。

dependency binding environmentは`1/1/0 -> 2/2/0`。binding 0はreserved `x`、source type
`14..17`。binding 1はproof context active `GivenWitness` `y`、context 1、resolver-local
scope `[0]`、declaration `76..77`、visible-after/source ordinal 1、normal、uncaptured、
diagnostic-free、Task269G snapshot内は`BindingTypeSite::Missing`。

### exact additive public API/atomic model

`source_type.rs`に次のsyntax-free public siblingを追加する。

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalGivenTypeHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    dependency: SourceProofLocalGivenBindingHandoff,
    dependency_fingerprint: String,
    binding_env: BindingEnv,
    binding_fingerprint: String,
    source_type: SourceTypeApplicationHandoff,
    source_type_fingerprint: String,
}
pub struct SourceProofLocalGivenTypeProducer;

impl SourceProofLocalGivenTypeProducer {
    pub fn build(
        dependency: SourceProofLocalGivenBindingHandoff,
        input: SourceTypeHandoffInput,
        symbols: &SymbolEnv,
        arena: &TypedArena,
    ) -> Result<SourceProofLocalGivenTypeHandoff, SourceProofLocalGivenTypeError>;
}

impl SourceProofLocalGivenTypeHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub const fn dependency(&self) -> &SourceProofLocalGivenBindingHandoff;
    pub fn dependency_fingerprint(&self) -> &str;
    pub const fn binding_env(&self) -> &BindingEnv;
    pub fn binding_fingerprint(&self) -> &str;
    pub const fn source_type(&self) -> &SourceTypeApplicationHandoff;
    pub fn source_type_fingerprint(&self) -> &str;
    pub fn debug_text(&self) -> String;
    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        arena: &TypedArena,
    ) -> Result<(), SourceProofLocalGivenTypeError>;
    pub(crate) fn validate_complete_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        arena: &TypedArena,
        installation_available: bool,
    ) -> Result<(), SourceProofLocalGivenTypeError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceProofLocalGivenTypeError {
    InvalidDependency,
    InvalidBindingEnvironment,
    InvalidSourceType,
    InvalidInstallation,
}
```

error typeはpublic standard error traitをexact
`impl std::error::Error for SourceProofLocalGivenTypeError {}`として実装する。このtrait
implementationの省略/置換はfrozen APIの範囲外。

variant順exact displayは`source proof-local given type dependency is invalid`、
`source proof-local given typed binding environment is invalid`、
`source proof-local given source type is invalid`、
`source proof-local given type installation is invalid`。precedenceはdependency、upgraded
binding environment、source-type input/arena/symbols、one-shot availability。partial publishなし。
source-backed Public Enum Policy/source inventoryはdocs prerequisiteでは変更せず、sourceに実在する
implementation commitでenum/literal itemを追加する。

producerはTask269Gをby-value consumeし`dependency()`でunchanged preserve。context/binding/
identity/lookup/diagnostic/non-type fieldをsort/repairせず再構成し、binding 1だけ`Missing`から
`Source(84..87)`、binding 0は`Source(14..17)`、cardinality `2/2/0`。Task269G fingerprintは
不変でupgraded environmentはown exact debug fingerprintを持つ。

embedded `SourceTypeApplicationHandoff`のapplications/expressions/arguments/definition-returns/
mode-RHS/structure-membersは`2/2/0/0/0/0`。application 0はbinding/source ordinal/root
`0/0/0`、application 1は`1/1/1`。両expressionはnormal/argument-free/`Bare`/builtin
`set`、written/head spelling `set`、range `14..17`/`84..87`。resolver head、argument、
attribute、type result、normalization、inhabitation、subtyping、coercion、evidence、semantic
assumption/conditionをinferしない。

generic `SourceTypeProducer::build`のadmissionは不変。private
`SourceTypeBindingProfile::ProofLocalGiven`だけがactive `BindingKind::GivenWitness`とmatching
`ResolverLocal` identity、proof `SourceStatement` context/parent/scope/range/ordinal、empty
capture/diagnostics、exact source type siteをadmit。generic/`ProofLocalLet`をbroadeningせず、
missing siteをtyped扱いしない。

### arena、fingerprint、Typed/final ownership

checker `TypedArena`はnormal/untyped/unlinked 3 nodes、root 2。node 0
`source.proof-local.given.reserve-type` `14..17`、node 1
`source.proof-local.given.type` `84..87`、node 2
`source.proof-local.given.type-root` `0..128` children `[0,1]`。resolved linkなし。
expression/headは各nodeのdistinct `TypedSiteRef::Role` `source.type.expression` /
`source.type.head`。producer/installerはexact node/site/range/recoveryをauthenticate。

`dependency.debug_text()`、upgraded `BindingEnv::debug_text()`、embedded source-type debugを
byte-exact fingerprintとしてfreeze。debugはterminal LF 1件でexact:

```text
source-proof-local-given-type-debug-v1
module: {package}::{module_path}
dependency-fingerprint: {dependency_fingerprint:?}
binding-fingerprint: {binding_fingerprint:?}
source-type-fingerprint: {source_type_fingerprint:?}
```

private field orderはdeclaration順、unit producer derive requirementなし。
`validate_installation`はdependency/environment/fingerprint/source type/arena、complete版はそれを
先に呼びavailabilityを最後にcheck。Typed/finalは同じarenaを渡しfinal arena再構成なし。
empty/Task269C/CT/G bytesは不変。

`TypedAst`/`ResolvedTypedAst`はGiven binding slot後にboxed optional
`source_proof_local_given_type` ownerを追加し、exact APIは次。

```rust
impl TypedAst {
    pub const fn source_proof_local_given_type(
        &self,
    ) -> Option<&SourceProofLocalGivenTypeHandoff>;
    pub fn with_source_proof_local_given_type(
        self,
        handoff: SourceProofLocalGivenTypeHandoff,
    ) -> Result<Self, TypedAstError>;
}
impl ResolvedTypedAst {
    pub const fn source_proof_local_given_type(
        &self,
    ) -> Option<&SourceProofLocalGivenTypeHandoff>;
}
```

error enumへ`InvalidSourceProofLocalGivenType`をappendし、displayはtyped
`source proof-local given type handoff is invalid`、resolved
`resolved typed AST source proof-local given type handoff is invalid`。profileはcomposite/3-node
arenaだけをownし、direct `source_type`/Given binding/Let fieldsはempty。dependencyはcomposite
経由。finalはcloneして3 nodesを`source.proof-local.given.type` roleでone-for-one map。
semantic table/node-hint inputは全empty。duplicate/wrong dependency/fingerprint/env/arena/site/
root/sibling/nonempty semanticはatomic reject。

### runner、tests、exclusion、impact、exit

dormant runnerはunchanged Task269Gを先にcallし、そのhandoffとTask269GP type rangeだけから
arena/inputをbuild。public dispatchなし。implementation ownershipはexact existing Rust 7 files:
`crates/mizar-checker/src/source_type.rs`、
`crates/mizar-checker/src/typed_ast.rs`、
`crates/mizar-checker/src/resolved_typed_ast.rs`、
`crates/mizar-test/src/runner/type_elaboration/source_proof_local_declaration.rs`、
`crates/mizar-test/src/runner/type_elaboration.rs`、
`crates/mizar-test/src/runner.rs`、
`crates/mizar-test/src/runner/tests/type_elaboration/source_proof_local_declaration.rs`。
後者production facade 2段はtest-only。checker `binding_env.rs`/proof-local declaration、runner
lower `crates/mizar-test/src/runner/type_elaboration/source_statement.rs`、parser/resolver/
fixture/sidecar/expectation/trace/metadata/Cargo/diagnostic/dispatchは変更しない。

checker exact tests 4件:
`task269gt_exact_transaction_fingerprints_and_overlay_are_stable`、
`task269gt_dependency_binding_source_type_and_precedence_fail_closed`、
`task269gt_typed_and_resolved_ownership_is_atomic`、
`task269gt_generic_and_neighbor_routes_remain_isolated`。runner exact tests 4件:
`task269gt_exact_type_composition_fingerprints_and_replay_are_stable`、
`task269gt_dependency_input_and_arena_corruption_fail_closed`、
`task269gt_typed_and_resolved_owners_are_one_shot_and_semantically_empty`、
`task269gt_near_miss_task269g_and_active_routes_remain_isolated`。overlay/payload/fingerprint、全
dependency/env/input/arena field/precedence、Typed/final one-shot/both-order cross-family、generic/
Let/Given-binding isolation、near miss、clone、semantic emptyをcover。

libraryは`494/556 -> 498/560`をproject、parser/resolver/syntax `226/148/59`不変。
production paths `30/37`不変、implementation後line/contentを再測定。path hashは
`c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5` /
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d`。
docs baseline production `30/169847` / `37/73118`、content
`e47862eebdb59b576160d4b64ab390549d91daecd69fd34f8bcfbc2952d6ca96` /
`2cae769737fdee4560ab1d1bca81f10d900ff8a1d9824aba720806f84e802711`、
list raw/normalized checker
`ce299dfafb8db5d5c27cb9e271dd77d08a09b45a7323d0efc17790e0d104a984` /
`6d8f1938b05118e129f8d0942bd7af77914435b6b45282bd46e636132891d4cb`、runner
`194b2884a9d933823e0d06b24460cd510fd9d16fbd6823b9e13584779acd1f03` /
`728a5b688c19acc42d66a9c2f5c13ad67d795949ec88a2d877b917c9607d80e8`。

`.miz`/sidecar/expectation/trace row/backlink/status/count/metadata/diagnostic/active/CLI変更なし。
corpus/requirements `428/395`、pass/fail `235/193`、warnings/errors `23/0`、active
`101/7/205/1`、type `259=247+12`、trace
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`。
plan/parse/declaration/type/proof hashは
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`、
`a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`、
`71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`、
`4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`、
`ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`。
broad proof-local fail fixtureはcondition/fact/later-use/consider/set/reconsider/acceptance/proof gap
によりdiagnosticのまま、active type/trace credit 0。

use/capture、condition/label/fact、existential/Skolem、type assumption/guard、goal/thesis、
proof skeleton、initial obligation、proof/discharge/acceptance、overload/normalization/
inhabitation、Core/CFG/VC、free-witness export、other proof-local form、Task270をexclude。
docs-only scopeはTask269Gと同じexisting Markdown 40件。EN/JA sync、spec review **NO
FINDINGS**、docs-only hard gate 9件uncapped `>=90/100`、exact docs commit、fresh lower
preflight、exact 7-file implementation、test/implementation/source-doc review **NO FINDINGS**、
full verification/count/hash、final gate/score、task-only commit、clean inventory、stash不変後に
later-use/captureまたはTask270をfresh select。

### documentation prerequisite verification status

EN/JA contractへ`std::error::Error`をexplicitにfreezeした後のspecification re-reviewは
**NO FINDINGS**。全docs-only executable/policy/metadata/CLI/test-list/production/canonical-
artifact/trace/whitespace checkはfrozen baselineでPASS。documentation-prerequisite review
時点ではthen-absent producerをclaimせず、下のimplementation statusがその状態をsupersede
する。prerequisiteのsource/documentation/final-quality reviewは**NO FINDINGS**、hard gate
9件はcapなし`100/100`で全PASSし、
`35bc97b92ce075226105e8fcd4c1e43c8621995c`としてcommit済み。

### Task 269GT implementation verification status

frozen API/transactionをexactに実装した。validation orderはdependency、upgraded binding environment、source-type input/symbols/arena、availability。dependency snapshotは`Missing`のまま、copyしたbinding 1だけが`Source(84..87)`。exact 2 application/expression rows、3 fingerprints、3-node arena、Typed/Resolved boxed owner、final source-preserved role、private Task269G-first runner routeを実装し、Generic/ProofLocalLet admissionを拡張しない。

checker/runner各4 testsはcomplete positive rows、全source-type input field、symbols mismatch、全3 arena nodesの全field、利用可能なdependency corruption seam、independent fingerprints/precedence、ownership/cross-family failure、nonempty final inputs、clone replay、near miss、exhaustive semantic emptinessをcoverする。focused/full crate suitesは`498/498` / `560/560` PASS、test-sufficiency/implementation reviewは **NO FINDINGS**。

checker/runner productionは`30/171383` / `37/73351`、content hashは`4a2635cbde94426652d75bfad176d9f167242630d6e1996ab4087ddf14e20abf` / `747a923200a6c23c58adfca7211c82724ff83e1a808b3e045cc73027054f4d07`。raw/normalized test-list hashはchecker `b6868cffc0a01b60f7a82bcacfd9e52f62ae98d2dbce5d72f832caf624870ff7` / `16cdd3c0bf618d7a16466ec71813ea0265b20d4e217a1fdafd0265c933cb9c00`、runner `da4b6c6049fbf4b0b10dd3fe49d840d0e61814ae250aaba5a47f2742a669c1f1` / `ca5fb8b3230848186435b8f29e8c9a5e542d2f9690faeb21d443165faa335ee2`。corpus/trace/fixture/expectation/metadata/5 CLI/diagnostic/semantic deferは不変。

final implementation/source-documentation/independent quality reviewは **NO FINDINGS**。
focused/crate/lint-policy/metadata/Cargo-metadata/format/Clippy/workspace/CLI/count-hash/
whitespace checkは全PASS。hard gate 9件はcapなし`100/100`でPASSし、staging、commit、
fresh inventoryだけparent-owned。

## Task 269GUP source-type exclusion

GUPは`BindingTypeSite::Missing`で終了。本module/API/profile/arena/test/direct Task269GT validationは
byte-identical。Task269GUPTだけがnew GUP binding handoffをby-value consumeして
`Source(84..87)`をoverlayし、Task269GUはさらにlater。
Completion evidence: [central Task-269GUP historical contract](../../task_contracts/ja/269GUP.md#completion-evidence)。

## Task 269GUPT frozen proof-`given` use-profile source type

### 選択、authority、分類

clean HEAD `076c142598e563be0f0bd2ac785c2643fc3a5b75`のfresh inventoryで
Task 269GUPTだけをselectする。canonical authorityはChapter 3 §§3.1--3.4、
Chapter 4 §§4.1/4.6、Chapter 8 §§8.1/8.3、Chapter 15 §§15.3.3/15.10/15.11.4、
Chapter 16 §§16.4.1--16.4.2である。exact sourceの`given y being set`はwritten
builtin-`set` typeとenclosing block残部のscopeを定めるが、condition fact、type
guard/assumption、existential/Skolem semantics、later term occurrence、capture、goal、
initial obligation、proof/discharge/acceptance、Core/CFG/VCをこのprerequisiteに
authorizeしない。

blocking `spec_gap`なし。implemented Task 269GUPはtype siteが意図的に`Missing`の
distinct validated binding handoffを供給し、lower dependencyはexact written type rangeと
source/resolver provenanceを保持する。GUPT composite、Typed/final owner、focused testsの
不在は`source_drift`/`test_gap`、stale next-task statusはこのdocs prerequisiteで直す
`design_drift`である。old 269G/GTを変更・再利用すること、`source_type.rs`でbindingを
reconstructすること、later `y` termまたはsemantic tableをpublishすることは
`boundary_violation`。`source_undocumented_behavior`/`test_expectation_drift`なし。
origin `0 behind / 7 ahead`はreport-only `repo_metadata_conflict`で、stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`は触らない。

### exact dependency、overlay、public API

sourceはone-final-LFのexact 128 bytes、SHA
`ec15ded78ae96022840a8419a85d74643de3b37337e9a202cbda77ee97aa7c01`。
normal 54-node Surfaceはroot 53、SHA
`c64297ce72e380a2e4146276966e085d780f8b38f2528d5abaa440a50c67db6d`。
reserve type/head `14..17`、theorem/proof/given/segment/name
`19..127`/`62..126`/`70..108`/`76..87`/`76..77`、given type/head `84..87`、
later identifier leaves `116..117`/`120..121`は不変。GUPTはpublic
`SourceProofLocalGivenUseBindingHandoff`をby-value consumeし、existing lowerは
authenticated type rangeだけに使う。syntax rescan/resolver identity reconstructionなし。

immutable dependencyは`1/1/0 -> 2/2/0`。binding 0はreserved `x`の
`Source(14..17)`、binding 1はproof context 1所有、resolver identity
`([0],1,76..77)`、visible-after/source ordinal 1、active/normal/uncaptured/
diagnostic-free `GivenWitness` `y`でtype siteは`Missing`。copied environmentでbinding 1
だけを`Source(84..87)`へ変え、context/identity/lookup/diagnostic/全non-type fieldを
byte-identicalに保つ。cardinalityは`2/2/0`。

`source_type.rs`に追加できるsyntax-free public siblingは次だけである。

```rust
pub struct SourceProofLocalGivenUseTypeHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    dependency: SourceProofLocalGivenUseBindingHandoff,
    dependency_fingerprint: String,
    binding_env: BindingEnv,
    binding_fingerprint: String,
    source_type: SourceTypeApplicationHandoff,
    source_type_fingerprint: String,
}
pub struct SourceProofLocalGivenUseTypeProducer;
pub enum SourceProofLocalGivenUseTypeError {
    InvalidDependency,
    InvalidBindingEnvironment,
    InvalidSourceType,
    InvalidInstallation,
}
```

handoffはEN canonical記載順のgetter、`debug_text`、`validate_installation`、
`validate_complete_installation`を持ち、producer `build`はGUP handoffをby-value、
`SourceTypeHandoffInput`、`SymbolEnv`、`TypedArena`を受ける。errorは
`Debug, Clone, PartialEq, Eq`、`#[non_exhaustive]`、`Display`、
`std::error::Error`を実装する。exact stringsは順に
`source proof-local given-use type dependency is invalid`、
`source proof-local given-use typed binding environment is invalid`、
`source proof-local given-use source type is invalid`、
`source proof-local given-use type installation is invalid`。precedenceはdependency、
copied typed binding env、exact source-type input/symbol/arena、one-shot installation。

embedded source typeはapplications/expressions/arguments/definition-returns/mode-RHS/
structure-members `2/2/0/0/0/0`。rowsは`(binding,ordinal,root)`が`(0,0,0)`と
`(1,1,1)`。両expressionはnormal、argument-free `Bare` builtin `set`、rolesは
`source.type.expression`/`source.type.head`、spelling `set`、rangesは`14..17`と
`84..87`。existing private `SourceTypeBindingProfile::ProofLocalGiven`だけをreuseし、
Generic/ProofLocalLetをbroadenしない。argument/attribute/normalized type/inhabitation/
subtyping/coercion/evidence/guard/condition/obligationなし。

### arena、fingerprint、Typed/final、runner

exact `TypedArena`はnormal/Unknown/unlinkedの3 nodes、root 2。node 0は
`source.proof-local.given-use.reserve-type` `14..17`、node 1は
`source.proof-local.given-use.type` `84..87`、node 2は
`source.proof-local.given-use.type-root` `0..127` children `[0,1]`。resolved linkなし。
dependency/binding/source-typeの完全debug textをfingerprintとし、exact debug headerは
`source-proof-local-given-use-type-debug-v1`、moduleと3 fingerprintをEN記載順で出力し、
terminal LFは1件。

`TypedAst`/`ResolvedTypedAst`はold Given-type slot直後にboxed optional
`source_proof_local_given_use_type`だけを追加し、両方にgetter、Typed側に
`with_source_proof_local_given_use_type`を追加する。error variantは両方
`InvalidSourceProofLocalGivenUseType`、stringは
`source proof-local given-use type handoff is invalid`と
`resolved typed AST source proof-local given-use type handoff is invalid`。全old owner/
nonempty semantic tableとのboth-order共存をrejectする。final node roleは全3件
`source.proof-local.given-use.type`。direct source-type/binding/term/use/statement/proof/
fact/obligation/diagnostic ownerはempty。

runner private outputは`typed_ast`、`resolved`順。mutation enumは`None`、
`WrongDependencyModule`、`WrongTypeRange`、`WrongArenaRoot`、`WrongArenaKind`。
`source_proof_local_given_use_type_output`とcfg-test `_with_mutation`はGUP routeと同じ
5 source argsを持ち、test seamだけmutationを追加する。selector mismatchは`None`、
selected failureは`Some(Err(_))`。route-local stringは
`Task269GUPT reserve type range is missing`だけ。existing GUP binding/lowerを呼び、
dependencyをby-value consumeし、exact input/arenaをbuild、empty TypedAstへinstallして
ResolvedTypedAstをassembleする。active dispatchには入らない。

### scope、tests、impact、deferral、exit

implementationはexact 7 existing Rust files: checker `source_type.rs`、`typed_ast.rs`、
`resolved_typed_ast.rs`; runner `type_elaboration/source_proof_local_declaration.rs`、
`type_elaboration.rs`、`runner.rs`、existing proof-local test leaf。
checker `source_proof_local_declaration.rs`/`binding_env.rs`/`source_term.rs`、runner
`source_statement.rs`、parser/resolver、canonical spec、fixture/sidecar/expectation/trace/
metadata/Cargo/diagnostic/public dispatch/CLI/active resultは変更禁止。

checker exact testsは`task269gupt_exact_transaction_fingerprints_and_overlay_are_stable`、
`task269gupt_dependency_binding_source_type_and_precedence_fail_closed`、
`task269gupt_typed_and_resolved_ownership_is_atomic`、
`task269gupt_prior_and_neighbor_routes_remain_isolated`。runner exact testsは
`task269gupt_exact_type_composition_fingerprints_and_replay_are_stable`、
`task269gupt_dependency_input_and_arena_corruption_fail_closed`、
`task269gupt_typed_and_resolved_owners_are_one_shot_and_semantically_empty`、
`task269gupt_near_miss_task269gup_and_active_routes_remain_isolated`。

docs-only scopeはchecker paired 26、runner paired 12、global ledger 2のexact 40 Markdown。
baselineはlibraries `502/564`、parser/resolver/syntax `226/148/59`、production
`30/172531`と`37/74826`、path hash `c89f43f...bad5`/`1f9e2c...a73d`、content hash
`e0342952...f7c5`/`8fe7c8c0...b1bc`、raw/normalized test-list checker
`059c34f7...d93`/`ba08b3db...de8`、runner `f43b3223...0fe`/`0083d9c0...990`。
implementationは`506/568`をprojectし、line/content/test-list hashをremeasureする。

cases/requirements `428/395`、pass/fail `235/193`、warnings/errors `23/0`、active
`101/7/205/1`、type coverage `259=247+12`、trace SHA `55b754c8...ca2b3`、5 CLI hash、
canonical fixture/sidecarは不変。exact CLI hashesはplan/parse/declaration/type/proof
`700f4bf5...718`/`a8a7aa63...a56`/`71e83ba0...3c74`/`4b2c7bd5...ab7f`/
`ccf3d2d4...8450`、parser source/sidecarは`bd9a2d47...7234`/`7361b50b...0f17`、
broad gap source/sidecarは`5fc4849a...ecd9`/`8e2c73b1...fa43`。active type/trace
creditは0。later occurrence、condition/
fact、existential/Skolem、guard/assumption、capture/export、goal/proof/acceptance、initial
obligation、IRはdefer。

EN/JA review **NO FINDINGS**、docs-only hard gate 9件capなし`>=90/100`、docs commit、
fresh lower-stage preflight、exact 7-file/8-test implementation、3 review **NO FINDINGS**、
full verification/count/hash、separate implementation commit、clean/origin/stash auditでexit。
次はTask 269GU。captureとTask 270はdeferする。

Completion evidence: [central Task-269GUPT historical contract](../../task_contracts/ja/269GUPT.md#completion-evidence)。

## Task 269GU type dependency boundary凍結

GUはcommitted GUPT handoffをby valueでownし、arena node 0--2をstandalone GUPT
arenaのexact private reconstructionへprojectしてcomplete binding/type payloadと
fingerprintを再authenticateする。standalone GUPT validation/source-type row/
public API/error string/3-node arena contractは不変。GUはtype application/
normalization/constraint/coercion/guard/obligationを追加しない。

Completion evidence: [central Task-269GU historical contract](../../task_contracts/ja/269GU.md#completion-evidence)。

## Task 269GCP frozen type deferral

lower rowはwritten bare builtin-`set` range `90..93`だけを保持。source-type
application、binding overlay、arena、normalization、constraint、guard、obligationは
なし。Task269GCTはfuture GC bindingをconsumeし、このsourceへexact GUPTをreuse
してはならない。

Completion evidence: [central Task-269GCP historical contract](../../task_contracts/ja/269GCP.md#completion-evidence)。

## Task 269GC frozen type deferral

binding 1は`BindingTypeSite::Missing`、`set@90..93`はGCP lower内だけ。source-
type application/overlay/normalization/constraint/guard/obligationなし。GCTだけが
GC handoffをby-value consumeしてexact written typeをoverlay可。

Completion evidence: [central Task-269GC historical contract](../../task_contracts/ja/269GC.md#completion-evidence)。

## Task 269GCT frozen Given-condition source-type composition

GCTはexact GC handoffをby value consumeするdistinct immutable composite。
validated `2/2/0` envをcopyしcontext/identity/status/capture/diagnostic/lookupを
変えず、binding 1だけ`Source(90..93)`へoverlay、binding 0は`Source(14..17)`。

common inputはdense application/expression各2、application `i`はbinding/
ordinal/root `i`、expressionは`set@14..17`/`set@90..93`、node 0/1の
`source.type.expression`/`source.type.head` role、same ranges/spelling、`Bare`、
`BuiltinSet`、`Normal`。argument/definition-return/mode-RHS/structure-memberは
empty。arena kind/rangeは
`source.proof-local.given-condition.reserve-type@14..17`、
`source.proof-local.given-condition.type@90..93`、
`source.proof-local.given-condition.type-root@0..133` children `[0,1]`、root 2。

public familyは`SourceProofLocalGivenConditionTypeHandoff`、`Producer`、
non-exhaustive `Error::{InvalidDependency,InvalidBindingEnvironment,
InvalidSourceType,InvalidInstallation}`。dependency/env/source-typeと各fingerprint、
read-only getter、`source-proof-local-given-condition-type-debug-v1` replayを
ownerする。generic admission/normalization/constraint/coercion/fact/guard/
obligation/condition occurrence/acceptanceは禁止。exact GCUだけがnext consumer。

handoff field orderは`source_id`、`module_id`、by-value
`SourceProofLocalGivenConditionBindingHandoff`、`dependency_fingerprint`、
`binding_env`、`binding_fingerprint`、`source_type`、
`source_type_fingerprint`。public read-only APIはEN canonical記載どおり
`source_id`/`module_id`/`dependency`/`dependency_fingerprint`/`binding_env`/
`binding_fingerprint`/`source_type`/`source_type_fingerprint`/`debug_text`のexact
getterだけ。

handoffは`#[derive(Debug, Clone, PartialEq, Eq)]`。crate-private
`validate_installation(&self, SourceId, &ModuleId, &TypedArena) -> Result<(),
SourceProofLocalGivenConditionTypeError>`と、末尾に`installation_available:
bool`を追加した`validate_complete_installation`をexactに持つ。unit-like public
`SourceProofLocalGivenConditionTypeProducer`の`build`はby-value GC dependency、
`SourceTypeHandoffInput`、`&SymbolEnv`、`&TypedArena`をこの順で受け、GCT handoff
またはGCT errorを返す。Errorは`#[derive(Debug, Clone, PartialEq, Eq)]`かつ
`#[non_exhaustive]`のexact 4 variantsで、`fmt::Display`と`std::error::Error`を実装。

debug grammarはheader
`source-proof-local-given-condition-type-debug-v1`、`module:`、Rust-debug quoted
complete GC `dependency-fingerprint:`、complete overlaid env
`binding-fingerprint:`、complete source-type `source-type-fingerprint:`の5行。
全行はlast fingerprintを含めexact LF 1個で終わり、blank/extra terminal lineなし。
nested fingerprintはembedded `\n`を含むretained bytesのRust `Debug` quote。
error Displayは順に
`source proof-local given-condition type dependency is invalid`、
`source proof-local given-condition typed binding environment is invalid`、
`source proof-local given-condition source type is invalid`、
`source proof-local given-condition type installation is invalid`。

producer/installation precedenceはGC dependency validation/identity/fingerprint、
exact overlay/fingerprint、complete input+arenaとcommon `validate_input`の
`ProofLocalGiven` profile、source-type shape/fingerprint/common install、最後にslot
availability。この4-tier precedenceをmulti-corruption testで固定する。

Completion evidence: [central Task-269GCT historical contract](../../task_contracts/ja/269GCT.md#completion-evidence)。

## Task 269GCU frozen type dependency

complete GCTをby-value consumeし、2 builtin-`set` rows、binding1 overlay、
fingerprint、arena nodes 0--2を不変保持。source-type/argument/normalization/
constraint/coercion/type fact/obligationを追加しない。type validationがterm
validationより先で、GCT再構築/緩和は禁止。

Completion evidence: [central Task-269GCU historical contract](../../task_contracts/ja/269GCU.md#completion-evidence)。

## Task 269SDP type deferral

Given type `set@95..98`はrange/spellingのみ。source-type handoff、typed
binding overlay、arena、guard/constraint/coercion/factは追加しない。

Completion evidence: [central Task-269SDP historical contract](../../task_contracts/ja/269SDP.md#completion-evidence)。

## Task 269SDC frozen source-type deferral

SDCはGiven binding 1を`BindingTypeSite::Missing`のまま保持する。SDPに
written `set@95..98`があってもsource-type row/arena/normalization/
constraint/guard/obligationを作らない。別に凍結するsuccessorだけがSDCを
by-value consumeしてexact typeをoverlayできる。2 Set name/RHSは
source-type ownership外。

Completion evidence: [central Task-269SDC historical contract](../../task_contracts/ja/269SDC.md#completion-evidence)。

## Task 269SDT Normative Checker ABI

これは [central Task-269SDT contract](../../task_contracts/ja/269SDT.md) の
durable derived checker ABIであり、language-specification authorityではない。

field orderとread-only ABIはexactに次のとおり。

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalGivenDescendantTypeHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    dependency: SourceProofLocalGivenDescendantBindingHandoff,
    dependency_fingerprint: String,
    binding_env: BindingEnv,
    binding_fingerprint: String,
    source_type: SourceTypeApplicationHandoff,
    source_type_fingerprint: String,
}

impl SourceProofLocalGivenDescendantTypeHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub const fn dependency(&self)
        -> &SourceProofLocalGivenDescendantBindingHandoff;
    pub fn dependency_fingerprint(&self) -> &str;
    pub const fn binding_env(&self) -> &BindingEnv;
    pub fn binding_fingerprint(&self) -> &str;
    pub const fn source_type(&self) -> &SourceTypeApplicationHandoff;
    pub fn source_type_fingerprint(&self) -> &str;
    pub fn debug_text(&self) -> String;
    pub(crate) fn validate_installation(
        &self, source_id: SourceId, module_id: &ModuleId, arena: &TypedArena,
    ) -> Result<(), SourceProofLocalGivenDescendantTypeError>;
    pub(crate) fn validate_complete_installation(
        &self, source_id: SourceId, module_id: &ModuleId, arena: &TypedArena,
        installation_available: bool,
    ) -> Result<(), SourceProofLocalGivenDescendantTypeError>;
}

pub struct SourceProofLocalGivenDescendantTypeProducer;
impl SourceProofLocalGivenDescendantTypeProducer {
    pub fn build(
        dependency: SourceProofLocalGivenDescendantBindingHandoff,
        input: SourceTypeHandoffInput,
        symbols: &SymbolEnv,
        arena: &TypedArena,
    ) -> Result<
        SourceProofLocalGivenDescendantTypeHandoff,
        SourceProofLocalGivenDescendantTypeError,
    >;
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceProofLocalGivenDescendantTypeError {
    InvalidDependency,
    InvalidBindingEnvironment,
    InvalidSourceType,
    InvalidInstallation,
}
```

4つのexact `Display` stringsは
`source proof-local given-descendant type dependency is invalid`、
`source proof-local given-descendant typed binding environment is invalid`、
`source proof-local given-descendant source type is invalid`、
`source proof-local given-descendant type installation is invalid`。
enumは `std::error::Error` を実装し、sourceや追加public methodsを持たない。

exact debug grammarは次のとおり。

```text
source-proof-local-given-descendant-type-debug-v1
module: {package}::{module}
dependency-fingerprint: {Rust-debug-quoted complete SDC debug text}
binding-fingerprint: {Rust-debug-quoted complete overlaid BindingEnv debug text}
source-type-fingerprint: {Rust-debug-quoted complete source-type debug text}
```

最終行を含む各行はexact 1 LFで終わる。nested fingerprintはRust `Debug`
quoteとembedded `\n` bytesを保持する。producer/install validation precedenceは、
SDC identity/complete fingerprint、`3/2/0` overlay/fingerprint、exact
input/common `SourceTypeBindingProfile::ProofLocalGiven`/source-type/arena、
installation availabilityの順。failureはpartial ownerをpublishしない。

common source-type inputはexact 2 dense application/expression rowsを持つ。
reserve `set@14..17` はbinding/ordinal/root `0/0/0`、Given
`set@95..98` は `1/1/1`。両方ともtyped nodes 0/1のexpression/head
rolesを持つnormal bare `BuiltinSet` headsである。argumentsと全extension
tablesはempty。

exact site-role stringsは `source.type.expression` と
`source.type.head`。arena kind stringsは
`source.proof-local.given-descendant.reserve-type`、
`source.proof-local.given-descendant.type`、
`source.proof-local.given-descendant.type-root`。root node 2は `0..179`、
children `[0,1]`、nodes 0/1は `14..17` と `95..98`。全nodeは
unresolved/unknown-typed/normal/link-freeである。

compositeはSDC dependencyをby valueで所有し、standalone slotをco-install
できない。atomic publicationとreciprocal exclusionは
[Typed](./typed_ast.md#task-269sdt-typed-ownership) および
[Resolved](./resolved_typed_ast.md#task-269sdt-resolved-ownership) sectionが所有する。

exact checker tests:

- `task269sdt_exact_descendant_type_composition_is_stable`
- `task269sdt_dependency_binding_input_and_arena_corruption_fail_closed`
- `task269sdt_typed_and_resolved_ownership_is_atomic`
- `task269sdt_generic_neighbor_and_descendant_use_routes_remain_isolated`
