# Source Formula Composition Transport

> canonical languageは英語。英語版:
> [../en/source_formula_composition.md](../en/source_formula_composition.md)。

## Responsibility And Authority

Checker Task 257B1はquantified formulaの最初のcross-family compositionとして、
explicit universal binder 1件、atomic equality body 1件、そのbinderのlexical
scopeにcaptureされるprimary-term reference 2件をownする。canonical authorityは
Chapter 4 §§4.1/4.5/4.6とChapter 14 §§14.4.1/14.4.4/14.5.2/14.7.5。
Task 252はvariable-reference occurrence 2件、Task 256はequality occurrenceと
operand edge、Task 257Aはuniversal occurrence、binder、body context、written
binder typeを引き続きownする。

equality/quantifier evaluation、binder type relativization、formula fact、
theorem acceptance、proof、CoreIr、ControlFlowIr、VCは作らない。
`BindingEntry::captured`は空のままである。このfieldはclosure-like binderの
free-variable capture用であり、本taskはlexical lookupで選ばれた通常のbound
occurrenceを別tableへ記録する。

## Exact Real Consumer

implementationは次のspec-derived type-elaboration fixtureを1件追加する。

```mizar
theorem FormulaQuantifierBoundUsePayloadBoundary: for x being set holds x = x;
```

trailing newline込み79 bytes、SHA-256は
`757872ac21c2a924c7c47f23328f5d76a8504255c195c17f113041c81bae5f3c`。
frozen half-open rangeはuniversal `50..77`、binder segment `54..65`、
binder identifier `54..55`、binder type/head `62..65`、equality `72..77`、
left occurrence `72..73`、right occurrence `76..77`である。implementation
preflightでreal parserから再測定し、差があればparser behavior変更のauthority
ではなくdocumentation `design_drift`として扱う。

sidecarはtype-elaboration passで、positive claimはsource-to-checker transport
成功だけである。equality/quantified formula truth、theorem status、accepted
fact、proofへのcreditは与えない。

## Task-257A Profile Extension

`SourceCompositeFormulaProducer`へTask-257A profile/debug bytesを変更しない
第2 exact profileを追加する。新profileのformula/wrapper/root/binder/type-site/
composite-edge/requestは`1/0/1/1/1/0/2`。context 0のuniversal 1件、
unassigned root 1件、explicit `x` binder/bare-`set` type site 1件、same-family
child edge 0件、quantifier-semantics/binder-type request 2件である。extended
binding environmentは`2/1/4`のまま。

exact real Task-257A `5/0/1/1/1/4/6` transaction、そのvalidation/debug、
installation、既存consumerはbyte-identicalを維持する。このpreservationは
exact profile partitionがretireしてTask 257B2へdeferする従来のsynthetic
nonempty-wrapper admissionを含まない。既存public input field/row meaningを
repurposeしない。profile discriminatorはvalidated table shapeからderiveし、
callerがsourceでmodeを指定しない。
exact 2 profileだけをacceptする。Task-257A cardinalityへTask-257B1
formula/request/binder/edgeを混ぜる形、そのinverse、otherwise well-formed
third shapeはatomicにfailする。

## Cross-Family Transaction

Task 257B1はpublic syntax-free `source_formula_composition` moduleを追加し、
atomic edgeとbound useの2 dense tableを持つ。

```rust
pub struct SourceFormulaCompositionHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub atomic_edges: Vec<SourceFormulaAtomicEdgeInput>,
    pub bound_uses: Vec<SourceQuantifierBoundUseInput>,
}

pub struct SourceFormulaAtomicEdgeInput {
    pub formula: SourceCompositeFormulaId,
    pub ordinal: usize,
    pub role: SourceFormulaAtomicEdgeRole,
    pub child: SourceAtomicFormulaId,
}

pub struct SourceQuantifierBoundUseInput {
    pub binder: SourceQuantifierBinderId,
    pub ordinal: usize,
    pub body_edge: SourceFormulaAtomicEdgeId,
    pub term: SourcePrimaryTermId,
    pub reference: SourcePrimaryTermReferenceId,
}
```

dense idは`SourceFormulaAtomicEdgeId`と`SourceQuantifierBoundUseId`。
private storage、`new`、`index`を持つ。immutable rowはread-only accessor、
tableは`get`/source-ordered `iter`/`len`/`is_empty`だけを公開する。
`SourceFormulaAtomicEdgeRole`は本sliceでは`UniversalBody`だけ。
`SourceFormulaCompositionError`と全public enumは`#[non_exhaustive]`。

## Public Enum Policy

| public enum | compatibility policy |
|---|---|
| `SourceConditionFormulaCompositionError` | `#[non_exhaustive]`。callerはcondition/formula validation failureをexhaustive matchしない。 |
| `SourceFormulaAtomicEdgeRole` | `#[non_exhaustive]`。callerはlater frozen cross-family body roleを許容する。 |
| `SourceFormulaCompositionError` | `#[non_exhaustive]`。callerはvalidation failureをexhaustive matchしない。 |
| `SourceFraenkelGeneratorBindingContextError` | `#[non_exhaustive]`。callerはFraenkel generator binding-context validation failureをexhaustive matchしない。 |
| `SourceFraenkelGeneratorBoundUseError` | `#[non_exhaustive]`。callerはFraenkel generator bound-use validation failureをexhaustive matchしない。 |
| `SourcePredicateChainCompositionError` | `#[non_exhaustive]`。callerはpredicate-chain composition validation failureをexhaustive matchしない。 |

この module が所有する exhaustive public enum exception はない。

exact producer/output surfaceは次のとおり。

```rust
impl SourceFormulaCompositionProducer {
    pub fn build(
        input: SourceFormulaCompositionHandoffInput,
        primary_terms: &SourcePrimaryTermHandoff,
        atomic_formulas: &SourceAtomicFormulaHandoff,
        composite_formulas: &SourceCompositeFormulaHandoff,
        arena: &TypedArena,
    ) -> Result<SourceFormulaCompositionHandoff, SourceFormulaCompositionError>;
}

pub struct SourceFormulaCompositionHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    primary_term_fingerprint: String,
    atomic_formula_fingerprint: String,
    composite_formula_fingerprint: String,
    atomic_edges: SourceFormulaAtomicEdgeTable,
    bound_uses: SourceQuantifierBoundUseTable,
}
```

3 fingerprintはTask-252/256/257 dependency順の各`debug_text()` exact owned
copyでnonempty。read-only `primary_term_fingerprint()`、
`atomic_formula_fingerprint()`、`composite_formula_fingerprint()`を公開する。
handoffは`source_id()`、`module_id()`、`atomic_edges()`、`bound_uses()`も
公開し、mutable/unchecked publication surfaceはない。

`debug_text()`はdeterministicで
`source-formula-composition-debug-v1\n`から始まる。module identity、Rust
debug-string escapeしたprimary/atomic/composite fingerprint、`atomic-edges: N`
と全edge row、`bound-uses: N`と全use rowの順。edge fieldは
id/formula/ordinal/role/child、use fieldは
id/binder/ordinal/body-edge/term/reference。role spellingは
`universal-body`。positive testはsubstringや2 incomplete output比較ではなく
single full literal renderingをfreezeする。

producerはtransaction全体のvalidation後だけimmutable handoffをpublishする。

real atomic-edge/bound-use aggregateは`1/2`。

| row | association |
|---:|---|
| atomic edge 0 | composite universal 0、ordinal 0、universal-body、atomic equality 0 |
| bound use 0 | binder 0、ordinal 0、edge 0、primary term 0、reference 0 |
| bound use 1 | binder 0、ordinal 1、edge 0、primary term 1、reference 1 |

Task-252 dependencyはterm/reference/numeric request `2/2/0`。両termはbody
context 1のnormal `VariableReference`/`Value`、spelling `x`で、
`BindingEnv::lookup`によりlexical scope `[0]`、use ordinal 1、binding 0へ
resolveする。Task-256 dependencyはformula/wrapper/head/candidate/type/
attribute/edge/request `1/0/0/0/0/0/2/2`。context 1のequality 1件、
primary-term operand edge 2件、unresolved operand-expected-type request 2件。

## Validation And Final Ownership

validationはsource/module identity、全dependency fingerprint、dense order、
exact universal/binder/body-context relation、atomic equality context/range、
equality operand edge 2件、primary-term reference 2件、binding lookup winner、
source order、containmentをauthenticateする。各bound-use termはatomic childの
direct operandで、各referenceはcomposition binderのbindingを選ぶ。term/
referenceのomit/duplicate/reorder/cross-context/out-of-range/別binder/formula
associationはfail closed。

既存`TypedAst::with_source_composite_formula`はcomplete Task-257A profile
専用のまま。new
`TypedAst::with_source_formula_composition(self, composite, composition)`は
combined one-shot installerで、Task-252/256は先行installを要求するが、第2
composite profileとcomposition handoffを同時にvalidate/publishする。atomic
body edge/bound-use rowなしの第2 profileはpublic `TypedAst`に現れない。
`source_context()`はabsent必須で、preinstalled Task-248 source-context
handoffをatomic rejectし、embedded source-derived `2/1/4` binding environment
のTask-257 sole ownershipを維持する。
legacy installerはvalid uninstalled Task-257B1 composite profileをrejectする。
combined installerはTask-257A composite handoffを既にownするASTをrejectする。
両failureはpre-existing handoff/debug byteをすべて保持し、第2 profile/
composition handoffを一切publishしない。
`TypedAst::source_formula_composition()`と
`ResolvedTypedAst::source_formula_composition()`がoptional immutable handoffを
公開する。typed/resolved debugはexisting Task-252 term、Task-256 atomic
formula、Task-257 composite formula、Task-257B1 compositionの順でrenderする。
absent時はlegacy byteをexactに保持する。
`ResolvedTypedAst::assemble`は全
fingerprintを再validateし、rowをrebuildせず同じhandoffをclone-preserveする。
missing/stale/substituted/reordered dependencyはatomic failure。Task-257A-only
ASTはcomposition handoffなしで引き続きvalidである。

## Tests And Exit Boundary

checker testは第2 composite profile、exact `1/2` composition、dependency
fingerprint、bound-use lookup/order、全field/association corruption、
deterministic replay、full literal debug、combined one-shot install、missing/wrong
dependency rejection、preinstalled Task-248 source-context rejection、
Task-257A debug byte不変、legacy-installer B1 rejection、combined-installer
Task-257A rejection、cross-profile/hybrid/third-shape rejection、rollback byte
preservation、final clone ownershipをcover
する。runner testはexact parser range、`2/2/0`、
`1/0/0/0/0/0/2/2`、`1/0/1/1/1/0/2`、`1/2` aggregate、same-arena
composition、selector isolation、semantic table/accepted fact非生成をcoverする。

implementationはnew pass sidecarだけへ
`spec.en.checker.type_elaboration.source_quantifier_bound_use_payload`をmapped
するcovered trace requirementを1件追加できる。既存Chapter-4/14、
Task-252/256/257A rowへstatus不変のreciprocal transport noteだけを追加できる。
projected countはplan `415/381`、type-elaboration `247/235`、pass/fail
`225/190`、active parse/declaration/type/proof `101/5/194/1`。

Task 257B2はconjunction/disjunction/`iff`/repetition/executable grouping、
Task 257B3はexistential/restricted/nested quantification、implicit reserved
binder、その追加scoped useをretainedする。Task 257Cはpredicate-chain/
conditioned-comprehension compositionをretainedする。

## Implementation Result

Task 257B1はこのfrozen boundaryを実装済みである。exact 79-byte pass consumerは
one arenaでTask-252 `2/2/0`、Task-256 `1/0/0/0/0/0/2/2`、第2
Task-257 `1/0/1/1/1/0/2`、formula-composition `1/2` transactionをbuildする。
direct `x` reference 2件はbody context 1のbinding 0をselectし、occurrence
ownershipはTask 252に残り、`BindingEntry::captured`は空のままである。

combined installer、legacy-profile partition、dependency fingerprint、full
literal debug rendering、corruption matrix、Task-248/Task-257A exclusion、
resolved clone ownershipはexecutableである。covered trace requirement
`spec.en.checker.type_elaboration.source_quantifier_bound_use_payload`はnew pass
sidecarだけにmapする。countはplan `415/381`、type-elaboration `247/235`、
pass/fail `225/190`、active parse/declaration/type/proof `101/5/194/1`、
warnings/errors `23/0`。次のdependency-ready formula sliceはTask 257B2である。

## Task 257B2 frozen connective/grouping addendum

Task 257B2はcrate planでfreezeしたexact 166-byte sourceだけについてこの
transportを拡張する。Task-257B1のexplicit `x being set` binder/body contextを
保持するが、`x` occurrenceは意図的に0件。bodyはgrouped left sideにrepeated
conjunction/disjunction、grouped right sideにfixed conjunction/disjunctionを
持つ1件の`iff`である。これはChapter-14 source transportでありconnective
evaluationではない。

第3 exact composite profileはformula/wrapper/root/binder/type-site/
same-family-edge/request `8/6/1/1/1/7/9`、binding environment `2/1/4`。
`Conjunction`、`RepeatedConjunction`、`Disjunction`、
`RepeatedDisjunction`、`Biconditional` kindを追加する。exact same-family
treeが追加するroleはdisjunction-left/rightとbiconditional-left/rightだけ。
conjunction/repeated nodeはcomposition tableでatomic childへ到達し、repeated
kindはkind/canonical spellingでdistinct。

real `ParenthesizedFormula` wrapper range 6件は`72..122`、`73..94`、
`98..121`、`127..164`、`128..143`、`147..163`。formula/ordinal group順、
independent typed site、context 1、normal recovery、exactly one ownerとの
associationとowner rangeのstrict containmentを保持し、formula/child edge/
request/semantic resultにはならない。outer wrapper内にdescendant rangeは
nestできるが、unrelated siblingをoverlap/containできない。

lower profileはTask 252 `16/0/16`、Task 256
`8/0/0/0/0/0/16/16`。全primary termがnumeralなのでbinder referenceは0件、
`BindingEntry::captured`もempty。compositionはatomic-edge/bound-use `8/0`。
new `ConjunctionLeft/Right`、`DisjunctionLeft/Right` roleで8件のTask-256
equalityをnearest composite parentへassociateし、Task-252/256 occurrenceを
copyしない。

validationはexact dependency profile/fingerprint、formula tree/context、
fixed/repeated kind/direct repetition token、wrapper、range/source order/
parent containment、atomic association、reference/bound-use absenceを認証する。
A/B1/B2 hybrid、第4 otherwise-valid profile、wrapper crossing/substitution、
fixed/repeated substitution、dependency replacement、omitted/duplicate/
reordered/cross-source associationをrejectする。

existing combined installerはexact Task-252/256 install後にB2 composite/
compositionをatomic publishする。legacy composite installerはA-only。
preinstalled Task-248 source contextまたはexisting A/B1
composite/compositionはprior byteを変えずB2 publicationをrejectする。final
assemblyはexact handoffを再validateしてclone-preserveする。
`source-formula-composition-debug-v1` headerと旧A/B1 renderingはbyte-identical。

implementationは
`pass_type_elaboration_formula_connective_grouping_payload_001`、covered row
`spec.en.checker.type_elaboration.source_connective_grouping_payload`、
status不変のreciprocal transport noteだけを追加できる。projected countはplan
`416/382`、type `248/236`、pass/fail `226/190`、active `101/5/195/1`。
connective truth、general repetition validation/expansion、theorem acceptance、
fact、proof/IR/VC、Task 257B3、Task 257C、Steps 6/7はdefer。

このaddendumはdocumentation prerequisiteだけで、production、fixture、
sidecar、trace metadata/count、executable coverageを変えない。baselineは
plan `415/381`、type `247/235`、pass/fail `225/190`、active
`101/5/194/1`、warnings/errors `23/0`。

## Task 257B2 Implemented Connective/Grouping Composition

third profileを実装し、8 ordered atomic edgesがrepeated/fixed conjunction/
disjunction rowsを8 Task-256 equalitiesへmapする。explicit binderはunusedなので
bound-useはemptyである。dependency spelling/context/numeric request/
fingerprint、wrapper/tree ownership、empty captureをfail-closedで再検証する。
combined publication/final cloneはatomicで、truth、repetition expansion、
theorem status、fact、proof、IRはdeferredのまま。

## Task 257B3 Frozen Nested-Quantifier Composition

fourth composition profileはexact `3/6`。atomic rowはouter restrictionとinner
restrictionをnew `UniversalRestriction` role、innermost equalityを
`UniversalBody`でassociateする。source-order bound-use 6件はTask-252
referenceを指し、`x` 3件はouter binder 1、`y` 1件はbinder 2、`r` 2件は
reserved binding 0ではなくinner binder 3をselectする。各associationはtermを
encloseするatomic edgeをnameする。
source compatibilityのためpublic `SourceQuantifierBoundUseInput::body_edge`、
immutable `SourceQuantifierBoundUse::body_edge()`、`body-edge` debug keyは
B1と同じ名前を保ち、B3 restriction useではowning atomic edgeへgeneralizeする。
exact owning-edge idは`0,0,1,1,2,2`、binder-row idは
`0,0,2,1,0,2`、per-binder ordinalは`0,1,0,0,2,1`。

validationはTask-48 reserve-default provenance、Task-252 `6/6/0`、
Task-256 `3/0/0/0/0/0/6/6`、Task-257B3
`3/0/1/3/3/2/6`、context ancestry、lexical lookup replay、shadow、
nearest-parent role、fingerprint、final ownershipをauthenticateする。direct
nested quantified useは`CapturedFreeVariables`にならない。quantified truth、
restriction discharge、witness、theorem closure、fact、acceptance、proof、IRを
produceしない。

## Task 257B3 implementation status

fourth profileと`3/6` association transactionはexact source consumerから
executableになった。checker/real-runner testsはparent role 3件、lookupで
選択したuse 6件、dependency fingerprint、deterministic rendering、atomic
installation/rollback、resolved cloneを認証する。atomic-edge validationは
deeper descendant composite formulaもatomをcontainする場合のouter assignmentを
rejectし、frozen nearest-parent/subtree exclusionを保持する。frozen semantic
deferralは不変。

## Task 257C1 prerequisite boundary

Task 257C1が供給するのはlower Task-256 predicate-segment graph/shared term
boundaryだけで、本moduleにnew rowは追加しない。predicate-chain implicit
conjunction/segment-local semantic negationはC1 implementation後に別途freeze
するTask-257C composition contractを必要とする。conditioned-comprehension
compositionも別Task-255 condition-bearing prerequisiteを待つ。

ただしimplementationでは、本production fileのexisting
`SourceAtomicFormulaHandoffInput` literal 3件すべてにempty
`predicate_segments` vectorを追加する。対応するmizar-test composition
literal 3件もemptyのまま。これはextended input shapeのmandatory
compatibility editで、composition row、selector admission、debug output、
semantic behaviorを追加しない。

compatibility editはinstall/verify済み。Task 257C1がpublishするのはlower
Task-256 handoffだけで、本moduleはpredicate-chain composition rowやsemantic
conjunction/negationをまだownしない。

## Task 257C2 frozen condition-formula composition

Task 257C2は本moduleにdedicatedな第2 transactionを追加する。Task-255C1
`SourceSetConditionId` 1件をdirect Task-256 `SourceAtomicFormulaId` 1件へ
associateし、synthetic `SourceCompositeFormulaHandoff`を作成・要求しない。
canonical authorityはChapters 10 §10.1、13 §§13.4/13.4.2、14
§§14.2/14.5.2/14.8。exact sourceはcommitted 191-byte conditioned-
comprehension fixtureのままで、final-LF SHA-256は
`8d9c3208d0e5a099e54c58f57642642046f0669c9b49e30d115549ba15a6eb3f`。

lower graphはTask-252 `4/0/4`、Task-253 `1/0/1/2/2`、Task-255
`1/0/1/1/1/1/2`、Task-256
formula/wrapper/segment/head/candidate/type/attribute/edge/request
`1/0/0/0/0/0/0/2/2`。Task 255は`177..182` `FormulaExpression` wrapper、
Task 256は同range/spelling `3 = 4`/context 0/normal recoveryを持つ別のdirect
`BuiltinPredicateApplication` equality siteをownする。operand edges 2件は
Task-252 primaries 2/3へ向く。new associationはsiteをownせずlower rowをcopyしない。

new public surfaceは`SourceConditionFormulaCompositionHandoffInput`、
`SourceConditionFormulaEdgeInput`、immutable
`SourceConditionFormulaCompositionHandoff`/`SourceConditionFormulaEdge`、
`SourceConditionFormulaEdgeTable`、dense `SourceConditionFormulaEdgeId`、
`SourceConditionFormulaCompositionProducer`、non-exhaustive
`SourceConditionFormulaCompositionError`。inputはsource/module identityと
`edges` vectorだけを持ち、edgeはcondition/dense ordinal/atomic formulaを保持。
sole exact rowは`0/0/0`。

producerはTask-252 primary、Task-253 application、Task-255 set term、
Task-256 atomic formula、arena dependencyを受け、exact nonempty debug
fingerprint 4件をその順で保持する。IDは`new`/`index`、tableは
`get`/`iter`/`len`/`is_empty`、row/handoffはread-only accessorだけを公開。
public producer/handoff signatureを次にfreezeする。

```rust
impl SourceConditionFormulaCompositionProducer {
    pub fn build(
        input: SourceConditionFormulaCompositionHandoffInput,
        primary_terms: &SourcePrimaryTermHandoff,
        applications: &SourceFunctorApplicationHandoff,
        set_terms: &SourceSetTermHandoff,
        atomic_formulas: &SourceAtomicFormulaHandoff,
        arena: &TypedArena,
    ) -> Result<
        SourceConditionFormulaCompositionHandoff,
        SourceConditionFormulaCompositionError,
    >;
}

impl SourceConditionFormulaCompositionHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub fn primary_term_fingerprint(&self) -> &str;
    pub fn application_fingerprint(&self) -> &str;
    pub fn set_term_fingerprint(&self) -> &str;
    pub fn atomic_formula_fingerprint(&self) -> &str;
    pub const fn edges(&self) -> &SourceConditionFormulaEdgeTable;
    pub fn debug_text(&self) -> String;
}
```

exact ID/table/row signatureはcanonical crate planにfreezeする。errorは
`DependencyMismatch`、`InvalidEdge { edge }`、`InvalidAggregate`。

debugはseparate `source-condition-formula-composition-debug-v1` header、
module identity、primary/application/set/atomic fingerprint、次のrowをrender。

```text
edges: 1
  edge#0 condition=0 ordinal=0 formula=0
```

validationはexact lower profile/fingerprint、equal source/module、direct
wrapper-to-atomic arena relation、equal condition/formula range/spelling/
context/recovery、exact operand edge/request、duplicate Task-255 ownership
なしを要求する。missing/duplicated/reordered/substituted/copied/stale/
wrong-profile inputはpublication前に失敗する。new typed/resolved optional
handoffはlower handoff 4件の後だけinstallし、bounded profileではTask-257
composite/Task-257B compositionをexcludeする。final assemblyはrevalidate/
clone-preserveする。`TypedAstError`と`ResolvedTypedAstError`はそれぞれ
dedicated `InvalidSourceConditionFormulaComposition` variantを追加する。

frozen pre-Task-256C1 baselineでは、このtransactionはseparate
condition-container compatibility prerequisiteがarbitrary overlap rejectionを
weakenせず、authenticated Task-255-encloses-Task-256 relationを両lower-
handoff installation orderでvalidにした後だけexecutableだった。Task 256C1は
両orderをpassした。completed prerequisiteのfresh post-commit preflight前に
C2 production editを開始しない。

既存Task-257B input literal、producer call、table、installer signature、
successful legacy fingerprint/debug byteはすべて不変。legacy Task-257A/
combined Task-257B installerはreciprocal checkを追加し、C2 install済みなら
それぞれexisting `InvalidSourceCompositeFormula`/
`InvalidSourceFormulaComposition` variantでatomicにrejectする。逆にC2
installerはA/B install済みを
`InvalidSourceConditionFormulaComposition`でrejectし、testsは両installation
orderとbyte-identical rollbackをcoverする。exact private consumerは
Task-255C1 selector、
Task-253 imported-`++` seam、reusable Task-256 equality builderをone
surface-indexed arenaでreuseする。fixtureは追加せず、existing fail sidecarは
same definition-intake diagnosticを保ち、reciprocal Task-257C2 spec reference
だけを追加できる。new covered trace row 1件だけがそのsidecarへmapできる。

equality truth、generator binding/reference/capture、predicate-chain
conjunction/segment negation、formula fact/result、sethood/result typing、
definition/theorem acceptance、proof/IR/VC、broader comprehension coverageは
deferred。本documentation prerequisiteはproduction/fixture/sidecar/trace/
count/test list/hashを変更しなかった。その後のseparate Task-257C2
implementation commitはfresh post-Task-256C1 preflight後にfrozen
transactionを完成した。dedicated public handoff/producer/table/dense ID/error
surface、typed/resolved ownership、exact private runner consumer、checker test
3件、runner test 4件、covered trace row 1件、reciprocal sidecar referenceを
追加し、fixture/semantic diagnosticは変更していない。measured exitはplan
`419/386`、type `252/240`、libraries `332/361`、active
parse/declaration/type/proof `101/5/198/1`である。

## Task 257C3 frozen predicate-chain composition

Task 257C3は本moduleのthird independent transactionである。existing
107-byte Task-257C1 pass consumerをreuseし、already validatedなpredicate
segment 2件のcompositionだけをauthenticateする。
`SourceCompositeFormula` rowやsemantic formula resultは作らない。

handoff inputはdense table 2件を持ち、exact profile `1/1`は次。

```text
conjunction#0 formula=0 ordinal=0 left_segment=0 right_segment=1 boundary=1
negation#0 formula=0 ordinal=0 segment=1
```

public familyは`SourcePredicateChainConjunction{Id,Input,Table}`、
`SourcePredicateChainNegation{Id,Input,Table}`、
`SourcePredicateChainCompositionHandoffInput`/`Handoff`/`Producer`、
non-exhaustive `SourcePredicateChainCompositionError`。rowはinput fieldだけ、
ID/tableはstandard dense accessor、handoffはsource/module、exact Task-252/
Task-256 debug fingerprint、両table、deterministic `debug_text()`を公開する。

exact public ID/row/table signatureは次。

```rust
impl SourcePredicateChainConjunctionId {
    pub const fn new(index: usize) -> Self;
    pub const fn index(self) -> usize;
}
impl SourcePredicateChainNegationId {
    pub const fn new(index: usize) -> Self;
    pub const fn index(self) -> usize;
}
impl SourcePredicateChainConjunction {
    pub const fn formula(&self) -> SourceAtomicFormulaId;
    pub const fn ordinal(&self) -> usize;
    pub const fn left_segment(&self) -> SourcePredicateSegmentId;
    pub const fn right_segment(&self) -> SourcePredicateSegmentId;
    pub const fn boundary(&self) -> SourceAtomicEdgeId;
}
impl SourcePredicateChainNegation {
    pub const fn formula(&self) -> SourceAtomicFormulaId;
    pub const fn ordinal(&self) -> usize;
    pub const fn segment(&self) -> SourcePredicateSegmentId;
}
impl SourcePredicateChainConjunctionTable {
    pub fn get(
        &self,
        id: SourcePredicateChainConjunctionId,
    ) -> Option<&SourcePredicateChainConjunction>;
    pub fn iter(
        &self,
    ) -> impl Iterator<
        Item = (
            SourcePredicateChainConjunctionId,
            &SourcePredicateChainConjunction,
        ),
    >;
    pub const fn len(&self) -> usize;
    pub const fn is_empty(&self) -> bool;
}
impl SourcePredicateChainNegationTable {
    pub fn get(
        &self,
        id: SourcePredicateChainNegationId,
    ) -> Option<&SourcePredicateChainNegation>;
    pub fn iter(
        &self,
    ) -> impl Iterator<
        Item = (
            SourcePredicateChainNegationId,
            &SourcePredicateChainNegation,
        ),
    >;
    pub const fn len(&self) -> usize;
    pub const fn is_empty(&self) -> bool;
}
```

producerはinput、`SourcePrimaryTermHandoff`、
`SourceAtomicFormulaHandoff`、common `TypedArena`を受ける。Task-252
`3/0/3`、Task-256 `1/0/2/2/2/0/0/3/2`、same-symbol imported candidate 2件、
positive segment 0、exact negative `does not` segment 1、canonical root
spellingをreauthenticateする。conjunction 0はboundary edge 1をsegment 0 right/
segment 1 leftとしてreuseし、このexisting `PredicateChainBoundary`はprimary 1
をtargetする。negation 0はsegment 1だけをtargetする。lower row/resolver
provenanceはcopyしない。

stable headerは`source-predicate-chain-composition-debug-v1`で、module、
primary/atomic fingerprint、conjunction count/row、negation count/rowの順。
exact error signatureは次。

```rust
#[non_exhaustive]
pub enum SourcePredicateChainCompositionError {
    DependencyMismatch,
    InvalidConjunction {
        conjunction: SourcePredicateChainConjunctionId,
    },
    InvalidNegation {
        negation: SourcePredicateChainNegationId,
    },
    InvalidAggregate,
}
```

producerはどちらのrow validatorより先に両table cardinalityを検査する。
conjunctionまたはnegationのcardinalityがexactly oneでなければ
`InvalidAggregate`。exact `1/1`ならconjunction 0、次にnegation 0をvalidateし、
invalid fieldには対応するstrongly typed ID付き`InvalidConjunction`または
`InvalidNegation`を返す。

typed/resolved ownershipはoptional/one-shot/revalidated/clone-preservedで、
`source_predicate_chain_composition()`から参照する。dedicated errorは
`InvalidSourcePredicateChainComposition`。Task-257A composite、Task-257B
composition、Task-257C2 condition compositionと全installation orderで
reciprocally exclusive。optional handoff absent時のexisting B/C2 successful
fingerprint/debug byteは不変。

C3-after-A/B/C2は
`TypedAstError::InvalidSourcePredicateChainComposition`にfailする。reverse
3 orderは順に`TypedAstError::InvalidSourceCompositeFormula`、
`TypedAstError::InvalidSourceFormulaComposition`、
`TypedAstError::InvalidSourceConditionFormulaComposition`にfailする。全6 pathは
何もpublishせずbyte-identical stateを保持し、replay可能。typed/resolved
debugではC3 chunkはTask-252 source-term、Task-256 source-atomic-formula、
A/B/C2 slotの後、existing node/table section直前のfinal mutually exclusive
formula-owner slotを占める。

later implementationはexisting Task-257C1 fixtureをreuseし、sidecar
reference/noteとcovered trace row
`spec.en.checker.type_elaboration.source_predicate_chain_composition` 1件だけを
変更できる。このrowはrequired、stage `type_elaboration`、status `covered`、
coverage `pass`。canonical sourceは
`doc/design/mizar-checker/en/source_formula_composition.md` section
`Task 257C3 Frozen Predicate-Chain Composition`で、sole mapped testはexisting
Task-257C1 sidecar。同sidecarのexact ordered spec-reference setはexisting
`spec.en.checker.type_elaboration.source_predicate_chain_segment_payload`、
次に`spec.en.checker.type_elaboration.source_predicate_chain_composition`。
new rowはsyntax-free associationだけをcreditする。
predicate signature answer、overload selection、
conjunction/negation truth、formula fact/result、theorem acceptance、proof、
IR/VC、broader chainはdeferred。本documentation prerequisiteはexecutable
artifactを変更せず、baselineはplan `419/386`、type `252/240`、libraries
`332/361`、active `101/5/198/1`、runner production 29 paths / 34,064 lines。

## Task 257C3 implementation result

frozen third transactionをpublic dense conjunction/negation ID/table、
immutable input/handoff/producer/error surface、lower debug fingerprint 2件、
exact accessor、stable debug textとして実装した。validationは両lower
installation/exact profileをreauthenticateした後、cardinality、conjunction
row 0、negation row 0の順に検証する。coherent wrong lower profile、
stale arena/fingerprint、substituted row、全cardinality/row precedence
combinationはfrozen typed errorでfailしreplay可能。

runnerはexisting fixtureからexact `1/1` handoffだけをpublishする。
predicate token/candidate/resolver contribution/lower edge/truth/fact/
diagnostic/semantic resultをduplicate/inferしない。checker 3 tests /
runner 4 testsがcomplete contractをcoverし、single covered trace rowは
syntax-free associationだけをcreditする。

## Task 258B4A downstream statement consumer

Task 258B4AはTask-257B1 explicit-universal composite/composition pairの最初の
upper consumerである。lower tables、fingerprints、
`1/0/1/1/1/0/2` / `1/2` profiles、
`SourceFormulaRootOwnership::UnassignedStatement`はbyte-for-byte
unchanged。new statement constructorはこれらpublic handoffを
reauthenticateし、`SourceCompositeFormulaId::new(0)`だけをtargetにする。

paired typed installerはTask 258 ownerであり、本moduleのownerではない。
existing composite/formula-composition handoffとstatementを一緒にinstallするか
何もpublishしない。lower API、row、root ownership、debug grammar、truth、
binder semantics、coverage creditを本moduleへ追加しない。documentation
reviewは**NO FINDINGS**である。independent final qualityは全9 hard gatesを
capなし、valid `100/100`でPASSした。remainingはstaging、commit、
post-commit inventoryだけである。

## Task 258B4A implemented downstream statement consumer

upper producerはalready validated Task-257B1 composite/composition pairを
public checker handoff経由でconsumeする。lower
`1/0/1/1/1/0/2` / `1/2` rows、fingerprints、debug text、selector、
`UnassignedStatement` ownershipはunchangedである。B4Aはupper
statement/candidateから`Composite(0)`へのlinksとpaired Task-258
transactionだけを追加する。runner visibility editはexisting validated
lower outputを`crate::runner`内だけにexposeし、public APIまたはlower
behaviorを変更しない。

## Task 258B4B downstream statement consumer

B4Bはexisting Task-257B2 composite/composition pairをunchangedにconsume
する。Task 257 `8/6/1/1/1/7/9` plus Task-257B2 `8/0`、Task-252
`16/0/16`、Task-256 `8/0/0/0/0/0/0/16/16`、binding `2/1/4`、
wrapper 6件、same-family edge 7件、atomic edge 8件、bound use 0件、
`UnassignedStatement` root ownershipである。private double-LF selectorは
lower range、row、fingerprint、debug byte、active 166-byte behaviorを
変更しない。equivalent historical Task-257B2 shorthandは
`8/0/0/0/0/0/16/16`で、上記の追加zeroはcurrent explicit empty
predicate-segment slotである。

upper statement/candidateだけがroot `Composite(0)`をreferenceする。inner
biconditional/disjunction/conjunction/repetition rows、wrappers、
equalities、numeralsはupper ownership/semantic resultを得ない。runner
helperはB4Aのcrate-private seam経由ですでにavailableなので、B4Bではこの
lower ownerはrequired no-opである。

## Task 258B4C downstream statement consumer

B4Cはexisting Task-257B3 composite/composition handoffs、binding
`4/4/0`、Task 252 `6/6/0`、Task 256
`3/0/0/0/0/0/0/6/6`、Task 257 `3/0/1/3/3/2/6`、composition
`3/6`をconsumeする。`67..136`、`92..136`、`110..136`のlower
composite roots、binder segments、equality roots 3件、references 6件、
bound-use edges 6件はunchangedである。composite root 0は
`UnassignedStatement`のままで、upper statement/candidateだけがtargetに
する。

active 138-byte sourceはsidecar/trace rowによりlower-onlyとしてfrozenで
ある。B4Cは同じsourceにsecond final LFを加えたexact 139 bytes、SHA-256
`36e5a68a92451590644951838a9af8926212bd78f88d1f90563f12b650b161c1`
を使う。upper work前のseparate lower-stage prerequisiteはrunner
`type_elaboration/source_formula.rs`と
`runner/tests/type_elaboration/source_formula_composition.rs`だけを更新し、
Task-257B3 selectorがactive 138-byte hash
`cbfd7077713e8e9630900e349d5f579251c19fba55434acb62170ea1dd940237`と
private 139-byte hashをacceptし、同じtextのzero/three final LFをreject
するようにする。

production runner `source_formula_composition.rs`とchecker lower owners
2件はprerequisite/B4Cともにunchangedである。structural matchingを
broadenせず、row/fingerprint/debug byteを変更せず、active upper behaviorを
creditしない。truth、restriction discharge、existential witnesses、
capture、facts、theorem acceptance、proof、IRはdeferredである。

## Task 257C4A Fraenkel generator binding context

Task257Cはcanonical [C4A contract](../../task_contracts/ja/CHECKER-FRAENKEL-GENERATOR-BINDING-257C4A.md)
のcompleted standalone handoffを所有する。本moduleはbinding-context ID/row/table、use-position
ID/row/table、handoff、producer、errorだけをexposeする。producerはexact 277C structural composition、R2 resolver
collection、`TypedAst`をconsumeし、opaque version/domain-tagged snapshotでfull fieldとR2→277C→typed binder relationをrevalidateする。
3 use rowはactual resolver positionのnormalizationだけでterm/reference/captureを所有しない。default-denyでformula/type/sethood/evidence/verdict/diagnostic、Typed/Resolved install、production routeなし。C4Bはuse/capture map前にhandoffをconsumeする。
implemented ownerは7303 lines、SHA-256 `f6da763061479e74e7b8f39169ecad311bb9bf879e91e93824d9899798017abc`。4 exact checker tests、550-test package library、format、workspace Clippy、full workspace tests、両lint、metadata、implementation/test reviewsはPASS。independent source-doc/bilingual/boundary/final-quality reviewsは**NO FINDINGS**。final-qualityは全`9/9` hard gates PASS、valid uncapped `100/100`（`20/20/15/15/10/10/5/5`）。historical pre-commit exact staging/cached reviewはPASS。task-only commit/immediate post-commit proof/accepted fresh-inventory dispositionはlanguage-local [historical checkpoint](../../task_contracts/ja/CHECKER-FRAENKEL-GENERATOR-BINDING-257C4A.md#historical-immediate-post-implementation-checkpoint)でclosed。C4Bはunselectedでseparate post-closure docs prerequisite freezeが必要。Task277Bはnot-ready/zero creditのまま。

## Task 257C4B Fraenkel generator bound-use transport

Frozen [C4B contract](../../task_contracts/ja/CHECKER-FRAENKEL-GENERATOR-BOUND-USE-257C4B.md)は本moduleに独立syntax-free associationを追加する。producerはcompleted C4A handoffだけをconsumeし、opaque full snapshotをrevalidateして3 normalized positionをexisting C4A `BindingEnv`へmapする。Task252 term/reference、role copy、capture、formula/type/sethood/request/verdict/diagnostic/install/routeは作らない。

Exact public familyは`SourceFraenkelGeneratorBoundUseId`、`SourceFraenkelGeneratorBoundUse`、`SourceFraenkelGeneratorBoundUseTable`、`SourceFraenkelGeneratorBoundUseHandoff`、`#[non_exhaustive] SourceFraenkelGeneratorBoundUseError`、`SourceFraenkelGeneratorBoundUseProducer`。IDは`new(index: usize) -> Self`/`index(self) -> usize`だけ。

Row getterはexact `use_position() -> SourceFraenkelGeneratorUsePositionId`、`binding_context() -> SourceFraenkelGeneratorBindingContextId`、`resolver_use_index() -> usize`、`source_ordinal() -> usize`、`lookup_ordinal() -> usize`、`context() -> BindingContextId`、`binding() -> BindingId`。Tableは`get(id) -> Option<&SourceFraenkelGeneratorBoundUse>`、`iter() -> impl Iterator<Item = (SourceFraenkelGeneratorBoundUseId, &SourceFraenkelGeneratorBoundUse)>`、`len() -> usize`、`is_empty() -> bool`。

Handoff getterはexact `source_id() -> SourceId`、`module_id() -> &ModuleId`、`dependency_summary() -> &str`、`bound_uses() -> &SourceFraenkelGeneratorBoundUseTable`、`debug_text() -> String`。summaryはexact C4A `source-fraenkel-generator-binding-context-v1|module=<package>.<path>|bindings=1|use-positions=3`でnon-authoritative。C4B debugは`source-fraenkel-generator-bound-use-v1|module=<package>.<path>|bound-uses=3`。

Producer signatureはexact `SourceFraenkelGeneratorBoundUseProducer::build(binding_context: &SourceFraenkelGeneratorBindingContextHandoff) -> Result<SourceFraenkelGeneratorBoundUseHandoff, SourceFraenkelGeneratorBoundUseError>`。public lower getterなし。private snapshot version/domainは`source-fraenkel-generator-bound-use-dependency-v1` / `source-fraenkel-generator-bound-use`。wrapper environment、version/domain、C4A full validation、summary、dense C4B rowの順にvalidateし、C4A validationが全R2/277C/TypedAst/binding/use-position/BindingEnv fieldをtransitive authenticateする。

Exact error/precedenceは`EnvironmentMismatch`、`InvalidBindingContextDependency`、`InvalidBoundUse { bound_use: SourceFraenkelGeneratorBoundUseId }`。wrapper source/module mismatch、snapshot/C4A/summary failure、最後にlowest invalid rowの順。wrong total countはID0。lower error/raw resolver identityをexposeしない。

F5 row 0/1/2はC4A use position 0/1/2、binding-context0、resolver/source ordinal 0/1/2、lookup 1/2/3、context1、binding0で、全lookupが`Local(binding0)`。ordinal0はC4A `ForwardReference` probeでC4B rowではない。range/spellingからordinalをinferしない。default-denyでmissing/extra/reorder/duplicate/recovery/stale/non-local/nested/multiple/shadowをatomic rejectする。contractの4 exact checker testsがABI/oracle、full snapshot corruption、row/lookup+precedence、deterministic non-mutationをcoverし、raw listは`550 -> 554`。

Implementationは本existing ownerでcompleteし、7958 lines / SHA-256 `90b339d9707f9f8d847b678721e8db0ef6a00e4a15dbb41474a0cf6980f47168`。上記exact family/getter、full retained C4A validation、three row、exact 4 checker testを実装した。Focused `4/4`、checker `554/554`、format、package/full-workspace Clippy、full workspace tests、metadata/public-enum suites、unchanged 5 CLI replay、diff checkはPASS。Structural/`TypedAst` dependency corruptionをexisting precedence testへ追加した後、implementation/test-sufficiency reviewは**NO FINDINGS**。sole Low baseline/current wording repair後、final source-doc reviewとindependent bilingual/boundary reviewも**NO FINDINGS**。final-qualityも**NO FINDINGS**、全`9/9` hard gate PASS、valid uncapped `100/100`（`20/20/15/15/10/10/5/5`）。contractのexact 23-path staging/cached reviewもPASS。task-only commit/immediate post-commit proof/accepted fresh semantic STOPはlanguage-local [historical checkpoint](../../task_contracts/ja/CHECKER-FRAENKEL-GENERATOR-BOUND-USE-257C4B.md#historical-immediate-post-implementation-checkpoint)でclosedし、successorはselectしない。

## Task 257C4C3 nested Fraenkel binder/mapper-use transport

Frozen [C4C3 contract](../../task_contracts/ja/CHECKER-FRAENKEL-NESTED-BINDER-USE-257C4C3.md)は
existing Task257C ownerへ独立zero-semantic transactionを追加する。Completed C4C2 resolver collectionと
exact normal `TypedAst`だけをconsumeし、exact-F5 C4A/C4B API/validatorをreuseまたはweakeningしない。

Frozen implementationがexposeするexact public familyは`SourceNestedFraenkelBinderUseId`、
`SourceNestedFraenkelBinderUse`、`SourceNestedFraenkelBinderUseTable`、
`SourceNestedFraenkelBinderUseHandoff`、non-exhaustive `SourceNestedFraenkelBinderUseError`、
`SourceNestedFraenkelBinderUseProducer`。Rowはresolver use index/binding ID、outer binder
`TypedNodeId`、inner mapper-use `TypedNodeId`、source ordinalだけ。Tableはdense read-only
`get`/`iter`/`len`/`is_empty`、handoffはsource/module/resolver summary/table/deterministic debugだけを公開する。

Resolver summaryはexact
`fraenkel-generator-variable-source-v1|module=<package>.<path>|bindings=2|uses=1`でnon-authoritative。
Exact handoff debug grammarは
`source-nested-fraenkel-binder-use-v1|module=<package>.<path>|binder-uses=1`。
Non-exhaustive errorはexact `EnvironmentMismatch`、`InvalidResolverDependency`、
`InvalidTypedDependency`、`InvalidBinderUse { binder_use: SourceNestedFraenkelBinderUseId }`だけを
このprecedenceで持つ予定である。

Producerはretained C4C2 exact `2 binding / 1 use`、unique normal Resolved→Typed mapping、typed structure/rangeを
full authenticateし、row0 `mapper use0 x@94..95 -> outer binding1 x@136..137`だけをpublishする予定である。
Task252 occurrence、`BindingEnv`/checker binding、capture、formula、type/sethood、request/result、verdict、
diagnostic、install、route、coverage creditは作らない予定である。Validation precedence/snapshot/default-deny/test/baselineは
completion evidenceまでcontractがownする。
