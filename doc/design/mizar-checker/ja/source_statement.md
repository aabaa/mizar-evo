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
