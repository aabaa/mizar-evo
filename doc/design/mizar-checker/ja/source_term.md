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
