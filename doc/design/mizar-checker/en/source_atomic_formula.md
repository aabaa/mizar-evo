# Source Atomic-Formula Transport

> Canonical language: English. Japanese companion:
> [../ja/source_atomic_formula.md](../ja/source_atomic_formula.md).

## Scope

Checker Task 256 owns a syntax-free immutable description of bounded source
atomic formulas: ordinary predicate applications, equality, inequality,
membership, bare builtin type assertions, and simple imported attribute
assertions. It transports source occurrence identity, transparent wrappers,
predicate-head and resolver-candidate provenance, formula-owned asserted-type
or assertion-attribute sites, nearest-family direct term edges, and unresolved
expected-input requests only.

The authority is Chapters 9 and 14, with Chapters 3, 6, 13, and 19 defining
the owned type, attribute, term, and resolver boundaries. Task 252 owns
primary terms, Task 253 owns applications, Task 254 owns structure terms, and
Task 255 owns set/choice/`qua` terms. Task 256 links their dense root IDs
without copying rows. Predicate chains, negation, connectives, quantifiers,
condition formulas, candidate selection, assertion truth, formula results,
theorem acceptance, facts, proof, and downstream IR remain deferred.

## Public Transaction

`SourceAtomicFormulaProducer::build` consumes
`SourceAtomicFormulaHandoffInput`, `BindingEnv`, `SymbolEnv`, the required
`SourcePrimaryTermHandoff`, optional Task-253/254/255 handoffs, and the shared
`TypedArena`. The input has eight source-ordered vectors:

- atomic formulas;
- transparent formula wrappers;
- ordinary predicate heads;
- individually resolver-authenticated predicate candidates;
- formula-owned bare asserted-type sites;
- formula-owned simple assertion attributes;
- ordered formula-to-nearest-term-family edges;
- unresolved operand, candidate-signature, type-reachability, and attribute
  admissibility requests.

The producer publishes eight dense immutable tables only after the entire
transaction validates. Public IDs expose zero-based `new` and `index`; tables
expose `get`, source-ordered `iter`, `len`, and `is_empty`; validated rows
expose read-only accessors. The handoff always fingerprints Task 252 and
conditionally fingerprints Task 253, Task 254, or Task 255 only when an edge
targets that family.

## Public Enum Policy

| Public enum | Compatibility policy |
|---|---|
| `SourceAtomicFormulaKind` | `#[non_exhaustive]`; callers must tolerate later frozen atomic source kinds. |
| `SourceAtomicFormulaRecovery` | `#[non_exhaustive]`; callers must tolerate later recovery classes. |
| `SourceAssertionTypeHead` | `#[non_exhaustive]`; callers must tolerate later frozen bare builtin heads. |
| `SourceAssertionAttributePolarityInput` | `#[non_exhaustive]`; callers must tolerate later source polarity forms. |
| `SourceAtomicEdgeRole` | `#[non_exhaustive]`; callers must tolerate later direct-slot roles. |
| `SourceAtomicTermTarget` | `#[non_exhaustive]`; callers must tolerate later frozen cross-family targets. |
| `SourceAtomicRequestKind` | `#[non_exhaustive]`; callers must tolerate later unresolved request kinds. |
| `SourceAtomicFormulaError` | `#[non_exhaustive]`; callers must not exhaustively match validation failures. |

No exhaustive public enum exceptions are owned by this module.

## Validation And Ownership

Validation authenticates source/module identity, dense source order, context,
recovery, ranges, typed-arena keys, canonical token spelling, formula-local
ordinals, table associations, resolver symbol/contribution provenance, and
single ownership. Formula keys distinguish predicate, equality, inequality,
membership, type assertion, and attribute assertion. Dedicated keys own
predicate heads, asserted type expression/head sites, attribute occurrence
and target sites, `non`, and transparent wrappers.

Each direct written term slot resolves to exactly one maximal root occurrence
from Task 252, 253, 254, or 255. Descendants remain with the nearest term
family. Duplicate, overlapping, partial, non-root, reverse-contained, or
cross-context targets fail atomically. An unrelated optional handoff may
coexist with an absent fingerprint only when its occurrences are disjoint
from all formula, wrapper, and direct-slot ranges.

Predicate formulas require one ordinary head, one or more authenticated
candidates, and one candidate-signature request per candidate. Equality and
inequality require two operand requests. Membership retains only its
right/container operand request. A bare type assertion requires one asserted
type site and one reachability request. Each simple attribute assertion
requires one or more authenticated attribute rows and one admissibility
request per attribute. Requests publish intent only; they contain no answer,
selected candidate, type, fact, or truth.

## AST Installation

`TypedAst::with_source_atomic_formula` is one-shot and requires every targeted
lower-family dependency first. Later Task-253/254/255 installers revalidate
an already installed Task-256 handoff so installation order cannot bypass
fingerprint or ownership checks. Replacement and non-equivalent dependency
substitution fail without changing the AST.

`ResolvedTypedAst::assemble` revalidates and clone-preserves the exact handoff
without rebuilding or renumbering rows. Typed and resolved debug renderings
include the handoff only when present. The handoff adds no semantic type,
fact, coercion, obligation, diagnostic, expression metadata, or cluster fact.

## Private Source Consumer

Raw `SurfaceAst`, source node IDs, and syntax kinds remain in
`mizar-test::runner::type_elaboration::source_atomic_formula`. Production
selects exactly eight existing active fixtures: numeral equality, inequality,
membership, bare builtin type assertion, imported predicate/functor,
positive and negative imported attribute assertions, and set-enumeration
equality.

Across the eight transactions the Task-256
formula/wrapper/predicate-head/candidate/type-site/attribute/edge/request
aggregate is `8/0/1/1/1/2/13/11`. The shared lower-family aggregate is Task
252 `16/0/16`, Task 253 `1/1/1/2/2`, and Task 255
`2/0/0/0/4/2`; no real Task-254 target exists. The private composer builds
each selected transaction in one arena, then runs the existing semantic route
unchanged, so all outcome and detail keys remain byte-identical.

## Verification Boundary

Checker tests cover dense tables, formula kinds, wrappers, canonical
spelling, provenance, request cardinality, arena and dependency identity,
nearest-family ownership, corruption, deterministic replay, installation,
and atomic failure. Runner tests cover all eight exact consumers, ordered
edges and requests, lower-family fingerprints, imported provenance and
anchors, same-arena composition, selector isolation, mutation failure, final
`TypedAst`/`ResolvedTypedAst` ownership, and unchanged external details.

The bounded trace row is
`spec.en.checker.type_elaboration.source_atomic_formula_payload`. Task 256
adds executable source-transport coverage only; semantic formula work and
Steps 6/7 remain unimplemented.

## Task 257B1 Consumer Addendum

Task 257B1 reuses this module's existing equality and two primary-term operand
edges as an authenticated dependency for one universal body. Atomic-formula
row ownership, validation, and semantic deferrals do not change; the new
formula-composition handoff stores only the cross-family parent association.

Task 257B2 reuses eight equality rows with exact profile
`8/0/0/0/0/0/16/16`. Their sixteen existing operand edges remain owned here;
the new composition table only associates those atomic roots with repeated or
fixed conjunction/disjunction parents and does not change atomic semantics.

## Task 257B3 Frozen Consumer Addendum

Task 257B3 reuses exactly three equality rows with profile
`3/0/0/0/0/0/6/6`: outer restriction `x = x`, inner restriction `r = y`,
and innermost body `x = r`. Their six Task-252 operand edges and six
unresolved operand-type requests remain owned here. Formula composition adds
only two restriction-parent associations and one body-parent association; it
does not change equality truth or operand typing.
Atom 0 and terms 0/1 use nested context 1; atoms 1/2 and terms 2..5 use
context 3. All three atoms are `Equality`/`Normal` with source ordinals
`0..2`. Source order, spelling, range containment, and request/edge ordinals
remain exact profile discriminators.

Task 257B3 is now an executable reciprocal consumer of these exact three
atoms and six operand rows. Atomic ownership and all semantic deferrals remain
unchanged.
