# Source Functor-Definition Transport

> canonical languageはEnglishです。canonical companion:
> [../en/source_functor_definition.md](../en/source_functor_definition.md)。

## Task 260 Scope / Authority

Checker Task 260はordinary `func` definitionとinitial correctness obligationの
syntax-free immutable source-to-checker intakeを所有します。authorityはChapter
10 §§10.1--10.6、Chapter 16 §§16.4/16.6.1/16.7.2、existing parser
functor-definition pass/recovery fixture、active predicate/functor definition
gap fixture、committed Tasks 248--256/259 public transportです。

本taskはmissing producerの`source_drift`とexact consumerのspec-derived
`test_gap`だけをcloseします。definition acceptance、correctness proof、fact/
axiom、FOL goal、discharge、VC、Core/ControlFlow IRは作りません。

## Frozen Exact Source

future active sourceはfinal LF込みでexactly次です。

```mizar
definition
  let x be set;
  let y be set;
  assume x = x;
  func Task260EqualsDef: task260_equals(x) -> set equals x;
  func Task260MeansDef: task260_means(y) -> set means x = y;
  existence by computation(steps: 1);
  uniqueness by computation(steps: 1);
end;
```

262 bytes/9 lines、SHA-256は
`9bbf50016c72faf8b86342a9a65f8d59bf7747b85b43b6c5bc3c624c7212416a`です。
normal definition block 1件、separately written builtin-set parameter 2件、
source guard 1件、`equals`/`means` functor各1件、builtin-set return type 2件、
primary-term/formula definiens各1件、explicit existence/uniqueness clause各1件を
含みます。import/reserve/predicate/property/theorem/proof block/conditional/
otherwise/redefinition/notation/recoveryはありません。

## Frozen Surface / Resolver Profile

frontendはzero diagnostics、dense Surface rows 108、root node 107/range
`0..261`/normalです。relevant rowはparameter type `62/63`・`66/67`、
DefinitionParameter `65/69`、guard subtree `70..77`、equals pattern/return/body/
definition `78..84`、means pattern/return/body/definition `85..95`、existence
computation/justification/correctness `97/98/99`、uniqueness
`101/102/103`、common DefinitionBlockItem `104`です。exact rangeはcanonical EN
tableをauthorityとします。

runnerはloaded byte/final LF、全108 rowのkind/range/recovery/ordered child、root、
direct sibling order、subtree partitionをauthenticateし、checkerへraw node ID/
syntax kindを渡しません。

resolverはshell 3、projection 2、diagnostic 0、Functor symbol/definition各2、
local contribution 1です。shell 0はblock `104/0..261`、shell 1はequals
`84/61..118`、shell 2はmeans `95/121..179`。definitionsは
`task260_equals ( x )` path `[4,0,9,0]`と`task260_means ( y )` path
`[4,0,9,1]`で、normal/local/exported/overloadable/conflict-freeです。
resolverのparameters/binders/arityはemptyなので、parameter、guard、style、
return type、definiens、correctness associationをそこからinferしません。

## Frozen Lower Bundle

exact sourceはTask 248 Profile B `1/2/2/2/2/2/0`、Task 249 + 249R
`2/4/0/2`（binding-linked parameter type 2、independent definition-return 2）、Task
252 `5/5/0`、Task 256 `2/0/0/0/0/0/0/4/4`をconsumeします。Tasks 253--255と
259はabsentです。Task-252 orderはguard `x/x`、equals body `x`、means body
`x/y`。Task-256 formula 0はguard、formula 1はmeans bodyです。pattern locus、
label、return token、correctness keyword、computation descendantをlower direct
discoveryからexcludeします。

Task-260 definiens targetはcommitted lower roots、すなわちTask-252 Primary、
Task-253 Application、Task-254 Structure、Task-255 SetTerm、Task-256
AtomicFormulaだけを表します。active sourceはPrimary(2)/AtomicFormula(1)です。
conditional/case/otherwise/composite formula/nested unsupported rootはadmitしません。

## Public Syntax-Free Contract

new `source_functor_definition.rs`はdefinition/parameter/guard/definiens/
correctnessのfive dense ID/tableを公開します。definition rowはresolver identity、
site/range/ordinal/context/recovery/spelling、Equals/Means style、
`SourceTypeDefinitionReturnId` return-type ID、
definiens ID。parameter rowはbinding/written type/site/ranges/context。guard rowは
atomic formula/site/range/context。definiens rowはowner/target/site/range/context。
correctness rowはowner、Existence/Uniqueness、site/range/justification anchor、
obligation IDを保持します。

全enum/errorは`#[non_exhaustive]`、IDは`new/index`、tableは
`get/iter/len/is_empty`、rowはread-only getterです。handoffはTask-248/249/252の
required fingerprintとTask-253/254/255/256のtarget-present時だけのoptional
fingerprintを公開し、Task-259 input/fingerprint/getterを持ちません。active
cardinalityは`2/2/1/2/2`です。parameter/guardはshared context rowで、definition
ごとにduplicateしません。

### Exact API / debug synchronization

canonical ENでfreezeしたexact Rust ABIをcompanionも同じ契約として参照します。
inputは`SourceFunctorDefinitionHandoffInput`とfive
`SourceFunctor{Definition,Parameter,Guard,Definiens,Correctness}Input`で、field
type/orderもEN code blockどおりです。styleは`Equals/Means`、targetは
`Primary(SourcePrimaryTermId)`、`Application(SourceFunctorApplicationId)`、
`Structure(SourceStructureTermId)`、`SetTerm(SourceSetTermId)`、
`AtomicFormula(SourceAtomicFormulaId)`、correctnessは
`Existence/Uniqueness`、recoveryは`Normal/Degraded`だけです。

immutable row/table/handoffのfield/getterと
`SourceFunctorDefinitionProjection::{base_initial_obligations,handoff,
initial_obligations,into_parts}`はcanonical ENのexact signatureです。
`SourceFunctorDefinitionProducer::build`はenv、required context/type/primary、
optional application/structure/set/atomic、baseline obligation、arenaをこの順で
受け取ります。error variantは`SourceIdentityMismatch`、`DependencyMismatch`、
index付き`InvalidResolverDefinition/Definition/Parameter/Guard/Definiens/
Correctness`、`InvalidObligation`、`InvalidArenaOwnership`、
`UnsupportedTaskShape`だけです。

debug headerはexact `source-functor-definition-debug-v1`で、module、required
fingerprint 3件、optional fingerprint 4件、definition/parameter/guard/definiens/
correctness rowをcanonical EN grammar順にfinal LF付きでemitします。active row
oracleはdefinition site `84/95`、parameter `65/69`、guard `77`、definiens
`83/94`、correctness `99/103`、context 1、lower IDとrange/spelling/origin pathは
canonical ENのexact valuesです。

## Public Enum Policy

| Public enum | compatibility policy |
| --- | --- |
| `SourceFunctorDefinitionStyle` | `#[non_exhaustive]`。callerはlater explicitly-frozen definition styleを許容する。 |
| `SourceFunctorDefiniensTarget` | `#[non_exhaustive]`。callerはlater explicitly-frozen lower-root targetを許容する。 |
| `SourceFunctorCorrectnessKind` | `#[non_exhaustive]`。callerはlater explicitly-frozen correctness kindを許容する。 |
| `SourceFunctorDefinitionRecovery` | `#[non_exhaustive]`。callerはlater recovery classを許容する。 |
| `SourceFunctorDefinitionError` | `#[non_exhaustive]`。callerはvalidation failureをexhaustive matchしない。 |

この module が所有する exhaustive public enum exception はない。

## Initial Obligations

`InitialObligationKind`にはexactly `FunctorExistence`と`FunctorUniqueness`を
追加します。Equalsはexistence/uniquenessをappendしません。Meansはexplicit
clause orderでPending row 2件をappendします。baseline lengthを`b`とするとIDは
`b`/`b+1`、owner/rangeはsite `99`/`103`と`182..217`/`220..256`です。
goalは
`source.definition.functor.correctness:definition=1:existence` / `...:uniqueness`、
provenanceは`source.definition.functor:definition=1:correctness=0` / `...=1`
です。assumptionsはempty、statusはPendingです。これはguardされないgoalをclaim
せず、Task 260がtyped-parameter/guard/return/formulaのFOL compositionをinvent
しないことだけを意味します。

computation justification subtreeはfuture proof owner用にSurfaceでpreserveし、
optionをreadせず、proof/discharge/acceptance/activation/fact/axiom/VCを作りません。

input baselineはunrelated existing obligation kindを保持できますが、
`FunctorExistence`、`FunctorUniqueness`、`PredicatePropertyCorrectness`を1件も
含められません。build/typed installationはpre-existing rowをrejectします。
Task-260 handoffありではcompleted tableのfunctor kindはlinked row 2件だけ、
handoffなしではfinal assemblyがorphan functor kindをすべてrejectします。

## Typed/Resolved Ownership / Task 259 Separation

producerはsource/module、fingerprint、arena、ordinal/context、resolver、style/
target、return、grouping、correctness cardinality/order、baseline obligationを
complete validateしてからpublishします。partial means、equals correctness、wrong
target、stale dependency、copied site、reorder、cross-source/context、overlap、
extra/missing rowをatomicにrejectします。

projectionはhandoff、unchanged baseline obligation、baseline-plus-twoを返します。
`TypedAst::with_source_functor_definition(SourceFunctorDefinitionProjection)
-> Result<Self, TypedAstError>`とtyped/final
`source_functor_definition() -> Option<&SourceFunctorDefinitionHandoff>`、
`TypedAstError::InvalidSourceFunctorDefinition`、
`ResolvedTypedAstError::InvalidSourceFunctorDefinition`をexactに追加します。
`TypedAstParts`/`ResolvedTypedAstInputs`はfieldを追加しません。

Task 259/260は本taskではmutually exclusiveです。Task 260はTask-259 handoffまたは
`PredicatePropertyCorrectness` baselineをrejectし、finalも両handoff coexistenceを
rejectします。cross-family install-order promiseやTask-259 compatibility editは
ありません。existing
mixed fixture/sidecar/expectation/traceはdocs prerequisiteでbyte-unchanged、later
implementationでもrebaselineせずisolationだけに使えます。

## Dedicated Consumer And Trace Intent

future active pairは
`pass_type_elaboration_functor_definition_payload_001.miz` / `.expect.toml`です。
sidecarはpass/type_elaboration/type_check、empty diagnostics/payloadでfuture
`spec.en.checker.type_elaboration.source_functor_definition_payload`だけをciteします。
pass creditはtransport/pending-obligation intakeだけです。one covered trace rowは
このsidecarだけをbacklinkします。

checker test 5件とrunner test 4件のexact names/write scopeはcanonical ENを
authorityとし、次のassertion allocationも同期します。

checker 1はfive `2/2/1/2/2` row/all getter/active field/style/target/
correctness/fingerprint `None/None/None/Some`/complete debug/serializer string/
obligation全field、2は全input row field/dense order/cardinality/context/style/
target/return/recovery/resolver/correctness associationのindependent corruption、
3は全lower fingerprint/ID/arenaとobligation field/link/pre-existing/orphan/extra
kind、4はnonempty unrelated baseline preservation、ID `b`/`b+1`、projection/
`into_parts`、one-shot/rollback/prior occupancy、5はno-handoff legacy debug、
final clone/replay/revalidation、Task-259 mutual exclusionとempty semantic outputを
coverします。

runner 1は262 bytes/final LF/hash、108 row/children/root/sibling/subtree、resolver
3-shell/2-definition、lower bundle/final output、2はsource/AST/excluded subtree/
resolver/lower mutationのowner停止、3はsource-only selection、expectation
non-selection、Task-259/mixed isolationとmetadata reciprocal sole backlink、4は
computation subtree non-consumption、goal/proof/discharge/acceptance/fact/axiom/
VC/IR/public diagnostic absenceおよびsix active-count consumerをcoverします。

future implementationはnew producer/support、lib/typed/final
（`typed_ast.rs` serializerを含む）、`type_checker.rs` /
`registration_resolution.rs`のexternal serializer 2件、lint policy、private runner route/facade/test、six mechanical
active-count assertions、新規fixture/sidecar/trace、derived EN/JAだけを変更できます。
parser/resolver/Cargo/canonical spec/existing `.miz`/existing expectation/lower ownerは
forbiddenです。

`source_spec_audit.md`のmodule-spec/crate-export/public-surface inventoryは上記exact
APIをenumerateします。`tests/lint_policy.rs`は
`source_functor_definition.rs`/`.md`をdocumented public module、public enum
policy、source/spec audit coverageの3 allowlistすべてへ追加します。existing
syntax-dependency scanはunchangedでexceptionを追加しません。

`typed_ast.rs`、`type_checker.rs`、`registration_resolution.rs`の3 serializerは
exactly `FunctorExistence => "functor_existence"`、
`FunctorUniqueness => "functor_uniqueness"`を追加します。

Task 249Rがchecker baselineを`435 -> 439`へ移し、その後Task 260はchecker
`439 -> 444`、runner `512 -> 516`、resolver/syntax
`144/59` unchanged。corpus/requirements `422/390 -> 423/391`、pass/fail
`229/193 -> 230/193`、active `101/7/199/1 -> 101/7/200/1`、type coverage
`254/242 -> 255/243`、warnings/errors `23/0`です。

## Deferrals / Exit

guard/parameter/return goal composition、FOL existence/uniqueness、proof、discharge、
acceptance/activation、fact/axiom、overload/call、conditional consistency/coverage、
dependent/attributed return semantics、recursion/redefinition/notation/property、
composite formula/imported/mixed acceptance、Core/CFG/VC、Task 261+はdeferredです。

docs prerequisiteはEN/JA sync、repeat review **NO FINDINGS**、executable artifact/
count/hash unchanged、all nine gates PASS、valid 90+ quality、exact staging、docs
commit、clean worktree、protected stash unchanged、fresh implementation selectionで
exitします。implementationも同じreview/hard gateとprojected executable count、
one logical-task commitを必須とします。
