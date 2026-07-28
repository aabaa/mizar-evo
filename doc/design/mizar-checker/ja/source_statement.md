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
| `SourceStatementKind` | `#[non_exhaustive]`; Task 258Aは`TheoremProposition`だけをaccept。 |
| `SourceStatementRecovery` | `#[non_exhaustive]`; callerは`Degraded`を許容し、exact routeは`Normal`だけをaccept。 |
| `SourceStatementFormulaTarget` | `#[non_exhaustive]`; Task 258AはTask-256 `Atomic` target 1件だけをaccept。 |
| `SourceStatementInputFactKind` | `#[non_exhaustive]`; Task 258Aは`ReservedTypeGuard`だけをaccept。 |
| `SourceStatementCandidateFactKind` | `#[non_exhaustive]`; Task 258Aは`UnverifiedProposition`だけをaccept。 |
| `SourceStatementError` | `#[non_exhaustive]`; callerはproducer/installation failureをexhaustive matchしない。 |

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
