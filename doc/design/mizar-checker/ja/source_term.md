# Source primary-term handoff

> canonical languageはEnglishである。
> [English source_term.md](../en/source_term.md)

## 目的とauthority

public `source_term` moduleはChecker Task 252を実装する。variable/local-constant
reference、`it`、numeral、transparent parenthesisのsource occurrenceをraw
syntax importなしでcheckerへtransportする。canonical authorityはChapter 04
§§4.1-4.3、4.4.1、4.6とChapter 13 §§13.1、13.8.1-13.8.2、13.8.8であり、
broader term/source-to-checker gapはMC-G017/MC-G020が追跡する。

moduleはtransport-onlyである。source shape、binding lookup、missing numeric-type
requestをauthenticateするが、numeric typeを選択せず、semantic term/formula、
current definition result type、fact/axiom、FOL/downstream IRを作らない。

## Task 269SDU Descendant Given Occurrence/Reference Contract

[central Task-269SDU contract](../../task_contracts/ja/269SDU.md) の下で、
`source_term.rs` はpublic `SourceProofLocalGivenDescendantUseTermHandoff`、producer、
non-exhaustive errorを実装する。handoffはcomplete SDT dependencyをby-value consumeし、
exact dependency/debug fingerprintをpreserveする。source-term rowは`y@118..119` 1件だけで、
typed node `3`、ordinal `0`、descendant context `2` (`[0,0]`)、
`VariableReference`/`Value`、`term 0 -> binding 1` の`Variable` reference 1件であり、
numeric requestは0。

validationはexact SDT dependency/prefix、exact term input/source-term fingerprint、
complete five-node arena、one-shot installationの順でatomicに行う。arenaは3-node SDT
prefix、node `3` `source.term.variable-reference@118..119`、children `[2,3]`のroot
node `4` `source.proof-local.given-descendant-use.term-root@0..179`を持ち、全nodeは
Unknown/Normal、unresolved、unlinkedのまま。profile限定`GivenWitness -> Variable`
admissionだけを許可しgeneric admissionは不変。error順は`InvalidDependency`、
`InvalidSourceTerm`、`InvalidInstallation`。

この実装済みownerはSet binding/abbreviation/capture、equality/formula/fact/guard、
proof completion、generic term publication、diagnostic、downstream IR、dispatch、
coverage creditをexcludeする。特に`z@114..115`、`q@129..130`、`z@133..134`はabsentのまま。

## Public model

`SourcePrimaryTermHandoffInput`はsource/module transaction 1件と、次のordered
input table 3件を持つ。

- `SourcePrimaryTermInput`
- `SourcePrimaryTermReferenceInput`
- `SourceNumericTypeRequestInput`

`SourcePrimaryTermProducer::build`はsyntax-free `BindingEnv`/`TypedArena`に
対してrowをauthenticateし、`SourcePrimaryTermHandoff`をatomicにpublishする。
immutable `SourcePrimaryTermTable`、`SourcePrimaryTermReferenceTable`、
`SourceNumericTypeRequestTable`がexposeするのはborrowed lookup、source-order
iteration、length、emptinessだけである。dense identityは
`SourcePrimaryTermId`、`SourcePrimaryTermReferenceId`、
`SourceNumericTypeRequestId`である。

term rowはnode site、exact source range、dense pre-order source ordinal、
binding context、recovery、token-normalized spelling、kind、role、optional
parentを保持する。reference rowはterm/binding identityとroleを保持し、lexical
scope/use ordinalはproducer-derived outputである。numeric requestはexact numeral
term/site/range/spellingとdense request ordinalを保持する。`debug_text()`は全tableを
deterministically renderする。

## Public Enum Policy

| Public enum | compatibility policy |
|---|---|
| `SourcePrimaryTermKind` | `#[non_exhaustive]`。callerはlater primary-term familyを許容する。 |
| `SourcePrimaryTermRole` | `#[non_exhaustive]`。callerはlater source roleを許容する。 |
| `SourcePrimaryTermReferenceRole` | `#[non_exhaustive]`。callerはlater authenticated binding roleを許容する。 |
| `SourcePrimaryTermRecovery` | `#[non_exhaustive]`。callerはlater recovery classを許容する。 |
| `SourcePrimaryTermError` | `#[non_exhaustive]`。callerはvalidation failureをexhaustive matchしない。 |
| `SourceProofLocalGivenUseTermError` | `#[non_exhaustive]`。callerはTask 269GU dependency/input/installation failureをexhaustive matchしない。 |
| `SourceProofLocalGivenConditionUseTermError` | `#[non_exhaustive]`。callerはTask 269GCU dependency/input/installation failureをexhaustive matchしない。 |
| `SourceProofLocalGivenDescendantUseTermError` | `#[non_exhaustive]`。callerはTask 269SDU dependency/input/installation failureをexhaustive matchしない。 |
| `SourceNestedFraenkelMapperPrimaryError` | `#[non_exhaustive]`。callerはC4C4 dependency/binding-environment/source-term projection failureをexhaustive matchしない。 |

この module が所有する exhaustive public enum exception はない。

## Validationとatomicity

term idと`source_ordinal`はequal dense pre-order indexである。全siteはunique
`TypedSiteRef::Node`で、arena kind/range/recoveryがrowとexact matchする。
identifier referenceはcanonical `mizar_lexer::is_identifier` predicateがaccept
するnonempty binding-authenticated spellingを要求する。first characterはASCII
alphabeticまたは`_`、remaining characterはASCII alphanumeric、`_`、
apostropheだけで、reserved wordをrejectする。このcheckはlexical vocabularyを
reuseするがraw syntaxをimportしない。`it`はexact `it`、numeralはASCII
digitだけ、parenthesis spellingはtoken間ASCII space 1件のexact
`( <child spelling> )`である。

各parentはsame contextのearlier parenthesisで、rangeがimmediate child 1件を
strict containする。parentだけがchildをownする。root/siblingはsource orderを
保ち、nested parentはTask-252 kind 5件だけのclosed acyclic pre-order treeを作る。
private runner extractorはlater term family descendantを含むparenthesized subtree
全体をexcludeする。

variable/constant rowはreference exactly 1件を持つ。generic profileのvariableは
`ReservedVariable`/`LetBinding`/`QuantifierBinder`/`DefinitionParameter`だけ、
constantは`LocalAbbreviation`だけを受ける。`it`/numeral/parenthesisはbinding
referenceを持たない。全numeralはnumeric request exactly 1件、他kindは0件である。

各referenceについてproducerはterm contextからlexical scopeをcloneし、
declaration range endがterm start以前であるbinding row数を`use_ordinal`とする。
previous referenceはordinalを進めない。normal binding groupはsource-order
singletonでvisibilityがdense indexと等しい。exact consecutive duplicate groupは
spelling/kind/owner context/`BinderIdentity`/rangeとgroup final row dense indexの
visibilityを共有する。このgroupを`BindingEnv::lookup`が`Ambiguous`としてreject
できるまで保持する。

producerはresolver payloadなしで`BindingLookupSite::new`を構築し、supplied
local binding exact winnerだけを要求する。forward/ambiguous/missing scope
payload/unresolved/different winner/lookup errorはfail closedである。このpathで
`Resolver`はstructurally unreachableである。inputをsort/repair/partial publish
しない。

## Ownershipとconsumer

`TypedAst::with_source_term`はsource/moduleと全arena nodeをrevalidateしてoptional
immutable handoff 1件をinstallし、replacementをrejectする。`ResolvedTypedAst`は
handoffをclone-preserveするだけで、`source_term()`をexposeする。

private `mizar-test::runner::type_elaboration::source_term` leafがraw
`SurfaceAst` extractionをownする。exact real selectorは次の3件である。

1. `fail_type_elaboration_term_formula_gap_001`
2. `pass_type_elaboration_reserved_variable_equality_001`
3. `pass_type_elaboration_parenthesized_reserved_variable_equality_001`

aggregate handoffはterm 7/reference 4/numeric request 2で、existing semantic
outcome/detail keyは不変である。synthetic testはsemantic acceptanceを追加せず
local constant、`it`、nested parenthesis、mixed-family exclusionをexerciseする。

## Verificationとdefer

checker testは全kind/role、dense order、binding-event order、shadow/forward/
ambiguous/missing/unresolved lookup、reference/numeric-request cardinality、
parent graph、source/module/site/range/kind/spelling/recovery/context corruption、
deterministic rendering、typed-AST installationをcoverする。runner testはexact
real selector、7/4/2 oracle、synthetic dependency boundary、isolation、
corruption、deterministic replay、final resolved preservationをcoverする。

covered trace requirementは
`spec.en.checker.type_elaboration.source_primary_term_payload`である。
application、structure/set/choice/comprehension/`qua` term、formula graph、
definition result semantics、real proof-local constant production、numeric
response、accepted fact/declaration/proof、downstream IR、Tasks 253+、Steps 6/7は
explicit ownerに残る。

## Task 257C4C4 nested Fraenkel mapper primary

Implemented [C4C4 contract](../../task_contracts/ja/CHECKER-FRAENKEL-NESTED-MAPPER-PRIMARY-257C4C4.md)は
C4C3-dependent/zero-semanticなspecialized Task252 transactionを1件追加した。
Exact public familyは`SourceNestedFraenkelMapperPrimaryHandoff`、non-exhaustive
`SourceNestedFraenkelMapperPrimaryError`、
`SourceNestedFraenkelMapperPrimaryProducer`。Handoffはcomplete C4C3 dependency、
exact 3-context/outer-x 1-binding/0-diagnostic `BindingEnv`、one-node immutable
projection arena、existing `SourcePrimaryTermHandoff` `1/1/0`をretainする。

Sole primaryはinner mapper `x@94..95`の`VariableReference`/`Value`。Sole referenceは
C4C3 resolver binding `1` / outer binder `x@136..137`を表すprojection-local checker
binding `0`をselectする。Binding ownerはouter-comprehension context、useはinner child
context。Profile-scoped logical ordinal `1`だけがsource上後置されたgeneratorをvisibleに
する。Inner `y`はrevalidated C4C3 dependencyだけに残りTask252へduplicateしない。

Private specialized profileはpublic generic Task252 source-order derivation/forward rejectionを
弱めない。Exact getter/debug/error precedence/context/binding/arena/row/default-deny matrixは
task contractだけがownする。Sole current consumerはexisting private nested-capture library
test leafである。Typed/Resolved installation、active route、capture identity、generated-core
parameter、type/sethood answer、semantic result、diagnostic、coverage creditは追加しない。

## Task 257B1 Consumer Addendum

Task 257B1はexact pass consumerへadditional `VariableReference`/`Value` row
2件とbinding reference 2件を追加する。両referenceはbody context 1でexplicit
quantifierのbinding 0をselectする。occurrence/lookup-winner ownershipはTask 252
だけに残り、formula-composition handoffはbinder-to-reference associationだけを
記録し、captured-free-variable metadataをrepurposeしない。

Task 257B2はbody context 1の16 numeral rows/16 numeric-type requestsをreuse
する。explicit `x` binderはunusedなのでreferenceは0、captured identitiesは
emptyであり、composition layerはbound-use rowをfabricateしない。

## Task 257B3 Frozen Consumer Addendum

Task 257B3はexact `VariableReference`/`Value` term 6件とTask-252
lookup-selected reference 6件をreuseする。source orderで`x` 3件はouter
quantifier binding 1、`y` 1件はbinding 2、`r` 2件はreserved binding 0ではなく
inner quantifier binding 3をselectする。term/reference 0・1はcontext 1、
2..5はcontext 3。termは`VariableReference`/`Value`/`Normal`、source
ordinal `0..5`、referenceはvariable roleを保持する。scope path/local identityは
source-derived resolver-shaped preflight fact、use ordinalはauthenticated
Task-252 producer outputである。formula compositionはowning-edge association
だけを記録し、occurrence/reference/spelling/lexical-scope/lookup-winner
ownershipはTask 252に残す。

Task 257B3はこのsix-row reciprocal consumerを実行し、binding ids
`1,1,3,2,1,3`とuse ordinals `2,2,4,4,4,4`を検証する。全occurrence/
referenceのownershipはTask 252に残る。

## Task 257C1 frozen consumer addendum

Task 257C1はsource term `1`、`2`、`3`のTask-252 `Numeral`/`Value`
primary/numeric requestをexact `3/0/3`でreuseする。primary 1
（`2`、`85..86`）は単一occurrenceで、新Task-256 shared-boundary edgeはterm/
requestをduplicateせず、隣接する両segment descriptionから同じrowを参照する。
occurrence、spelling、range、arena、numeric-request ownershipはTask 252、
predicate grouping/polarityはTask 256に残る。

Task 257C1 active pass consumerはこのfrozen backlinkを実行する。実測
`3/0/3` profileとsingle middle-primary identityを保持し、Task-252 API/
semantic numeric resultは変更しない。

## Task 255C1 frozen backlink

exact conditioned-comprehension prerequisiteはone immutable Task-252
`4/0/4` handoffをbuildする。primaries 0/1はTask-253 mapper argument、
primaries 2/3はauthenticated condition range内のequality operandである。後者は
ordinary Task-252 occurrence/numeric-request rowのままだがTask-255 edgeを
持たない。formula/numeric semanticsをここへ与えず、later Task 256がtargetする
exact objectを保持する。

## Task 255C1 transport result

exact routeはこのsingle `4/0/4` handoffをpublishする。copied、omitted、
range-substituted condition primaryはcomplete Task-255 transactionをfailさせる。
authentic condition operands 2件はTask-255 edgeを持たず、later Task-256
installation向けに保持される。

## Task 257C2 frozen consumer boundary

Task 257C2は同じimmutable `4/0/4` handoffをreuseする。Task-256 equality
edgesはprimaries 2/3をdirectにtargetし、Task-257C2 associationがtargetするのは
condition/formula IDだけ。Task-252 row/request/parent/context/fingerprint/
debug byte/numeric semantic meaningは変更しない。frozen pre-Task-256C1
baselineではrouteはseparate lower taskをgateとしていた。Task 256C1は両
installation orderをpassし、Task 252自体にcompatibility editは不要なまま、
completed Task-257C2 routeはfingerprint/debug byteを変更せずexact rowを
reuseする。

## Task 257C3 frozen downstream consumer

Task 257C3はexisting Task-252 `3/0/3` numeral handoffをexact fingerprintと、
primary 1をtargetするTask-256 shared boundary edge経由だけで
reauthenticateする。term/reference/request row、parent edge、ownership、
Task-252 APIを追加しない。本documentation prerequisiteでは全Task-252
byte/testが不変。

## Task 257C3 downstream consumption result

implementationはexact immutable `3/0/3` handoff/debug fingerprintをreuseし、
Task-252 production API/rowを追加しない。same source/module/arena上のcoherent
two-term test-only handoffは個別validate後、C3 exact-profile boundaryだけで
failする。

## Task 258A frozen downstream consumer

Task 258Aは`74..75`/`78..79`のnormal `VariableReference`/`Value` primary
2件とreserved binding 0へlookup-authenticatedされたreference 2件だけを
reuseする。profileは`2/2/0`、1 binding rowが両useより前にcompleteするため
Task-252 `SourcePrimaryTermReference::use_ordinal()`は両方1。これはrunner
upstream binding/use source-event lookup ordinal 1/2とはdistinct。
statement input factはreference IDs `[0, 1]`を指すだけでbinding/spelling/
range/lookup winner/source ordinalをcopyしない。Task 252が全occurrence/
reference ownership/semanticsを保持する。本prerequisiteはTask-252 API/
source/test/debug byteを変更しない。

## Task 269GUP source-term exclusion

GUPはexact sibling binding envだけを作る。checker/runner source-term、Task252 allowlist、
term/reference/request table/testはbyte-identical。`116..117`/`120..121`はselector-only。
GUPT後のTask269GUだけがfuture GivenWitness Variable admission/occurrenceをownする。
Completion evidence: [central Task-269GUP historical contract](../../task_contracts/ja/269GUP.md#completion-evidence)。

## Task 269GU proof-`given` later-use term/reference 凍結契約

### 選択、authority、classification

fresh clean HEAD `c529245138b6d40be65c590ba701fef4f4ea0881`はcommitted
GUPT source-type prerequisiteを含み、Task 269GUだけを選択する。canonical
Chapter 4 §4.6.1(5)、Chapter 15 §§15.3.3/15.10、Chapter 16 §16.3.3
item 5a/§16.4.2とuser確認は、`given` witnessがdeclaration condition内をbindし、
その後はinnermost enclosing proof/reasoning blockの残りと、shadowされないchild
blockで有効、parent/siblingでは無効であることを定める。Chapter 8 §8.1は既に
実装済みのdeclared typeだけ、Chapter 3は通常のin-scope variable解釈だけを与え、
Chapter 13 §§13.1.1/13.8.1がin-scope identifier occurrenceをvariable referenceとする。

exact sourceは128 byte・final LF 1件、source SHA
`ec15ded78ae96022840a8419a85d74643de3b37337e9a202cbda77ee97aa7c01`、
54 Surface node/root 53、snapshot SHA
`c64297ce72e380a2e4146276966e085d780f8b38f2528d5abaa440a50c67db6d`である。
later leafはunrecovered `TermReference` node 41/43、`y@116..117`/
`y@120..121`だけである。GUPはshell/theorem/resolver provenance、proof
`62..126`、given `70..108`、name `76..77`、scope `[0]`/ordinal 1を認証済み、
GUPTはexact `2/2/0` typed BindingEnvと`2/2/0/0/0/0` source typeをownする。

blocking `spec_gap`はない。未実装composition/testはbounded `source_drift`/
`test_gap`、stale ledgerは`design_drift`。generic Task-252 pathへの無制限
`GivenWitness` admission、old GUP/GUPT変更、formula/proof semantics出版は
`boundary_violation`。origin `0/9`はreport-only `repo_metadata_conflict`である。

### Public compositeとexact payload

`source_term.rs`へpublic `SourceProofLocalGivenUseTermHandoff`、
`SourceProofLocalGivenUseTermProducer`、non-exhaustive
`SourceProofLocalGivenUseTermError { InvalidDependency, InvalidSourceTerm,
InvalidInstallation }`を追加する。handoff getterはsource/module、owned GUPT
dependency、そのcomplete debug fingerprint、`SourcePrimaryTermHandoff`とその
complete fingerprintをborrowする。producerはGUPTをby valueでconsumeし、
`SourcePrimaryTermHandoffInput`と`TypedArena`からatomicに構築する。
handoffとerror enumはいずれも`Debug + Clone + PartialEq + Eq`を実装し、errorは
さらに`std::error::Error`を実装する。

exact profileは`2/2/0`である。term 0/1はsite node 3/4、range
`116..117`/`120..121`、source ordinal 0/1、context 1、Normal、spelling `y`、
`VariableReference`/`Value`、parentなし。reference 0/1は各term、binding 1、
`Variable`、producer-derived scope `[0]`、use ordinalは双方2。reserve binding 0と
witness binding 1が両use以前にcompleteし、first occurrenceはbinding eventを
増やさないためである。numeric requestは0。context 1/scope `[0]` lookupは両方
local binding 1を返す。

generic `SourcePrimaryTermProducer::build`の既存allowlist behaviorは不変。
private GU profileだけがexact compositeで`GivenWitness -> Variable`を許可する。
これによりcanonical決定をunauthenticated global admissionへ拡張しない。

arenaはGUPT prefixを保つdistinct 6 nodeである。node 0/1/2は
`reserve-type@14..17`、`type@84..87`、`type-root@0..127 children [0,1]`。
node 3/4は`source.term.variable-reference` at `116..117`/`120..121`。
node 5は`source.proof-local.given-use.term-root@0..127 children [2,3,4]`でroot。
全nodeはresolved nodeなし、Unknown、Normal、empty links。dependencyはnode
0--2を含む全public component/fingerprintで再認証し、standalone GUPTの3-node
contractを弱めない。source/module/payload/fingerprint/lookup/arena mismatchは
atomic failureである。

debug headerは`source-proof-local-given-use-term-debug-v1`、module、quoted
complete dependency fingerprint、quoted complete source-term fingerprint。
error stringは`source proof-local given-use term dependency is invalid`、
`source proof-local given-use source term is invalid`、`source proof-local
given-use term installation is invalid`。
validation precedenceはdependency/GUPT prefix再認証、exact source-term input/
profile検証、exact full-arena/one-shot installation検証の順である。どのfailureも
partial handoffまたはAST ownerをpublishしない。

### Typed/final ownership、runner、scope

`TypedAst`/`ResolvedTypedAst`はboxed optional
`source_proof_local_given_use_term`だけをownし、getter/one-shot installerと
exact Invalid errorを追加する。direct GUPT/binding/type/term fieldはinstallしない。
six nodesはfinalで`source.proof-local.given-use.term`へone-for-one projectionする。
全semantic table/node-hint inputはempty、全old ownerとboth-order mutually exclusive。

dormant runner outputは`{ typed_ast, resolved }`、mutationは`None`、
`WrongDependencyModule`、`WrongTermRange`、`WrongReferenceBinding`、
`WrongArenaRoot`、`WrongArenaKind`。selector argsはGUPTと同じでcfg-test版だけ
mutationを追加する。mismatchは`None`、selected failureは`Some(Err(_))`。
route-local stringは`Task269GU GUPT dependency is missing`だけ。exact GUPT
private outputからauthenticated owned dependencyをcloneし、2 term/referenceと
6-node arenaだけを構築する。public dispatchから到達不能。

implementationはexact 7 Rust files、checker `source_term.rs`/`typed_ast.rs`/
`resolved_typed_ast.rs`とrunner proof-local leaf/facade 2件/test leafだけ。
`source_type.rs`、`source_proof_local_declaration.rs`、`binding_env.rs`、runner
`source_statement.rs`、parser/resolver、canonical spec、fixture/sidecar/
expectation/trace/metadata/Cargo/diagnostic/dispatch/CLI/active resultは禁止。

checker/runner各4 testで、2 occurrence/lookup、全fingerprint、全dependency/input/
arena corruptionとprecedence、clone replay、one-shot/both-order ownership、generic/
Let/old-Given/GUP/GUPT/near-miss isolation、zero semantic publicationを網羅する。
docsはchecker 14 pairs、runner 6 pairs、global 2の42 Markdown。baselineは
libraries `506/568`、production `30/174332`/`37/75074`、implementation projection
`510/572`。path/content/test-list hash、corpus `428/395`、pass/fail `235/193`、
warnings/errors `23/0`、stage `101/7/205/1`、type coverage `259=247+12`、trace
SHA `55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`と
全CLI/fixture hashはEN canonical記載値を共有し、artifact/countは変更しない。

GUのownershipはlater primary-term occurrence/referenceだけ。equality/
BuiltinPredicateApplication/proposition/`thus`、condition/label/fact、existential/
Skolem、guard/assume、capture/export、goal/thesis、initial obligation、proof/
discharge/acceptance、theorem acceptance、Core/CFG/VCは明示deferする。

exitはEN/JA spec review NO FINDINGS、docs-only 9 hard gates uncapped `>=90/100`、
docs prerequisite commit、fresh preflight、exact seven-file/eight-test implementation、
test/implementation/source-docs各review NO FINDINGS、全verification/count/hash、
exact stage/implementation commit、clean tree/origin report/stash不変、その直後の
fresh next-task inventoryである。

Completion evidence: [central Task-269GU historical contract](../../task_contracts/ja/269GU.md#completion-evidence)。

## Task 269GCP frozen term deferral

condition leaf `107..108`/`111..112`はexact selector evidenceだけ。GCPは
`SourcePrimaryTermHandoff`、profile admission、Typed/final term ownerを追加しない。
Task269GCUはexact GC binding/GCT type dependencyをby-value取得後だけtransport可。

Completion evidence: [central Task-269GCP historical contract](../../task_contracts/ja/269GCP.md#completion-evidence)。

## Task 269GC frozen term deferral

GCはterm/occurrence/reference/use-site resolver/equality operand/Typed-final term
nodeをpublishしない。GCP-authenticated `107..108`/`111..112`はopaque。exact GCT
composite後GCUだけがtransport可、descendant useはlater。

Completion evidence: [central Task-269GC historical contract](../../task_contracts/ja/269GC.md#completion-evidence)。

## Task 269GCT frozen term deferral

written typeにterm argumentがないためsource-type argument tableはempty。term/
occurrence/reference/use-site resolver/equality operand/Typed-final term nodeなし。
condition leaf `107..108`/`111..112`はexcludedでexact GCT後GCUだけがtransport可。

Completion evidence: [central Task-269GCT historical contract](../../task_contracts/ja/269GCT.md#completion-evidence)。

## Task 269GCU frozen given-condition term/reference composition

GCUはexact `SourceProofLocalGivenConditionTypeHandoff`をby-value consumeし、
declaration conditionの`y@107..108`/`y@111..112`だけをpublishする。inputは
typed node 3/4、ordinal 0/1、context 1、normal、spelling `y`、
`VariableReference`/`Value`、parentなしのterm 2 rowsと、term 0/1から
`BindingId(1)`へ`Variable`としてlinkするreference 2 rows、numeric request 0。
common producerはuse ordinal 2をderiveし、scope `[0]`、type site
`Source(90..93)`のGCT-owned `GivenWitness`へunique lookupする。

private `SourcePrimaryTermBindingProfile::ProofLocalGivenConditionUse`だけが
`GivenWitness -> Variable`をadmitする。generic/older GU profileは不変。arenaは
GCT reserve-type/type/type-root nodes 0--2、variable-reference nodes 3/4、
`source.proof-local.given-condition.term-root@0..133` node 5 children
`[2,3,4]`のexact 6 nodes。root 5、全node unresolved/unknown/normal/link-free。

public handoff field orderはsource_id、module_id、
`SourceProofLocalGivenConditionTypeHandoff` dependency、dependency fingerprint、
`SourcePrimaryTermHandoff`、term fingerprint。getter、`debug_text`、crate-private
installation validatorを持つ。producer signatureは
`build(dependency, SourcePrimaryTermHandoffInput, &TypedArena)`。
non-exhaustive errorは`InvalidDependency`、`InvalidSourceTerm`、
`InvalidInstallation`で、この順にprecedenceを持つ。

debug headerは
`source-proof-local-given-condition-use-term-debug-v1`、続いてmodule、Rust-
Debug quoted complete GCT fingerprint、quoted source-term fingerprintを各LFで
renderする。Displayは順に
`source proof-local given-condition-use term dependency is invalid`、
`source proof-local given-condition-use source term is invalid`、
`source proof-local given-condition-use term installation is invalid`。

GCUはidentifier occurrence/reference transportだけをownする。label `G`、
equality/formula/condition/fact/guard/proof/obligation、later/descendant
occurrence、capture/export、generic source-term、active credit/dispatch、IRはexcluded。

Completion evidence: [central Task-269GCU historical contract](../../task_contracts/ja/269GCU.md#completion-evidence)。

## Task 269SDP term deferral

`y@118..119`と`z@133..134`はSurface TermReference syntaxだけ。primary-term/
reference winner/use ordinal/arena/type/captureはpublishしない。descendant
`y` occurrenceは後続、`z` replayは`set` reconciliation待ち。

Completion evidence: [central Task-269SDP historical contract](../../task_contracts/ja/269SDP.md#completion-evidence)。

## Task 269SDC frozen term/occurrence deferral

context-2 lookup oracleはabstract `BindingEnv`の継承可視性だけを証明する。
SDCは`y@118..119`のterm arena/reference row/request/source-site mappingを
作らず、descendant occurrence/capture creditは0。`z@114..115`、
`z@133..134`、`q@129..130`はopaque。occurrence consumerは別のGiven
type owner後で、全Set closure/captureはcanonical reconciliationまでblocked。

Completion evidence: [central Task-269SDC historical contract](../../task_contracts/ja/269SDC.md#completion-evidence)。
