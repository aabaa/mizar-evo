# Source Primary-Term Handoff

> Canonical language: English. Japanese companion:
> [../ja/source_term.md](../ja/source_term.md).

## Purpose And Authority

The public `source_term` module implements Checker Task 252. It transports
source occurrences of variable and local-constant references, `it`, numerals,
and transparent parentheses into the checker without importing raw syntax.
The canonical authority is Chapter 04 §§4.1-4.3, 4.4.1, and 4.6 and Chapter
13 §§13.1, 13.8.1-13.8.2, and 13.8.8. MC-G017 and MC-G020 track the broader
term and source-to-checker gaps.

The module is transport-only. It authenticates source shape, binding lookup,
and missing numeric-type requests. It does not choose a numeric type, create a
semantic term or formula, type a current definition result, publish a fact or
axiom, or create FOL/downstream IR.

## Public Model

`SourcePrimaryTermHandoffInput` carries one source/module transaction plus
three ordered input tables:

- `SourcePrimaryTermInput`;
- `SourcePrimaryTermReferenceInput`; and
- `SourceNumericTypeRequestInput`.

`SourcePrimaryTermProducer::build` authenticates those rows against a
syntax-free `BindingEnv` and `TypedArena`, then atomically publishes
`SourcePrimaryTermHandoff`. Its immutable
`SourcePrimaryTermTable`, `SourcePrimaryTermReferenceTable`, and
`SourceNumericTypeRequestTable` expose only borrowed lookup, source-ordered
iteration, length, and emptiness. Their dense identities are
`SourcePrimaryTermId`, `SourcePrimaryTermReferenceId`, and
`SourceNumericTypeRequestId`.

Term rows retain a node site, exact source range, dense pre-order source
ordinal, binding context, recovery, token-normalized spelling, kind, role,
and optional parent. Reference rows retain term and binding identity plus
role; lexical scope and use ordinal are producer-derived output. Numeric
requests retain the exact numeral term/site/range/spelling and a dense request
ordinal. `debug_text()` renders every table deterministically.

## Public Enum Policy

| Public enum | Compatibility policy |
|---|---|
| `SourcePrimaryTermKind` | `#[non_exhaustive]`; callers must tolerate later primary-term families. |
| `SourcePrimaryTermRole` | `#[non_exhaustive]`; callers must tolerate later source roles. |
| `SourcePrimaryTermReferenceRole` | `#[non_exhaustive]`; callers must tolerate later authenticated binding roles. |
| `SourcePrimaryTermRecovery` | `#[non_exhaustive]`; callers must tolerate later recovery classes. |
| `SourcePrimaryTermError` | `#[non_exhaustive]`; callers must not exhaustively match validation failures. |

No exhaustive public enum exceptions are owned by this module.

## Validation And Atomicity

Term ids and `source_ordinal` are equal dense pre-order indices. Every site is
a unique `TypedSiteRef::Node` whose arena kind, range, and recovery exactly
match the row. Identifier references use nonempty binding-authenticated
spellings accepted by the canonical `mizar_lexer::is_identifier` predicate:
an ASCII alphabetic or `_` start, ASCII alphanumeric, `_`, or apostrophe
continuations, and rejection of reserved words. This reuses lexical vocabulary
without importing raw syntax. `it` is exactly `it`, numerals contain only ASCII
digits, and a parenthesis spelling is exactly `( <child spelling> )` with one
ASCII space between tokens.

Each parent is an earlier parenthesis in the same context whose range strictly
contains its one immediate child. Only parents own children. Roots and
siblings remain source ordered, and nested parents form a closed acyclic
pre-order tree over the five Task-252 kinds. The private runner extractor
excludes an entire parenthesized subtree if any descendant belongs to a later
term family.

Variable and constant rows have exactly one reference. Variables accept only
`ReservedVariable`, `LetBinding`, `QuantifierBinder`, or
`DefinitionParameter`; constants accept only `LocalAbbreviation`. `it`,
numerals, and parentheses have no binding reference. Every numeral has
exactly one numeric request, and no other term kind has one.

For each reference, the producer clones lexical scope from the term context
and derives `use_ordinal` as the number of binding rows whose declaration
ranges end no later than the term start. Previous references do not advance
that ordinal. Normal binding groups are source ordered singletons with
visibility equal to their dense index. An exact consecutive duplicate group
shares spelling, kind, owner context, `BinderIdentity`, range, and the final
group row's dense index as its visibility ordinal. This preserves the whole
group until `BindingEnv::lookup` can reject it as `Ambiguous`.

The producer constructs `BindingLookupSite::new` with no resolver payload and
requires the exact supplied local binding winner. Forward, ambiguous, missing
scope payload, unresolved, different-winner, and lookup-error results fail
closed. `Resolver` is structurally unreachable on this path. Inputs are never
sorted, repaired, or partially published.

## Ownership And Consumers

`TypedAst::with_source_term` installs one optional immutable handoff after
revalidating source/module and every arena node; replacement is rejected.
`ResolvedTypedAst` only clone-preserves the handoff and exposes
`source_term()`.

The private `mizar-test::runner::type_elaboration::source_term` leaf owns raw
`SurfaceAst` extraction. Its exact real selector is:

1. `fail_type_elaboration_term_formula_gap_001`;
2. `pass_type_elaboration_reserved_variable_equality_001`; and
3. `pass_type_elaboration_parenthesized_reserved_variable_equality_001`.

Their aggregate handoff is seven terms, four references, and two numeric
requests. Existing semantic outcomes and detail keys remain unchanged.
Synthetic tests exercise local constants, `it`, nested parentheses, and
mixed-family exclusion without adding semantic acceptance.

## Verification And Deferrals

Checker tests cover every kind and role, dense order, binding-event order,
shadow/forward/ambiguous/missing/unresolved lookup behavior, reference and
numeric-request cardinality, parent graphs, source/module/site/range/kind/
spelling/recovery/context corruption, deterministic rendering, and typed-AST
installation. Runner tests cover the exact real selector, 7/4/2 oracle,
synthetic dependency boundaries, isolation, corruption, deterministic replay,
and final resolved preservation.

The covered trace requirement is
`spec.en.checker.type_elaboration.source_primary_term_payload`. Applications,
structure/set/choice/comprehension/`qua` terms, formula graphs, definition
result semantics, real proof-local constant production, numeric responses,
accepted facts/declarations/proofs, downstream IR, Tasks 253+, and Steps 6/7
remain with their explicit owners.
