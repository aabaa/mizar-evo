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
| `SourceStatementCitationTarget` | `#[non_exhaustive]`; Tasks 258B1/B5Aは`Local(SourceStatementLabelId)`、Task 258B5Bはlocal label rowをfabricateせず`Imported`を使う。 |
| `SourceStatementCitationKind` | `#[non_exhaustive]`; Tasks 258B1/B5Aは`SimpleLocal`、Task 258B5Bは`SimpleImported`をaccept。 |
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
    pub target: SourceStatementCitationTarget,
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
    pub const fn target(&self) -> SourceStatementCitationTarget;
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
pub enum SourceStatementCitationTarget {
    Local(SourceStatementLabelId),
    Imported,
}

#[non_exhaustive]
pub enum SourceStatementCitationKind {
    SimpleLocal,
    SimpleImported,
}
```

両IDはexisting dense-ID deriveと`new`/`index` accessorを持つ。input/
immutable row/table/handoffは`Debug, Clone, PartialEq, Eq`、enum 3件はexisting
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

Lexer Task 258B3M2P1後のfresh inventoryはbroad B3M2
“other witness-term shapes” umbrellaを最初にB3M2A/B3M2Bへ分解した。
B3M2Aはunnamed numeral witness 1件だけを所有する。later B3M2B1は
reserved-variable childを持つparenthesized wrapperを所有し、B3M2B2は
compound、application、selector、update、set、choice、other
authority-valid termを保持する。`it`はChapter-13-valid `means` contextだけ。
Task 258B4はB3M2B2までblocked。

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

## Task 258B3M2B1 frozen parenthesized-witness slice

fresh post-B3M2A inventoryはremaining B3M2B umbrellaを
dependency-ordered B3M2B1/B3M2B2へdecomposeする。B3M2B1はexact one
unnamed parenthesized reserved-variable witness `take (x);`だけをownする。
B3M2B2はapplication、structure、selector、update、set、choice、
compound、other authority-valid witness termを保持する。`it`はChapter 13
§13.1.2が許すvalid `means` definition/property contextだけで後続となり、
このtheorem proofで`take it;`をauthorizeしない。B4はB3M2B2後。

canonical authorityは`doc/spec/en/15.statements.md` §15.4.4のunnamed
`term_expression`と`doc/spec/en/13.term_expression.md`
§§13.1/13.1.3/13.8.8/13.9のtype-preserving parenthesized primary。
`tests/miz/pass/parser/pass_parser_simple_statements_001.miz`とその
expectationはunnamed `take` syntaxをauthenticateする。active
`tests/miz/pass/types/pass_type_elaboration_parenthesized_reserved_variable_equality_001.miz`、
そのexisting expectation、covered
`spec.en.checker.type_elaboration.source_primary_term_payload` trace rowは
real wrapper/child/reserved-binding provenanceをauthenticateする。
B3/B3M1 proof-scope contractもcomposeできる。Task 272はtype obligation、
existential matching、substitution、remaining goal、proof acceptanceを
保持し、これらexisting consumers/trace artifactsは変更しない。

exact future corpus-dormant consumerはfinal-LF 113 bytes、SHA-256
`f09815b49d1b4598218f656a366ef73ec0dffd1f581a1018f07aa2ebcf410bf2`：

```mizar
reserve x for set;
theorem FormulaStatementParenthesizedWitnessSmoke: x = x proof
  take (x);
  thus x = x;
end;
```

equality goalはexistentialではなくvalid proofでないため、active accepted
corpus caseにしない。fresh frontendはdiagnostics 0、53 unrecovered
nodes、root 52。tokensはexactly
`0:reserve@0..7, 1:x@8..9, 2:for@10..13, 3:set@14..17,
4:;@17..18, 5:theorem@19..26,
6:FormulaStatementParenthesizedWitnessSmoke@27..68, 7::@68..69,
8:x@70..71, 9:=@72..73, 10:x@74..75, 11:proof@76..81,
12:take@84..88, 13:(@89..90, 14:x@90..91, 15:)@91..92,
16:;@92..93, 17:thus@96..100, 18:x@101..102, 19:=@103..104,
20:x@105..106, 21:;@106..107, 22:end@108..111,
23:;@111..112`でchildなし。structural nodes：

| IDs | exact kind、range、ordered children |
| --- | --- |
| 24–27 | `TypeHead 14..17 [3]`; `TypeExpression 14..17 [24]`; `ReserveSegment 8..17 [1,2,25]`; `ReserveItem 0..18 [0,26,4]` |
| 28–33 | `TermReference 70..71 [8]`; `TermExpression 70..71 [28]`; `TermReference 74..75 [10]`; `TermExpression 74..75 [30]`; `BuiltinPredicateApplication 70..75 [29,9,31]`; `FormulaExpression 70..75 [32]` |
| 34–39 | `TermReference 90..91 [14]`; `TermExpression 90..91 [34]`; `ParenthesizedTerm 89..92 [13,35,15]`; `TermExpression 89..92 [36]`; `Witness 89..92 [37]`; `TakeStatement 84..93 [12,38,16]` |
| 40–47 | `TermReference 101..102 [18]`; `TermExpression 101..102 [40]`; `TermReference 105..106 [20]`; `TermExpression 105..106 [42]`; `BuiltinPredicateApplication 101..106 [41,19,43]`; `FormulaExpression 101..106 [44]`; `Proposition 101..106 [45]`; `ConclusionStatement 96..107 [17,46,21]` |
| 48–52 | `ProofBlock 76..111 [11,39,47,22]`; `TheoremItem 19..112 [5,6,7,33,48,23]`; `ItemList 0..112 [27,49]`; `CompilationUnit 0..112 [50]`; `Root 0..112 [0..23,51]` |

resolver provenanceはone local public/exported theorem owner/label、
contribution 0、owner range `19..112`、label `27..68`、structural origin
`[2,1]`、normal recovery。import、proof-step label、citation、witness-name
symbol、新resolver handoffはない。existing private owner enrichmentで十分。

syntax-free lower composition：

- Task 48 `2/1/0`：module/proof contexts `0/1`、proof owner `76..111`、
  reserved binding 0がscope `[0]`でvisible、proof binding/diagnosticなし。
- Task 252 `6/5/0`：surface roots `28/30/36/40/42`の5本からdense
  primary 6 rowsを作る。terms 0/1はnodes/ranges `28/70..71`,
  `30/74..75`のcontext-0 variable `x`。term 2は
  `36/89..92`、spelling `( x )`、context 1、parentなしの
  parenthesized wrapper。term 3は`34/90..91`、context 1、parent term
  2のvariable child。terms 4/5は`40/101..102`, `42/105..106`の
  context-1 variable。dense references `0/1/2/3/4`はterms
  `0/1/3/4/5`をtargetし、binding 0、use ordinal 1、scopes
  `[]/[]/[0]/[0]/[0]`。term 2にreference/numeric requestなし。
- Task 256 `2/0/0/0/0/0/0/4/4`：equality nodes 32/44はprimary pairs
  `[0,1]` / `[4,5]`だけをtargetし、witness wrapper/child terms 2/3を
  全edge/requestからexclude。
- base `1/2/2/2/2`：theorem `49/19..112`、conclusion
  `47/96..107`、source ordinals `0/2`、contexts `0/1`。input factsは
  references `[0,1]` / `[3,4]`。

runnerは5 extraction rootsと6 expected primary rowsを別に表現する。
root countをprimary countにreuseせず、conclusion atomic startを
`root_count - 2`からderiveしない。frozen atomic startsは`[0,4]`、
input-fact reference startsは`[0,3]`。これはprivate Task-258 consumer
adjustmentで、Task-252/256 defectではない。

witness companionはexactly `1 witness / 0 names`。witness 0はowner 0、
binding context 1、`Primary(2)`、take `39/84..93`、item
`38/89..92`、source ordinal 1、within-take ordinal 0、
token-normalized spelling `( x )`、`Unnamed`、normal recovery、nameなし。
combined source orderは`[0,1,2]`。wrapperがwitness targetで、inner
variableはTask-252 child/referenceだけ。

typed ownershipはnode 36=`source.term.parenthesized`、34=
`source.term.variable-reference`、38=`source.statement-witness.item`、39=
`source.statement-witness.take`、32/44=
`source.formula.atomic.equality`、47=`source.statement.conclusion`、49=
`source.statement.theorem`。nodes 35/37はunowned wrappers。witness
subtreeはtake/proof/theorem内、both equality subtree/conclusionとdisjoint。
Tasks 253–255にapplication/structure/selector/update/set/choice
payload/wrapper/cross-family edgeを渡さない。

public type、variant、field、error、table、accessor、producer、installer、
route、detail keyは追加しない。public Tasks 248–253 lower familiesは
`SourceBindingContextHandoff`、`SourceTypeApplicationHandoff`、
`SourceAttributeHandoff`、`SourceEvidenceHandoff`、
`SourcePrimaryTermHandoff`、`SourceFunctorApplicationHandoff`。Task 254
`SourceStructureHandoff`はnext excluded family。B3M2B1はTask-248
contextとTask-252 projectionをreuseし、Tasks 249–251/253–254は
empty/excluded。existing `SourcePrimaryTermKind::Parenthesized`、
`SourcePrimaryTermInput::parent`、reference table、
`SourceStatementWitnessTermTarget::Primary`、witness/name producer/table、
paired typed/final APIで十分。private profilesだけを追加する。
validation precedenceはcomplete
dependency/fingerprint/arena、aggregate、witness 0、empty names：
`DependencyMismatch` > `InvalidAggregate` >
`InvalidWitness { witness: 0 }`、`InvalidName`はunreachable。

paired typed/final ownerはstandalone、base/witness hybrid、B3/B3N/B3M1/
B3M2A/B3M2B1 cross-profile、parent/child/reference corruption、全
Task-248/253–257/other-258 family both orders、stale fingerprint、
nonempty semantic/proof/goal tableをrejectする。successはpairを
clone-preserveし全semantic outputをemptyにする。B3M2Aとはexact bytes、
49-vs-53 nodes、term count/kind、numeric request、fingerprintでisolate。

checker test contractはexactly 4本：

1. `task258b3m2b1_exact_parenthesized_witness_api_debug_and_compatibility_are_stable`;
2. `task258b3m2b1_dependencies_parent_child_witness_precedence_and_all_nodes_fail_closed`;
3. `task258b3m2b1_paired_ownership_hybrids_and_all_family_orders_are_atomic`;
4. `task258b3m2b1_final_clone_revalidation_and_semantic_deferrals_are_stable`。

checker test 2は、child referenceを残したままwrapper term 2へreference
rowを追加するmutation、child reference 2のremove/remap/duplicate、
Task-256 edge/requestへのterm 2 contaminationとterm 3 contaminationを
それぞれ独立にrejectする。一つのrejectionを別caseの代用にしない。

runner test contractはexactly 5本：

1. `task258b3m2b1_real_frontend_freezes_parenthesized_witness_contract`;
2. `task258b3m2b1_validation_precedence_mutation_and_replay_fail_closed`;
3. `task258b3m2b1_selector_and_byte_subtree_near_misses_are_exact`;
4. `task258b3m2b1_family_and_active_route_isolation_is_atomic_in_both_orders`;
5. `task258b3m2b1_typed_final_clone_debug_rollback_and_empty_semantics_are_stable`。

runner test 2は各wrapper-reference、child-reference、term-2/term-3
Task-256 contamination mutationをreal route constructionで試行する。
malformed rowがvalid Task-252/256 handoffを構築できない場合はowning public
lower producerが先にrejectし、lower construction成功時にはpaired statement
consumerをexerciseする。runner test 3はselector/subtree near missがpartial
wrapper/child ownershipやdetached child referenceをpublishできないことも固定する。

exact identity、5-root/6-primary separation、parent/child/reference、
precedence、all 53 nodes/bytes/subtrees、B3M2A/Tasks253–255 isolation、
family/active order、rollback/replay、debug/final clone、empty semanticsを
coverする。near missesは`x`、`101`、`(101)`、`((x))`、named/multiple、
application/structure/selector/update/set/choice、recovery、changed theorem、
existential/composite root。authority-invalid theorem-proof `take it;`も
near miss。

exact private detail projectionは`Some(Vec::new())`、paired witness、base
statements 2、lookup `1/1`、reference-use `[1; 5]`を要求する。selector
missは`None`、owned invalidはexisting
`type_elaboration.checker.typed_ast_invalid`。

本documentation prerequisiteはproduction/test source、canonical spec、
existing `.miz`、fixture、expectation、sidecar、trace row/status/count、
active route、test list/count/hashを変更しない。fresh baselineはlibraries
`362/399`、checker sizes `15746/4660/7202/3156`、runner sizes
`4185/691/2505/8611`、production 30 paths / 38,571 lines。checker
raw/normalized hashesは
`af5f3c7030167087367ebbf534b9ebde03fcfcb3b406dcacbd4eccd1841a25e7` /
`4b95f5557e65e4d9ec4e9df90f3f61e77318570a0498996a7929b29500f127d7`、
runnerは
`a9557e877ad59d5d5da47861f41beaecd2e6a28b9a7a090381bb966096ecea13` /
`88a2d2e70f04c7606a78630c42ae66b7506a15df2f0cd91b4dbb3945181ad847`。
production hashesは
`98f3b264a59fed5b08c3e8f20e7ca58ff54efaa154eab16a7572a69ce923f275` /
`30640df237d236a980cd0daf013996e3d37dc36fbabd0e9badadac8a0e57c4c2`。
plan/type `419/387`, `253/241`、pass/fail `228/191`、active
parse/declaration/type/proof `101/5/198/1`、warnings/errors `23/0`、CLI
hashes
`4cc13ea6bee6c1a6458d4a7d027a7eea685b711eda8410edafad8faa01809d54`,
`a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
`210055108c257ff65c6f45fb654c82e506653ec4617b68d111893bb3aa1da5a8`,
`f87a743b914d2d51d6b9a8dbcf3c8d93bbc1403b44907fd85123a1865f84edd5`,
`ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`
は不変。

implementationはchecker/runner exactly 4/5 tests、libraries `366/404`を
projectし、changed sizes/hashesは実測する。broad umbrellaと
root/primary conflationの2件の`design_drift`をcloseし、future code/testは
bounded `source_drift`/`test_gap`。blocking `spec_gap`、unsafe test
intent、lower defect、`source_undocumented_behavior`、
`test_expectation_drift`、language/crate `boundary_violation`はない。
review-only writerによるB3M2B1 docs duplicateは
`repo_metadata_conflict`ではなくoperational `boundary_violation`であり、
parentがrepository metadataを変更せずtask-owned documentationを
reconcileした。

coverage auditはfollow-up ownershipだけ変更し、
`spec.en.checker.formula_statement.source_payloads`は`deferred`,
`tests = []`、creditなし。exitはEN/JA sync、no-findings reviews、all hard
gates、read-only quality >=90、task-only staging、dedicated docs commit。
implementationはcommit直後のfresh preflight後だけ開始する。

## Task 258B3M2B1 implementation result

checkerはone private syntax-free `Task258B3M2B1` profileを実装した。
complete 53-node arena、module/proof binding contexts、Task-48 `2/1/0`、
Task-252 `6/5/0`のterm 2 / child 3とrefs
`0/1/2/3/4 -> 0/1/3/4/5`、Task-256 `[0,1]` / `[4,5]`、base
`1/2/2/2/2`、one unnamed outer-term witness / zero namesをauthenticateする。
paired typed/final pathは両halvesをatomic publishし、near missはpartial
ownershipを残さない。

tests 4/5がpass。libraries `366/404`、checker sizes
`17569/4661/7203/3156`、runner statement leaf/facade/root/test
`4676/695/2508/9902`、production 30 paths / 39,069 lines。checker
raw/normalized hashesは
`0e43763c92ee171b18b5a2f80b92cd278b49ac9895d95410ca52ca787bcac3c8` /
`7685e21bc0d76bb8d824dd821e800707d251e8c025682ef69b2db798d6888e5d`、
runnerは
`a28c33e517d8efdd635e23e6f2273c29b966aa6102efb321eed73335ab11483c` /
`f8e8dc6ef605cbd8f8ad722983793434339b3cad21bf53703ab6c21f0b8742a5`。
production path/contentは
`98f3b264a59fed5b08c3e8f20e7ca58ff54efaa154eab16a7572a69ce923f275` /
`04bf563fcc99ccbc3b789a8596d953ade05453b2267639ef0ce3d8d54cbd6b45`。
five CLI counts/hashesは不変。

canonical/fixture/expectation/trace/active/public/binding/semantic ownershipは
不変。coverageは`deferred`, `tests = []`でcreditなし。public Task-252/256
fail-closeをbypassするtest seam/APIは追加せず、malformed lower rowsはpaired
consumer前にrejectする。B3M2B2がB4前のnext。

## Task 258B3M2B2A frozen nested-parenthesized-witness slice

post-B3M2B1 fresh inventoryはbroad B3M2B2をdependency-ordered
B3M2B2A/B3M2B2Bへdecomposeする。B3M2B2AはTask-252 nested-primary graph
だけに依存するexact one unnamed two-level parenthesized witness
`take ((x));`を所有する。B3M2B2Bはtriple/deeper parentheses、
application、structure constructor/selector/update、set、choice、compound、
other authority-valid witness termsを保持し、future cross-family workは
Task 253 → 254 → 255のlower-owner orderで再分解してからB4/B5へ進む。

canonical authorityは`doc/spec/en/15.statements.md` §15.4.4、
`doc/spec/en/13.term_expression.md` §§13.1/13.1.3/13.8.8/13.9、
`doc/spec/en/16.theorems_and_proofs.md` §§16.3.3/16.7.3。
parenthesesはarbitrary nesting、type preserving、FOL-transparentで、
unnamed exampleは任意の`term_expression`。existing
`pass_parser_simple_statements_001.miz`、`pass_parser_primary_terms_001.miz`、
Task-252 `task252_nested_parentheses_exclude_mixed_subtrees_and_keep_siblings`、
covered `source_primary_term_payload` rowがsyntax/lower transportを
authenticateする。existing source/expectation/traceは変更せず、
formula-statement creditを追加しない。

exact future corpus-dormant consumerはfinal-LF 121-byte、SHA-256
`35396db1f7e22abfbe94861709b2ab9bca38d4464712dfbce114533d2ab4d71d`：

```mizar
reserve x for set;
theorem FormulaStatementNestedParenthesizedWitnessSmoke: x = x proof
  take ((x));
  thus x = x;
end;
```

equality goalはexistential introductionをauthorizeしないためactive
accepted corpus caseにしない。fresh frontendはdiagnostics 0、57
unrecovered nodes、root 56、26 tokens：
`0:reserve@0..7, 1:x@8..9, 2:for@10..13, 3:set@14..17,
4:;@17..18, 5:theorem@19..26,
6:FormulaStatementNestedParenthesizedWitnessSmoke@27..74,
7::@74..75, 8:x@76..77, 9:=@78..79, 10:x@80..81,
11:proof@82..87, 12:take@90..94, 13:(@95..96, 14:(@96..97,
15:x@97..98, 16:)@98..99, 17:)@99..100, 18:;@100..101,
19:thus@104..108, 20:x@109..110, 21:=@111..112,
22:x@113..114, 23:;@114..115, 24:end@116..119,
25:;@119..120`。全tokenはchildrenなし。structural nodes：

| IDs | exact kind, range, ordered children |
| --- | --- |
| 26–29 | `TypeHead 14..17 [3]`; `TypeExpression 14..17 [26]`; `ReserveSegment 8..17 [1,2,27]`; `ReserveItem 0..18 [0,28,4]` |
| 30–35 | `TermReference 76..77 [8]`; `TermExpression 76..77 [30]`; `TermReference 80..81 [10]`; `TermExpression 80..81 [32]`; `BuiltinPredicateApplication 76..81 [31,9,33]`; `FormulaExpression 76..81 [34]` |
| 36–43 | `TermReference 97..98 [15]`; `TermExpression 97..98 [36]`; `ParenthesizedTerm 96..99 [14,37,16]`; `TermExpression 96..99 [38]`; `ParenthesizedTerm 95..100 [13,39,17]`; `TermExpression 95..100 [40]`; `Witness 95..100 [41]`; `TakeStatement 90..101 [12,42,18]` |
| 44–51 | `TermReference 109..110 [20]`; `TermExpression 109..110 [44]`; `TermReference 113..114 [22]`; `TermExpression 113..114 [46]`; `BuiltinPredicateApplication 109..114 [45,21,47]`; `FormulaExpression 109..114 [48]`; `Proposition 109..114 [49]`; `ConclusionStatement 104..115 [19,50,23]` |
| 52–56 | `ProofBlock 82..119 [11,43,51,24]`; `TheoremItem 19..120 [5,6,7,35,52,25]`; `ItemList 0..120 [29,53]`; `CompilationUnit 0..120 [54]`; `Root 0..120 [0..25,55]` |

resolver provenanceはone local public/exported theorem owner/label、
contribution 0、owner `19..120`、label `27..74`、structural origin
`[2,1]`、normal recovery。import、proof-step label、citation、
witness-name symbol、新resolver handoffなし。distinct theorem labelは必須で、
existing B3M2B1のsame-label `(x)`→`((x))` mutationはselector near missの
まま維持する。

syntax-free composition：

- Task 48 `2/1/0`：module context 0 / proof context 1、proof owner
  `82..119`、reserved binding 0はscope `[0]`でvisible、proof binding /
  diagnosticなし。
- Task 252 `7/5/0`：five roots `30/32/40/44/46`からseven rows。
  terms 0/1は`30/76..77`,`32/80..81`の`x`。term 2はouter
  `40/95..100`, `( ( x ) )`, parentなし。term 3はinner
  `38/96..99`, `( x )`, parent 2。term 4はvariable
  `36/97..98`, parent 3。terms 5/6は`44/109..110`,
  `46/113..114`。terms 2–6はcontext 1。refs
  `0/1/2/3/4 -> terms 0/1/4/5/6`、全て
  binding 0 / use ordinal 1、scopes `[]/[]/[0]/[0]/[0]`。terms 2/3に
  referenceなし、numeric request 0。
- Task 256 `2/0/0/0/0/0/0/4/4`：equality nodes 34/48はpairs
  `[0,1]` / `[5,6]`。complete witness chain `2 -> 3 -> 4`を全
  edge/requestからexclude。
- base `1/2/2/2/2`：theorem `53/19..120`、conclusion
  `51/104..115`、source ordinals 0/2、contexts 0/1。input factsは
  refs `[0,1]` / `[3,4]`。

five extraction rootsとseven primary rowsを分離し、atomic starts
`[0,5]`、input-fact reference starts `[0,3]`を固定する。witness/nameは
`1/0`。witness 0はowner 0、context 1、`Primary(2)`、take
`43/90..101`、item `42/95..100`、source ordinal 1、witness ordinal 0、
spelling `( ( x ) )`、normal recovery、nameなし。combined source order
`[0,1,2]`。

typed ownershipはnode 36 `source.term.variable-reference`、nodes 38/40
`source.term.parenthesized`、node 42 witness item、node 43 take、nodes
34/48 atomic equality、node 51 conclusion、node 53 theorem。
`TermExpression`/`FormulaExpression` surface wrappersはunowned。
Tasks 249–251/253–255/257はrows/ownershipなし。

public type/variant/field/error/table/accessor/producer/installer/route/detail
key/debug grammarを追加しない。existing Task-248/252/256/base、
witness/name tables、parent links、`Primary(2)`、paired typed/final ownershipで
十分。private profiles/exact selectorだけを追加する。dependency/
fingerprint/arena validationをaggregate cardinality、witness row 0、empty
name rowsより先に行う。standalone、hybrid、
stale、parent-chain/reference corruption、cross-family、
semantic-coexisting stateはpartial publicationなしでrejectする。

checker test contractはexactly 4本：

1. `task258b3m2b2a_exact_nested_parenthesized_witness_api_debug_and_compatibility_are_stable`;
2. `task258b3m2b2a_dependencies_parent_chain_witness_precedence_and_all_nodes_fail_closed`;
3. `task258b3m2b2a_paired_ownership_hybrids_and_all_family_orders_are_atomic`;
4. `task258b3m2b2a_final_clone_revalidation_and_semantic_deferrals_are_stable`.

test 2はvalid leaf reference 2がterm 4をtargetしたままouter wrapper
term 2へのextra reference rowを追加するmutationと、同じvalid leaf rowを
保持したままinner wrapper term 3へextra reference rowを追加するmutationを
separately rejectする。さらに`2 -> 3 -> 4` parent/referenceの
removal/remap/duplicate/detachとterms 2/3/4のTask-256 edge/request
contaminationを独立にrejectする。
test 3はprior statement profilesとTasks 253–255のboth ordersをcoverする。

runner test contractはexactly 5本：

1. `task258b3m2b2a_real_frontend_freezes_nested_parenthesized_witness_contract`;
2. `task258b3m2b2a_validation_precedence_mutation_and_replay_fail_closed`;
3. `task258b3m2b2a_selector_and_byte_subtree_near_misses_are_exact`;
4. `task258b3m2b2a_family_and_active_route_isolation_is_atomic_in_both_orders`;
5. `task258b3m2b2a_typed_final_clone_debug_rollback_and_empty_semantics_are_stable`.

121 bytes/57 nodes、resolver、five-root/seven-primary、two-parent chain、
reference/subtree ownership、独立corruptions、B3M2B1 same-label/exact-source
isolation、all prior family/active orders、rollback/replay/debug/final clone、
empty semanticsを固定する。near missesは`x`、`(x)`、`(((x)))`、
`((101))`、named/multiple、application/structure/selector/update/set/choice、
recovery、changed theorem/label、composite/existential roots。
theorem-proof `take it;`はauthority-invalid。

runner test 2はvalid term-4 leaf referenceを保持したboth extra-wrapper-
reference mutations、全parent/leaf-reference mutations、terms 2/3/4の
Task-256 mutationsをreal route constructionで試す。invalid rowから
Task-252/256 handoffを作れない場合はowning lower producerがfirst rejectし、
constructible handoffはpaired statement consumerまで到達する。

detail projectionは`Some(Vec::new())`、statements 2、lookups `1/1`、
reference uses `[1; 5]`。selector missは`None`、owned invalidは
`type_elaboration.checker.typed_ast_invalid`。Task 269はno-op、Task 272が
typing/existential matching/substitution/remaining goal/proof acceptanceを
保持し、formula truth/fact、Core/ControlFlow/VC/goal outputsはempty。

documentation prerequisiteはproduction/test source、canonical spec、
existing `.miz`、fixture、expectation、sidecar、trace row/status/count、
active route、test list/count/hashを変更しない。baselinesはlibraries
`366/404`、checker `17569/4661/7203/3156`、runner
`4676/695/2508/9902`、production 30 paths / 39,069 lines。checker
raw/normalized hashes
`0e43763c92ee171b18b5a2f80b92cd278b49ac9895d95410ca52ca787bcac3c8` /
`7685e21bc0d76bb8d824dd821e800707d251e8c025682ef69b2db798d6888e5d`、
runner
`a28c33e517d8efdd635e23e6f2273c29b966aa6102efb321eed73335ab11483c` /
`f8e8dc6ef605cbd8f8ad722983793434339b3cad21bf53703ab6c21f0b8742a5`、
production path/content
`98f3b264a59fed5b08c3e8f20e7ca58ff54efaa154eab16a7572a69ce923f275` /
`04bf563fcc99ccbc3b789a8596d953ade05453b2267639ef0ce3d8d54cbd6b45`。
plan/type `419/387`,`253/241`、pass/fail `228/191`、active
`101/5/198/1`、warnings/errors `23/0`、five CLI hashesは不変。

implementationはchecker/runner exactly `4/5`、libraries `370/409`をproject。
このprerequisiteはbroad-umbrella `design_drift`をcloseし、future codeは
bounded `source_drift`、future testsは`test_gap`。blocking `spec_gap`、
unsafe intent、lower-stage defect、`source_undocumented_behavior`、
`test_expectation_drift`、language/crate `boundary_violation`なし。
historical external-origin movementはreport-only `repo_metadata_conflict`で、
task/commit baseはunambiguous。

coverage auditはfollow-up ownershipだけ更新し、
`spec.en.checker.formula_statement.source_payloads`は`deferred`,
`tests = []`、creditなし。exitはEN/JA sync、independent no-findings
reviews、all hard gates、read-only quality >=90、task-only staging、
dedicated docs commit。implementationはcommit後のfresh
parser/resolver/lower/count/hash preflight後だけ開始する。

## Task 258B3M2B2A implementation result

checkerはone private `Task258B3M2B2A` profileをrecognizeする。exact
57-node arena、Task-48 `2/1/0`、Task-252 `7/5/0`とparent chain
`2 -> 3 -> 4` / refs `0/1/4/5/6`、Task-256 equality pairs
`[0,1]` / `[5,6]`、base `1/2/2/2/2`、one unnamed outer-term
witness/no names、source partition `[0,1,2]`をauthenticateする。
dependency/fingerprint/arena validationはaggregate、witness 0、empty-name
validationより先。wrapper refsとterms 2/3/4のindependent Task-256
contaminationはfail-closeする。

checker/runner tests 4/5がpass。libraries `370/409`、checker sizes
`19571/4662/7204/3156`、runner statement sizes
`5188/699/2513/11234`、production 30 paths / 39,590 lines。
raw/normalized test-list hashesは
`18cae89ddf8a5a21cca3741fd2c3e19a6d23b53c9ffe8e482dca63310445245c` /
`a1c328b0a1fef79df97b3fc5cb353dac8ac1ecc7a8477f27c11124de9f390d84`
および
`7e76d1de5b01b7a6fbe7fa8c88a8bffc3f957ec35a7d8a27cd456031d70d9299` /
`8eae5a5a084f0feeaba678c3b0aa11f47956c7f98946d7205b82984a8b5eb23a`。
production path hashは
`98f3b264a59fed5b08c3e8f20e7ca58ff54efaa154eab16a7572a69ce923f275`、
contentは
`291da8a26e90f75e7f54e221314c1fcb9ebba375c238a07b02a161f7af6dfe66`。

canonical artifact、fixture、expectation、sidecar、trace status/count、
active route、public API、binding、semantic/proof/goal ownerは変更しない。
formula-statement rowは`deferred`, `tests = []`、creditなし。
B3M2B2BがB4前のnext witness-term slice。

## Task 258B3M2B2B lower-owner decomposition

broad B3M2B2Bは最初にprivate Task-253 proof-context reuse seamである
Task 258B3M2B2B1Pへ依存する。seamは独立lower-stage logical taskであり、
witness targetへの`Application`追加、statement table install、
application/statement coexistence許可を行わない。B1P commitとfresh
inventory後、B3M2B2B1Aがexact imported-infix application-witness contractを
freezeできる。他のTask-253 forms、Task-254 structure
constructor/selector/update、Task-255 set/choice/qualification、remaining
compound termsはdeferする。

## Task 258B3M2B2B1P dependency completion

private Task-253 proof-context prerequisiteはcomplete/verifiedだが、
statement/witness tableはinstallしない。Task-258 behaviorと
application-to-witness ownership edgeは未実装のまま。fresh inventory後、
B3M2B2B1Aをseparate documentation taskとしてfreezeできる。他の
Task-253/254/255/compound shapesはdeferred。

## Task 258B3M2B2B1A application-witness ownership

B1AはB1Pのexact `take 1 ++ 2;` sourceだけをconsumeする。witness rowは
owner 0、proof context 1、source ordinal 1/witness ordinal 0、take
node/range `49/111..123`、witness-container node/range
`48/116..122`、spelling `1 ++ 2`、unnamed/normal/no-name、
`SourceStatementWitnessTermTarget::Application(SourceFunctorApplicationId(0))`
である。

parser containmentは`49 -> 48 -> 47 -> 46`だが、ownership edgeは
witness row 0からTask-253 application row 0だけ。node 48はowned witness
site、node 47はunowned transparent traversal node、node 46がapplication
target。
node 47はTask-253 wrapper/Task-252 primaryを作らない。Task 252はnumeral
arguments `Primary(2/3)`、Task 253はapplication/head/candidate/arguments/
requests、Task 256はsubtree全体をexcludeする。consumerはlower rowを
copy/retargetしない。

既存`#[non_exhaustive] SourceStatementWitnessTermTarget`へ
`Application(SourceFunctorApplicationId)` variantを追加する。immutable
handoffは`application_fingerprint: Option<String>`とread-only accessorを
追加し、legacy primary targetは`None`。既存`build(...)`はlegacy caller
について不変で、new dependencyなしのapplication targetをrejectする。
`build_with_application(...)`だけがsame input/base/primary/arenaに加えて
exact Task-253 handoffを受け、B1Aだけ`Some(application.debug_text())`を
produceする。

additive public signaturesは次でfreezeする。

```rust
pub fn application_fingerprint(&self) -> Option<&str>;

pub fn build_with_application(
    input: SourceStatementWitnessHandoffInput,
    statements: &SourceStatementHandoff,
    primary_terms: &SourcePrimaryTermHandoff,
    application: &SourceFunctorApplicationHandoff,
    arena: &TypedArena,
) -> Result<SourceStatementWitnessHandoff, SourceStatementWitnessError>;

pub fn with_source_application_statement_witnesses(
    self,
    application: SourceFunctorApplicationHandoff,
    statements: SourceStatementHandoff,
    witnesses: SourceStatementWitnessHandoff,
) -> Result<Self, TypedAstError>;
```

installationは(1) source application/fingerprintなしのlegacy profiles、
(2) exact application/matching fingerprint/`Application(0)`のB1Aだけ。
全Some/None、missing/orphan、wrong ID/context/range、stale fingerprint、
substituted candidate、cross-profile hybridをrejectする。dependency/
source/module/fingerprint/lower handoffをaggregate `1/0`、witness row 0、
empty namesより先にvalidateする。legacy
`source-statement-witness-debug-v1` bytesは不変。B1Aだけ
`application-fingerprint: Some(...)`と`term=application#0`を追加する。

sole TypedAst entry pointは
`with_source_application_statement_witnesses(application, statements,
witnesses)`で、complete validation後だけ3 handoffsをpublishする。既存
individual installersはpartial B1Aをrejectし続け、ResolvedTypedAstも同じ
bundleをrevalidate/clone-preserveする。Task 256はcombined validation時に
Task-253を見るが、equalities `[0,1]`/`[4,5]`だけを使いapplication
fingerprintは持たない。Task-253 consumerはwitness handoffだけ。

contractはsource provenance/ownershipで終了する。witness type、goal
matching、existential substitution、remaining goal、formula truth、proof
acceptance、Core/ControlFlow/VC、diagnostic/active behaviorはTask 272以降。
other application forms、application parentheses、Tasks 254/255、
named/multiple witnesses、broader proof shapesはlater B1B+。

exact checker testsは
`task258b3m2b2b1a_exact_application_witness_api_debug_and_legacy_compatibility_are_stable`、
`task258b3m2b2b1a_dependencies_application_witness_precedence_and_all_nodes_fail_closed`、
`task258b3m2b2b1a_combined_ownership_hybrids_and_all_family_orders_are_atomic`、
`task258b3m2b2b1a_final_clone_revalidation_and_semantic_deferrals_are_stable`
の4件。

## Task 258B3M2B2B1A implementation result

`SourceStatementWitnessTermTarget::Application(SourceFunctorApplicationId)`
とoptional application fingerprintをadditiveに実装した。
`SourceStatementWitnessProducer::build_with_application`はexact B1A
statement/application pairだけを受け、legacy builderは
application-free profilesだけを受けてdebug bytesを維持する。exact
143-byte/63-node profileはwitness node 48からtraversal node 47を経て
node 46のTask-253 applicationをtargetし、wrapper/primary duplicateを
追加しない。producerはreal imported symbol、local/FQN lookup、complete
contribution/path/export provenance、Task-252 arguments/requests、Task-256
equality-only exclusion、base `1/2/2/2/2`、unnamed witness 1/names 0、
dependency fingerprints 2件をauthenticateする。

上記checker tests 4件とrunner tests 5件はexhaustive byte/subtree/
provenance/dependency/precedence/family/rollback/replay/clone matricesを
passした。librariesは`374/416`、checker modulesは
`21664/4742/7224/3156`。semantic、proof、type、substitution、goal meaningを
inferせず、canonical、fixture、active、expectation、sidecar、trace
artifactは不変。

## Task 258B3M2B2B1B1P lower-owner deferral

fresh inventoryは`take (1 ++ 2);`を次のB1B1 statement shapeとして選ぶが、
B1B1Pがfreezeするのはmissing runner-private wrapped Task-253 reuse seam
だけ。158-byte/67-node motivating sourceはTask-252 `6/4/2`とTask-253
`1/1/1/2/2`を持つ。wrapper node 50は`129..137`、inner application
node 48は`130..136`をownし、future witnessは引き続き
`Application(0)`をtargetする。B1B1Pはnew statement/witness profile、
checker API、atomic installer、test、semantic behaviorをauthorizeしない。

documentation commitとlower-seam implementation commitの後、B1B1を
separateにfresh-inventoryしてfreezeしなければならない。他operator/
operandのparenthesized applications、nested wrappers/applications、
named/multiple witnesses、Task-254/255 witness terms、goal matching、
type obligations、substitution、proof acceptanceはdeferredのまま。

## Task 258B3M2B2B1B1P dependency completion

runner-private wrapped Task-253 prerequisiteはstatement/witnessをpublishせず、
exact tests 2件をpassしてcompleteした。future B1B1 consumerはapplication
0とwrapper containmentに対してfresh-inventory可能になったが、本completion
からB1B1 selector、checker row、installer、semantic behaviorを推定しない。

## Task 258B3M2B2B1B1 wrapped application-witness ownership

B1B1は`take (1 ++ 2);`を含むfinal-LF 158-byte/67-node sourceだけを
consumeする。B1B1P Task-253 application/wrapper handoffとexisting B1A
public application-witness schemaをreuseし、public type/method/table/
fingerprint grammarを追加しない。

exact containment pathは`take 53 -> witness 52 -> unowned 51 ->
wrapper 50 -> unowned 49 -> application 48`。Task 258はtake/witness
nodes 53/52とdirected `Witness(0) -> Application(0)` edgeをownする。
Task 253はwrapper/application 50/48をownし続け、wrapper 0はcontainment
metadataでtargetではない。Task 252はnumeral primaries 2/3をownし、
Task 256はsubtree全体をexcludeする。

owner 0はexact local theorem
`FormulaStatementParenthesizedApplicationWitnessSmoke`: site/range
`63/48..157`、label `56..108`、contribution 0、`LocalSource` anchor
`29..47`、checked origin `48..157`、structural path `[2,1]`、
public/exported/normal。resolver symbol、definition、label、contribution、
checked ownerは一致する。resolverはone import、witness-name symbolなし。

base rows:

| Row | Frozen value |
| --- | --- |
| statement 0 | owner/context `0/0`; `Atomic(0)`; site/range `63/48..157`; ordinal 0; `TheoremProposition`; normalized complete-theorem spelling |
| statement 1 | owner/context `0/1`; `Atomic(1)`; site/range `61/141..152`; ordinal 2; `Conclusion`; `thus x = x ;` |
| context 0/1 | statements 0/1; binding contexts 0/1; ranges `48..157` / `141..152`; visible binding `[0]` |
| input fact 0/1 | corresponding statement/context; ordinal 0; `ReservedTypeGuard`; binding 0; refs `[0,1] -> Primary(0/1)` / `[2,3] -> Primary(4/5)` |
| candidate fact 0/1 | corresponding statement/context; ordinal 0; `UnverifiedProposition`; `Atomic(0/1)` |

witness 0はowner/context `0/1`、source/witness ordinal `1/0`、
normal/unnamed/nameなし。takeはsite/range `53/124..138`、children
`[17,52,23]`。witnessはsite/range `52/129..137`、normalized spelling
`( 1 ++ 2 )`、child `[51]`、target `Application(0)`。theoremは
take/witnessをcontainし、conclusionはcontainせず、statement-witness
ownership kindsはnodes 53/52だけ。

lower tablesはTask-48 `2/1/0`、Task-252 `6/4/2`、wrapped Task-253
`1/1/1/2/2`、equality-only Task-256 `2/0/0/0/0/0/0/4/4`。
application 0はnode/range/context `48/130..136/1`、wrapper 0は
`50/129..137/1`、headは`20/132..134/++`、argumentsは
`Primary(2/3)`、candidate provenanceはexact imported
`parser.type_fixtures::++#12`、contribution 2、origin `7..27`、
path `[12]`、public/exported/signatureなし。

implementationはexplicit crate-private B1B1 statement/witness profileを
1件追加する。`SourceStatementWitnessTermTarget::Application`、
`SourceStatementWitnessProducer::build_with_application`、
`TypedAst::with_source_application_statement_witnesses`をreuseし、B1Aを
broadenしない。validationはselector/owner、lower dependencies/
fingerprints、aggregate、全base rows、witness、empty names、atomic typed
install、final revalidationの順。failureはatomic、clean replayは
byte-identical。

crate plan記載のexact checker tests 4件/runner tests 5件で、158 bytes、
67 nodes+root全fields、resolver substitutions 5件、eight-entry reparse
matrix、B1A compatibility、family/active-route isolation、validation
precedence、rollback/replay/clone、empty downstream semanticsをcoverする。
type checking、goal matching、substitution、proof acceptance、Task-254/255
forms、他application/witness shapesはdeferする。

## Task 258B3M2B2B1B1 implementation result

private B1B1 profileはexact 158-byte/67-node owner、base statements、one
unnamed application witness、lower fingerprints、wrapper containmentを
authenticateする。checker tests 4件/runner tests 5件は、mutation
precedence、B1A/family/active isolation、rollback/replay、final cloneを含め
全てpass。bounded `source_drift`/`test_gap`はclosedで、semantic、proof、
goal、type-substitution behaviorは追加していない。

## Task 258B3M2B2B2P statement-owner deferral

B2PはTask-258 owner、statement、context、input/candidate fact、witness、
witness name、typed coexistence row、final-statement profileをpublishしない。
このlower seamのimplementation/fresh inventory後、future B2A contractだけが
`SourceStatementWitness(0) -> SourceStructureTerm(0)`を追加できる。
したがってtake node 62/witness node 61はB2Pではunowned、transparent node
60はexcluded、constructor node 59はTask-254-ownedのまま。witness
obligation、substitution、proof/fact acceptance、goal dischargeはdeferする。

## Task 258B3M2B2B2P statement-owner result

B2PはTask-258 owner、statement、context、fact、witness、witness name、
coexistence row、final-statement profileをpublishせずにimplementした。
take 62、witness 61、transparent term 60はB2P-unownedのまま。fresh
inventoryでは、directed
`SourceStatementWitness(0) -> SourceStructureTerm(0)` edgeのsole next
ownerをB2Aとする。

## Task 258B3M2B2B2A frozen structure-witness ownership

B2Aは172-byte sourceのexact Task-258 theorem owner/base rowsとunnamed
witness 1件をcomposeする。base countsは`1/2/2/2/2`、witness/nameは
`1/0`。owner row 0はtheorem node/range `72/48..171`、spelling
`FormulaStatementStructureConstructorWitnessSmoke`、role/status
`Theorem/Unmodified`、normal recovery。statement 0はowner/context
`0/0`、`Atomic(0)`、`72/48..171`、ordinal 0、
`TheoremProposition`、normal recovery、literal normalized spelling
`theorem FormulaStatementStructureConstructorWitnessSmoke : x = x proof take TypeCaseStruct ( x : 1 , y : 2 ) ; thus x = x ; end ;`。
statement 1はowner/context `0/1`、`Atomic(1)`、`70/155..166`、
ordinal 2、`Conclusion`、normal recovery、literal spelling
`thus x = x ;`。contexts 0/1はstatements/binding contexts 0/1、
corresponding ranges、visible `[0]`。input facts 0/1はcorresponding
statement/context、ordinal 0、`ReservedTypeGuard`、binding 0、exact
reference uses `[0,1]` / `[2,3]`。candidate factsはordinal 0、
`UnverifiedProposition`、`Atomic(0/1)`。

witness 0はcontext 1、ordinal `1/0`、take `62/120..152`、item
`61/125..151`、normalized spelling
`TypeCaseStruct ( x : 1 , y : 2 )`、unnamed/no nameで、`Structure(0)`
だけをtarget。base transactionはtheorem/conclusion rows 72/70、B2A
extensionはtake/witness 62/61とdirected edgeをownする。

frozen additive schemaは
`SourceStatementWitnessTermTarget::Structure(SourceStructureTermId)`、
`structure_fingerprint(&self) -> Option<&str>`、canonical checker planの
full `build_with_structure`/`with_source_structure_statement_witnesses`
signatures。fingerprint pairsはlegacy `(None,None)`、application
`(Some,None)`、B2A `(None,Some)`、`(Some,Some)`はinvalid。
conditional structure fingerprint lineはexisting application positionの後、
B2Aだけ`term=structure#0`。legacy primary/application
builders/fingerprints/debug/installersはexact。Task 256は`Structure` edge/
fingerprintなしで、combined typed/final pathsだけが`Some(&structure)`で
revalidateする。

contractはsource provenance/ownershipで終了し、existential matching、
type obligation、substitution、remaining goal、formula truth、proof fact、
theorem acceptanceはpublishしない。

## Task 258B3M2B2B2A structure-witness result

syntax-free producerはexact B2A profileをenumerateし、authenticated
Task-258 base上のunnamed `Structure(0)` witness 1件をpublishする。
`build_with_structure`はexact structure fingerprintをstoreし、legacy/
application pairsは`(None,None)`/`(Some,None)`のまま、hybridはreject。
debugはB2Aだけconditional structure fingerprintと`term=structure#0`を
renderする。

checker 4 testsがpublic API、base/witness rows、dependency substitutions、
fingerprint isolation、atomic installation、final cloneをvalidateする。
`source_statement.rs`は27,194 lines。
existential matching、obligation、goal、proof fact、theorem acceptance、
active route、coverage creditは追加しない。

## Task 258B3M2B2B2BP lower-prerequisite exclusion

B2BPはstatement-witness profileではない。171-byte selector sourceは
private Task-254 proof-context reuse seamだけをmotivateし、Task 258は
theorem/statement/context/fact/witness/name rowをownしない。
`SourceStatementWitnessTermTarget`、fingerprint、builder、TypedAst
installer、final clone rule、public API、debug grammarは変更しない。

later B2B consumerはB2BP separate implementation commit後だけexact
witness-to-selector edgeをownできる。B2C functional-update/`FieldUpdate`、
selector identity/type、proof/goal/theorem semanticsはdeferred。

B2BP private lower seamはfrozen prerequisite後にimplementedとなった。
statement witnessのinstallやchecker API変更はない。fresh dependency
inventoryは次のseparate logical taskとしてB2Bをfreezeできる。B2Cと
semantic/proof/goal behaviorはdeferred。

## Task 258B3M2B2B2B frozen structure-selector witness contract

exact 171-byte、final-LF sourceはexisting Task-258 base tableへ
syntax/provenance-only witness edgeを1件追加する。baseは`1/2/2/2/2`の
まま。theorem site node `75`、conclusion statement node `73`、context
`1`からcontext `0`がvisible、input-fact references `[0,1]`/`[2,3]`、
atomic statements `0/1`がcandidates。witness tableはexact `1/0`。
`take` node `65`がwitness expression node `64`をownし、selector spelling
をpreserve、nameなし、Task-254 `Structure(0)`だけをtargetとする。
selector base `Structure(1)`はlower-stage childで、second witness target
ではない。

exact consumersは`SourceStatementProducer`、
`SourceStatementWitnessProducer::build_with_structure`、combined TypedAst
installer、
final-AST clone path。checker implementationの変更範囲は
`source_statement.rs`、`typed_ast.rs`、`resolved_typed_ast.rs`だけで、
public API/debug grammarは追加しない。Task 252 owns
`47/49/55/58/66/68`、Task 254 owns `62/61/29/20/24`、Task 256 owns
`51/70`、Task 258 base owns `75/73`、B2B owns nodes `65/64`と
witness-to-`Structure(0)` edgeだけ。formula containers `52/71`と
private numeric roots `56/59`はunownedのまま。

required checker tests 4件:

- `task258b3m2b2b2b_exact_structure_selector_witness_api_debug_and_legacy_compatibility_are_stable`
- `task258b3m2b2b2b_dependencies_structure_selector_witness_precedence_and_all_nodes_fail_closed`
- `task258b3m2b2b2b_combined_ownership_hybrids_and_all_family_orders_are_atomic`
- `task258b3m2b2b2b_final_clone_revalidation_and_semantic_deferrals_are_stable`

canonical `take` semanticsはexistential goalをrequireするが、このsmoke
sourceのconclusionは`x = x`。よって本taskはsyntax/provenanceだけを
authenticateする。existential matching、proof facts、obligations、goals、
theorem acceptance、selector identity/type/result、B2C functional update、
`FieldUpdate`はdeferred。

## Task 258B3M2B2B2B implementation result

source-statement producerはexact 171-byte/79-node B2B profileを
authenticateし、base `1/2/2/2/2`とunnamed witness 1件/names 0件を
publishする。witness 0はtake/item nodes `65/64`をownし、Task-254
selector `Structure(0)`だけをtargetとする。constructor
`Structure(1)`、members、roots、primaries、applications、transparent
containersはwitness targetではない。Task-256 nodes `51/70`はowned、
containers `52/71`はunownedのまま。

existing structure-aware builder、fingerprint、atomic installerをpublic
API growthなしでreuseする。frozen checker 4 testsはexact dependency
precedence、all-node failure、B2A/B2B hybrid rollback、final-clone
revalidation、empty semantic deferralsを含めてPASS。
`source_statement.rs`は29,941 lines。selector meaning、proof/goal effect、
theorem acceptance、B2C update/`FieldUpdate`、corpus active-route status、
trace creditは追加していない。

## Task 258B3M2B2B2CP statement-owner deferral

B2CPはstatement-witness profileではない。181-byte functional-update
sourceはprivate Task-254 proof-context reuse seamだけをmotivateする。
このprerequisite/implementation中、Task 258はtheorem、statement、
context、fact、take、witness、name、directed witness targetをownしない。
`SourceStatementWitnessTermTarget`、fingerprint、producer、TypedAst
installer、final-clone rule、public API、debug grammarは変更なし。

B2CP implementationをseparately commitした後だけ、B2Cはsame sourceを
fresh-inventoryし、theorem 82/conclusion 80のtheorem/statement/context/
fact rowsとlocal owner/labelを含むcomplete Task-258 base transaction/
provenanceをfreezeする。そのexact countsはB2CPではfreezeしない。
B2C witness extensionがownできるのはtake/witness nodes `72/71`とexact
witness-to-functional-update `Structure(0)` edgeだけ。Task 256がlater
ownするのはnodes `55/77`だけで、formula containers `56/78`はunowned、
formula tableはupdate subtreeをexcludeする。update/member/
`FieldUpdate` semantics、replacement/result typing、functional-copy
meaning、existential obligations/substitution、proof、goal、theorem
acceptanceはdeferred。smoke theoremのgoalは`x = x`なので、この`take`
occurrenceはsemantic acceptance claimを供給しない。

## Task 258B3M2B2B2CP implementation result: statement surface unchanged

CPC1 correction commit `ee267d9c`はcompleteし、B2CP private lower reuse
seamはimplemented。frozen runner tests 2件がPASSし、prerequisite
`design_drift`、bounded `source_drift`、`test_gap`をclose。final
test-sufficiency/implementation re-reviewsはfindingsなし。
source-statement production/testsはunchangedで、B2CPはTask-258 statement、
witness、name、target edge、fingerprint、TypedAst/final row、public API、
active routeをpublishしない。

specification、`.miz`、fixture、expectation、sidecar、trace status/count/
backlink/credit、semantic behaviorは変更なし。formula rowは`deferred`、
`tests = []`、coverage impactはnarrative-only。functional-copy/update
meaning、type/result identity、B2C ownership、proof/goal/theorem acceptance、
IRはdeferred。concurrent ownershipはreport-only
`repo_metadata_conflict`でmetadata repairなし。fmt、Clippy、tests、
全count/hash gatesはPASS。final source/documentation re-reviewは
findingsなし。independent final qualityはfindingsなし、全9 hard gates
PASS、valid `98/100`。B2CP implementation commit
`b146f0f72dceac2233c9d679b7820e264974b227`はcomplete。

## Task 258B3M2B2B2C frozen statement/witness contract

exact 181-byte/86-node B2C sourceはTask-258 base
`1 owner / 2 statements / 2 contexts / 2 input facts / 2 candidate facts`
と`1 witness / 0 names`をpublishする。local contribution 0は
`LocalSource` reserve anchor `29..47`、checked owner originは
`48..180/[2,1]`、label `56..99`はpublic/exported/normal。owner 0は
theorem site 82、spelling
`FormulaStatementStructureUpdateWitnessSmoke`、role/status
`Theorem/Unmodified`、normal recovery。

statement 0はowner/context `0/0`、`Atomic(0)`、site/range
`82/48..180`、ordinal 0、kind `TheoremProposition`、normalで、
`theorem FormulaStatementStructureUpdateWitnessSmoke : x = x proof take TypeCaseStruct ( x : 1 , y : 2 ) with ( x := 3 ) ; thus x = x ; end ;`
とspellする。statement 1はowner/context `0/1`、`Atomic(1)`、conclusion
site/range `80/164..175`、ordinal 2、kind `Conclusion`、normalで、
`thus x = x ;`とspellする。context rows 0/1はstatements 0/1をnameし、
binding contexts 0/1、copy ranges `48..180` / `164..175`、visible `[0]`。
input facts 0/1はbinding 0、references `[0,1]` / `[2,3]`のordinal-0
`ReservedTypeGuard` rows。candidate facts 0/1は`Atomic(0/1)`をtargetする
ordinal-0 `UnverifiedProposition` rows。

witness 0はowner 0/context 1、source/within-take ordinals `1/0`、
unnamed/normal/nameless。takeは`72/115..161`、itemは`71/120..160`、
spellingは`TypeCaseStruct ( x : 1 , y : 2 ) with ( x := 3 )`で、sole
targetはfunctional-update `Structure(0)`。exact B2CP structure debug
fingerprintをrecordし、application fingerprintはなく、
`term=structure#0`とrenderする。transparent 70、constructor
`Structure(1)`、root/member/`FieldUpdate`、primary、application、
formula、container rowsはwitness targetではない。

Task-256 formula 0はnode/range `55/101..106`、ordinal/context `0/0`、
formula 1は`77/169..174`、ordinal/context `1/1`。両方ともnormal equality
`x = x`で、edges/requestsはexactに`Primary(0/1)` / `Primary(5/6)`を
targetする。direct structure targetもstructure fingerprintもなく、
update subtree全体をexcludeする。

existing public `Structure` target、structure fingerprint、structure-aware
producer、combined TypedAst installer、final cloneをunchangedでreuseする。
future checker tests exactly 4件はAPI/debug/legacy compatibility、
dependency precedence/all-node failure、combined ownership/family-order
atomicity、final-clone/semantic deferralsをfreezeする。future runner tests
exactly 5件はreal frontend、corruption/replay、update/byte/subtree
near-miss、family/active-route isolation、typed/final/debug/empty-semantics
behaviorをfreezeする。validation順序はexact source/arena、local/imported
provenance、Tasks 48/252/254/256、Task-258 base、witness、atomic
publication、final clone。goal `x = x`のため、このsourceはtransport
onlyであり、statement semanticsはすべてdeferred。

## Task 258B3M2B2B2C implemented statement/witness contract

statement producerはexact B2C Task48/252/256/base profileをrecognizeし、
witness producerはexisting Task254 functional-update handoffもauthenticate
する。proof contextのunnamed witness 1件だけをpublishし、sole term targetは
`Structure(0)`、debug spellingは`term=structure#0`。name row、public
table/API、reverse edge、fact、obligation、proof result、goal progress、
theorem acceptance追加なし。

frozen checker 4 tests/runner 5 testsはPASS。final test-sufficiency/
implementation reviewsはfindingsなしで、final source/docs/quality reviewsは
pending。

## Task 258B3M2B2B2C broad statement verification

broad fmt/Clippy/crate/workspace gates、focused `4/4`/`5/5`、sibling
`12/12`/`21/21` suitesはunchanged counts/hashesでPASS。statement/witness
transport contractの変更やsemantic credit追加は不要。independent final
source/docs/quality reviews、commit、post-commit inventoryはpending。

## Task 258B3M2B2B2C final statement review status

independent final source/docs consistency/final qualityは**NO FINDINGS**。
全9 hard gates PASS、valid `98/100`。exact statement/witness evidenceと
semantic deferralsはunchanged。pendingはcached-diff/staging audit、
implementation commit、post-commit inventory/fresh-next-task gatesだけ。

## Task 258B3M2B2B3P statement-owner deferral

B2C implementationは
`e8373c683448e524cb98edde83fdf8de83a125cd`としてcommit済みで、
post-commit invariantsはclean。次のlower prerequisite B3Pは117-byte
set-enumeration sourceのproof context 1にあるTask-255 enumeration term 0、
node/range `40/90..96`だけをauthenticateする。`SourceStatement`、
`SourceStatementWitness`、statement-to-term edge、checker API/testはownしない。

theorem statement、`take` witness、proof、全containersはB3Pではunowned。
upper B3Aはseparate future logical taskで、`SourceStatementWitness ->
SetTerm(0)`、public witness schema/installers、checker 4/runner 5 testsを
freeze/implementできる。B3Pはそのedgeを先取りせず、witness、
existential、substitution、type、goal、proof、theorem semanticsをclaimしない。

## Task 258B3M2B2B3P documentation review status

documentation phaseの4 review tracksはすべて**NO FINDINGS**で、record済み
source/count/hash/scope/trace-no-op verificationはすべてPASS。B3Pの
statement-owner exclusionをconfirmし、later B3A consumerはcloseしない。
future B3P implementation `source_drift`/`test_gap`はplannedで、
final quality、commit、post-commit、fresh inventoryはpending。

## Task 258B3M2B2B3P final quality status

final qualityは**NO FINDINGS**、全9 hard gates PASS、valid `98/100`
（`20/20/15/14/10/10/5/4`）。pendingはstage/commit、post-commit、
fresh implementation inventoryだけ。

## Task 258B3M2B2B3P implemented statement-owner exclusion

prerequisite `285a1f11c310bb313c4c6b4feae914eb11f74754`のprivate lower
implementationはTask-252/Task-255 set-enumeration transportだけをpublish。
statement/witness/proof/theorem/term-expression containersはunownedのまま、
`SourceStatementWitness -> SetTerm(0)` edgeもstatement semantic rowもない。
exact 2 testsがactive/adjacent profilesを含めこのexclusionを固定。

lower B3P `source_drift`/`test_gap`はupper B3A ownershipを消費せずclosed。
次dependencyはB3A。test-sufficiency/implementation reviewsは
**NO FINDINGS**。source/docs consistency repeatとdocumentation/boundary
repeatも**NO FINDINGS**。lint-policy `15/14`、metadata `137`、
focused/library/fmt、workspace Clippy/tests、CLI/manifests/test-list hashes、
diff、exact30 scopeはPASS。independent final qualityは**NO FINDINGS**、
全9 hard gates PASS、valid `98/100`（`20/20/15/14/10/10/5/4`）。
pendingはcommit/post-commit、fresh B3A inventoryだけ。

## Task 258B3M2B2B3A frozen source-statement witness contract

source ownerは`53/19..116`、local-only/public-exported、label spelling
`FormulaStatementSetEnumerationWitnessSmoke` at `27..69`、reserve anchor
`0..18`、origin `19..116/[2,1]`、`LocalSource` contribution 0、
import/recoveryなし。B3Aはresolver labelと`CheckedStatementOwner`を
fresh-authenticateし、B3P empty-label oracleで代替しない。

Task258 baseはowner1、statements/contexts/input facts/candidates各2。
statement0はowner/context `0/0`、`Atomic(0)`、`53/19..116`、ordinal0、
`TheoremProposition`、normalized
`theorem FormulaStatementSetEnumerationWitnessSmoke : x = x proof take { 1 , 2 } ; thus x = x ; end ;`。
statement1は`0/1`、`Atomic(1)`、`51/100..111`、ordinal2、
`Conclusion`、`thus x = x ;`。binding contexts0/1は`[0]`、
statement context rangesは`19..116`/`100..111`、
`ReservedTypeGuard` refs `[0,1]`/`[2,3]`、candidates `Atomic(0/1)`。

B3Aはwitness0/names0だけ：owner/context `0/1`、source/take ordinals
`1/0`、take `43/85..97`、item `42/90..96`、spelling `{ 1 , 2 }`、
unnamed `Normal`、target `SetTerm(0)`。APIはexact：

```rust
SourceStatementWitnessTermTarget::SetTerm(SourceSetTermId)

pub fn set_term_fingerprint(&self) -> Option<&str>;

pub fn build_with_set_term(
    input: SourceStatementWitnessHandoffInput,
    statements: &SourceStatementHandoff,
    primary_terms: &SourcePrimaryTermHandoff,
    set_terms: &SourceSetTermHandoff,
    arena: &TypedArena,
) -> Result<SourceStatementWitnessHandoff, SourceStatementWitnessError>;

pub(crate) fn validate_installation_with_set_term(
    &self,
    source_id: SourceId,
    module_id: &ModuleId,
    statements: &SourceStatementHandoff,
    primary_terms: &SourcePrimaryTermHandoff,
    set_terms: Option<&SourceSetTermHandoff>,
    arena: &TypedArena,
) -> Result<(), SourceStatementWitnessError>;
```

accepted application/structure/set fingerprint tupleは
`None/None/None`、`Some/None/None`、`None/Some/None`、B3A
`None/None/Some`で、他は全てinvalid。debugはexisting optional fields後に
set fingerprint、target `set-term#0`。producer/handoff validationは
`SourceStatementWitnessError::DependencyMismatch`、typed installationと
final statement/witness revalidationは既存`InvalidSourceStatement` variantを
保持。先行final lower-stage failureは`InvalidSourceSetTerm`、
`InvalidSourceAtomicFormula`を含むowner variantを保持。error variant/
display/legacy bytesは変更しない。

checker4+runner5 testsはbytes/LF、57 AST nodes/root、resolver label、
Tasks48/252/255/256/258、partition/graph、fingerprints、set/label near
misses、family hybrids/orders、rollback/replay、final clone、empty semantics
をexhaust。precedenceはsource/AST、resolver+label、Tasks48、252、255、
256、258 base、witness、atomic publication、final clone。

source transportだけで、result/numeric/set/element typing、existential
goal matching、witness guards/obligations、substitution、goal progress/
discharge、proof/theorem acceptance、facts、overload/coercion、Core/CFG/VC、
imported set、broader forms、B4/B5、active/corpus/diagnostic creditはdeferred。

## Task 258B3M2B2B3A implemented source-statement witness closure

source-statement ownerはexactly `SetTerm(SourceSetTermId)`、optional set
fingerprint/getter、`build_with_set_term`、crate-private set-aware
installation seamを提供する。exact witness1/names0 profileはresolver label、
Tasks48/252/255/256/258、全`57` nodes/root、ownership partition、sole
witness-to-set edgeをauthenticateする。checker4+runner5 testsはfrozen
mutation、fingerprint tuple、near miss、family order、rollback/replay、
final clone、empty semanticsをcoverする。specification/test-sufficiency/
implementation reviewsは**NO FINDINGS**。semantic deferralsは全て維持し、
2回目のsource/documentation consistency repeatとfinal documentation/
boundary rereadも**NO FINDINGS**。crate plans記載のparent final
verificationはexact `39`-file scopeを含めPASS。independent final
read-only quality reviewは**NO FINDINGS**。全9 hard gates PASS、score
capなし、valid `98/100`（`20/20/15/14/10/10/5/4`）。記載済み
semantic/coverage deferralsはunchanged residual risk。pendingは
dedicated implementation commit、postcommit invariant verification、
fresh next-task inventoryだけ。

## Task 258B3M2B2B3B frozen empty-enumeration statement contract

B3Aは`a147bad88f1963c504f796051ba0b855eca71d07`でclosedした。generic
SetTerm carrierによってexact `{ 1 , 2 }` statement profileが`{}`をaccept
することにはならない。そのためB3Bは118-byte
`FormulaStatementEmptySetEnumerationWitnessSmoke` source、diagnostics 0、
50 nodes/root 49、local public/exported theorem `46/19..117`、label
`27..74`、proof context `1` at `82..116`をfreezeする。

Task 252はreference roots `{27,29,37,39}`をownする。Task 255はempty
enumeration `33/95..97`、spelling `{ }`、profile
`1/0/0/0/0/0/1`、child edge 0件、`ResultType` request 1件だけをownする。
Task 256はformula roots `{31,41}`、Task-258 baseは`{44,46}`をownする。
B3Bはwitness/take `{35,36}`とsole
`Witness(0) -> SetTerm(0)` edgeをownする。他の全nodesはcrate planで
freezeしたとおりunownedである。

B3BはB3A SetTerm APIとset-only fingerprint tupleをreuseする。future
private exact profileはpublic schema/debug grammarを変更しない。
checker4+runner5 matrixは全bytes/nodes/resolver/lower rows、zero-edge
nonvacuity、precedence、family isolation、replay/rollback、final clone、
empty semanticsをfreezeする。singleton/nonempty enumeration、choice、
comprehension、`qua`、named/multiple witnesses、semantic typing、
existential/proof behavior、B4/B5、active/trace creditはdeferredのままで
ある。

## Task 258B3M2B2B3B implemented statement profile

private selectorはall 118 bytes、50 nodes/root 49、resolver provenance、
Tasks 48/252/255/256/258 rows、zero-edge graph、one unnamed witness、
family isolationをauthenticateしてからB3A SetTerm-aware routeへpublish
する。8 base-resolver mutations、両方向family order、non-vacuous
zero-edge corruptionを含むchecker 4 / runner 5 testsがfail-closed
precedence、replay/rollback、final clone、empty semanticsをcoverする。
public schema/error/debug、active route、semantic/trace creditはunchanged。
initial 3 findingsとrepeat reviewのcurrently mutable Task-48/252/255
mutation/replay gapはremediatedし、後者はexact `32/55/23` matricesを持つ。
post-auth injectionとstage-prefix/non-generic-guard assertionsが
authenticationをcompleteした。全test-sufficiency repeatsとfinal
implementation repeatは**NO FINDINGS**。source/documentation
consistency repeat、final documentation/boundary、independent qualityも
**NO FINDINGS**。全9 protocol hard gates PASS、score capなし、valid
`98/100`（`20/20/15/14/10/10/5/4`）である。

## Task 258B3M2B2B3C frozen statement profile

exact `110`-byte/`52`-node choice sourceはtheorem owner 1、
statement/context/guard/candidate各2、unnamed witness 1/name 0を持つ。
Task 258 ownershipは`{48,46}`、B3Cは`{38,37}`、witness
`37/82..89`はproof context `1`の`SetTerm(0)`をtargetにする。
complete cross-family graph/owner partitionはcrate planでfreeze済み。
implementationはexisting SetTerm target/fingerprint/install/clone APIを
reuseしpublic/error/debug/semantic surfaceを追加しない。exact checker 4 +
runner 5 testsと`32/55/39/72/62/21` matricesをfreezeした。

## Task 258B3M2B2B3C implemented choice statement

source-statement producerはexact 110-byte、52-node/root-51
`take the set;` profileだけをacceptし、target `SetTerm(0)`のunnamed witness
1件をinstallする。syntax-free handoff publish前にcomplete
Task-48/252/255/256/258 tables、exact owner partition、local resolver
provenance、Task-255 edge 0、set fingerprint、choice/witness subtree
exclusionをvalidateする。

frozen checker 4 + runner 5 testsは全bytes/final LF、`52 x 4` node
surfaces/root、resolver mutation、`32/55/39/72/62/21` fields、family
orders、immediate replay、clone/rollback/debug、empty semanticsをexercise
する。resolver replayとexact upper stage prefix/non-generic rejectionで
initial medium `test_gap` 2件をcloseした。B3A-hard-coded branchは両
enumeration siblingをretainしてB3Cへ限定し、`source_drift`/`test_gap`を
closeした。repeat reviewsは**NO FINDINGS**。public API/error/debug grammar/
semanticsはunchanged。new private dormant exact selector branchをactive
corpus sourceはselectしないため、existing active-corpus routing/outcomeは
unchanged。

## Task 258B3M2B2B3D frozen qua statement profile

exact 109-byte/54-node qua sourceはtheorem owner 1件、statement/context/
guard/candidate rows各2件、unnamed witness 1件/name 0件をcontributeする。
Task 258は`{50,48}`、B3Dは`{40,39}`をownし、`39/79..88`のwitnessは
proof context `1`の`SetTerm(0)`をtargetする。Task 255は
`{35,36,37}`と`QuaBase -> Primary(2)` edgeをownし、complete
owner/unowned graphはcrate planでfreezeする。existing SetTerm
fingerprint、producer、typed install、final replay、error、debug APIsを
reuseする。checker 4 + runner 5 testsと`32/70/44/72/62/21` matricesを
freezeする。

## Task 258B3M2B2B3D implemented qua statement profile

private exact selectorは109-byte/54-node source、local theorem/label
provenance、Task-48/252/255/256/258 rowsをauthenticateし、proof context
`1`のunnamed witness 1件を`SetTerm(0)`へpublishする。ownershipはfrozen
partitionのまま、graphはformula-to-primary、`QuaBase` set-to-primary、
witness-to-set edgesだけを含む。existing `SetTerm` target/fingerprint/
producer APIをreuseし、legacy/application/structure siblingsを変更しない。

checker tests 4件は全nodes、dependencies、hybrids/24 family orders、final
clone/semantic deferralsをcoverし、全field matricesは
`32/70/44/72/62/21`。focused `4/4`、checker package `406+15`はPASSし、
test-sufficiency reviewは**NO FINDINGS**。statement moduleは`41452`
lines、raw/normalized test-list hashesは
`11a6c54d3b0190c5b929565bf264dd4170c1e02d66c957c47e308764ec6f4b09` /
`d50e4a826c3cfe8d482f04e0dc4819af92d9f3ce9b7fb8ff52bf2f62f3378081`。
independent implementation reviewは**NO FINDINGS**。source/docs
consistencyとboundary repeatも24-order/qua-edge/review-state wording修正後に
**NO FINDINGS**。package/fmt/full Clippy/workspace tests/5 CLI/count/hash
final rerunsはPASS。

independent final read-only quality reviewは**NO FINDINGS**、全9 hard
gates PASS、no cap、valid `100/100`
（`20/20/15/15/10/10/5/5`）。CLI `23/0` warnings/errorsとlarge
repeated-test diff review volumeはnonblocking residual。残るのはexact
staging/cached diff、implementation commit、post-commit/fresh-nextだけ。

## Task 258B3M2B2B3E frozen comprehension statement profile

exact sourceはfinal-LF 139 bytes、28 tokens、60 nodes/root 59である。
Task-258 baseはone theorem resolver owner、two statements、two atomic
inputs、two candidates、profile `1/2/2/2/2`を保持する。theorem statement
node/range `56/19..138`はcontext `0`、source ordinal `0`、Atomic `0`、
conclusion node/range `54/122..133`はproof context `1`、source ordinal
`2`、Atomic `1`である。local theorem labelはnode/range
`6/27..68`、origin path `[2, 1]`、reserve contribution anchor
`0..18`である。

B3Eはone unnamed normal witness、zero namesを追加する。
`TakeStatement(46/84..119)`、`Witness(45/89..118)`、owner statement
`0`、proof context `1`、source ordinal `1`、witness ordinal `0`、
spelling `{ 3 where candidate255 is set }`、target `SetTerm(0)`である。
Task-255 set termはnode/range `43/89..118`、kind
`Comprehension`、generator node `16`、type expression/head `41/40`、
condition 0件、`ComprehensionMapper -> Primary(2)`、
ordered `GeneratorSethood`/`ResultType`を保持する。

complete ownershipはTask-252 `{32,34,38,47,49}`、Task-255
`{16,40,41,43}`、Task-256 `{36,51}`、Task-258 `{54,56}`、B3E
`{45,46}`。generator segment `42`とtransparent term nodes `39/44`は
unownedである。statement graphが追加するupper edgeは
`Witness(0) -> SetTerm(0)`だけで、condition/formula edgeまたはgenerator
binding/referenceをsynthesizeしない。

future checker tests 4件はexact `32/70/53/72/62/21` field matrices、
all nodes、resolver provenance、dependency/generator validation precedence、
ownership/hybrids、five-family `120` orders、final clone/revalidation、
empty semantic deferralsをcoverする。private selector/build pathはexisting
`build_with_set_term`とset-only fingerprintをreuseし、public API/error/
debug grammarを変更しない。documentation-only test-sufficiencyと
implementation-boundary reviewsは**NO FINDINGS**である。future source
implementation/test reviewはseparate implementation taskに残す。

## Task 258B3M2B2B3E implemented comprehension statement inventory

private selectorはexact final-LF 139-byte/60-node sourceとtheorem/label
provenanceをauthenticateし、exact Task-48/252/255/256/258 tupleから
witness/take `{45,46}`、target `SetTerm(0)`のunnamed witness 1件をpublish
する。existing public witness DTO/producerをreuseし、new profile/validatorは
privateである。

checker 4 testsはexact/legacy、stage precedence/all nodes、ownership/120
orders、clone/deferralsをcoverする。matrices、coherent near miss、
non-generic guard、repeated failure、clean replayがPASSする。witness
matching、substitution/goal、proof/facts、Core/CFG/VC、B4/B5、active routeは
追加しない。

bounded design correction後のfinal source/docs consistencyは
**NO FINDINGS**、full verificationはPASSした。independent final qualityは
**NO FINDINGS**、全9 gates PASS、valid `100/100`。staging/post-commit
gatesはimplementation commit
`e4479691db3b0a8785bb16e94d386bd71a394274`でcloseし、fresh inventoryは
Task 258B4Aをselectした。

## Task 258B4A frozen composite statement root

B4Aはprivate 80-byte/double-LF explicit-universal theoremと、既に認証済みの
Task-257B1 composite `1/0/1/1/1/0/2` + composition `1/2`をconsumeする。
theorem owner 1、theorem statement 1、context 1、input fact 0、unverified
candidate 1をpublishする。statement/candidateはどちらも
`SourceStatementFormulaTarget::Composite(SourceCompositeFormulaId::new(0))`
をtargetとし、context 0はbinding context 0とempty visible-binding setを
保持する。

owner 0はchecked local theorem symbol、contribution `0`、Surface site
`22` / range `0..78`、label spelling
`FormulaQuantifierBoundUsePayloadBoundary`、
`Theorem` / `Unmodified` / `Normal`である。statement 0はowner/context
`0`、site `22` / range `0..78`、ordinal `0`、
`TheoremProposition` / `Normal`、normalized spelling
`theorem FormulaQuantifierBoundUsePayloadBoundary : for x being set holds x = x ;`
である。context 0はstatement 0、binding context 0、range `0..78`、
visible `[]`。candidate 0はstatement/context/ordinal `0`、
`UnverifiedProposition`、`Composite(0)`である。

`SourceStatementFormulaTarget`には`Composite` variantだけを追加する。
`SourceStatementHandoff`はoptional composite-formula/formula-composition
fingerprintを保持し、
`composite_formula_fingerprint(&self) -> Option<&str>` /
`formula_composition_fingerprint(&self) -> Option<&str>`で公開する。
existing atomic routeは両方absentのままbyte-identical debug textを保持する。
present valuesはcorresponding lower handoffの`debug_text()` bytesに一致する。
dedicated `SourceStatementProducer::build_with_formula_composition`の引数順は
input、symbols、bindings、primary terms、atomic formulas、composite
formulas、formula composition、arenaである。constructorは
Task-252/256/257/B1、resolver owner contribution `0` / origin `[2,0]`、
complete `1/1/1/0/1` table profile、exact `Composite(0)` links、subtree
exclusionをpublication前にvalidateする。

B4A handoffだけは`debug_text()`がexisting atomic-formula fingerprintの後、
owner 0の前にRust-Debug quoteした
`composite-formula-fingerprint: {:?}`、続いて
`formula-composition-fingerprint: {:?}`をinsertする。atomic handoffは両lineを
omitするため、complete debug bytesは変更しない。

lower `UnassignedStatement` root ownershipはunchanged。binder fact、truth、
proof acceptance/publication、fact、goal、justification、diagnostic、
semantic resultを推測しない。active one-final-LF 79-byte Task-257B1 fixtureは
upper-route negativeで、private double-LF 80-byte sourceだけがB4Aをselect
できる。repeated read-only documentation reviewは**NO FINDINGS**である。
independent final qualityは全9 hard gatesをcapなし、valid `100/100`で
PASSした。remainingはstaging、commit、post-commit inventoryだけである。

## Task 258B4A implemented composite statement root

`SourceStatementFormulaTarget`は`Composite(0)`をadmitし、statement
handoffはTask-257 composite/composition dependenciesをoptionalに
fingerprintする。dedicated producerはpublication前にfrozen 80-byte
routeのsyntax-free input、resolver owner contribution 0/origin `[2,0]`、
lower profiles、exact owned lower sites/ranges、complete
`1/1/1/0/1` upper tablesをauthenticateする。runner selectorがsource
bytesと全26 Surface rows/root 25をseparately authenticateする。atomic
statement routeはoptional fingerprint absentとbyte-identical debug textを
保持する。

B4A debug textが追加するのはfrozen quoted fingerprint lines 2件だけである。
upper-input mutations 38件、coherent rooted-arena/relocated-term lower near
misses、final statement corruptions 19件、missing-lower tuples、route
isolation、replayがfailure boundaryを証明する。lower typed arenaはrootless、
Surface root 25はauthenticatedのままで、`UnassignedStatement`を書き換えない。
truth、fact、theorem acceptance、proof、goal、justification、diagnostic、
semantic resultは追加しない。

## Task 258B4B frozen connective/grouping statement root

exact private sourceは167 bytes、末尾LF 2件、hashは
`3145e60413841ae005977400f1acd21f0974c7bad635f37fe3df6eeae7700748`
で、zero diagnostics、124 Surface nodes/root 123としてparseする。theorem
ownerはnode 120/range `0..165`、label node 1/range `8..48`、universal
root node 118/range `50..164`である。raw real-frontend resolver ownershipは
public/exported local theorem contribution 0、origin `[2,0]`で、label
projection/import/recoveryを持たない。runnerはhandoff前にexactly one
public/exported theorem `LabelProjection`をenrichする。そのspelling、
namespace、origin、range anchor、contribution、normal recovery、
contribution label effectはすべてownerとmatchし、exact enriched resolver
cardinalitiesは`1/1/1/1/0`である。

statement handoffはexactly owner 1件、statement 1件、context 1件、input
facts 0件、candidate 1件である。owner spellingは
`FormulaConnectiveGroupingPayloadBoundary`である。statement/candidateは
`Composite(0)`をtargetとし、statement spellingは
`theorem FormulaConnectiveGroupingPayloadBoundary : for x being set holds ( ( 0 = 0 & ... & 0 = 3 ) or ( 0 = 0 or ... or 0 = 3 ) ) iff ( ( 0 = 0 & 0 = 0 ) or ( 0 = 0 or 0 = 0 ) ) ;`
である。context 0はbinding context 0/range `0..165`とvisible `[]`を
referenceする。

public DTO、producer、accessor、installer、error、debug grammarは追加しない。
`build_with_formula_composition`とexisting optional lower fingerprints
2件はexact matched Task-257B2 profileだけへextendする。crate-private
cardinality-only `is_task_258b4a_profile`はB4Aのexact owner
spelling/rangeへnarrowし、symmetric exact `is_task_258b4b_profile`を追加
する。call sitesはB4AをTask-257B1だけに、B4BをTask-257B2だけにpair
しなければならない。lower-owned nodes 42件と`UnassignedStatement` rootは
unchangedで、Task 258がownするのはtheorem node 120とupper root links
2件だけである。active 166-byte source、B4A、atomic statement families、
rooted/relocated lower near misses、全profile hybridはmandatory fail-closed
negativesである。runner mutation matrixはこのenriched theorem label
projectionの全fieldとcontribution label effectをauthenticateし、
independently corruptする。raw preflightがlabel-freeであることは
checker-consumed label contractを免除しない。
runner-private route outputはこのzero-reference profileにlookup telemetry
`0/0/[]`を使用する。zero 2件はreference ordinalではなくsentinelである。
transport-detail guardはexact matched Task-257B2/B4B profileだけでそれを
acceptし、B4Aを`1/1/[1,1]`のまま保たなければならない。このdormant
runner conventionはpublic checker DTO/statement semanticを変更しない。

## Task 258B4B implemented connective/grouping statement root

prerequisite commit
`b8a7b8257a682f7c88de943ceaa35b67c0585bc4`でfrozen contractをcloseした
後、existing composite-statement APIをTask-257B2/B4Bへexactにextend
した。`is_task_258b4a_profile`はB4A owner spelling/rangeへnarrowし、
`is_task_258b4b_profile`は
`FormulaConnectiveGroupingPayloadBoundary` / `0..165`だけをrecognize
する。producer、stored-handoff validation、typed installer、final
revalidationはB1/B4AまたはB2/B4Bのmatched pairだけをadmitする。

lower `16/0/16`、`8/0/0/0/0/0/0/16/16`、
`8/6/1/1/1/7/9`、`8/0`、binding `2/1/4`とrootless 124-node arenaは
unchangedである。Task 258はnode 120だけをownし、`42/1/81`
partitionを形成する。upper tablesは`1/1/1/0/1`で、statementと
candidateだけが`Composite(0)`を参照する。two optional lower
fingerprint linesはexisting debug position/grammarをpreserveする。

checker 4 testsとrunner 5 testsはPASSし、separate test-sufficiency/
implementation reviewsは**NO FINDINGS**である。active 166-byte route、
B4A、atomic families、hybrids、partial stateはfail closedである。public
API/error/debug grammar、semantic tables、corpus、traceは変更していない。
final source/documentation、bilingual、boundary consistency reviewsも
**NO FINDINGS**である。focused checker `4/4` / runner `5/5`、full
`cargo test --offline`、`cargo fmt --all -- --check`、warnings deniedの
full offline Clippy、5 CLI、全count/hash、exact scope、audit no-op、
forbidden-artifact、unchanged-stash gatesはPASSした。independent final
qualityは**NO FINDINGS**、全9 hard gates PASS、capなし、valid
`100/100`（`20/20/15/15/10/10/5/5`）である。staging/cached-diff
review、implementation commit、post-commit inventory、B4Cはpendingで
ある。

## Task 258B4C Frozen Nested-Quantifier Statement Root

exact private sourceはfinal LF 2個の139 bytes、SHA-256
`36e5a68a92451590644951838a9af8926212bd78f88d1f90563f12b650b161c1`
である。diagnostic 0、normal Surface nodes 66/root65、reserve item
35=`0..18`、theorem item 62=`19..137`、label token6=`27..65`、
outer composite root60=`67..136`である。raw resolverは
`1/0/1/1/0`で、public/exported theorem owner range `19..137`、
origin `[2,1]`、contribution0 anchor `0..18`である。runnerがexact
theorem label projection/effectを追加した後は`1/1/1/1/0`となる。

active 138-byte sourceはTask-257B3 lower-onlyのままである。現行exact
guardはprivate double-LF sourceをadmitしないため、B4C upperより前に
別lower-stage prerequisiteを作る。scopeはprivate Task-257B3 selector
とそのrunner testsだけで、exact 138/139 bytesをacceptしfinal LF
0個/3個をrejectする。B4C upper commitへ混在させず、production
`source_formula_composition.rs`は変更しない。

matched lower profileはbinding `4/4/0`、primary `6/6/0`、atomic
`3/0/0/0/0/0/0/6/6`、composite `3/0/1/3/3/2/6`、composition
`3/6`である。rootless 66-node arenaのexact lower-owned idsは
`{9,17,22,32,33,36,37,38,39,41,43,44,45,46,47,48,50,52,53,55,57,58,59,60}`。
Task 258はtheorem node62だけをownし、41 nodesはunowned、lower root
ownershipは`UnassignedStatement`のままである。

upper publicationは`1/1/1/0/1`。owner/statementはsite62/range
`19..137`、statement/candidateは`Composite(0)`をtargetする。
context0はbinding context0、visible `[0]`。reserved binding0をselect
するTask-252 referenceがないためinput factは0件である。six
referencesのbinding idsは`1,1,3,2,1,3`、ordinalsは
`2,2,4,4,4,4`、private telemetryは
`2/2/[2,2,4,4,4,4]`である。exact pairingはB4A/B1、B4B/B2、
B4C/B3だけである。

public API/error/debug grammarは変更しない。B4Bと同じseven eventual
upper consumers、checker 4/runner 5 testsをfreezeする。truth、
restriction discharge、existential witness、implicit closure、facts、
theorem acceptance/publication、proof、downstream IRはdeferredである。

## Task 258B4C 実装済み Nested-Quantifier Statement Root

別 commit の selector prerequisite
`42356f38ed0e679d7b878caf0e647c6aa8148d82` が exact private 139-byte
lower transaction を供給する。producer は全66 Surface row、raw
`1/0/1/1/0` provenance、enriched `1/1/1/1/0`、matched Task-257B3
handoff を認証してから upper `1/1/1/0/1` を publish する。statement と
candidate はともに `Composite(0)`、context 0 は exact `[0]` を expose
し、input fact はない。

checker revalidation は全 lower fingerprint、exact B1/A-B2/B-B3/C
pairing、rootless arena の全 anchor と normal recovery state、exact
`24/1/41` ownership を cover する。resolver、row、arena、
family/Task-248、telemetry、rollback/replay、clone/debug、empty-semantic
matrix は exact checker 4 test / runner 5 test で cover される。
test-sufficiency と implementation review は **NO FINDINGS** である。

これは syntax/provenance transport のみである。public API、debug/error
grammar、active artifact、trace/coverage state、truth、restriction/witness
semantics、fact、theorem acceptance、proof、IR は変更しない。

## Task 258B5A frozen ancestor/descendant statement transaction

private 185-byte sourceはexact one theorem ownerとfive syntax-free rowsを
publishする。theorem node 89/context 0/`Atomic(0)`、labeled proof-step
node 67/context 1/`Atomic(1)`、conclusion node 65/context 2/`Atomic(2)`、
outer conclusion node 87/context 1/`Atomic(3)`、descendant conclusion
node 85/context 3/`Atomic(4)`である。source ordinalは`0..4`、各contextは
reserved binding `[0]`、各inputはTask-252 references `[2i,2i+1]`を
consumeし、各candidateはunverifiedである。

reference handoff label row 0はstatement/context/candidate 1、range
`95..96`、origin `<package>::<module>::proof::A`、visible-after ordinal 1、
scope `[0]`、private/local-only SemanticOrigin `[12]`をnameする。citation
row 0はstatement/context 4、range `170..171`、`LabelRefId(0)`、scope
`[0,1]`、simple-local、resolver node 82/SemanticOrigin `[82]`をnameする。
exact prefix visibilityはprovenanceだけで、fact/acceptance/proof/goal/
semantic outputを作らない。

## Task 258B5A implemented ancestor/descendant statement transaction

private producerはfrozen base `1/5/5/5/5`とreference `1/1` profileをconstruct
する前に、exact 185-byte source、全93 normal Surface row/root 92、raw/enriched
resolver provenance、unchanged BindingEnv/Task-252/Task-256 handoffを
authenticateする。ownするのはterm 10、formula 5、statement 5 nodesだけで、
label/citation/proof-block/wrapperとother 73 nodesはarena provenanceのまま。

labelはstatement/context/candidate 1、`95..96`、scope `[0]`、
visible-after ordinal 1、private/local-only contribution 0を維持する。
citationはstatement/context 4、`170..171`、scope `[0,1]`、resolver node
82、`LabelRefId(0)`を維持する。reference validationは全resolver node kind
もauthenticateし、coherent arena-kind mutationがfrozen Surface/resolver
identityをbypassできない。B1/B5A cross-pair、row、scope、ownership、
fingerprint、relocation/recovery、replay mismatchはatomically failする。
fact、acceptance、proof、goal、diagnostic、public APIを追加しない。

## Task 258B5B frozen imported citation transaction

private 146-byte/final-LF B5B sourceは57 normal Surface rows/root 56を持つ。
mandatory separate import-summary prerequisite後、producerはraw resolver
`1/0/1/1/0`、opt-in augmented resolver `8/1/1/3/1`、BindingEnv
`2/1/0`、Task-252 `4/4/0`、Task-256
`2/0/0/0/0/0/0/4/4`、Task-258 base `1/2/2/2/2`、reference local-label/
citation `0/1`をauthenticateする。

owner 0はtheorem node 53/range `48..145`。statement 0はcontext 0のその
theorem、`Atomic(0)`、references `[0,1]`。statement 1はproof context 1/
scope `[0]`のconclusion node 51/range `122..140`、`Atomic(1)`、
references `[2,3]`。各rowはreserved-type-guard input 1件とunverified
candidate 1件を持つ。transactionはterms `35,37,41,43`、formulas
`40,46`、statements `51,53`だけをownし、exact `8/49` ownershipを維持する。

primary term/ref ids 0..3はnode/range/context/source-ordinal
`35/108..109/0/0`、`37/112..113/0/1`、`41/127..128/1/2`、
`43/131..132/1/3`で、全て`x`、binding 0/stored use ordinal 1、normal。
全てmatching reference id、`VariableReference`/`Value`、reference role
`Variable`、scope none/none/`[0]`/`[0]`。
atomic formulas 0/1はnormal equality nodes `40/108..113`と
`46/127..132`、context/source ordinal `0/0`と`1/1`、spelling `x = x`で、
crate planでfreezeしたpaired left/right primary edgeとexact request
triples `(0,0,0), (0,1,1), (1,0,2), (1,1,3)`を持つ。

ownerはcontribution 0のcurrent-module public/exported theorem symbolで、
current-source/current-module origin anchor `48..145`、path `[2,1]`、
import edgeなし、normal recovery。statement source ordinalは0/1、
normalized spellingは
`theorem FormulaStatementImportedPublicTheoremCitationSmoke : x = x proof thus x = x by Ref ; end ;`
と`thus x = x by Ref ;`。input-fact/candidate table idは0/1だが各row自身の
ordinal fieldは0。input usesは`[0,1]`/`[2,3]`、candidate targetは
`Atomic(0)`/`Atomic(1)`。

local label rowは0件。citation id 0はstatement/context 1、node/range
`48 / 136..139`、scope `[0]`、dense citation-row ordinal 0。resolver
reference candidateは独立にsource-statement ordinal 1を持つ。citationは
`LabelRefId(0)`、`ProofOrTheorem`、spelling `Ref`、normal recovery。
singular resolver
projectionは`parser.type_fixtures`由来のimported/public/exported theorem
provenanceで、exact opt-in origin path
`summary:parser.type_fixtures::Ref:label:Ref`、structural path `[1,0]`、
current-module namespace、imported contribution 2、anchor `7..27`を持つ。

resolved import id 0はresolver node 29（`ImportAliasDecl`）がownし、range
`7..27`、exact spelling `import parser.type_fixtures;`、aliasなし、
resolution `Resolved(<package>::parser.type_fixtures)`。originはcurrent
source/current module、range anchor `7..27`、path `[0]`、import edgeなし、
normal recovery。nodes 28/29/30はそれぞれ
`ModulePath`/`ImportAliasDecl`/import-item identity、range
`7..27`/`7..27`/`0..28`、path `[28]`/`[29]`/`[30]`、normal recovery、
`NotApplicable`、reference keyなしを維持する。node 48だけがkeyed node。

imported projection declaration rangeは`7..27`、semantic originはcurrent
source、declaring module `<package>::parser.type_fixtures`、range anchor
`7..27`、path `[1,0]`、import edgeなし、normal recovery。reference
candidate originはcurrent source/current module、range anchor
`136..139`、path `[48]`、import edgeなし、normal recovery。producer/
final-clone testsは全import-row fieldと両origin tupleを独立にmutateする。

imported citationにはlocal label idがないため、later upper taskは
non-exhaustive public
`SourceStatementCitationTarget::{Local(SourceStatementLabelId), Imported}`
を追加する。citation input/immutable row/getterはmandatory
`label`/`label()`の代わりに`target`/`target()`を使い、B1/B5Aはunchanged
`Local`を使う。`SourceStatementCitationKind::SimpleImported`を追加する。
B5B debugはimported projectionと`target=imported`をrenderし、absent local
label nodeを表し、`label#0` lineを出さない。B1/B5A debug bytesはexact維持。

B5B public debug schemaの全体は次の通り:

```text
source-statement-reference-debug-v1
module: <package>::<module>
statement-fingerprint: <quoted source-statement debug>
resolver-ast root=56 nodes=57 name_refs=0 label_refs=1 imports=1 exports=0 label_node=absent reference_node=48 reference_state=resolved reference_key=label#0
resolver-projection source=imported origin=summary:parser.type_fixtures::Ref:label:Ref module=parser.type_fixtures namespace=<module> range=7..27 contribution=2 path=[1,0] kind=theorem visibility=public export=exported spelling="Ref"
resolver-reference node=48 range=136..139 source_ordinal=1 scope=[0] expectation=proof-or-theorem spelling="Ref"
resolver-result index=1 references=1 ids=[0] diagnostics=0
citation#0 statement=1 context=1 target=imported label_ref=0 scope=[0] range=136..139 ordinal=0 kind=simple-imported recovery=normal
```

placeholderはvalidate済みruntime module/fingerprint valueを表し、他の全token、
field order、line orderはliteral。`label#0` lineは存在しない。checker test 1、
runner test 1、final-clone coverageはschema全体をassertし、B1/B5A outputを
byte-identicalに維持する。

producerはabsent/duplicate/private/local-only/re-exportedまたは他のwrong
export status/wrong-kind/module/namespace/contribution/origin/range/path、
recovered/stale/relocated/cross-profile/wrong dense citation-row ordinal/
wrong resolver source-statement ordinal/partial/wrongly-keyed rowを
atomically rejectする。checker test 2、runner test 2、final-clone
coverageは`Exported`から`ReExported`へのmutationを独立に実行する。
B5Cと全semantic resultは
deferred。このprerequisiteはsource、fixture、expectation、trace row、
public runner schemaを変更しない。

## Task 258B5B implemented imported citation transaction

documentation commit
`141dc44a757555e8d4837756515e1577f672348b`とisolated lower commit
`46dd9db56ced2fcc57799420de9d5fed06f284f5`後、upper transactionは
frozen 146-byte routeをthree checker/four runner consumersだけでimplement
する。57-node/root-56 resolver arenaからexact Task-258 base
`1/2/2/2/2`、reference `0/1`、root-preserving `8/49` ownershipをpublish。

citation row 0は`target=Imported`、`SimpleImported`、statement/context 1、
`LabelRefId(0)`、scope `[0]`、range `136..139`、dense ordinal 0。local
label rowはない。producerはpublication前にresolved import 0、imported/
public/exported theorem projection、reference node 48、resolution key 0、
source-statement ordinal 1、independent source/module/range/anchor/path/
recovery provenanceをauthenticateする。debug outputは
`label_node=absent`/`source=imported`を含み、`label#0` rowを出さず、
B1/B5A local debug bytesを維持する。

上のprimary API sketch/Public Enum Policyはactual non-exhaustive target
enum、`target` field/accessor、`SimpleImported` variantと一致する。
dependency/aggregate/import/projection/reference/row/cross-profile/
installation/final-clone mutationはatomically failしvalid replayを保持。
four checker/five upper runner testsがexact routeをcoverし、separate lower
commitはtwo testsを維持する。B5Bだけは両operandがformula wrapperのsole
immediate childを共有するためfull nested operand child pathを比較し、全
pre-existing statement profileは従来のimmediate-child ordering ruleを
維持する。exact-profile testはこの区別をrecordする。fact、acceptance、
proof、goal、diagnostic、downstream IRはempty、B5Cとactive corpus/trace
coverageはdeferred。

## Task 258B5C frozen unresolved-reference exclusion

R-032Aはvalidated structural arena、R-032Bは`A`についてscope `[0,0]`/
completion後ordinal 3 visibleのone private/local-only proof-step
projectionと、enclosing scope `[0]`またはsibling scope `[0,1]`のone simple
unqualified reference candidateをsupplyする。両resolution resultは
`has_unresolved = true`でexact one `UnresolvedLabelRef`を持つ。

`SourceStatementReferenceHandoff`はこのstateを意図的にrejectし、reference
nodeにkeyed `Resolved` resultを要求する。したがってtwo B5C negativesは
`SourceStatementLabelInput`、`SourceStatementCitationInput`、immutable
label/citation row、resolver projection replay、statement/reference profile、
owned-node partition、debug outputをpublishしない。declaration-symbol
runnerはR-032A structural validationとR-032B collection後にresolver failureを
直接observeし、local label id、resolved node、scope、imported target、
citation ordinal、statement contextをmanufactureしない。

全Surface nodeはsyntax-ownedのまま。structure constructor、selector、
functional/field update、Task-252 term、Task-253 formula、B1/B5A/B5B
profile、fact、proof progress、acceptance、downstream semanticsへのB5C edgeは
ない。

resolver source formはnormal
`ConclusionStatement -> JustificationClause -> ReferenceList -> Reference`
pathに限定する。module-global owning-statement ordinalとcanonical
`proof-step-v1` provenanceはhandoffより下位に留まり、source-byte runner
selection/`proof_scope_input` failureはsource-statement rowをpublishしない。

complete lower allowlistはexact
`Root -> CompilationUnit -> ItemList -> direct TheoremItem -> direct
ProofBlock`から始まり、direct normal compact/
conclusion statementだけをadmitし、compact proposition labelだけをinspectし、
supported statementからdirect proof/justification childだけへdescendする。
exact simple-reference chainは上記の通り。Root/CompilationUnitは各exact
one normal structural child、ItemListはdirect normal theorem childだけをscan
する。missing/additional/wrong upper child、direct Root/Compilation theorem
relocation、`VisibleItem` wrapping、all other excluded/mixed formは
no-row/no-ordinal/no-descentである。

runnerはenv/module、module-derived namespace、exact one id-0 LocalSource
contribution record/public `ast.source_id`、各projection module/namespace/
contributionをindependently validateする。全field mutationはinput-onlyで、
このDTOをpublishできない。

## Task 269A frozen upper consumer boundary

Task 269Aはexact Task-258B3N `SourceStatementHandoff`/
`SourceStatementWitnessHandoff`をconsumeするが変更しない。witness 0、name 0、
RHS `Primary(2)`はimmutable lower identityであり、complete debug byteをupper
fingerprintとして保持する。new handoffはdefinition-site associationとextended
binding environmentだけをownする。

existing name node 13は`source.statement-witness.name`、witness node 36は
`source.statement-witness.item`、take node 37は
`source.statement-witness.take`のまま。`source.proof-local.*` nodeは作らない。
existing Task-258B3N route/debug resultはstable lower oracleとして残る。

## Task 269A active upper consumer boundary

separate proof-local producerはunchanged Task-258B3N statement/witness/primary
bundleをfingerprint/consumeする。adjacent checker test 4件はprivate B3N oracleを
reuseするだけでlower public API/node/row/legacy debug byteを変更しない。new
binding ownershipはすべて`source_proof_local_declaration`に残る。

## Task 269B B3M1 lower-consumer boundary

Task269Bはfrozen Task-258B3M1 `2 witnesses / 1 name` profileをbyte-for-byte
consumeする。complete 56-node authentication/replayはexisting distributed node
ownershipを維持する。source-statementがownするのはtake、witness-item、
witness-name、dense within-take order、witness/statement handoffであり、RHS
reference nodeはTask252、formula nodeはTask256が引き続きownする。5番目の
fingerprintはlower source-statement inputではなくfinal binding environmentを
authenticateする。binding incrementは`0/0/2`だけをlinkし、unnamed witness1の
binding/lower API/debug byte/left-to-right goal semanticsを変更しない。

## Task 269B active B3M1 lower-consumer boundary

implemented upper consumerはB3M1 lower row、node、range、ordinal、debug byteを
すべて不変に保つ。two-row witness handoffをfingerprintし、named row0/name0/RHS
primary2だけをassociate、unnamed row1がchecker bindingをallocateしないことを
直接検証する。all-node/isolated cross-profile testsはadjacent private testsの
ままで、lower APIやsemantic meaningを追加しない。

## Task 269CP lower statement boundary

runner-private exact sourceはtheorem ordinal0、proof-local let ordinal1、conclusion
ordinal2を持つが、269CPは`SourceStatementKind`、generalization table、statement
handoffを追加しない。nodes 47/46/37/36/13/35/34はcomplete normal 51-node
Surface snapshot内のrole anchorとしてauthenticateする。runnerはroot node 50、
absent expression root、token nodes 0..23、および全nodeのsource identity、range、
recovery、ordered childrenもauthenticateする。Task-258/269A/B debug/profileは
不変で、checker statement edgeにはlater frozen contractが必要。

## Task 269C no-statement binding boundary

binding-only transactionはTask-269CP theorem/proof/let rangeをconsumeするが、
`SourceStatementKind`、statement context/fact/candidate row、formula edge、statement
semanticを追加しない。`SourceStatement(59..98)`はbinding-context owner tagだけ。
goal/thesis/conclusion ownershipはdeferred。

implemented transactionもこのboundaryを維持する。context owner tagはprovenance
としてだけvalidateし、statement/formula/thesis/conclusion/fact/proof rowをemitしない。

## Task 269CT no-statement boundary

source-type prerequisiteはTask-269CP theorem/proof/let provenanceとTask-269C bindingをreuseするが、
statement row、current goal、thesis transition、proof-skeleton node、conclusion、fact、acceptanceを
publishしない。statement API/fingerprint/testは不変。

## Task 269CT implemented no-statement boundary

final compositeはexact source-type node 3件だけをmapする。complete 3-row
`source.statement.transport` hint setでも`InvalidSourceProofLocalLetType`としてrejectし、
silent consume/overrideできないことをregressionで固定した。statement handoff/semantics/
proof/fingerprintはempty/unchanged。

## Task 269GP no-statement lower boundary

runnerは`GivenStatement(70..108)`をsyntax owner rangeとしてだけauthenticateし、
binding scope/visibilityをpublishしない。condition、
proposition、label、thesis、conclusion、fact、statement context、proof rowはprivate lower
outputからexclude。source-statement API/checker fingerprint変更なし。

implemented runner-only projectionもexactlyこのexclusionを保持し、checker
source-statement file/APIは変更なし。

## Task 269GS no-statement-owner reconciliation

canonical block-lifetime ruleはsource-statement lowering/existing 269GP private rowを変更しない。
condition、label、formula、statement payload追加なし。binding-only consumptionはTask269G。

## Task 269G statement boundary

existing exact `given` lowerをbyte-for-byte consumeし、statement/condition/label/formula/
use-site rowは追加しない。checker binding handoffはenclosing proofだけをidentify。
statement semantics/proof effectはdefer。

implemented transactionはこのboundaryを保持する。proof contextはprovenanceとしてだけ
authenticateし、statement/condition/label/formula/fact/thesis/conclusion/proof rowをemitしない。

## Task 269GT statement boundary

type compositeはexisting proof statement contextをbinding provenanceとしてだけauthenticate。
statement/condition/label/formula/fact/thesis/conclusion/proof rowを追加せず`such that`を
reinterpretしない。

### Task 269GT implemented statement boundary

statement ownerやstatement/proof hintを追加しない。dormant consumerはdispatch外で、final assemblyはstatement/condition/fact/proof/semantic inputが空の場合だけGiven-type compositeをacceptする。

## Task 269GUP statement boundary

`thus y = y;` conclusion全体はselector-only。statement/term/conclusion/formula/equality/
condition/label/fact/proof/acceptance ownerなし。private lower/binding routeはdispatch外。
Completion evidence: [central Task-269GUP historical contract](../../task_contracts/ja/269GUP.md#completion-evidence)。

## Task 269GUPT statement boundary

already authenticated `given` declarationのwritten typeだけをownする。`such that G: thesis` condition/label、conclusion equality、later `y` leaves、proof goal/acceptanceはselector-onlyでstatement/fact/semantic rowなし。existing statement API/production dispatchはbyte-identical。

### Task 269GUPT implemented statement boundary

authenticated written source typeだけをtransport。statement API、condition/label/
equality/later-use subtree、proof state、acceptance、production dispatchは不変。

## Task 269GU statement boundary

GUは`thus y = y;`内の`TermReference` leaf 2件だけをconsumeする。
`TermExpression` wrapper、`BuiltinPredicateApplication`、equality formula、
proposition、`ConclusionStatement`、`such that` condition/label、proof block、
goal/fact/acceptanceはselector-only。existing statement API/production dispatch
は不変。

### Task 269GU implemented statement boundary

primary-term/reference leaf 2件だけをtransport。statement API、condition/label/
equality/formula shell、fact、proof state、acceptance、production dispatchは不変。

## Task 269GCP frozen statement exclusion

selectorはexact source identityのためGiven statement、condition list、labeled
proposition、equality subtree、final conclusionをauthenticateするだけ。
`source_statement`にはpublishせず、condition/label fact、statement semantics、
proof state、acceptanceはabsent。

### Task 269GCP implemented statement exclusion

runnerはexact Given/condition statement subtreeをselector evidenceとしてだけ
authenticateする。`source_statement` payload、condition/label fact、assume、proof
state、acceptance resultはpublishしない。

## Task 269GC frozen statement exclusion

theorem/proof/Given/segment/name rangeはbinding authenticate/scopeだけに使用。
`source_statement` handoff、proposition/condition list/label/fact/assume/
conclusion/proof-state/acceptance rowなし。condition occurrenceはGCT後GCU owner。

### Task 269GC implemented statement exclusion

authenticated rangeだけでbinding handoffを実装。`source_statement` payload、
condition/label fact、assume、conclusion、proof state、acceptance rowは不変。
condition occurrenceはopaqueのままGCT後GCU owner。

## Task 269GCT frozen statement exclusion

GCP/GC statementはsource/type dependencyとしてだけauthenticate。statement
handoff/proposition/condition list/label/fact/assume/conclusion/proof state/
acceptance rowなし。condition equalityとwitness 2 occurrencesはopaque/GCU owner。
`source_statement.rs`変更は禁止。

Completion evidence: [central Task-269GCT historical contract](../../task_contracts/ja/269GCT.md#completion-evidence)。

## Task 269GCU frozen statement deferral

unchanged 54-node Surfaceは`TermReference` leaves `107..108`/`111..112`
だけをselector evidenceとして供給。statement owner/proposition/condition
list/equality/formula/label/fact/proof/conclusion/acceptance rowはpublishせず、
Given statementとenclosing/subsequent structureはopaque。

Completion evidence: [central Task-269GCU historical contract](../../task_contracts/ja/269GCU.md#completion-evidence)。

## Task 269SDP source-statement ownership

`source_statement.rs`だけがexact source、68 Surface nodes/tokens、2 shells、
theorem provenance、Given/now/two-Set ranges、debugを検証するprivate lower
ownerとなる。statement semantics、proof context、fact/result、binding、term、
capture、diagnosticは生成しない。

Completion evidence: [central Task-269SDP historical contract](../../task_contracts/ja/269SDP.md#completion-evidence)。

## Task 269SDC frozen source-statement consumer boundary

SDCは既存Task-269SDP lowerをconsumeし、`source_statement.rs`を変更しない。
lower source/module identity、theorem provenance、theorem/proof/Given/
segment/name/descendant ranges、Given name spelling/source ordinal、complete
lower debug fingerprintだけをsyntax-free binding producerへ渡す。Given type
getters、2 Set rows/RHS、conclusion gettersはauthenticated lower evidenceの
ままでsemanticに読まない。
`y@118..119` occurrence、`z`/`q` declaration、formula/conclusion/fact/
block-result/proof payloadをpublishしない。

Completion evidence: [central Task-269SDC historical contract](../../task_contracts/ja/269SDC.md#completion-evidence)。
