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
