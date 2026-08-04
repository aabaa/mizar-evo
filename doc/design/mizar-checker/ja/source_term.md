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

variable/constant rowはreference exactly 1件を持つ。variableは
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
