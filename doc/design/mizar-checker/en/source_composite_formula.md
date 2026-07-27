# Source Composite-Formula Transport

> Canonical language: English. Japanese companion:
> [../ja/source_composite_formula.md](../ja/source_composite_formula.md).

## Responsibility And Authority

Checker Task 257A owns a syntax-free immutable transport for the exact
implication/universal/negation/contradiction source tree and its one explicit
quantifier binder. The canonical behavior comes from Chapter 14 formula and
quantifier rules, Chapter 4 binder identity and scope rules, Chapter 3's
written `set` type, and the unchanged
`fail_type_elaboration_formula_connective_quantifier_gap_001.miz` intent.
Raw `SurfaceAst` traversal remains private to `mizar-test`.

This module does not evaluate connectives, relativize quantifiers, assign
truth, create formula facts, own theorem semantics, search proofs, or produce
Core/CFG/VC output. Broader connective and quantifier shapes, bound uses and
capture, predicate chains, and conditioned comprehensions remain Tasks 257B
and 257C.

## Public Transaction

`SourceCompositeFormulaHandoffInput` carries seven dense vectors in the frozen
order: formulas, transparent wrappers, roots, binders, binder type sites,
formula edges, and unresolved requests. The dense identities are
`SourceCompositeFormulaId`, `SourceFormulaWrapperId`,
`SourceFormulaRootId`, `SourceQuantifierBinderId`,
`SourceBinderTypeSiteId`, `SourceFormulaEdgeId`, and
`SourceFormulaRequestId`.

The flat input rows are `SourceCompositeFormulaInput`,
`SourceFormulaWrapperInput`, `SourceFormulaRootInput`,
`SourceQuantifierBinderInput`, `SourceBinderTypeSiteInput`,
`SourceFormulaEdgeInput`, and `SourceFormulaRequestInput`. After validation,
the immutable rows `SourceCompositeFormula`, `SourceFormulaWrapper`,
`SourceFormulaRoot`, `SourceQuantifierBinder`, `SourceBinderTypeSite`,
`SourceFormulaEdge`, and `SourceFormulaRequest` expose read-only accessors.
Their tables expose only `get`, source-ordered `iter`, `len`, and `is_empty`.

`SourceCompositeFormulaProducer::extend_bindings` validates the exact normal
Task-248-era module shell and atomically returns the Task-257A `2/1/4`
environment. `SourceCompositeFormulaProducer::build` revalidates the same
input and exact extended environment, clone-owns the environment, and returns
`SourceCompositeFormulaHandoff`. `SourceCompositeFormulaError` reports atomic
validation failure; neither step has a partial publication path.

The exact real table counts are `5/0/1/1/1/4/6`. Formula ids are
parent-before-child preorder. The sole root is unassigned to a statement.
Edges retain implication-left, implication-right, universal-body, and
negated-formula roles. Requests retain only unresolved connective, constant,
quantifier, binder-type, and negation intent.

## Binder Environment

The input is extended from one normal module context, no bindings, and four
canonical external-gap diagnostics. Context 1 is a normal expression child
owned by `BindingContextOwner::SourceFormula`, anchored to the universal
range, with `LocalTermScope([0])`. It owns and exposes binding 0 only.

Binding 0 is the source-derived `x` quantifier binder with a resolver-local
identity, declaration range `78..79`, visible-after ordinal 0, and written
type site `Source(86..89)`. The binder row retains its segment and identifier
sites and links body context 1 and type-site 0. The type-site row is evaluated
in context 0 and retains the written `TypeExpression` and `TypeHead` sites for
builtin `set`. No Task-248 `SourceBindingContextHandoff` is fabricated.

## Validation And Ownership

Validation authenticates source/module identity, the exact base and extended
binding environments, dense row order, source ranges and typed-arena keys,
canonical spellings, normal recovery, a single complete tree, the unique
context transition, binder scope and identity, type association, and all
request associations. Task 257B1 supersedes the former Task-257A-only
synthetic-wrapper admission: both currently admitted exact profiles require
an empty wrapper table. The public wrapper row/table shape remains reserved,
but executable parenthesized formula occurrences and their nesting contract
are deferred to Task 257B2; any nonempty wrapper shape is currently rejected
atomically as an unowned third profile.

`TypedAst::with_source_composite_formula` is one-shot, revalidates the complete
handoff, and rejects an already installed source-context handoff.
`TypedAst::source_composite_formula` and
`ResolvedTypedAst::source_composite_formula` expose the immutable handoff.
Final assembly clone-preserves and revalidates it without rebuilding from raw
source. `debug_text` renders the complete embedded binding environment and
all seven tables deterministically; legacy AST debug bytes remain conditional
and unchanged when Task 257A is absent.

## Real Consumer And Tests

The private `mizar-test::runner::type_elaboration::source_composite_formula`
leaf is the sole real consumer. It extends the existing exact selector so it
retains binder segment, identifier, type expression, and type head sites. It
constructs the dedicated `1/0/4` base, runs the public extension and build,
installs the handoff, and assembles the resolved AST before the older semantic
route runs. The existing two semantic detail keys remain unchanged.

Checker tests cover the seven-table aggregate, full literal debug oracle,
deterministic replay, rejection of the retired synthetic-wrapper/third-profile
shapes, binding extension, arena vocabulary, cross-table corruption, one-shot
installation, and legacy debug bytes.
Runner tests cover the real sites and corrected parser ranges, exact selector
isolation, unchanged external details, corruption recovery, clone-preserving
final ownership, and preinstalled Task-248 rejection.

## Public Enum Policy

| Public enum | Compatibility policy |
|---|---|
| `SourceCompositeFormulaKind` | `#[non_exhaustive]`; callers must tolerate later frozen composite source kinds. |
| `SourceCompositeFormulaRecovery` | `#[non_exhaustive]`; callers must tolerate later recovery classes. |
| `SourceFormulaRootOwnership` | `#[non_exhaustive]`; callers must tolerate later authenticated root owners. |
| `SourceBinderTypeHead` | `#[non_exhaustive]`; callers must tolerate later frozen binder type heads. |
| `SourceFormulaEdgeRole` | `#[non_exhaustive]`; callers must tolerate later composite child roles. |
| `SourceFormulaRequestKind` | `#[non_exhaustive]`; callers must tolerate later unresolved request kinds. |
| `SourceCompositeFormulaError` | `#[non_exhaustive]`; callers must not exhaustively match validation failures. |

No exhaustive public enum exceptions are owned by this module.

## Task 257B1 Profile Addendum

Task 257B1 adds the second exact `1/0/1/1/1/0/2` profile without changing
the Task-257A `5/0/1/1/1/4/6` input or debug bytes. The producer derives the
profile from validated table shape and rejects A/B hybrids or any third
shape. The legacy `with_source_composite_formula` installer remains
Task-257A-only; the second profile is publishable only with its Task-252/256
dependencies and `1/2` cross-family handoff through the combined installer
specified in [source_formula_composition.md](./source_formula_composition.md).

## Task 257B2 Implemented Third Profile

The producer now admits exactly `8/6/1/1/1/7/9`, with the frozen universal,
`iff`, disjunction, repeated/fixed conjunction/disjunction preorder, six
independent wrappers, seven exact child edges, and nine unresolved requests.
Validation rejects wrapper crossing/order drift, fixed/repeated substitution,
A/B1/B2 hybrids, and an otherwise coherent unsupported fourth profile.
Legacy installation remains Task-257A-only; B1/B2 publish only through the
combined formula-composition installer.

## Task 257B3 Frozen Fourth Profile

The future fourth profile is exactly `3/0/1/3/3/2/6`: source-preorder
`Universal`, new `Existential`, `Universal`; no wrappers; one unassigned
root; three binders and type sites; outer `UniversalBody` plus new
`ExistentialBody` edges; and quantifier/binder-type request pairs in formula
order. The first two binder type sites are written bare `set` expressions.
The third authenticates the Task-48 reserved-default `set` source and binding
for an unqualified segment; it does not invent a child type expression.
The exact kind/context/spelling rows are `Universal@0 "for st"`,
`Existential@1 "ex st"`, and `Universal@2 "for st holds"`; binder segments
are `"x being"`, `"y being"`, and `"r"`.

Binding extension starts from the exact one-reserve module environment, adds
three resolver-local quantifier binders in nested contexts, and preserves the
inner `r` binder's shadow of the reserved `r`. Exact local scopes and ranges
are preflight-measured. Validation rejects missing or substituted reserve
provenance, flattened contexts, wrong shadow targets, A/B1/B2/B3 hybrids, and
any otherwise coherent fifth profile. B3 remains publishable only through the
combined formula-composition installer.

## Task 257B3 Implementation Status

`Existential`, `ExistentialBody`, the exact fourth profile, and cumulative
multi-binder extension are implemented. The exact reserve base extends to
four contexts/four bindings with zero diagnostics, preserving inner-`r`
shadowing and empty capture.

## Task 257C3 Frozen Non-Ownership

Predicate-chain implicit conjunction and segment negation use the separate
syntax-free Task-257C3 association handoff. They do not fabricate
source-composite formula, wrapper, root, edge, binder, type-site, or request
rows. Existing Task-257A/B profiles and debug bytes remain unchanged, and
their installers will only add reciprocal exclusion against the future C3
owner.

## Task 257C3 Implemented Non-Ownership

The C3 owner is now installed separately. Task-257A/B production rows and
debug bytes remain unchanged; their installers add only the reciprocal C3
occupancy guard. Mutation-sensitive tests exercise A/B-after-C3 and
C3-after-A/B without relying on unrelated dependency mismatch.
