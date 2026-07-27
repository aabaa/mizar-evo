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
| `SourceFormulaAtomicEdgeRole` | `#[non_exhaustive]`。callerはlater frozen cross-family body roleを許容する。 |
| `SourceFormulaCompositionError` | `#[non_exhaustive]`。callerはvalidation failureをexhaustive matchしない。 |

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
