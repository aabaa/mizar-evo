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
