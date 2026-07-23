# mizar-checker: source attribute projection

> canonical languageはEnglish。English canonical:
> [../en/source_attribute.md](../en/source_attribute.md)。

## Purpose And Authority

`source_attribute`は[`00.crate_plan.md`](./00.crate_plan.md)でfreezeしたTask 250
raw attribute-chain/qualification/provenance producerを実装する。canonical
authorityはChapters 03 §§3.2-3.3、06 §§6.2/6.6/6.9、11 §§11.2-11.3、12
§§12.3/12.5/12.6.1/12.7、およびChapter-17 §§17.3/17.10
restricted-adjective boundaryである。bounded audit ownerはMC-G014/MC-G020で、
Tasks 248-249がinput dependencyである。

## Boundary And Model

moduleは`SurfaceAst`、`SurfaceNodeId`、syntax kindを受け取らない。
syntax-free `SourceAttributeHandoffInput`はdense chain、attribute、qualifier、
argument-group、argument tableを所有する。nonempty chainはauthenticated
Task-249 `SourceTypeExpressionId`をexactly one linkする。attribute rowはsource
order、polarity、full occurrence/target-name site・range・spelling、recovery、
resolver-authenticated attribute symbol/contribution identityを保持する。negative
rowはwritten `non` occurrenceを独立に保持する。

optional qualifier rowはwritten structure disambiguatorとauthenticated structure
symbol/contributionだけを保持し、owner compatibility/admissibility evidenceでは
ない。prefix groupとparenthesized-argument-list groupは独立したwritten formの
ままである。groupはexact delimiter/comma/hyphen provenanceを、actual rowは
selected `BindingId`、type result、normalized termなしで
`PrefixIdentifier`、`PrefixNumeral`、`TermSite` typed siteと
`SemanticOrigin`を保持する。

## Validation And Atomicity

`SourceAttributeProducer`はparent `SourceTypeApplicationHandoff`、
`BindingEnv`、`SymbolEnv`、`TypedArena`をauthenticateする。cross-source/module、
missing/duplicate parent、empty/reordered chain、dangling/multiply owned row、
stale site/range/spelling/recovery、overlapping source element、
polarity/`non` mismatch、wrong symbol role/contribution、useより後のlocal
declaration、invisible/out-of-closure imported symbol、unauthenticated qualifier、
invalid group form/punctuation/cardinality、invalid actual order/kind/originを
rejectする。spelling authenticationはactualからgroup、attributeまで
compositionalに行い、complete source-ordered attribute spellingは既に
authenticatedなTask-249 expression spellingのprefixでなければならない。

dense public vectorはsupplied orderのままvalidateする。sort、repair、normalize、
partial handoff publishは行わない。全typed siteは`TypedAst` install時にも
revalidateされる。failure時はtyped/resolved ASTにTask-250 payloadを残さない。

## Ownership, Consumers, And Exclusions

optional immutable handoffは`TypedAst`が所有する。`ResolvedTypedAst`はtyped AST
からcloneするだけで、separately replaceable resolved inputを持たない。handoff
absent時はconditional debug renderingによりlegacy byteを維持する。

real runner consumerはexactly Tasks 81/67/84/85である。aggregateはTask-249
`4/4/0`、Task-250
`4 chain / 4 attribute / 1 qualifier / 1 group / 1 actual`、polarity
positive 3/negative 1、attribute provenance local 2/imported 2、type head
builtin `set` 3/local structure 1である。synthetic private extractor probe
`p-ranked (q,2)-graded set`はpublic producerを通してtwo ordered attribute、
single/parenthesized prefix、exact punctuation、three prefix actualをcoverする。

Task 250はlater semantic owner向けにprefix/argument-list spellingを保持する。
arity/term typing、normalized instance/prefix-list equivalence decision、
admissibility、structure-owner compatibility、evidence request/result、
fact/truth/closure、accepted declaration/proof、Core/CFG/VCは実行しない。
legacy normalized `AttributeInput`は別bridgeのままである。

## Public Enum Policy

| Public enum | Compatibility policy |
|---|---|
| `SourceAttributePolarityInput` | `#[non_exhaustive]`。callerはlater polarity source formを許容する。 |
| `SourceAttributeArgumentGroupKind` | `#[non_exhaustive]`。callerはlater syntax-free group formを許容する。 |
| `SourceAttributePrefixForm` | `#[non_exhaustive]`。callerはlater prefix spellingを許容する。 |
| `SourceAttributeActualKind` | `#[non_exhaustive]`。callerはlater syntax-free actual kindを許容する。 |
| `SourceAttributeError` | `#[non_exhaustive]`。callerはvalidation failureをexhaustive matchしない。 |

この module が所有する exhaustive public enum exception はない。

## Task 250 Classification

| Class | Result |
|---|---|
| `test_gap` | exact four real handoffとsynthetic extractor/corruption matrixについてのみclose。 |
| `source_drift` | complete raw chain/polarity/qualifier/argument/provenance/final-handoff transportをrepair。 |
| `design_drift` | frozen contractとpaired component/plan/todo/audit/runner docsでrepair。 |
| `boundary_violation` | なし。raw syntaxはrunner-owned、later semantic decisionはdeferredのまま。 |
| `spec_gap` | bounded raw input-handoff sliceにはなし。 |
| `test_expectation_drift` | なし。authorized Task-67/81 pending progressionだけを変更し、Task-84/85 outcomeを維持。 |
| `repo_metadata_conflict` | 観測なし。 |

## Task 251 evidence-association addendum

attributed Task-251 requestはTask-249 root expressionをnameするexact Task-250
chainをconsumeする。request ownerはexpression siteのまま、request
site/range/recoveryはchain由来である。chainのdense source ordinalは独立に
authenticateされ、requestのapplication-ordinal fieldへcopyされない。result
kindは常に`AttributedTypeInhabitation`である。Task 251はこのimmutable handoffを
reuseし、selectorをwidenせず、syntax traversalをrepeatせず、attribute
admissibilityやtruthをpublishしない。
