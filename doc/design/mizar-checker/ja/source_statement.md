# Source Statement Transport

> canonical languageはEnglish。英語版:
> [../en/source_statement.md](../en/source_statement.md)。

このcomponentはsource theorem owner、statement shell、visibility-scoped
input、未検証candidate propositionのsyntax-free checker boundaryである。
statement parse、label resolve、formula proof、theorem fact publicationは行わない。

## Task 258A Frozen Reserved-Variable Theorem Slice

Task 258AはChecker Task 258の最初のbounded sliceである。exact future
`MT10-FS` sourceは次。

```mizar
reserve x for set;
theorem FormulaStatementReservedVariableEqualitySmoke: x = x;
```

final LF込みexactly 81 UTF-8 bytes、SHA-256は
`341aad596ef6891dfa33c189895df2350d357ac8edaf3747f160bbad7a2ddd96`。
normal rangeはreserve `0..18`、written `set` `14..17`、theorem/owner
`19..80`、label `27..72`、equality/formula `74..79`、left `x`
`74..75`、`=` `76..77`、right `x` `78..79`。

authorityはChapter 4 §§4.3/4.7.1、Chapter 14 §14.5.2、Chapter 15
§§15.8/15.10、Chapter 16 §§16.1/16.2/16.7.1/16.9。Chapter 4により
free reserved theorem identifierはreserved type付きimplicit universal
closureとなる。Chapter 16はnamed theorem ownerを要求し、unmodified itemの
omitted justificationを許すが、automatic proof/publicationはverification後だけ。
従ってTask 258Aはreserved type guardをvisible input、equalityをunverified
candidateとしてtransportし、equality truth/theorem acceptanceを主張しない。

### Exact lower inputsとresolver provenance

private extractorはreal final-LF frontend ASTとreal resolver `SymbolEnv`を
consumeする。exactly one normal reserveの後に、exact labelとdirect
unparenthesized reserved-variable equalityを持つnormal unmodified theoremが
1件だけある形を選択する。lower profileは次。

- Task-48由来normal module `BindingEnv`: context 0、active/visibleな
  `ReservedVariable` binding 0 (`x`)、reserve item `0..18`内のidentifier
  declaration `8..9`、type site `14..17`、first-use ordinal 1。
- Task 252 `2/2/0`: 2件の`VariableReference`/`Value` primaryとbinding 0へ
  independentにauthenticateしたTask-252 stored use ordinal 1/1の
  reference 2件。
- Task 256 `1/0/0/0/0/0/2/2`: normal `Equality` 1件、Task-252 primaries
  0/1へのoperand edge 2件、unresolved expected-type request 2件。
- theoremがdirect `FormulaExpression` wrapperをcontainし、そのwrapperが
  atomic formula occurrenceを、formula subtreeがleft-to-right orderで両term
  occurrenceをcontainするshared typed arena。

Task 248はcanonical binding/context modelを供給するが、現行exact
`SourceBindingContextHandoff` profileはreserve-plus-theoremを表せないため、
ここでfabricate/extendしない。Task 249/253–255はabsent。exact formulaは
atomicなのでTask 257 formula-owner handoffもabsent。

ownerは全resolver viewでauthenticateする。current source/moduleのlocal
source theorem symbolはexactly one。
`CheckedStatementOwner::validate_exact_local_theorem`がacceptする。
`SymbolEntry`/`DefinitionEntry`/`LabelEntry`/checked ownerは
source/module/contribution/normal theorem origin `19..80`/spelling/
visibility/exportで一致する。runnerのexact label selector/projection rangeは
`27..72`であり、published `LabelEntry`は別のdeclaration-range fieldではなく
shared theorem originを保持する。
module-wide `SourceContribution`はtheorem-local contributionをfabricateせず、
real resolverのfirst declaration shellであるreserve `0..18`をanchorにする。
kindは`SymbolKind::Theorem`/`DefinitionKind::Theorem`/
`LabelKind::Theorem`、visibility/exportは`Public`/`Exported`、
contributionは`LocalSource`でtheorem symbol/definition/label effectを含み
import edgeを持たない。recovered/imported/summary/missing/duplicate/stale/
cross-contribution/wrong-kind/private/local-only/source mismatchはpublication前に
failする。

### Frozen syntax-free API

later implementationはpublic `source_statement` moduleを追加し、dense ID/table
は`SourceTheoremOwnerId`、`SourceStatementId`、
`SourceStatementContextId`、`SourceStatementInputFactId`、
`SourceStatementCandidateFactId`の5系統とする。input/handoff/row/producer/
errorのfieldとsignatureはcanonical English sectionのRust contractと
token-for-token同じidentifierを使う。

input rowは次を保持する。

- owner: resolver `SymbolId`/`SourceContributionId`、site/range/spelling、
  `Theorem`、`Unmodified`、recovery。
- statement: owner/context、`Atomic(SourceAtomicFormulaId)`、site/range/
  ordinal/spelling、`TheoremProposition`、recovery。
- context: statement、`BindingContextId`、range、visible binding vector。
- input fact: statement/context/ordinal、`ReservedTypeGuard`、binding、
  Task-252 reference use vector。
- candidate fact: statement/context/ordinal、`UnverifiedProposition`、
  atomic formula target。

全public enumは`#[non_exhaustive]`。dense IDは`new/index`、tableは
`get/iter/len/is_empty`だけ、immutable rowはread-only accessorだけを公開。
handoffはproducer-validated `BindingEnv`のexact cloneをownし、
`binding_env()`で公開する。さらにsource/module、quoted
`binding_env.debug_text()` fingerprint、Task-252/256 fingerprint、checked
owner、5 table、deterministic `debug_text()`を公開する。owned environmentは
equality/clone preservationの一部で、mutable/unchecked constructorを持たない。

exact output typeは`SourceTheoremOwner`、`SourceStatement`、
`SourceStatementContext`、`SourceStatementInputFact`、
`SourceStatementCandidateFact`と対応する5 table。owner rowは
symbol/contribution/site/range/spelling/role/status/recovery、statement rowは
owner/context/formula/site/range/source ordinal/spelling/kind/recovery、
context rowはstatement/binding context/range/visible bindings、input rowは
statement/context/ordinal/kind/binding/uses、candidate rowは
statement/context/ordinal/kind/formulaのread-only accessorだけを持つ。各tableの
exact `get/iter/len/is_empty`、handoffの
`source_id/module_id/binding_env/binding_fingerprint/
primary_term_fingerprint/atomic_formula_fingerprint/checked_owner/owners/
statements/contexts/input_facts/candidate_facts/debug_text`、private
`validate_installation(source_id, module_id, primary_terms, atomic_formulas,
arena)` signatureはcanonical EnglishのRust blockと同じ。

5 IDは`Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash`。
input/immutable row/table/handoffは`Debug, Clone, PartialEq, Eq`、7 data
enumは`Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash`。
`SourceStatementError`は`Debug, Clone, PartialEq, Eq`と`Display`/`Error`、
`SourceStatementProducer`は`Debug, Clone, Copy, Default`。Task 258Aに他public
trait/construction surfaceはない。

`SourceStatementProducer::build`はcomplete inputと`&SymbolEnv`、
`&BindingEnv`、`&SourcePrimaryTermHandoff`、
`&SourceAtomicFormulaHandoff`、`&TypedArena`を受け、transaction全体を
validateしてからimmutable handoffを返す。errorはnon-exhaustiveで
`DependencyMismatch`、owner/statement/context/input-fact/candidate-factの
strongly typed row-local variant、`InvalidAggregate`だけ。

### Exact `1/1/1/1/1` transaction

5 vectorは各exactly one rowでID/ordinal 0。owner 0はauthenticated theorem
label。statement 0は`19..80`、normal、exact single-space token spelling、
`TheoremProposition`、owner/context 0、
`Atomic(SourceAtomicFormulaId::new(0))`。formulaはTask-256 equality
`74..79`で、theorem arena nodeは別statement/proof/justification subtreeなしに
direct formula-expression wrapperをcontainし、そのwrapperがatomic occurrenceと
ordered term descendantをcontainする。

context 0はtheorem range、binding context 0、visible bindings exact `[0]`。
binding environmentはnormal、same source/module、diagnosticなし。input fact 0は
`ReservedTypeGuard`、binding 0、reference uses exact `[0, 1]`でcontext 0に
visible。両referenceはactive normal reserved binding 0をTask 252で
independentに選び、binding declaration/type siteはtheorem/usesより前。

candidate fact 0は`UnverifiedProposition`でstatement/context 0とsame atomic
equalityを指す。input factではなく、statement自身やlater statementにvisible
でなく、`TypeFactId`、checked formula、theorem result、axiom、accepted
premise、discharged goalでもない。

stable debug schemaはcanonical Englishの
`source-statement-debug-v1` blockと同じ。全validation entry pointは同じ
total precedenceを使う。最初にそのentry pointで利用可能なsource/module、
owned `BindingEnv`、Task-252/256 handoff、exact profile/stored fingerprint、
shared arena、required lower installationをauthenticateし、failureは
`DependencyMismatch`。次にrowを検査する前に5 input/tableすべての
cardinalityとdense aggregate orderを検査し、missing/extra/duplicate/
reordered aggregateは`InvalidAggregate`。最後にowner、statement、context、
input fact、candidate fact順のfirst invalid rowへstrongly typed row-local
errorを返す。

live `SymbolEnv`を受けるのは`SourceStatementProducer::build`だけ。その
owner-row tierでresolver theorem identity、contribution membership、label
effect、range、spelling、visibility/export、kind/origin、
`CheckedStatementOwner::validate_exact_local_theorem`をowner row 0のproperty
として検査し、failureは`InvalidOwner { owner: 0 }`で
`DependencyMismatch`ではない。`validate_installation`、typed-AST
installation、final assemblyはresolver viewを再照会せず、dependency/
aggregate tier後にimmutable stored `CheckedStatementOwner`とowner row 0の
整合、その後remaining rowをsame orderで検査する。従ってmixed
dependency/cardinality corruptionは常に`DependencyMismatch`、mixed
aggregate/row corruptionは常に`InvalidAggregate`。frozen inputを超える
resolver reauthenticationは主張しない。その他のcopied/substituted/stale/
recovered/wrong source/module/range/spelling/site/ordinal/kind/status/owner/
contribution/fingerprint/binding/reference/formula target、input/candidate
aliasもpublication前にfailする。

### Typed/final ownershipとexclusion

`TypedAst`/`ResolvedTypedAst`はoptional `SourceStatementHandoff`、accessor、
deterministic debug、revalidation、dedicated `InvalidSourceStatement`を追加。
installationはone-shotでexact Task-252/256 handoffが先に必要。
final assembleはsame objectをrevalidate/clone-preserveし、rowを再構築しない。
debug chunkは全Task-257 formula-owner chunkの後、existing node/table sectionの
前。failureはfieldをpublishせずbyte-identical state/replayを保持する。

handoff-owned `BindingEnv`はproducer inputからcloneしたexact object。
installation/final assemblyはsource/module、`binding-env-debug-v1`
fingerprint、normal module context、exact active reserved binding、
visibility、declaration/type range、first-use ordinal、diagnostic absence、
Task-252 reference winner、statement-context useをrevalidateする。従ってpublic
installer/final assemblyへ`BindingEnv` parameterを追加せずstale/substituted
binding provenanceを観測できる。

Task 248/Task 258Aはmutually exclusive。唯一のproduction orderはconstructor-
supplied Task-248 `SourceBindingContextHandoff`後の
`with_source_statement`で、`TypedAstError::InvalidSourceStatement`。
Task 248にpost-construction installerはなくTask 258Aも追加しない。reverse
logical attemptはspecifically named `#[cfg(test)]`
`with_source_context_for_test`でsame private typed-AST validationを実行し、
`TypedAstError::InvalidSourceContext`。final coexistenceはspecifically named
`#[cfg(test)]` `inject_source_statement_for_test`だけでprepareし、
`ResolvedTypedAstError::InvalidSourceStatement`。全3 failureはsecond owner/
final outputをpublishせずprior debug bytesを保ちvalid owner replayを許す。
exact profileはTask 248をinstallしない。test-only signatureはcanonical
English Rust blockと同じ。

Task 258Aは`TypedAst::facts`、checked formulas、existing Task-266
`statement_semantics`、Task-268 checked proof/node/terminal goal、cluster fact、
diagnostic、CoreIr/ControlFlowIr/VC/cache/artifactをemptyのままにする。
Task-266/268 standalone contradiction routeがexisting checked tableをsole own
する。new `StatementSemanticInput`、`StatementProofIntentInput`、
`CheckedStatementOwner` constructor、accepted-status pathは追加しない。

selectorはlemma/role alias/status modifier/justification/proof block、
proof-local statement/assumption/conclusion/witness/citation/local label、
composite/other atomic/parenthesized formula、multiple reserve/theorem、
shadow/import/definition、exact bytesを変えるcomment、missing final LF、
recovered/synthetic-only AST、全named near missをexcludeする。broader familyは
separate Task 258BまたはTasks 269–272。

## Public Enum Policy

| Public enum | Compatibility policy |
|---|---|
| `SourceTheoremRole` | `#[non_exhaustive]`; Task 258Aは`Theorem`だけをaccept。 |
| `SourceTheoremStatus` | `#[non_exhaustive]`; Task 258Aは`Unmodified`だけをaccept。 |
| `SourceStatementKind` | `#[non_exhaustive]`; Task 258Aは`TheoremProposition`、Task 258B1はexactな`ProofStepProposition`と`Conclusion` row、Task 258B2はexactなunlabeled `Assumption` row 1件もaccept。 |
| `SourceStatementRecovery` | `#[non_exhaustive]`; callerは`Degraded`を許容し、exact routeは`Normal`だけをaccept。 |
| `SourceStatementFormulaTarget` | `#[non_exhaustive]`; Task 258AはTask-256 `Atomic` target 1件だけをaccept。 |
| `SourceStatementInputFactKind` | `#[non_exhaustive]`; Task 258Aは`ReservedTypeGuard`だけをaccept。 |
| `SourceStatementCandidateFactKind` | `#[non_exhaustive]`; Task 258Aは`UnverifiedProposition`だけをaccept。 |
| `SourceStatementWitnessTermTarget` | `#[non_exhaustive]`; Task 258B3はexactなTask-252 `Primary` term 2だけをaccept。 |
| `SourceStatementWitnessKind` | `#[non_exhaustive]`; Task 258B3は`Unnamed` witness 1件だけをaccept。 |
| `SourceStatementLabelKind` | `#[non_exhaustive]`; Task 258B1はresolver-authenticatedな`ProofStep` label 1件だけをaccept。 |
| `SourceStatementCitationKind` | `#[non_exhaustive]`; Task 258B1は`SimpleLocal` backward citation 1件だけをaccept。 |
| `SourceStatementError` | `#[non_exhaustive]`; callerはproducer/installation failureをexhaustive matchしない。 |
| `SourceStatementReferenceError` | `#[non_exhaustive]`; callerはreference dependency、aggregate、label、citation failureをexhaustive matchしない。 |
| `SourceStatementWitnessError` | `#[non_exhaustive]`; callerはwitness dependency、aggregate、row failureをexhaustive matchしない。 |

この module が所有する exhaustive public enum exception はない。

### Tests、traceability、exit

checker testsはcomplete API/debug、resolver/lower profile、Task-252 reference
ID 0/1と独立したstored use ordinal 1/1、全row-local error/aggregate
precedenceをcoverする。どちらかのstored ordinalを1以外へ変えると
`DependencyMismatch`となるdependency mutationで、no publication、prior
debug byte preservation、valid replayを要求する。さらにowned-binding
substitution、production Task-248-first rejection、named reverse test-only
validation seam、injected final coexistence、install/final clone、
rollback/replay、empty semantic tableをcover。
runner testsはexact 81-byte sourceをreal frontend/resolverでparse/
resolveし、private dormant `MT10-FS` route、range/provenance/profile、
measured left/right Task-252 stored use ordinal sequence 1/1と独立した
upstream binding/use source-event lookup sequence 1/2、final ownership、
loaded-source/final-LF/named/recovered/subtree/lower near miss、active
type-elaboration isolationをcoverする。targetはchecker 3 tests、
mizar-test library 4 tests。

implementationはfuture `.miz`/sidecarを追加せず、existing
`pass_type_elaboration_reserved_variable_equality_001`をreclassify/modifyせず、
trace rowを追加/coverしない。existing deferred
`spec.en.checker.formula_statement.source_payloads`はTask 258、Tasks 269–272、
`MT10-FS`完了までempty test list/deferredのまま。

Task 258Aはsyntax-free transaction、real parser/resolver test route、
typed/resolved ownership、exact empty-semantic boundary、reviews/verificationで
completeするが、Task 258 umbrellaはcloseしない。explicit assumption/
conclusion/witness、local label/citation、composite root、nested context、
broader visibilityはseparately frozen Task 258Bに残す。

## Task 258A implementation result

frozen language/test intentを拡張せずtransactionを実装した。
`SourceStatementProducer`はexact five dense rowsだけをpublishし、validated
binding environmentをownする。current module namespace、symbol、definition、
label、contribution、checked-owner viewを横断してtheoremをauthenticateする。
installationはTask-252/256 debug fingerprint、stored reference-use ordinal
2件、arena topology、direct formula-expression wrapper、excluded descendantを
再検証する。

typed installationは全existing semantic tableをrejectする。final assemblyも
output構築前にcluster/overload/expression/statement-semantic/proof/diagnostic
coexistenceをrejectする。node hintはempty、またはsole role
`source.statement.transport`のcomplete dense source-preserved setだけをaccept
する。このhintはsyntax nodeをpreserveするだけでsemantic factを生成しない。
Task-248-first、named reverse test-only、injected final coexistenceはいずれも
atomicにfailし、valid replayを保持する。

checkerはexactly Task-258A compound tests 3件、dormant real runner routeは
exactly 4件。fixture/sidecar/expectation/trace row/status/active countは不変。
broader statement shapeと全semantic acceptance/proof decisionはTask 258Bまたは
Tasks 269–272に残る。

## Task 258B decomposition

fresh post-Task-258A inventoryでは、original Task 258B umbrellaは安全な1
logical taskではない。explicit assumption/witnessにはnew statement payloadと
後続proof-local binding ownerが必要で、composite theorem rootはTask-257
cross-family composition、imported/nested visibilityは別resolver profileを
必要とする。全部を結合するとsource transportとTasks 269–272 semanticsが
混在する。

そこでTask 258Bを次のように分割する。

1. **Task 258B1**は下でfreezeするexact theoremについてnested proof
   context、labeled local proposition、explicit conclusion shell 2件、
   resolver-authenticated local citation 1件だけをtransportする。accepted
   factをpublishせずproof/justification semanticsを実行しない。
2. **Task 258B2+**はexplicit assumption/witness、composite theorem root、
   broader imported/outer/inner visibility profileを保持し、それぞれlater
   separate frozen contractを要求する。
3. Tasks 269–272はproof-local declaration/binding、closure/capture/
   substitution、`reconsider` intent、proof skeleton decomposition、
   justification selection、proof resultを引き続きownする。

このdecompositionは`design_drift` closureでありlanguage changeではない。

## Task 258B1 frozen nested-conclusion/local-citation slice

### Authority、exact source、lower-stage evidence

canonical authorityはreserved variable/type guardについてChapter 4
§§4.3, 4.6, 4.7.1、equalityについてChapter 14 §14.5.2、direct
conclusion/full statement proof/label/citation/scopeについてChapter 15
§§15.4.1, 15.8.1–15.8.2, 15.10, 15.12、theorem owner/proof block/
proof-step visibility/later proof semanticsについてChapter 16
§§16.1–16.2, 16.4.1–16.4.2, 16.5.1, 16.7.1–16.7.3, 16.8, 16.9。
existing `pass_parser_theorems_proofs_001.miz`、
`fail_type_elaboration_statement_proof_gap_001.miz`、parser tests、
resolver `labels` testsはunchanged lower-stage oracle。

private dormant consumerはexact 139-byte final-LF sourceだけをacceptする。

```mizar
reserve x for set;
theorem FormulaStatementNestedContextSmoke: x = x proof
  A: x = x proof
    thus x = x;
  end;
  thus x = x by A;
end;
```

SHA-256は
`e5b87121e97e4ec4160b0189eff598d05f3ed5193698238226461f00593a907b`。
fresh real-frontend inventoryはnormal root `0..138`と次のhalf-open rangeを
測定した。

| Occurrence | Range |
|---|---:|
| reserve item / type | `0..18` / `14..17` |
| theorem owner / label / theorem equality | `19..138` / `27..61` / `63..68` |
| outer proof block | `69..137` |
| labeled compact statement / label / equality | `77..114` / `77..78` / `80..85` |
| nested proof block | `86..113` |
| nested conclusion / equality | `96..107` / `101..106` |
| outer conclusion / equality | `117..133` / `122..127` |
| simple justification / citation `A` | `128..132` / `131..132` |
| outer `end;` | `134..138` |

equality operand pair 4件は`63..64`/`67..68`、`80..81`/`84..85`、
`101..102`/`105..106`、`122..123`/`126..127`。parser shapeはreserve
1件、unmodified theorem 1件、direct theorem `FormulaExpression` 1件、
outer `ProofBlock` 1件、direct formulaとnested `ProofBlock`各1件を持つ
labeled `CompactStatement`、nested `ConclusionStatement` 1件、後続outer
`ConclusionStatement` 1件（`JustificationClause`とsimple `Reference`各1件）。
全nodeはnormal。

ordinary declaration/symbol collectionはpublic/exported local theorem
symbol/definition各1件と、source anchorがreserve `0..18`の`LocalSource`
contributionを提供する。Task 258Aと同様にprivate runnerがtheorem label
projectionを構築・authenticateする。normal symbol environmentがproof-step
label indexを持たないのはintentional。Task 258B1はexisting public resolver
boundary、すなわちexact parser-backed `ResolvedAst` 1件、
`LabelProjection::proof_step` 1件、
`LabelReferenceCandidate::unqualified_citation` 1件、
`LabelResolver::resolve`を使用する。label `A`は`Private`/`LocalOnly`、
`LabelKind::ProofStep`、outer proof scope `[0]`でstatement ordinal 1より
後だけvisible。citationはsame scopeのstatement ordinal 3でそのoriginへ
resolveする。nested proof scopeは`[0, 0]`。missing/ambiguous/forward/
sibling/inner-to-outer substitution/theorem-kind/imported/recovered/
cross-source/module/contribution/wrong-range/spelling provenanceをrejectする。

### Frozen lower profileとbase statement transaction

lower dependency graphは次のとおり。

```text
Task 48 reserve x:set base 1 context / 1 binding / 0 diagnostics
  -> Task 258B1 proof-context extension 3 contexts / 1 binding / 0 diagnostics
  -> Task 252 primary/reference/numeric 8/8/0
  -> Task 256 atomic/wrapper/segment/head/candidate/type/attribute/edge/request
     4/0/0/0/0/0/0/8/8
  -> Task 258 base owner/statement/context/input/candidate 1/4/4/4/4
  -> Task 258B1 label/citation composition 1/1
```

`BindingContextOwner`へnon-exhaustive
`SourceStatement { source_range: SourceRange }` variant 1件を追加する。
context 0はreserved binding 0を持つunchanged module context。context 1は
`SourceStatement { 69..137 }`、parent 0、layer `Proof`。context 2は
`SourceStatement { 86..113 }`、parent 1、layer `Proof`。両方ともbindingを
ownせずvisible bindings `[0]`を保持する。context 1のlexical scopeは`[0]`、
context 2は`[0,0]`、module context 0は`None`を保持し、3件ともnormal
recovery。environmentはexact `3/1/0`で、proof-local variable/capture/
diagnosticを追加しない。
`BindingEnv::try_new`は各`SourceStatement` rangeがnonemptyかつenvironment
source由来であることを要求し、exact
`source-statement(<start>..<end>)`としてrenderする。pre-B1 binding-
environment debug byteはすべて不変。

Task 252はsource orderの`VariableReference`/`Value` 8 rowsとreserved
binding 0へのreference 8件をownする。binding contextは
`0,0,1,1,2,2,1,1`、producer-stored reference use ordinalはすべて1。
Task 256はcontext `0,1,2,1`のnormal `Equality` 4 rows、rowごとのbuiltin
operand edge 2件とunresolved operand-expected-type request 2件をownする。
Task-248 context handoff、Task-249/253–257 handoffはinstallしない。

existing five input vector/public APIはsource-compatibleのまま。
`SourceStatementProducer::build`はvalidated row/dependencyからTask-258A
`1/1/1/1/1`とTask-258B1 `1/4/4/4/4`をrecognizeし、caller profile flagを
追加しない。Task 258B1が追加するpublic enum variantは次だけ。

```rust
pub enum BindingContextOwner {
    SourceStatement { source_range: SourceRange },
    // existing variants remain unchanged
}

pub enum SourceStatementKind {
    TheoremProposition,
    ProofStepProposition,
    Conclusion,
}
```

existing base-producer/installation precedenceは両profileで不変。source/
module、binding environment、Task-252/256 fingerprint、shared arena、
required lower installationが最初に`DependencyMismatch`、profile
cardinality/dense aggregate orderが次に`InvalidAggregate`、その後first
invalid owner、statement、context、input-fact、candidate-fact rowの順でfail
する。mixed dependency/cardinality corruptionはdependency-first、mixed
aggregate/row corruptionはaggregate-first。

statement 4 rowsはsource preorder。`Spelling`はbase handoffがstore/
validate/printするexact single-space token rendering。

| Row | Kind | Context row / binding context | Range | Formula | Spelling |
|---:|---|---:|---:|---:|---|
| 0 | `TheoremProposition` | `0 / 0` | `19..138` | atomic 0 | `theorem FormulaStatementNestedContextSmoke : x = x proof A : x = x proof thus x = x ; end ; thus x = x by A ; end ;` |
| 1 | `ProofStepProposition` | `1 / 1` | `77..114` | atomic 1 | `A : x = x proof thus x = x ; end ;` |
| 2 | `Conclusion` | `2 / 2` | `96..107` | atomic 2 | `thus x = x ;` |
| 3 | `Conclusion` | `3 / 1` | `117..133` | atomic 3 | `thus x = x by A ;` |

全rowはowner 0、normal recovery、exact source ordinal `0..3`、direct parser
formula pathを使用する。context rowはsame statement rangeとvisible binding
`[0]`。input fact row `i`はstatement/context `i`、binding 0、ordinal 0、
Task-252 reference pair `[2i, 2i+1]`の`ReservedTypeGuard`。candidate row
`i`はstatement/context `i`、ordinal 0、atomic formula `i`の
`UnverifiedProposition`。source label/citationがあってもcandidateはvisible/
acceptedにならない。

owner 0はexact theorem site/range `19..138`、label range `27..61`、
spelling `FormulaStatementNestedContextSmoke`、role `Theorem`、status
`Unmodified`、normal recovery。symbol/contributionはordinary Task-258A
owner pathのsole authenticated current-module public/exported theorem
declarationであり、B1 branchはsecond owner/contribution/theorem label/
source anchorをinvent/substituteしない。

### Frozen local-label/citation composition API

Task 258B1はTask-258A debug byteを変更せず、existing public
`source_statement` moduleへdense table 2件を追加する。

```rust
pub struct SourceStatementLabelId(usize);
pub struct SourceStatementCitationId(usize);

pub struct SourceStatementReferenceHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub labels: Vec<SourceStatementLabelInput>,
    pub citations: Vec<SourceStatementCitationInput>,
}

pub struct SourceStatementLabelInput {
    pub statement: SourceStatementId,
    pub context: SourceStatementContextId,
    pub candidate: SourceStatementCandidateFactId,
    pub origin_path: LabelOriginPath,
    pub proof_scope: LabelScopePath,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub visible_after_ordinal: usize,
    pub spelling: String,
    pub kind: SourceStatementLabelKind,
    pub recovery: SourceStatementRecovery,
}

pub struct SourceStatementCitationInput {
    pub statement: SourceStatementId,
    pub context: SourceStatementContextId,
    pub label: SourceStatementLabelId,
    pub label_ref: LabelRefId,
    pub proof_scope: LabelScopePath,
    pub source_range: SourceRange,
    pub ordinal: usize,
    pub kind: SourceStatementCitationKind,
    pub recovery: SourceStatementRecovery,
}

pub struct SourceStatementLabel { /* immutable validated label fields */ }
impl SourceStatementLabel {
    pub const fn statement(&self) -> SourceStatementId;
    pub const fn context(&self) -> SourceStatementContextId;
    pub const fn candidate(&self) -> SourceStatementCandidateFactId;
    pub const fn origin_path(&self) -> &LabelOriginPath;
    pub const fn proof_scope(&self) -> &LabelScopePath;
    pub const fn source_range(&self) -> SourceRange;
    pub const fn source_ordinal(&self) -> usize;
    pub const fn visible_after_ordinal(&self) -> usize;
    pub fn spelling(&self) -> &str;
    pub const fn kind(&self) -> SourceStatementLabelKind;
    pub const fn recovery(&self) -> SourceStatementRecovery;
}

pub struct SourceStatementCitation { /* immutable validated citation fields */ }
impl SourceStatementCitation {
    pub const fn statement(&self) -> SourceStatementId;
    pub const fn context(&self) -> SourceStatementContextId;
    pub const fn label(&self) -> SourceStatementLabelId;
    pub const fn label_ref(&self) -> LabelRefId;
    pub const fn proof_scope(&self) -> &LabelScopePath;
    pub const fn source_range(&self) -> SourceRange;
    pub const fn ordinal(&self) -> usize;
    pub const fn kind(&self) -> SourceStatementCitationKind;
    pub const fn recovery(&self) -> SourceStatementRecovery;
}

pub struct SourceStatementLabelTable { /* dense source-order rows */ }
impl SourceStatementLabelTable {
    pub fn get(&self, id: SourceStatementLabelId) -> Option<&SourceStatementLabel>;
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (SourceStatementLabelId, &SourceStatementLabel)>;
    pub const fn len(&self) -> usize;
    pub const fn is_empty(&self) -> bool;
}

pub struct SourceStatementCitationTable { /* dense source-order rows */ }
impl SourceStatementCitationTable {
    pub fn get(
        &self,
        id: SourceStatementCitationId,
    ) -> Option<&SourceStatementCitation>;
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (SourceStatementCitationId, &SourceStatementCitation)>;
    pub const fn len(&self) -> usize;
    pub const fn is_empty(&self) -> bool;
}

#[non_exhaustive]
pub enum SourceStatementLabelKind {
    ProofStep,
}

#[non_exhaustive]
pub enum SourceStatementCitationKind {
    SimpleLocal,
}
```

両IDはexisting dense-ID deriveと`new`/`index` accessorを持つ。input/
immutable row/table/handoffは`Debug, Clone, PartialEq, Eq`、enum 2件はexisting
public data-enum deriveと`#[non_exhaustive]`を持つ。
`SourceStatementLabel`/`SourceStatementCitation`は対応する全input fieldの
read-only accessorを公開し、`SourceStatementLabelTable`/
`SourceStatementCitationTable`はtyped `get`、source-ordered `iter`、
`len`、`is_empty`だけを公開する。

`SourceStatementReferenceProducer::build`のexact syntax-free signatureは:

```rust
pub fn build(
    input: SourceStatementReferenceHandoffInput,
    statements: &SourceStatementHandoff,
    resolver_ast: &ResolvedAst,
    projection: &LabelProjection,
    reference: &LabelReferenceCandidate,
    resolution: &LabelResolutionResult,
    arena: &TypedArena,
) -> Result<SourceStatementReferenceHandoff, SourceStatementReferenceError>;
```

immutable handoffはvalidated `ResolvedAst`、`LabelProjection`、
`LabelReferenceCandidate`、`LabelResolutionResult`のexact cloneとTask-258
base debug fingerprintをownし、mutable/unchecked constructorを持たない。
exact arena/table accessorは:

```rust
pub const fn source_id(&self) -> SourceId;
pub const fn module_id(&self) -> &ModuleId;
pub fn statement_fingerprint(&self) -> &str;
pub const fn resolver_ast(&self) -> &ResolvedAst;
pub const fn label_projection(&self) -> &LabelProjection;
pub const fn reference_candidate(&self) -> &LabelReferenceCandidate;
pub const fn label_resolution(&self) -> &LabelResolutionResult;
pub const fn labels(&self) -> &SourceStatementLabelTable;
pub const fn citations(&self) -> &SourceStatementCitationTable;
pub fn debug_text(&self) -> String;
```

errorは:

```rust
#[non_exhaustive]
pub enum SourceStatementReferenceError {
    DependencyMismatch,
    InvalidLabel { label: SourceStatementLabelId },
    InvalidCitation { citation: SourceStatementCitationId },
    InvalidAggregate,
}
```

reference production、typed installation、final assemblyは1つのtotal
precedenceを使用する。source/module、statement fingerprint、resolver-AST
identity、resolver replay result、shared typed arenaがproducerで最初に
`DependencyMismatch`。typed/final installationはstatement handoffの
binding/lower fingerprintと実際にinstallされたTask-252/256 valueへ同じ
dependency-first classを適用する。exact `1/1` cardinality、dense ID、source-order
aggregate structureが次に`InvalidAggregate`。続いてfirst invalid
projection/label pairが`InvalidLabel`、first invalid reference/citation pairが
`InvalidCitation`。mixed dependency/cardinality corruptionはdependency-
first、mixed aggregate/row corruptionはaggregate-first。later entry pointは
stored resolver object 4件をrevalidateし、replacement provenanceを構築
しない。

admitするprofileは`1/1`だけ。label 0はstatement/context 1、candidate 1、
range `77..78`、ordinal 0、visible-after statement ordinal 1、scope `[0]`、
spelling `A`、kind `ProofStep`、normal。exact origin pathは
`<package>::<module>::proof::A`。citation 0はstatement/context 3、label 0、
resolver reference 0、range `131..132`、ordinal 0、scope `[0]`、kind
`SimpleLocal`、normal。

projectionは同じorigin path、spelling/range、owner-0 source/module/
contribution、`Private`/`LocalOnly`、visible-after ordinal 1、proof scope
`[0]`を持つcurrent-module proof-step projection。trusted namespaceは
`NamespacePath::new(statements.module_id().path().as_str())`だけからderiveし、
projection module/namespaceはauthenticated statement module/derived
namespaceと一致しなければならない。さらにproof-step semantic originは
normal/non-imported、anchor `77..78`、structural path `[12]`で、real label
token resolver node 12のoriginとexactly同じであり、owner 0と
source/module/contributionをshareするがanchor/pathはshareしない。reference
candidateはsite spelling/range、current-source semantic origin、source
ordinal 3、scope `[0]`がcitation 0と一致するunqualified `ProofOrTheorem`
citation。そのsemantic originはnormal/non-imported、anchor `131..132`、
structural path `[68]`で、`ReferenceSite` nodeであるresolver node 68
（real `SurfaceNodeKind::Reference`、`131..132`）のoriginとexactly同じ。

runnerは`ResolvedNodeId`をexpose/inventせず、deterministic two-pass
`ResolvedArenaBuilder` adapterを使用する。各passはparser arena orderでreal
surface node 77件をexactly once insertし、returned resolver idと
corresponding surface indexの一致を要求し、全node kind/range/child list/
recoveryをpreserveする。current source/module range originとdeterministic
structural path `[index]`を使いreal root 76でfinishする。nodeのomit/
generate/reorder、id生成だけのdummy nodeを禁止する。preliminary passは全node
を`NotApplicable`/no reference keyに保ち、candidate構築に必要なgenuine
node-68 idを供給する。`LabelResolver::resolve`後、final passはnode 68だけを
`NodeResolutionState::Resolved` /
`NodeReferenceKey::Label(resolution.ids()[0])`へ変更し、他nodeはすべて
`NotApplicable`/no keyのままにする。

checkerはresolver/typed node各77件、root 76、all-index anchor/child/
recovery parity、no-import current source/module origin、sole resolved/keyed
node 68、resolver node 12とproof-step projection originの一致、resolver
node 68とcandidate origin/same-range typed citation nodeの一致を要求する。
node 68が`Reference`であることを含むexact `SurfaceNodeKind` parityはrunnerの
real-parser selector/testだけがenforceする。checker production boundaryは
`SurfaceNodeKind`をname/match/stringify/interpretせず、normal/runtime
`mizar-syntax` dependencyを追加しない。下でfreezeするchecker test matrix
だけがtest-only dev-dependencyを追加できる。runnerは続いて、final arena、
empty name-reference/import/export table、resolver-produced
label-reference tableのexact cloneを`ResolvedAst::try_new`へ渡す。
resulting `ResolvedAst`はcurrent source/module、root 76、77 nodes、
name reference 0、label reference exactly 1、import/export 0で、
label tableが`resolution.table()`と等しく、node-68 label keyがそのsame
table entryを指さなければならない。これはfull
driver label lowering/synthetic arenaではなくbounded B1 parser-to-public-
resolver adapterである。

producerはexactly
`LabelResolver::new(&[projection.clone()]).resolve(statements.module_id(),
&derived_namespace, &[reference.clone()])`をreplayし、supplied resolutionとの
structural equalityを要求する。runnerは`LabelRefTable`/result/replacement
reference outcomeをpopulate/mutateせず、resolver-produced tableを
`ResolvedAst::try_new`へcloneするだけ。trusted namespaceをprojectionから
deriveしない。
resultはproof-step index entry 1件、resolved reference 1件、ids `[0]`、
diagnosticなし、unresolved/ambiguousなしで、result/indexはtheorem owner
projectionを含まずproof-step `A`だけを含む。したがってmutually altered
scope/ordinal inputをlossy result tableで隠せない。

stable new debug schema:

```text
source-statement-reference-debug-v1
module: <package>::<module>
statement-fingerprint: <quoted source-statement debug>
resolver-ast root=76 nodes=77 name_refs=0 label_refs=1 imports=0 exports=0 label_node=12 reference_node=68 reference_state=resolved reference_key=label#0
resolver-projection origin=<package>::<module>::proof::A namespace=<module> range=77..78 visible_after=1 scope=[0] kind=proof-step visibility=private export=local-only spelling="A"
resolver-reference node=68 range=131..132 source_ordinal=3 scope=[0] expectation=proof-or-theorem spelling="A"
resolver-result index=1 references=1 ids=[0] diagnostics=0
label#0 statement=1 context=1 candidate=1 origin=<package>::<module>::proof::A scope=[0] range=77..78 source_ordinal=0 visible_after=1 kind=proof-step recovery=normal spelling="A"
citation#0 statement=3 context=3 label=0 label_ref=0 scope=[0] range=131..132 ordinal=0 kind=simple-local recovery=normal
```

### Installation、exclusion、semantics、tests、audit

`TypedAst`はexact field/accessor/combined one-shot installerを追加する。

```rust
source_statement_references: Option<SourceStatementReferenceHandoff>,

pub const fn source_statement_references(
    &self,
) -> Option<&SourceStatementReferenceHandoff>;

pub fn with_source_statement_references(
    self,
    statements: SourceStatementHandoff,
    references: SourceStatementReferenceHandoff,
) -> Result<Self, TypedAstError>;
```

installerはfresh statement slot、exact Task-252/256 dependency、B1
base/reference pair、`3/1/0` binding environment、shared arenaを要求する。
全failureは`TypedAstError::InvalidSourceStatement`で、complete validation後
だけ両handoffをpublishする。existing
`with_source_statement(self, statement) -> Result<Self, TypedAstError>`は
Task-258A-only installerのままでdebug byte/validationは不変。

`ResolvedTypedAst`はsame optional fieldとexact
`source_statement_references(&self) ->
Option<&SourceStatementReferenceHandoff>` accessorを追加する。`assemble`はB1
pairをtogether revalidate/cloneし、全failureを
`ResolvedTypedAstError::InvalidSourceStatement`とする。typed/resolved両debug
textでbase `source_statement.debug_text()`の直後、node/table output前に
`source_statement_references.debug_text()`を出す。Task-258Aはreference
chunkを持たずbyte-identical。

Task-248 source context、全Task-257 family、preinstalled statement profile、
referencesなしB1 base、matching baseなしreferences、mixed A/B1 row、
他source ownerとの両install orderはpartial mutationなしでfail。

typed arenaはexact theorem/proof-block/compact-statement/conclusion/
proposition-formula-wrapper/equality/term/justification/reference topologyを
preserveする。owned statement siteはtheorem `19..138`、compact
`77..114`、conclusion `96..107`/`117..133`で、formula targetはそのdirect
structural descendant。compact statementがcontainできるのはfrozen nested
proof/conclusion subtreeだけ。exact admitted statement containment treeは
row 0がrows 1/3をcontainし、row 1がrow 2をcontainする形で、statement
site/formula target 4件はすべてdistinct。これ以外のancestor/descendant/
sibling crossing、duplicate site reuse、citationのnested proof移動、
assumption/witness/second label/reference追加、proof/justification node
substitutionはfail closed。

exact selectorはmissing final LF、byte/name/status/role/reserve change、
missing/extra item、direct以外・parenthesized/composite/non-equality formula、
proof block/statementのmissing/extra/reorder、`hence`、`then`、assumption、
witness、`given`、`consider`、`now`、`hereby`、case/suppose、iterative
equality、theorem citation、imported label、forward citation、local-label
shadow、recovery、byteを変えるcomment、全active corpus/Task-258A near missを
rejectする。

Task 258B1は`TypedAst::facts`、checked formula、statement semantics、
checked proof/node/goal、cluster/overload/expression output、diagnostic、
CoreIr、ControlFlowIr、VC、cache、artifact、proof acceptance、theorem
publicationをemptyのままにする。`ProofStepProposition`はunverified candidate
のまま、`SimpleLocal` citationはresolver-resolvedだがsemantically
unacceptedなproof intentであり、input fact/accepted premiseではない。
semantic interpretationはTasks 269–272が保持。

checker tests 4件はB1 base/reference API/debug、complete dependency/
aggregate/row/subtree error precedence、resolver scope/ordinal/origin
corruption（independently stale resultは`DependencyMismatch`、coherently
replayed projection/reference mutationは`InvalidLabel`/`InvalidCitation`）、
full resolver/typed-arena parityとnode/origin corruption、exact binding-
owner range/debug validation、Task-258A byte compatibility、typed/final
ownership、semantic coexistence rejection、rollback/clone/replayをfreezeする。
mizar-test library tests 5件はreal frontend range、theorem/local label
resolver provenance、two-pass/final-keyed 77-node resolver AST、replay
bundle、binding context、8 references/4 atomics、2 handoff、selector
isolation、mutation、empty final outputをfreezeする。

checker test moduleはfrozen resolver ASTをconstructする必要がある一方、
production checker codeは`SurfaceNodeKind`をnameしてはならない。そのため
implementationは`mizar-syntax`を`crates/mizar-checker/Cargo.toml`の
`[dev-dependencies]`にだけ追加し、対応する`Cargo.lock` dependency edgeは
mechanicalとする。production dependency/runtime syntax accessは追加しない。
このbounded test-construction gapはB1 `source_drift`/`test_gap`の一部で、
independent resolver changeではない。

本documentation prerequisiteはproduction source、fixture、sidecar、
expectation、trace TOML/status/count、executable route、test list、hashを
変更しない。baselineはplan `419/387`、type `253/241`、pass/fail
`228/191`、active parse/declaration/type/proof `101/5/198/1`、
warnings/errors `23/0`、checker/runner libraries `338/369`、runner
production 30 paths / 34,955 lines、path/content hash
`98f3b264a59fed5b08c3e8f20e7ca58ff54efaa154eab16a7572a69ce923f275` /
`dd399648aecadf2e7a63f685ad87577b7ebae9a9064fbfaba429a07d25ed9912`。
test-list/five CLI hashはTask-258A completion valueのまま。

missing contractは`design_drift`、absent B1 profile/reference handoff/
installer/dormant route/matrixはbounded `source_drift`/`test_gap`。
blocking `spec_gap`、`test_expectation_drift`、
`source_undocumented_behavior`、`boundary_violation`、unresolved
`repo_metadata_conflict`はない。
`spec.en.checker.formula_statement.source_payloads`は`tests = []`/
deferredのまま。coverage auditはfollow-up ownershipだけを変更する。
documentation exitはEN/JA sync、independent no-findings review、measured
artifact不変、全hard gate、quality 90/100以上、task-only staging、
dedicated documentation commitを要求。そのcommitとfresh preflight後だけ
Task-258B1 implementationへ進む。

### Task 258B1 implementation status

frozen base/reference transactionをexactに実装した。base producerはcaller
flagではなくauthenticated dependencyからTask-258A/Task-258B1 profileを
selectする。reference producerはexact resolver projection/reference/resultを
replayし、全resolver nodeをsame-index typed nodeと比較してからproof-step
label 1件とsimple local citation 1件だけをpublishする。dependency、
aggregate、label、citation error precedenceはfail-closedかつreplay-safe。

combined `TypedAst`/`ResolvedTypedAst` ownerはbase/reference pairをatomicに
publishする。checker 4本がcomplete API/debug、dependency/row/provenance
corruption、owner exclusion/rollback、final revalidation/clone、empty semantic
boundaryをcoverする。Task-258A installer/debug byteは不変。broader statement
shapeと全proof semanticは本implementation外に残る。

## Task 258B2 frozen single-assumption slice

Task 258B2はTask 258B1後のnext dependency-ready transport sliceである。
authorityは`doc/spec/en/15.statements.md` §§15.3.1、15.4.1、15.8.2、
15.10、Chapters 13–14のequality formula/term rules、Chapter 4のreserve
visibility、existing `pass_parser_simple_statements_001.miz`、Task-88/89
parser/resolver fixtures、public Task-48/252/256/258A/258B1 APIである。
これらはunlabeled single assumptionをsource intentとしてsupportするが、
factとしてacceptすることやproof effectのinterpretationはauthorizeしない。

future corpus-dormant exact consumerは次の113-byte final-LF source、
SHA-256
`c9d77d864ab899865bac77c29c57ff5785d553f8b119ef2274e4e9caf031a125`:

```mizar
reserve x for set;
theorem FormulaStatementSingleAssumptionSmoke: x = x proof
  assume x = x;
  thus x = x;
end;
```

fresh parser/resolver inventoryは次のidentityをfreezeする。

| object | exact identity |
| --- | --- |
| surface arena | 55 nodes、root 54、全node unrecovered |
| reserve/theorem | reserve `0..18`; theorem item node 51、`19..112`; label `27..64` |
| theorem owner | local public/exported theorem 1件、contribution 0、origin path `[2,1]` |
| proof | node 50、`72..111` |
| statement rows | theorem node 51 `19..112`; assumption node 41 `80..93`; conclusion node 49 `96..107` |
| atomic targets | nodes 32/38/46、range `66..71`、`87..92`、`101..106` |
| primary terms | nodes 28/30/34/36/42/44、range `66..67`、`70..71`、`87..88`、`91..92`、`101..102`、`105..106` |
| resolver labels | proof-step label、citation、label-reference keyはいずれも0件 |

syntax-free lower compositionはexact Task-48 `2/1/0`である。context 0はmodule、
context 1は`72..111`の`BindingContextOwner::SourceStatement` proof contextで、
両方がreserved binding 0をexposeする。Task-252は`6/6/0`、context sequence
`0,0,1,1,1,1`、stored use ordinal 1。Task-256は
`3/0/0/0/0/0/0/6/6`、formula contexts `0,1,1`。source-statement baseは
`1/3/3/3/3`で、`SourceStatementReferenceHandoff`はinstallしない。

`SourceStatementKind`が追加するのは`Assumption`だけ。exact statement kindは
`TheoremProposition`、`Assumption`、`Conclusion`で、normalized spellingは順に
`theorem FormulaStatementSingleAssumptionSmoke : x = x proof assume x = x ;
thus x = x ; end ;`、`assume x = x ;`、`thus x = x ;`。
各rowはdirect atomic formula target 1件、context 1件、2つのTask-252
referenceに対する`ReservedTypeGuard` input 1件、
`UnverifiedProposition` candidate 1件を持つ。assumption kindはsource intent
だけをrecordする。`SourceStatementProducer::build` signatureは不変で、
caller flagではなくexact authenticated row/dependencyからprofileをselectする。

existing base-only `TypedAst::with_source_statement` installerはTask-258A-only
からexact Task-258AまたはTask-258B2 base profileへwidenする。Task-258B1は
paired reference installerを引き続きrequireする。installationとfinal
`ResolvedTypedAst` assemblyはsame shared arenaをrevalidateしbaseをatomicに
cloneする。Task-248、全Task-257 family、Task-258A/B1/B2 cross-profile
hybrid、preinstalled statement/reference payload、全semantic table、foreign
source ownerとの両ownership orderはpartial mutationなしでrejectする。
Task-258A/B1 debug byteは不変。

statement containment graphはtheorem row 0がsibling rows 1/2をcontainし、
proof statement同士はcontainしない。formula targetはowning statementの
direct proposition/formula descendantで、相互にdistinctでなければならない。
proof-block、proposition、formula-wrapper、punctuation、unrelated surface
nodeはunowned validation contextのまま。duplicate site、row crossing、
ancestor/descendant substitution、other statementのformula、
recovered/degraded node、extra label/citation、labeled/collective
assumption、`given`、`consider`、`take`、`then`、`hence`、`now`、
`hereby`、case/suppose、iterative equality、composite/non-equality formula、
extra/reordered statement、source byte changeはfail closed。

Task 258B2はaccepted premise、fact、checked formula、statement semantic、
proof node/goal、diagnostic、theorem status、IR、VC、cache、artifactをpublish
しない。特に`Assumption`と`UnverifiedProposition`はformulaをproof contextへ
追加するauthorityではない。assumption/proof-skeleton/justification meaningは
Task 272、local declaration/closure-capture-substitution/reconsider intentは
Tasks 269–271が保持する。Task 258B3はwitness transport、Task 258B4は
composite theorem root、Task 258B5はbroader imported/outer/inner visibilityを
保持する。

future implementation scopeはexisting checker `source_statement.rs`、
typed/final profile validation、lint-policy tests、existing private runner
statement leaf/facade、checker tests 4本、runner tests 5本に限定する。既存
`BindingContextOwner::SourceStatement` contractをreuseしなければならず、
`binding_env.rs` source changeは禁止する。new lower-stage defectを発見した場合は
separate prerequisiteを必要とする。testsは全exact row/accessor/debug field、
Task-48/252/256
fingerprint、parser/resolver provenance、all-index typed/surface parity、
profile/semantic ownership両order、complete mutation/replay、exact selector
near miss、active-corpus isolation、clone/rollback/empty semanticsをcoverする。
production syntax dependency、fixture、sidecar、expectation、trace edit、
active route、corpus creditは禁止。

missing Task-258B2 contractは`design_drift`。absent exact profile/dormant routeは
bounded `source_drift`、absent checker/runner 4/5 test matrixは`test_gap`。
blocking `spec_gap`、`source_undocumented_behavior`、
`test_expectation_drift`、`boundary_violation`、unresolved
`repo_metadata_conflict`はない。
`spec.en.checker.formula_statement.source_payloads`は`tests = []`/deferredの
ままで、coverage auditはfollow-up ownershipだけを変更する。

本documentation prerequisiteはsource、fixture、sidecar、expectation、trace
row/status/count、route、test list、hashを変更しない。baselineはplan/type
`419/387` / `253/241`、pass/fail `228/191`、active
parse/declaration/type/proof `101/5/198/1`、warnings/errors `23/0`、
checker/runner libraries `342/374`、runner production 30 paths /
35,854 lines、path/content hash
`98f3b264a59fed5b08c3e8f20e7ca58ff54efaa154eab16a7572a69ce923f275` /
`f2d133e6fc42bd62058e95c274944aa03d80e9f8f2b5a0394a4d11e58ec3a66e`。
4 test-list hashと5 CLI hashもTask-258B1 valueのまま。exitはEN/JA
documentation sync、independent no-findings review、全hard gate、read-only
quality 90/100以上、task-only staging、dedicated documentation commitを要求。
implementationはそのcommitとfresh parser/resolver/lower-API/count/hash
preflight後だけ開始できる。

## Task 258B2 implementation closure

frozen 113-byte profileをlanguage meaningのwideningなしに実装した。
`SourceStatementKind::Assumption`、exact syntax-free `1/3/3/3/3` producer
profile、base-only typed/final installation、dormant runner leafが、
Task-48 `2/1/0`、Task-252 `6/6/0`、Task-256
`3/0/0/0/0/0/0/6/6`上でtheorem/assumption/conclusionをtransportする。
resolver authenticationはexact local public/exported theorem label 1件、
contribution 0、origin path `[2,1]`、import/citation/reference handoffなしを
requireする。

checker 4本とrunner 5本がbounded `source_drift`/`test_gap`をcloseし、
all-index arena parity、lower/resolver mutation replay、subtree exclusion、
Task-248/257/258 cross-family ownership、clone/debug、empty semanticsをcover
する。`Assumption`は`UnverifiedProposition`とのpairだけであり、premise、
fact、checked formula、statement semantic、proof、goal、diagnostic、
accepted theoremを作らない。Task 258B3はwitness、Task 258B4はcomposite
root、Task 258B5はbroader visibility、Tasks 269–272はproof semanticsを
保持する。

## Task 258B3 frozen single-witness slice

Task 258B3はTask 258B2後のnext dependency-ready transport sliceである。
canonical authorityは`doc/spec/en/15.statements.md` §§15.4.4、15.11.5、
reserve/term/equality shellのChapters 4、13、14、existing
`pass_parser_simple_statements_001.miz`のnamed/unnamed `take` syntax、
parser/resolver fixture、public Task-48/252/256/258A/258B1/258B2 APIである。
grammarはunnamed `take x;` source shapeとleft-to-right witness orderを
authorizeする。§15.11.5のexistential-goal matching、type obligation、
substitution、named abbreviation effectはlater semanticsに属し、ここでは
実行しない。

exact future corpus-dormant consumerは次のfinal-LF 104-byte source、
SHA-256
`76fb48354fc0dfb17047900a047a5b28b806df60d139a3133e606f0ef12a3f82`:

```mizar
reserve x for set;
theorem FormulaStatementSingleWitnessSmoke: x = x proof
  take x;
  thus x = x;
end;
```

equality theorem rootはwitness transportをTask-258B4 composite-root slice
から意図的に分離する。そのためこのdormant sourceをsemantically validな
proofとは主張しない。`take`にはexistential goalが必要であり、Task
258B3でactive corpus caseやaccepted theoremにはできない。

fresh parser/resolver inventoryは次のidentityをfreezeする:

| object | exact identity |
| --- | --- |
| surface arena | 49 nodes、root 48、すべてunrecovered |
| reserve/theorem | reserve node 25 `0..18`; theorem node 45 `19..103`; label `27..61` |
| theorem owner | local public/exported theorem 1件、contribution 0、range `19..103`、origin path `[2,1]`; importなし |
| proof | node 44、`69..102`、lexical scope `[0]` |
| formula statements | theorem node 45 + transparent `FormulaExpression` wrapper 31 + Task-256 atomic site 30 `63..68`; conclusion node 43 + wrapper 41 + atomic site 40 `92..97` |
| witness | `TakeStatement` node 35 `77..84`; `Witness` node 34 `82..83`; transparent `TermExpression` wrapper 33とTask-252 term/reference site 32 `82..83` |
| formula terms | transparent wrappers 27/29とTask-252 sites 26/28 `63..64`/`67..68`; wrappers 37/39とsites 36/38 `92..93`/`96..97` |
| resolver labels | proof-step label、citation、label-reference key、resolver companionなし |

syntax-free lower compositionはexactである。Task 48は`2/1/0`: module
context 0と
`BindingContextOwner::SourceStatement { source_range: 69..102 }`がownする
proof context 1を持ち、parent 0、proof layer、scope `[0]`、local binding
なし、visible reserved binding 0、normal recoveryである。binding 0は
`8..9`のreserved `x`、`14..17`の`set` typeを保持する。

Task 252は`5/5/0`。dense term/reference ID 0–4はtransparent wrappers
27/29/33/37/39配下のactual owned sites 26/28/32/36/38でranges
`63..64`、`67..68`、`82..83`、`92..93`、`96..97`をcoverし、binding
contexts `0,0,1,1,1`、scopes `[],[],[0],[0],[0]`、source ordinal 0–4、
stored use ordinalはすべて1。全termはnormal spelling `x`、kind
`VariableReference`、role `Value`、parentなし、全referenceはbinding 0への
normal `Variable` referenceである。Task 256は
`2/0/0/0/0/0/0/4/4`: equality formulas 0/1は`63..68`/`92..97`、contexts
0/1でordered left/right primary targetを持つ。formula 0はterms 0/1、
formula 1はterms 3/4をtargetにする。primary term 2は全atomic
edge/requestからexcludeされ、witness transactionだけがownする。Task-256
formula IDs 0/1はtransparent `FormulaExpression` wrappers 31/41配下の
`BuiltinPredicateApplication` sites 30/40をownする。
application/structure/set-term fingerprintはすべてabsent。

base `SourceStatementHandoff`はformula-onlyを維持し、exact cardinality
`1/2/2/2/2`:

| table row | exact contract |
| --- | --- |
| owner 0 | authenticated theorem symbol/contribution; node 45; `19..103`; spelling `FormulaStatementSingleWitnessSmoke`; `Theorem` / `Unmodified` / normal |
| statement 0 | owner/context 0; atomic formula 0; node 45; `19..103`; source ordinal 0; `TheoremProposition`; normalized complete-theorem spelling |
| statement 1 | owner 0/context 1; atomic formula 1; node 43; `87..98`; source ordinal 2; `Conclusion`; spelling `thus x = x ;` |
| context 0/1 | statement 0/1; binding context 0/1; ranges `19..103`/`87..98`; visible bindings `[0]` |
| input fact 0/1 | statement/context 0/1; ordinal 0; `ReservedTypeGuard`; binding 0; uses `[0,1]`/`[3,4]` |
| candidate 0/1 | statement/context 0/1; ordinal 0; `UnverifiedProposition`; atomic formula 0/1 |

normalized theorem spellingは
`theorem FormulaStatementSingleWitnessSmoke : x = x proof take x ; thus x
= x ; end ;`。dense base table IDはglobal proof-source orderを消さない:
base source ordinalはexactly 0/2、companion witness source ordinalは1。
combined partitionはgap/duplicate/reorderなしのexact `[0,1,2]`。

witnessを`SourceStatementKind`へ追加してはならない。全base rowはformula、
formula-statement context、input fact、candidate factをrequireするが、
`take x;`はtermを持ちpropositionを持たない。Task 258B3は次のseparate
syntax-free transactionを追加する:

- dense `SourceStatementWitnessId`;
- `SourceStatementWitnessHandoffInput`と
  `SourceStatementWitnessInput`;
- non-exhaustive `SourceStatementWitnessKind::Unnamed`と
  `SourceStatementWitnessTermTarget::Primary`;
- immutable `SourceStatementWitnessHandoff`、`SourceStatementWitness`、
  `SourceStatementWitnessTable`;
- `SourceStatementWitnessProducer`とnon-exhaustive
  `SourceStatementWitnessError`。

exact public construction surfaceはcanonical EnglishのRust blockと同一で、
`SourceStatementWitnessHandoffInput { source_id, module_id, witnesses }`、
`SourceStatementWitnessInput { owner, binding_context, term, take_site,
take_range, site, source_range, source_ordinal, ordinal, spelling, kind,
recovery }`、`SourceStatementWitnessProducer::build(input, statements,
primary_terms, arena)`を公開する。

`SourceStatementWitnessId`は
`Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash`。
input/row/table/handoffは`Debug + Clone + PartialEq + Eq`。kind/targetは
`Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash`、
producerは`Debug + Clone + Copy + Default`、errorは
`Debug + Clone + PartialEq + Eq`かつ`Display`/`Error`をimplementする。
`SourceStatementWitnessHandoffInput`は`source_id`、`module_id`、
`witnesses`を持つ。immutable handoffはsource/module identity、derived exact
`statement_fingerprint`/`primary_term_fingerprint`、witness row exactly
1件を持つ。handoff accessorは`source_id`、`module_id`、両fingerprint、
`witnesses`、deterministic `debug_text`、table accessorは`get`、`iter`、
`len`、`is_empty`を公開する。

witness row 0はowner 0、direct `BindingContextId(1)`、primary target 2、
take site/range node 35/`77..84`、witness site/range node 34/`82..83`、
source ordinal 1、within-`take` ordinal 0、spelling `x`、kind `Unnamed`、
normal recovery。accessorはsyntax typeなしで全fieldを公開する。typed
arenaはnode 35だけに`source.statement-witness.take`、node 34だけに
`source.statement-witness.item`を割り当て、transparent
`TermExpression` wrapper 33は`source.surface.unowned`、Task 252は
`TermReference` node 32をownする。companionはordered containment
35 → 34 → 33 → 32、exact term/reference range、binding 0、context 1、
scope `[0]`、use ordinal 1、Task-256 edgeからのabsenceをvalidateする。
tokenをcopyせず、formula/binding/resolver node/projection/reference/resultを
inventしない。

`SourceStatementWitness`は`owner`、`binding_context`、`term`、`take_site`、
`take_range`、`site`、`source_range`、`source_ordinal`、`ordinal`、
`spelling`、`kind`、`recovery` accessorをexisting statement rowと同じ
borrowed/value return styleで公開する。

`SourceStatementWitnessProducer::build(input, statements, primary_terms,
arena)`はexact Task-258B3 base profileをauthenticateし、両debug
fingerprintをstoreする。failure precedenceはsource/module/base/lower/
fingerprint/shared-arena dependency firstの`DependencyMismatch`、exact
one-row cardinality secondの`InvalidAggregate`、その後first invalid
field/ordinal/site/containment/target/context/binding/scope/recoveryの
`InvalidWitness { witness }`。revalidationも同じprecedence。new resolver
bundleなしでprovenanceは十分である。base transactionが
`SymbolEnv`-authenticated theorem ownerを保持し、Task 252がwitnessで使う
authenticated reserved-variable referenceを保持する。

exact debug grammar:

```text
source-statement-witness-debug-v1
module: <package>::<module>
statement-fingerprint: <quoted source-statement debug>
primary-term-fingerprint: <quoted source-primary-term debug>
witness#0 owner=0 binding_context=1 term=primary#2 take_range=77..84 take_site=35 range=82..83 site=34 source_ordinal=1 ordinal=0 kind=unnamed recovery=normal spelling="x"
```

`SourceStatementWitnessError`はexactly `DependencyMismatch`、
`InvalidWitness { witness: SourceStatementWitnessId }`、
`InvalidAggregate`を持つ。textはdependency mismatch、invalid dense
witness ID、invalid witness aggregateをそれぞれnameする。

`TypedAst`はoptional field/accessor
`source_statement_witnesses: Option<SourceStatementWitnessHandoff>` /
`source_statement_witnesses()`を追加し、
`with_source_statement_witnesses(statements, witnesses)`だけがB3 pairを
publishできる。existing base-only installerはTask-258A/258B2-only、
reference-paired installerはTask-258B1-onlyを維持する。final
`ResolvedTypedAst`もsame field/accessorを追加し、same pairを
revalidate/clone-preserveする。orphan half、standalone B3 base、stale/
cross-profile fingerprint、B1 references+witness、Task-248、全Task-257
family、他source family、全semantic table、いずれのownership orderも
atomicにfailする。debug orderはlower handoffs、base
`source-statement-debug-v1`、witness
`source-statement-witness-debug-v1`、nodes。earlier debug bytesは不変。
B3 productionはproducer validation用にformula-only baseをindependentに
buildできるが、matching witness handoffなしでtyped/final ownerへinstall
してはならない。

exact containment graphではtheorem row 0がconclusion/take/witnessとlower
descendantをcontainし、conclusion row 1は自身のformula/termsだけをcontain
する。baseはformula row 2件だけ、companionはtake/witness wrapperだけを
ownする。duplicate site、crossing row、別rowのformula/term substitution、
witness termのatomic edge接続、recovered/degraded node、wrong child order、
named/multiple witness、他term、missing/extra/reordered statement、
assumption、citation/label、composite theorem root、broader visibility、
全source byte changeはfail closed。

future checker matrixはexactly compound tests 4本: complete API/debug/lower
profile publication、exhaustive dependency/aggregate/base/witness/all-index/
provenance mutation+replay、typed ownershipと全Task-248/257/258
cross-family order+rollback、final clone/orphan/stale-half rejection+empty
semantics。mutationはbase ordinals `0/2`、witness source/within-take
ordinals `1/0`、combined partition、term 2の0/1/3/4へのsubstitution、
binding context 0/foreign proof scope、take/witness site swap、
wrapper/reference substitution、全range/spelling/kind/recovery field、
independently stale statement/primary fingerprint、coherent replayを
explicitにcoverする。complete API testはRust type/public-surface levelで
witness inputが`BindingContextId`をexposeし`SourceStatementContextId`
fieldを持たないこともfreezeする。これはruntime mutationではない。
runner matrixはexactly
compound tests 5本: real frontend/resolver/lower identity、complete
mutation/replay、named/multiple/missing/extra witness、`take y`、
reordered/extra statement、composite/existential rootを含むselector/
subtree/byte near miss、active route/全A/B1/B2 family isolation両order、
typed/final debug clone+empty semantic output。testはexisting syntax
dev-dependencyを使えるが、checker production codeはsyntax-freeを維持。

Task 258B3はaccepted witness、existential match、type obligation、
substitution、local abbreviation、fact、premise、checked formula、
statement semantic、proof node/goal、diagnostic、theorem status、IR、VC、
cache、artifactをpublishしない。Tasks 258B3N/Mがnamed/multiple/other
witness-term transportを明示的に保持し、B3後/B4前にseparately freezeする。
Task 258B4はcomposite theorem root、Task 258B5はbroader
imported/outer/inner visibility、Tasks 269–272は
binding、closure/substitution、reconsider、proof-skeleton、justification、
goal semanticsを保持する。

missing B3 contractはresolved `design_drift`。absent exact producer/paired
ownership/dormant routeはbounded `source_drift`、absent checker/runner
4/5 matrixは`test_gap`。blocking `spec_gap`、
`source_undocumented_behavior`、`test_expectation_drift`、
`boundary_violation`、unresolved `repo_metadata_conflict`はない。
`spec.en.checker.formula_statement.source_payloads`はdeferred
`tests = []`を維持し、coverage auditはownershipだけを記録してcreditを
与えない。

本documentation prerequisiteはsource、fixture、sidecar、expectation、
trace row/status/count、active route、test list、hashを変更しない。current
baselineはplan/type `419/387` / `253/241`、pass/fail `228/191`、active
parse/declaration/type/proof `101/5/198/1`、warnings/errors `23/0`、
checker/runner libraries `346/379`、runner production 30 paths /
36,479 lines。checker test-list hashesは
`83fbd231030ff57c3c2c152c9374ca10579eb50797bd0b455a22a576b9f6edd5` /
`aa34d2780713de5b89ff75e24cc152797260daefdac064410120358980555119`、
runner hashesは
`3642d5057d7dc2f47c1b739b61f9c4272b823fe200bc72270e9345386df59586` /
`467fe747add608900943eaee02e333c8d672a3a4a433f9a4efa3fea4f4b21e5a`。
runner path/content hashesは
`98f3b264a59fed5b08c3e8f20e7ca58ff54efaa154eab16a7572a69ce923f275` /
`a0553883c61b5a113b3af509f58296cc97a7b1dfd31b6f82b1d71b95ff0f8bcb`。
current checker module sizesは`source_statement.rs` 7,334 lines、
`typed_ast.rs` 4,550、`resolved_typed_ast.rs` 7,172、unchanged
`binding_env.rs` 3,156。5 CLI hashesは
`4cc13ea6bee6c1a6458d4a7d027a7eea685b711eda8410edafad8faa01809d54`、
`a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`、
`210055108c257ff65c6f45fb654c82e506653ec4617b68d111893bb3aa1da5a8`、
`f87a743b914d2d51d6b9a8dbcf3c8d93bbc1403b44907fd85123a1865f84edd5`、
`ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`
を維持。
implementationはlibraries `350/384`、production paths 30をprojectし、
exact line countとchanged test/content hashは予測せず実測する。

exitはEN/JA documentation sync、independent no-findings review、全hard
gate、read-only quality 90/100以上、task-only staging、dedicated
documentation commitをrequireする。implementationはそのcommitとfresh
parser/resolver/lower-API/count/hash preflight後だけ開始できる。

## Task 258B3 implementation result

frozen producer、row/table/handoff/error API、exact B3 base profile、
fingerprint、containment check、combined `[0,1,2]` order、deterministic debugを
implementした。checker tests 4本がpublication、dependency/aggregate/row/
provenance corruption、paired ownership、final revalidation、replay、empty
semanticsをcoverする。bounded `source_drift`/`test_gap`はcloseし、全semantic
deferralとdeferred trace rowは不変。

## Task 258B3N frozen named-witness slice

Task 258B3NはTask 258B3 implementation後のnext dependency-ready slice。
旧B3N/M umbrellaを分解し、B3NはRHSが既存reserved-variable termである
named witness 1件だけを所有する。Task 258B3Mはmultiple witnessとその他の
witness-term shapeを保持し、Task 258B4は両方の完了までblocked。

canonical authorityは`doc/spec/en/15.statements.md` §§15.4.4、15.11.5、
`doc/spec/en/04.variables_and_constants.md` §4.4.3、Chapters 13、14、
existing `pass_parser_simple_statements_001.miz`、parser/resolver fixture、
public Tasks 48/252/256/258A/B1/B2/B3 API。grammarは`take y = x;`を
authorizeし、Chapter 4は`y`をlocal nameと分類する。§15.11.5の
local-name useとexistential-witness effectはlater semanticsに属する。
B3Nはexact name occurrenceだけを記録し、`BindingId`、local
abbreviation、substitution、fact、obligation、proof result、accepted
theoremを作らない。Task 269がnamed `take`のfuture local `BindingId`、RHS
link、capture-by-resolved-binding abbreviation replay、context transitionを
所有する。Task 272がordered existential-binder matching、witness
type-obligation request、capture-avoiding goal substitution、remaining
goalを所有する。Task 270は`deffunc`/`defpred` closureだけ、Task 271は
`reconsider`だけを引き続き所有する。

exact future corpus-dormant consumerは次のfinal-LF 107-byte source、
SHA-256
`a57022c4b75991dd4308943477e03819f5bfe2c0d23ea1030730256252d7d329`:

```mizar
reserve x for set;
theorem FormulaStatementNamedWitnessSmoke: x = x proof
  take y = x;
  thus x = x;
end;
```

equality rootによりcomposite-root behaviorはTask 258B4に残る。`take`は
existential goalを必要とするため、本sourceをvalid proofまたはactive
corpus caseとして扱わない。

fresh real parser/resolver inventory:

| object | exact identity |
| --- | --- |
| surface arena | 51 nodes、root 50、同一source、全unrecovered |
| reserve/theorem | reserve node 27 `0..18`; theorem node 47 `19..106`; label token 6 `27..60` |
| theorem owner | local public/exported theorem 1件、contribution 0、range `19..106`、origin `[2,1]`; importなし |
| proof | node 46 `68..105`、lexical scope `[0]` |
| formula statements | theorem node 47 + wrapper 33 + atomic node 32 `62..67`; conclusion node 45 + wrapper 43 + atomic node 42 `95..100` |
| named witness | `TakeStatement` node 37 `76..87`; `Witness` node 36 `81..86`; name token 13 `81..82` spelling `y`; `=` token 14 `83..84`; RHS wrapper 35 + term/reference node 34 `85..86` spelling `x` |
| formula terms | wrappers 29/31 + Task-252 nodes 28/30 `62..63`/`66..67`; wrappers 39/41 + nodes 38/40 `95..96`/`99..100` |
| resolver labels | theorem owner projectionのみ。proof-step label、citation、label-reference key、新resolver companionなし |

exact syntax-free lower compositionはTask-48 `2/1/0`: module context 0、
`BindingContextOwner::SourceStatement { source_range: 68..105 }`所有のproof
context 1、reserved binding 0が1件、diagnosticなし、proof-context bindings
empty、visible bindings `[0]`。name token `y`はB3NでTask-48 bindingに
しない。Task 252は`5/5/0`で、term/reference nodes
`28/30/34/38/40`、ranges `62..63`、`66..67`、`85..86`、`95..96`、
`99..100`、source ordinals `0..4`、contexts `0/0/1/1/1`、use ordinal 1。
name tokenはprimary termではない。Task 256は
`2/0/0/0/0/0/0/4/4`のまま。equality formulasはnodes 32/42、edgesは
primary terms `[0,1,3,4]`をtargetとし、witness RHS term 2をexclude。

base statement profileは`1/2/2/2/2`: owner 47、source ordinals 0/2の
theorem/conclusion、contexts 0/1、reserved guards、unverified candidates。
witness companionは`1 witness / 1 name`。witness 0はowner 0、proof binding
context 1、primary term 2、take node/range `37`/`76..87`、witness
node/range `36`/`81..86`、spelling `y = x`、source ordinal 1、within-take
ordinal 0、kind `Named`、normal recovery、`name = Some(name#0)`。
name row 0はwitness 0へlinkし、token node/range `13`/`81..82`、spelling
`y`、normal recoveryを所有する。base+witness partitionはexact
`[0,1,2]`。

frozen public-table extension:

- `SourceStatementWitnessNameId`はexisting dense-ID contract
  `Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash`と、
  public `const fn new(usize) -> Self` /
  `const fn index(self) -> usize`を持つ;
- public `SourceStatementWitnessNameInput`は
  `Debug + Clone + PartialEq + Eq`をderiveし、exact public fields
  `witness: SourceStatementWitnessId`、`site: TypedSiteRef`、
  `source_range: SourceRange`、`spelling: String`、
  `recovery: SourceStatementRecovery`だけを持つ;
- immutable public row `SourceStatementWitnessName`は
  `Debug + Clone + PartialEq + Eq`をderiveし、同じ5 fieldsをprivateに保持。
  `pub const fn witness(&self) -> SourceStatementWitnessId`、
  `pub const fn site(&self) -> &TypedSiteRef`、
  `pub const fn source_range(&self) -> SourceRange`、
  `pub fn spelling(&self) -> &str`、
  `pub const fn recovery(&self) -> SourceStatementRecovery`を公開;
- immutable public `SourceStatementWitnessNameTable`は
  `Debug + Clone + PartialEq + Eq`をderiveし、standard dense
  `pub fn get(&self, SourceStatementWitnessNameId)
  -> Option<&SourceStatementWitnessName>`、
  `pub fn iter(&self)
  -> impl Iterator<Item = (SourceStatementWitnessNameId,
  &SourceStatementWitnessName)>`、`pub const fn len(&self) -> usize`、
  `pub const fn is_empty(&self) -> bool`を公開;
- `SourceStatementWitnessHandoffInput`へexact
  `pub names: Vec<SourceStatementWitnessNameInput>`を追加し、handoffはtableを
  storeして
  `pub const fn names(&self) -> &SourceStatementWitnessNameTable`を公開;
- `SourceStatementWitnessInput`へ
  `pub name: Option<SourceStatementWitnessNameId>`を追加し、immutable witness
  rowは
  `pub const fn name(&self) -> Option<SourceStatementWitnessNameId>`を公開。
  `SourceStatementWitnessKind`は`Named`だけを追加;
- name rowはresolver symbol、`BindingId`、type、substitution、semantic
  statusを持たない。Task 258B3は`name = None`、empty name tableの
  `Unnamed` witness 1件として維持;
- valid aggregateはexact B3 `(1 witness, 0 names)`またはB3N
  `(1 witness, 1 name)`だけ。dependencyとshared arena全体、aggregate
  cardinality、witness rows、name rowsの順でvalidateする。bad profile/countは
  `InvalidAggregate`、wrong witness kind/name option/forward linkは
  `InvalidWitness { witness }`、bad name row/reverse witness linkはnew
  `InvalidName { name: SourceStatementWitnessNameId }`。display textは
  `source statement witness name {index} is invalid`。

`debug_text()`は`source-statement-witness-debug-v1` headerを維持し、既存
Task-258B3 bytesを全て不変にする。named witnessだけがexisting witness
lineの末尾に` name={name.index()}`をappendする。dense name rowは全witness
rowの後にexact
`witness-name#{id} witness={witness} range={start}..{end} site={site} recovery={recovery} spelling={spelling:?}`
で出力する。したがってempty B3 namesはnew bytesを出さず、B3N name
identity/orderはdeterministic。hybrid/orphan/duplicate/sparse/reordered/
stale-fingerprint/cross-profile tableは上記precedenceでfailする。

`SourceStatementWitnessProducer`、`TypedAst`、`ResolvedTypedAst`はpaired
base/witness install APIを維持する。B3Nはauthenticated
base/witness/name bundleだけをinstallできる。standalone half、B3/B3N
hybrid、reference hybrid、Task-248/257/other-258 ownership、semantic
coexistenceはatomicにfailする。51 nodesすべてでfrozen range/kind/normal
recovery/ordered childrenを一致させる。

checker testはexact 4 compound tests:

1. complete API/debug、B3 compatibility、B3N lower/base/witness/name、
   resolver owner、全accessor、empty semantics;
2. exhaustive dependency/aggregate/row/name/fingerprint/provenance、
   all-51-node range/kind/Recovered/Degraded/child mutation + replay;
3. paired typed ownership、B3/B3N hybrid rejection、existing
   Task-248/257/258全order + rollback;
4. final clone/revalidation、orphan/stale-half/reference-hybrid、全
   semantic-table/proof/goal coexistence rejection。

runner testはexact 5 compound tests:

1. real bytes/hash、parser/resolver、Task-48/252/256/base、witness/name row、
   combined ordinal、arena parity、paired output;
2. exhaustive lower/base/witness/name/fingerprint/resolver/all-index mutation
   + deterministic replay;
3. unnamed、changed/missing name、missing `=`、multiple witness、
   non-primary RHS、reordered/extra statement、composite/existential root、
   recoveryを含むselector/byte/subtree near miss;
4. B3N/B3/B2/B1/A/active route isolationのboth ownership order;
5. typed/final clone/debug、rollback、empty semantic output。

本prerequisiteはproduction/test source、`doc/spec`、`.miz`、fixture、
sidecar、expectation、trace row/status/count、active route、test list、count、
hashを変更しない。baselineはchecker/runner libraries `350/384`、checker
modules `9812/4644/7195/3156`、runner production
leaf/facade/root/test leaf `2806/681/2495/4291`、30 paths / 37,172 lines、
plan/type `419/387` / `253/241`、pass/fail `228/191`、active
parse/declaration/type/proof `101/5/198/1`、warnings/errors `23/0`。
test-list hashは
`67b97e6594a4208aa0e0413c072b7f21809e9f88c7ab97671d6a9dea16c831a7` /
`cef91e5ce85dde5101147206de5c066b229651b7d4d4a99a3543c09e618e4651`
および
`4a077d6ab1fa4d881ae4d8d46afd003e785be573d8438772e9fbffe37374cd2f` /
`9d0c11fe6e48f136525ef4b0ca61235d8b4d0a16b703b12ba2c378d1f947b2ae`。
production path/content hashは
`98f3b264a59fed5b08c3e8f20e7ca58ff54efaa154eab16a7572a69ce923f275` /
`adfc81c21e69a91b194161525856aa40eb0e3ea76facfc2146dcb00b473ab3c2`、
5 CLI hashはTask-258B3 valueのまま。

implementationはchecker 4/runner 5 compound tests追加、libraries
`354/389`をprojectする。changed module sizeとcontent/test hashは予測せず
実測する。missing named-witness contractはresolved `design_drift`、future
producer/routeはbounded `source_drift`、future matrixは`test_gap`。blocking
`spec_gap`、`source_undocumented_behavior`、`test_expectation_drift`、
`boundary_violation`、`repo_metadata_conflict`はない。

`spec.en.checker.formula_statement.source_payloads`は`tests = []`の
deferredを維持し、coverage auditはfrozen B3N ownershipだけを記録する。
semantic creditは付与しない。exitはEN/JA同期、independent no-findings
review、全hard gate、read-only quality 90/100以上、task-only staging、
dedicated documentation commit。そのcommitとfresh parser/resolver/lower/
count/hash preflight後だけB3N implementationへ進む。Task 258B3Mはnext
documentation prerequisite、Task 258B4はB3Mまでblocked。

## Task 258B3N 実装結果

syntax-only named witnessをfrozen contractどおり実装した。1 `Named` witness
rowがtoken `y`のdense name row 1件を指し、B3はname row 0件の`Unnamed`と
byte-identical v1 debugを維持する。validationはexact base/lower
fingerprint、51-node arena、forward/reverse name link、subtree boundary、
dependency/aggregate/witness/name error precedenceをauthenticateする。
binding、abbreviation、obligation、fact、proof result、goal transition、
accepted theoremは作成しない。checker 4本/runner 5本のcompound testsが
bounded `source_drift`/`test_gap`をcloseし、Task 258B3Mがnextである。

## Task 258B3M1 frozen mixed multiple-witness slice

fresh inventoryにより、open-endedだったTask 258B3Mを依存順の2 sliceへ
分解する。Task 258B3M1は1つの`take`内でnamed reserved-variable rowの後に
unnamed reserved-variable rowが続くexact 2-row transportだけを所有する。
Task 258B3M2はnon-reserved-variableを含む他の全witness-term shapeを保持する。
Task 258B4はB3M2までblockedのままである。

canonical authorityは`doc/spec/en/15.statements.md` §§15.4.4/15.11.5、
`doc/spec/en/16.theorems_and_proofs.md` §16.3.3 item 5、
`doc/spec/en/04.variables_and_constants.md` §4.4.3である。既存
`pass_parser_simple_statements_001.miz` fixtureはmixed shape
`take a = x, y;`を含み、parser testは1 `TakeStatement` / 2 `Witness`
nodesを要求する。parserはcomma区切りwitnessをsource orderで読み、
`identifier = term_expression`または`term_expression`を受理する。
このauthorityがfreezeするのはsyntax transportだけである。Task 269は将来の
`y` binding、RHS link、abbreviation replay、context transitionを保持し、
Task 272はordered existential-binder matching、witness type obligation、
capture-avoiding substitution、remaining goalを保持する。Tasks 270/271は
`deffunc`/`defpred` closureと`reconsider`だけを保持する。

exact future corpus-dormant consumerは次のfinal-LF 113-byte sourceで、
SHA-256は
`412a6a7f8fddebd67418f3482855ea89a1e7da922b42ebb93463971d8e49c186`:

```mizar
reserve x for set;
theorem FormulaStatementMultipleWitnessSmoke: x = x proof
  take y = x, x;
  thus x = x;
end;
```

goalはexistential claimでないequalityなのでvalid proofではなく、active
accepted corpus caseにしてはならない。

fresh real frontend inventoryはone source、unrecovered 56 nodes、root 55を
freezeする。token nodesはexactに
`0:reserve@0..7, 1:x@8..9, 2:for@10..13, 3:set@14..17,
4:;@17..18, 5:theorem@19..26,
6:FormulaStatementMultipleWitnessSmoke@27..63, 7::@63..64,
8:x@65..66, 9:=@67..68, 10:x@69..70, 11:proof@71..76,
12:take@79..83, 13:y@84..85, 14:=@86..87, 15:x@88..89,
16:,@89..90, 17:x@91..92, 18:;@92..93, 19:thus@96..100,
20:x@101..102, 21:=@103..104, 22:x@105..106, 23:;@106..107,
24:end@108..111, 25:;@111..112`で、全てchildを持たない。structural
nodesは次の通り:

| IDs | exact kind、range、ordered children |
| --- | --- |
| 26–29 | `TypeHead 14..17 [3]`; `TypeExpression 14..17 [26]`; `ReserveSegment 8..17 [1,2,27]`; `ReserveItem 0..18 [0,28,4]` |
| 30–35 | `TermReference 65..66 [8]`; `TermExpression 65..66 [30]`; `TermReference 69..70 [10]`; `TermExpression 69..70 [32]`; `BuiltinPredicateApplication 65..70 [31,9,33]`; `FormulaExpression 65..70 [34]` |
| 36–42 | `TermReference 88..89 [15]`; `TermExpression 88..89 [36]`; `Witness 84..89 [13,14,37]`; `TermReference 91..92 [17]`; `TermExpression 91..92 [39]`; `Witness 91..92 [40]`; `TakeStatement 79..93 [12,38,16,41,18]` |
| 43–50 | `TermReference 101..102 [20]`; `TermExpression 101..102 [43]`; `TermReference 105..106 [22]`; `TermExpression 105..106 [45]`; `BuiltinPredicateApplication 101..106 [44,21,46]`; `FormulaExpression 101..106 [47]`; `Proposition 101..106 [48]`; `ConclusionStatement 96..107 [19,49,23]` |
| 51–55 | `ProofBlock 71..111 [11,42,50,24]`; `TheoremItem 19..112 [5,6,7,35,51,25]`; `ItemList 0..112 [29,52]`; `CompilationUnit 0..112 [53]`; `Root 0..112 [0..25,54]` |

resolver provenanceはexact one local public/exported theorem owner/label、
contribution 0、owner range `19..112`、label range `27..63`、structural
origin `[2,1]`、normal recoveryである。import、proof-step label、citation、
label-reference key、witness-name symbol、新resolver companion bundleはない。

syntax-free lower compositionはTask-48 `2/1/0`: module context 0、source
range `71..111`所有のproof context 1、one reserved binding 0、visible
binding `[0]`、empty proof-owned binding list、no diagnostic。token `y`は
`BindingId`ではない。Task 252は`6/6/0`で、term/reference nodes
`30/32/36/39/43/45`、ranges `65..66`, `69..70`, `88..89`,
`91..92`, `101..102`, `105..106`、source ordinals `0..5`、contexts
`0/0/1/1/1/1`、binding 0、scope `[0]`、use ordinal 1。Task 256は
`2/0/0/0/0/0/0/4/4`のままで、equality nodes 34/47はprimary terms
`[0,1,4,5]`をtargetとし、witness terms 2/3を除外する。

base statement profileは`1/2/2/2/2`のまま: owner/theorem node 52と
conclusion node 50のsource ordinalsは0/2、contextsは0/1。witness
companionはexact `2 witnesses / 1 name`になる:

| Row | frozen syntax-only identity |
| --- | --- |
| witness 0 | owner 0; context 1; primary term 2; take node/range `42`/`79..93`; item node/range `38`/`84..89`; source ordinal 1; within-`take` ordinal 0; spelling `y = x`; `Named`; normal; `Some(name#0)` |
| witness 1 | owner 0; context 1; primary term 3; take node/range `42`/`79..93`; item node/range `41`/`91..92`; source ordinal 1; within-`take` ordinal 1; spelling `x`; `Unnamed`; normal; no name |
| name 0 | witness 0だけへlink; token node/range `13`/`84..85`; spelling `y`; normal |

2 rowsはone source `take` itemに属するためsource ordinal 1を共有し、dense
within-`take` ordinalsだけがsyntax orderを保持する。combined source-item
orderはtheorem 0、両witness rows 1、conclusion 2であり、left-to-right
goal effectを主張しない。

public type、enum variant、field、installerは追加しない。既存dense
witness/name tables、`Named`/`Unnamed` kinds、primary-term target、
`SourceStatementWitnessProducer`、
`TypedAst::with_source_statement_witnesses`、final `ResolvedTypedAst`
ownershipだけで十分である。private validatorはexact B3M1 profileだけを
追加する。dependency/fingerprint/complete shared-arena validationの後に
aggregate cardinality、dense順witness rows、name rowsを検証する。
kind/name/term/ordinal linkのreorder、orphan/duplicate name、B3/B3N/B3M1
hybrid、sparse row、copied dependencyは既存
`DependencyMismatch` / `InvalidAggregate` /
`InvalidWitness { witness }` / `InvalidName { name }` precedenceでatomicに
failする。

typed arenaはnode 42を`source.statement-witness.take`、nodes 38/41を
`source.statement-witness.item`、token 13だけを
`source.statement-witness.name`にする。TermExpression wrappers 37/40は
unownedのまま、Task 252がreferences 36/39を所有する。takeはexactにnamed
witness 0、comma、unnamed witness 1をこの順に含む。nameはwitness 0だけの
descendantで、2つのRHS wrapper/reference subtreeはdistinct siblings。
両witnessはtheorem/proof/takeのdescendantでconclusion subtreeから除外され、
Task 256も両方を除外する。

`source-statement-witness-debug-v1`は不変。B3/B3N debug bytesを
byte-identicalに維持し、B3M1は既存grammarでwitness rows 0、1、その後
name row 0をemitする。paired typed/final ownerはstandalone halves、
reference hybrid、全Task-248/257/other-258 familyのboth order、nonempty
semantic/proof/goal tableをrejectする。成功時はpairをclone-preserveし、
全semantic outputをemptyに保つ。

checker test contractはexact 4 compound tests:

1. complete API/debug、B3/B3N compatibility、exact B3M1 lower/base/
   witness/name publication、resolver provenance、empty semantics;
2. dependency/cardinality、statement/primary fingerprint、各
   witness/name/order/link/provenance field、mixed-fault precedence
   `DependencyMismatch` → `InvalidAggregate` → witness 0 → witness 1 →
   name row、全56 nodesそれぞれのrange/kind/child corruptionと
   `NodeRecoveryState::Recovered`/`Degraded`両状態、deterministic replay;
3. paired typed ownership、B3M1/B3N/B3 hybrid、全existing
   Task-248/257/258 ownership orderとrollback;
4. final clone/revalidation、orphan、privateなstatement/primary
   fingerprintそれぞれの独立stale、reference-hybrid、全
   semantic/proof/goal coexistence rejection。

runner test contractはexact 5 compound tests:

1. exact bytes/hash、parser/resolver identity、Task-48/252/256/base、
   both witness rows、name row、ordinals、shared arena、paired output;
2. exhaustive lower/base/witness/name/resolver/all-index mutation、
   publicなstatement/primary fingerprintのpositive equality、copied
   cross-profile handoffとaggregate/cardinality corruption、mixed-fault
   dependency/aggregate/witness-0/witness-1/name precedence、
   deterministic replay;
3. reversed named/unnamed、both named、both unnamed、missing/extra/reordered
   witness、changed comma/name/`=`、non-primary RHS、recovery、
   composite/existential rootsを含むselector/byte/subtree near misses;
4. B3M1/B3N/B3/B2/B1/Aとactive-route isolationのboth ownership orders;
5. typed/final debug clone、rollback、empty semantic output。

本documentation prerequisiteはproduction/test source、`doc/spec`、既存
`.miz`、fixture、sidecar、expectation、trace row/status/count、active
route、test list、count、hashを変更しない。current baselineは
checker/runner libraries `354/389`、checker modules
`12114/4644/7200/3156`、runner statement leaf/facade/root/test leaf
`3183/684/2498/5799`、production 30 paths / 37,555 lines、plan/type
`419/387` / `253/241`、pass/fail `228/191`、active
parse/declaration/type/proof `101/5/198/1`、warnings/errors `23/0`。
test-list hashesは
`3b4eb710711061fed2c008e7e7f10e3c433398c5ddca050464d8e0d2dc9fc3af` /
`3be45d9cbe826df9fc4562feda0350c751fbcfeb776296ffba676f8cc0d54cae`
および
`bb6cbbad01b281ac0e55b2944ddc83bee73903ededa2501f4343a4b4ffb645ce` /
`65e097ba6f86648b45cf3b7bcf5a888a7e3b0498ea30ee88277960d49af60ccf`。
runner production path/content hashesは
`98f3b264a59fed5b08c3e8f20e7ca58ff54efaa154eab16a7572a69ce923f275` /
`2289634cb6126854e1382093f1adedd6d0608d0e1d241ff33b1eedd48a4716eb`
のまま。five CLI hashesもTask-258B3N値のまま。

implementationはexact 4 checker / 5 runner testsをprojectし、librariesは
`358/394`になる。broad B3M wordingの分解とB3M1 freezeは
`design_drift`を解消する。future codeはbounded `source_drift`、future
testsは`test_gap`。blocking `spec_gap`、
`source_undocumented_behavior`、`test_expectation_drift`、
`boundary_violation`、`repo_metadata_conflict`はなく、lower-stage
prerequisiteもない。coverage auditはfollow-up ownershipだけを変更し、
`spec.en.checker.formula_statement.source_payloads`は`deferred` /
`tests = []`を維持してcreditを得ない。

exitにはEN/JA同期、independent no-findings reviews、全protocol hard gates、
read-only quality 90/100以上、task-only staging、dedicated documentation
commitが必要。implementationはそのcommit後のfresh
parser/resolver/lower/count/hash preflightを経てから開始できる。

## Task 258B3M1 implementation result

frozen mixed rowをsemantic/API拡張なしでimplementした。private profileは
raw parser tuples / typed nodes全56件、resolver-owned `y`のないexact
resolver owner、6 primary terms、2 atomic formulas、2 base statements、
2 dense witnesses、1 nameをauthenticateする。validationは
dependency/fingerprint、aggregate、witness 0、witness 1、nameのprecedenceを
維持し、B3/B3N v1 bytesは不変。

checker exactly 4本 / runner exactly 5本のcompound testsがpassし、全node
mutation、exhaustive base/witness/name/resolver replay、全ownership order、
全final coexistence stages、near miss、active isolationをcoverする。
libraryは`358/394`、module sizesは`14045/4659/7201/3156`、runner sizesは
`3724/688/2501/7246`、runner productionは30 paths / 38,103 lines。
binding/semantic ownershipはTasks 269/272へdeferし、B3M2がB4前のnext。

## Task 258B3M2A frozen numeral-witness slice

Lexer Task 258B3M2P1後のfresh inventoryで、broad B3M2
“other witness-term shapes” umbrellaを依存順のB3M2A/B3M2Bへ分解する。
B3M2Aはunnamed numeral witness 1件だけを所有する。B3M2Bはcompound、
application、selector、update、set、choice、`it`、parenthesizedを含む残りの
non-reserved-variable witness shapeを保持する。Task 258B4はB3M2Bまでblocked。

canonical authorityは`doc/spec/en/15.statements.md` §15.4.4
（unnamed exampleを`term_expression`とし`take 101;`を明示）、
`doc/spec/en/13.term_expression.md` §§13.1/13.1.4/13.9
（numeralはprimary term）、`doc/spec/en/04.variables_and_constants.md`
§4.4.3（unnamed witnessはlocal nameを導入しない）、
`doc/spec/en/16.theorems_and_proofs.md` §16.3.3 item 5。
Chapter 15 §15.11.5は後続witness type obligation/existential substitutionを
所有する。本taskはsyntax transportだけをfreezeする。Task 252はnumeral
occurrence/unresolved numeric-type request、Task 272はtype inference、
existential matching、substitution、remaining goal、proof acceptanceを保持する。
unnamedなのでTask 269にはbinding workを追加しない。

exact future corpus-dormant consumerは次のfinal-LF 107-byte sourceで、
SHA-256は
`7b424949e98761b0179758065db5d164ad7d0a640f082801986683a54c43a2d1`:

```mizar
reserve x for set;
theorem FormulaStatementNumeralWitnessSmoke: x = x proof
  take 101;
  thus x = x;
end;
```

equality goalはexistentialではないためvalid proofではなく、active accepted
corpus caseにしてはならない。dedicated lexer prerequisiteは完了し、fresh real
frontend runはdiagnostic 0件。

fresh frontend inventoryはone source、49 unrecovered nodes、root 48をfreeze。
token nodesはexactに
`0:reserve@0..7, 1:x@8..9, 2:for@10..13, 3:set@14..17,
4:;@17..18, 5:theorem@19..26,
6:FormulaStatementNumeralWitnessSmoke@27..62, 7::@62..63,
8:x@64..65, 9:=@66..67, 10:x@68..69, 11:proof@70..75,
12:take@78..82, 13:101@83..86, 14:;@86..87, 15:thus@90..94,
16:x@95..96, 17:=@97..98, 18:x@99..100, 19:;@100..101,
20:end@102..105, 21:;@105..106`で、各childはない。structural nodes:

| IDs | Exact kind, range, ordered children |
| --- | --- |
| 22–25 | `TypeHead 14..17 [3]`; `TypeExpression 14..17 [22]`; `ReserveSegment 8..17 [1,2,23]`; `ReserveItem 0..18 [0,24,4]` |
| 26–31 | `TermReference 64..65 [8]`; `TermExpression 64..65 [26]`; `TermReference 68..69 [10]`; `TermExpression 68..69 [28]`; `BuiltinPredicateApplication 64..69 [27,9,29]`; `FormulaExpression 64..69 [30]` |
| 32–35 | `NumeralTerm 83..86 [13]`; `TermExpression 83..86 [32]`; `Witness 83..86 [33]`; `TakeStatement 78..87 [12,34,14]` |
| 36–43 | `TermReference 95..96 [16]`; `TermExpression 95..96 [36]`; `TermReference 99..100 [18]`; `TermExpression 99..100 [38]`; `BuiltinPredicateApplication 95..100 [37,17,39]`; `FormulaExpression 95..100 [40]`; `Proposition 95..100 [41]`; `ConclusionStatement 90..101 [15,42,19]` |
| 44–48 | `ProofBlock 70..105 [11,35,43,20]`; `TheoremItem 19..106 [5,6,7,31,44,21]`; `ItemList 0..106 [25,45]`; `CompilationUnit 0..106 [46]`; `Root 0..106 [0..21,47]` |

resolver provenanceはlocal public/exported theorem owner/label各1件、
contribution 0、owner range `19..106`、label `27..62`、structural origin
`[2,1]`、normal recovery。import、proof-step label、citation、label-reference
key、witness-name symbol、companion resolver handoffはない。private runnerは
既存exact theorem-owner enrichmentを再利用してよいが、新resolver APIは公開しない。

syntax-free lower composition:

- Task 48 `2/1/0`: module/proof contexts 0/1、proof owner `70..105`、
  reserved binding 0、visible `[0]`、proof-owned binding/diagnosticなし;
- Task 252 `5/4/1`: nodes `26/28/32/36/38`、ranges `64..65`,
  `68..69`, `83..86`, `95..96`, `99..100`、source ordinals `0..4`、
  contexts `0/0/1/1/1`。dense reference IDs `0/1/2/3`はそれぞれterms
  `0/1/3/4`をtargetし、binding 0、exact lexical-scope vector
  `[]/[]/[0]/[0]`、use ordinal 1。numeral term 2はkind `Numeral`、
  spelling `101`、normal `Value`、referenceなし、numeric request 0は
  node/range `32`/`83..86`、request ordinal 0;
- Task 256 `2/0/0/0/0/0/0/4/4`: equality nodes 30/40はprimary
  `[0,1]` / `[3,4]`をtargetし、numeral witness term 2を全atomic
  edge/requestからexclude;
- base statement `1/2/2/2/2`: theorem node 45 / conclusion node 43、
  source ordinals 0/2、contexts 0/1。theorem input-fact rowはdense
  references `[0,1]`からterms `[0,1]`、conclusion rowはreferences
  `[2,3]`からterms `[3,4]`をuseする。numeral holeを跨いでterm IDを
  reference IDとしてreuseしてはならない。

witness companionはexact `1 witness / 0 names`。witness 0はowner 0、
context 1、primary term 2、take node/range `35`/`78..87`、item
`34`/`83..86`、source ordinal 1、within-`take` ordinal 0、spelling
`101`、`Unnamed`、normal、nameなし。combined source-item partitionはexact
`[0,1,2]`で、syntax orderだけを表しexistential goal effectを主張しない。

public type/variant/field/error/table/accessor/producer/installerは追加しない。
既存`SourcePrimaryTermKind::Numeral`、`SourceNumericTypeRequestTable`、
`SourceStatementWitnessTermTarget::Primary`、witness/name tables、
`SourceStatementWitnessProducer`、
`TypedAst::with_source_statement_witnesses`、final `ResolvedTypedAst`
ownershipで十分。private base/witness selectorにB3M2A exact profileだけを
追加する。validation precedenceはdependency/fingerprint + complete arena、
aggregate、witness 0、empty name rowsで、`DependencyMismatch` →
`InvalidAggregate` → `InvalidWitness { witness: 0 }`。empty name tableでは
`InvalidName`は到達しない。numeral/numeric requestはTask-252-ownedで、
witness tableはtype semanticsをduplicateしない。

typed arenaはnode 32を`source.term.numeral`、34を
`source.statement-witness.item`、35を`source.statement-witness.take`、
30/40をequality、43をconclusion、45をtheoremとして所有する。wrapper 33は
unowned。witness subtreeはtheorem/proof/take descendantで、両equality/
conclusion subtreeからdisjoint。Task 256はprimary term 2をexcludeし続ける。

`source-statement-witness-debug-v1`とB3/B3N/B3M1 bytesは不変。paired
typed/final ownerはstandalone half、B3/B3N/B3M1/B3M2A hybrid、
reference hybrid、全Task-248/257/other-258 family order、stale fingerprint、
numeric-request corruption、nonempty semantic/proof/goal tableをreject。
successはpairをclone-preserveし全semantic outputをemptyにする。

checker compound testsはexact 4本:

1. `task258b3m2a_exact_numeral_witness_api_debug_and_compatibility_are_stable`;
2. `task258b3m2a_dependencies_numeric_request_witness_precedence_and_all_nodes_fail_closed`;
3. `task258b3m2a_paired_ownership_hybrids_and_all_family_orders_are_atomic`;
4. `task258b3m2a_final_clone_revalidation_and_semantic_deferrals_are_stable`。

runner compound testsはexact 5本:

1. `task258b3m2a_real_frontend_freezes_numeral_witness_contract`;
2. `task258b3m2a_validation_precedence_mutation_and_replay_fail_closed`;
3. `task258b3m2a_selector_and_byte_subtree_near_misses_are_exact`;
4. `task258b3m2a_family_and_active_route_isolation_is_atomic_in_both_orders`;
5. `task258b3m2a_typed_final_clone_debug_rollback_and_empty_semantics_are_stable`。

testsはexact lower/base/witness、public API no-op、prior debug bytes、
dense reference-to-term mapping、両input-fact `uses` rows、
dependency/aggregate/witness precedence、numeric request、全49 node
mutation/replay、paired/final order、empty semanticsをfreezeする。near missは
別numeral、sign/token差、named/multiple、identifier/`it`/parenthesized/
application/selector/update/set/choice RHS、recovery、theorem shape差、
existential/composite roots。dormant selectorはB3M1/B3N/B3/B2/B1/Aより先で、
public route/detail keyは追加しない。exact authenticated B3M2A outputでは
existing private detail projectionは`Some(Vec::new())`を返し、paired witness
handoff、base statements 2件、lookup ordinals `1/1`、reference use ordinals
exact `[1; 4]`を要求する。`None`はselector missのまま、owned invalid
outputはexisting `type_elaboration.checker.typed_ast_invalid` detailを維持する。
real-frontend/final compound testsはdense reference-to-term mapping、両
input-fact `uses` rows、このexact detail projectionをassertする。

本documentation prerequisiteはproduction/test source、canonical spec、既存
`.miz`、fixture、expectation、sidecar、trace row/status/count、active route、
test list/count/hashを変更しない。fresh baselineはlibraries `358/394`、
checker modules `14045/4659/7201/3156`、runner statement
leaf/facade/root/test `3724/688/2501/7246`、production 30 paths /
38,103 lines。checker raw/normalized hashesは
`39c9d84a4fe990f3a74d69554aeb5be6d41349bd8dfe40d0bc269eacab5355d5` /
`cd4e902f325c08226c10deeec64c3b8de1d11f346d82a81f1008687f009c372f`、
runnerは
`e729eaf60f00a53a9767375d8718ea8179c27bf3c660c5a936eaeeea2ef8d00a` /
`af7e5ed68cec3e3feda6fb2264471b359443e849cf0f67ed4d111207e008bb12`。
production path/content hashesは
`98f3b264a59fed5b08c3e8f20e7ca58ff54efaa154eab16a7572a69ce923f275` /
`b5b3523f13bd5b0ef5da10bd003db75fc89fd98d9d23300071f468ec22746c19`。
plan/type `419/387` / `253/241`、pass/fail `228/191`、active
`101/5/198/1`、warnings/errors `23/0`。five CLI hashesはexactに
plan
`4cc13ea6bee6c1a6458d4a7d027a7eea685b711eda8410edafad8faa01809d54`、
parse
`a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`、
declaration
`210055108c257ff65c6f45fb654c82e506653ec4617b68d111893bb3aa1da5a8`、
type
`f87a743b914d2d51d6b9a8dbcf3c8d93bbc1403b44907fd85123a1865f84edd5`、
proof
`ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`。

implementationはchecker/runner tests exactly 4/5本、libraries `362/399`を
projectするが、changed size/hashはtargetではなく実測する。B3M2分解とB3M2A
freezeは`design_drift`をcloseし、future private codeはbounded
`source_drift`、future testsは`test_gap`。blocking `spec_gap`、
`source_undocumented_behavior`、`test_expectation_drift`、
`boundary_violation`はない。contract draft後にreport-only
`repo_metadata_conflict`が発生した。local HEADは
`076555b7ba61788e30c4266c0a9fd0375004c4de`のまま、remote-tracking
`origin/main`が2026-07-28 19:46:29 +0900に外部から`1e81db7a`から同commitへ
移動し、measured ahead countが12から0へ変わった。task-owned paths、clean
committed base、untouched `stash@{0}`は一意なのでnonblockingであり、本taskでは
修復しない。completed lexer prerequisiteがsole lower-stage defectをcloseした。

coverage auditはfollow-up ownershipだけを変更し、
`spec.en.checker.formula_statement.source_payloads`は`deferred` /
`tests = []`でcreditなし。exitはEN/JA同期、independent no-findings
reviews、全hard gates、read-only quality 90/100以上、task-only staging、
dedicated documentation commit。そのcommit後のfresh
parser/resolver/lower/count/hash preflight後だけimplementationを開始できる。

## Task 258B3M2A implementation result

checkerはone private `Task258B3M2A` syntax-free profileをrecognizeする。
runnerがdispatch前にfinal-LF 107-byte sourceをauthenticateし、checkerは
その49-node arena projection全部/root 48、exact module/proof binding
contexts、primary terms 5件、scopes `[]/[]/[0]/[0]`を持つdense
references `0/1/2/3 -> 0/1/3/4`、numeral term 2のnumeric request 0、
`[0,1,3,4]`だけを使うTask-256 equality 2件、base `1/2/2/2/2`を
fail-closed authenticateする。witness validatorはterm 2をtargetするone
unnamed witness、0 names、source partition `[0,1,2]`だけをpublishし、
dependency/aggregate/witness validation precedenceを明示する。

paired typed/final consumerはこのbase/witness pairだけをacceptする。
standalone、stale、reordered、cross-family、subtree、resolver、lower-table、
numeric-request、全node/byte mutationはpartial ownershipなしでfailする。
checker 4本 / runner 5本のcompound testsがpassした。private detail
projectionはlookup ordinals `1/1`、reference use ordinals `[1;4]`を持つ
`Some(Vec::new())`のままで、全semantic/proof/goal tablesはempty。

librariesは`362/399`。checker module sizesは
`15746/4660/7202/3156`、runner statement leaf/facade/root/test sizesは
`4185/691/2505/8611`、runner productionは30 paths / 38,571 lines。
canonical spec、`.miz`、fixture、expectation、sidecar、trace
row/status/count、active route、public API、binding、semantic ownerは変更して
いない。bounded `source_drift`/`test_gap`を閉じ、B3M2BがB4前のnext。
