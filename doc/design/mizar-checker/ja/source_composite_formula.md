# Source Composite-Formula Transport

> canonical languageは英語。英語版:
> [../en/source_composite_formula.md](../en/source_composite_formula.md)。

## Responsibility And Authority

Checker Task 257Aはexact
implication/universal/negation/contradiction source treeと1件のexplicit
quantifier binderをsyntax-free immutable transportとしてownする。canonical
behaviorはChapter 14のformula/quantifier rule、Chapter 4のbinder identity/scope
rule、Chapter 3のwritten `set` type、およびunchanged
`fail_type_elaboration_formula_connective_quantifier_gap_001.miz` intentに
従う。raw `SurfaceAst` traversalはprivate `mizar-test` ownershipのままである。

このmoduleはconnective evaluation、quantifier relativization、truth、formula
fact、theorem semantics、proof search、Core/CFG/VCを作らない。broader
connective/quantifier shape、bound use/capture、predicate chain、conditioned
comprehensionはTasks 257B/257Cに残す。

## Public Transaction

`SourceCompositeFormulaHandoffInput`はformula、transparent wrapper、root、
binder、binder type site、formula edge、unresolved requestの7 dense vectorを
frozen orderで持つ。dense identityは`SourceCompositeFormulaId`、
`SourceFormulaWrapperId`、`SourceFormulaRootId`、
`SourceQuantifierBinderId`、`SourceBinderTypeSiteId`、
`SourceFormulaEdgeId`、`SourceFormulaRequestId`である。

flat input rowは`SourceCompositeFormulaInput`、
`SourceFormulaWrapperInput`、`SourceFormulaRootInput`、
`SourceQuantifierBinderInput`、`SourceBinderTypeSiteInput`、
`SourceFormulaEdgeInput`、`SourceFormulaRequestInput`である。validation後の
immutable row `SourceCompositeFormula`、`SourceFormulaWrapper`、
`SourceFormulaRoot`、`SourceQuantifierBinder`、`SourceBinderTypeSite`、
`SourceFormulaEdge`、`SourceFormulaRequest`はread-only accessorを公開する。
tableは`get`、source-ordered `iter`、`len`、`is_empty`だけを公開する。

`SourceCompositeFormulaProducer::extend_bindings`はexact normal
Task-248-era module shellをvalidateし、Task-257A `2/1/4` environmentをatomicに
返す。`SourceCompositeFormulaProducer::build`は同じinputとexact extended
environmentを再validateしてclone-ownし、`SourceCompositeFormulaHandoff`を
返す。`SourceCompositeFormulaError`はatomic validation failureを表し、
partial publication pathはない。

real table countは`5/0/1/1/1/4/6`。formula idはparent-before-child preorder。
sole rootはstatementにunassigned。edgeはimplication-left/right、
universal-body、negated-formula roleを保持し、requestはunresolved
connective/constant/quantifier/binder-type/negation intentだけを保持する。

## Binder Environment

inputはnormal module context 1件、binding 0件、canonical external-gap
diagnostic 4件からextendする。context 1はuniversal rangeでanchorされた
`BindingContextOwner::SourceFormula` ownershipのnormal expression childで、
`LocalTermScope([0])`を持ちbinding 0だけをown/visibleにする。

binding 0はresolver-local identity、declaration range `78..79`、
visible-after ordinal 0、written type site `Source(86..89)`を持つsource-derived
`x` quantifier binderである。binder rowはsegment/identifier siteを保持し、
body context 1/type-site 0をlinkする。type-site rowはcontext 0で評価され、
builtin `set`のwritten `TypeExpression`/`TypeHead` siteを保持する。Task-248
`SourceBindingContextHandoff`はfabricateしない。

## Validation And Ownership

validationはsource/module identity、exact base/extended binding environment、
dense order、source range/typed-arena key、canonical spelling、normal recovery、
single complete tree、unique context transition、binder scope/identity、
type association、全request associationをauthenticateする。synthetic
transparent wrapperはstrict nesting、normal、context-preserving、single
represented formula ownershipを満たす。dense ordinalはsource-ordered
outermost-to-innermostで、canonical spellingは次のinner occurrenceをrecursiveに
wrapする。

`TypedAst::with_source_composite_formula`はone-shotで全handoffを再validateし、
preinstalled source-context handoffをrejectする。
`TypedAst::source_composite_formula`と
`ResolvedTypedAst::source_composite_formula`はimmutable handoffを公開する。
final assemblyはraw sourceからrebuildせずclone-preserve/revalidateする。
`debug_text`はembedded binding environmentと7 tableをdeterministicにrenderし、
Task 257A absent時のlegacy AST debug byteはunchangedである。

## Real Consumer And Tests

private `mizar-test::runner::type_elaboration::source_composite_formula` leafが
sole real consumerである。既存exact selectorをbinder segment/identifier/type
expression/type head site保持へ拡張し、dedicated `1/0/4` base、public extension/
build、install、resolved assemblyをolder semantic routeより前に実行する。
既存2 semantic detail keyはunchangedである。

checker testは7-table aggregate、full literal debug oracle、deterministic replay、
nested wrapper、binding extension、arena vocabulary、cross-table corruption、
one-shot install、legacy debug byteをcoverする。runner testはreal site/corrected
parser range、exact selector isolation、unchanged external detail、corruption
recovery、clone-preserving final ownership、preinstalled Task-248 rejectionを
coverする。

## Public Enum Policy

| Public enum | compatibility policy |
|---|---|
| `SourceCompositeFormulaKind` | `#[non_exhaustive]`。callerはlater frozen composite source kindを許容する。 |
| `SourceCompositeFormulaRecovery` | `#[non_exhaustive]`。callerはlater recovery classを許容する。 |
| `SourceFormulaRootOwnership` | `#[non_exhaustive]`。callerはlater authenticated root ownerを許容する。 |
| `SourceBinderTypeHead` | `#[non_exhaustive]`。callerはlater frozen binder type headを許容する。 |
| `SourceFormulaEdgeRole` | `#[non_exhaustive]`。callerはlater composite child roleを許容する。 |
| `SourceFormulaRequestKind` | `#[non_exhaustive]`。callerはlater unresolved request kindを許容する。 |
| `SourceCompositeFormulaError` | `#[non_exhaustive]`。callerはvalidation failureをexhaustive matchしない。 |

この module が所有する exhaustive public enum exception はない。
