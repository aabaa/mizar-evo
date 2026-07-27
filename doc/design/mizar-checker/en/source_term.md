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

## Task 257B1 Consumer Addendum

Task 257B1 adds one exact pass consumer with two additional
`VariableReference`/`Value` rows and two binding references. Both references
select the explicit quantifier's binding 0 in body context 1. Task 252 keeps
exclusive occurrence and lookup-winner ownership; the formula-composition
handoff records only binder-to-reference associations and does not repurpose
captured-free-variable metadata.

Task 257B2 reuses sixteen numeral rows and sixteen numeric-type requests in
body context 1. It intentionally creates zero references: the explicit `x`
binder is unused, captured identities remain empty, and the composition layer
does not invent bound-use rows.

## Task 257B3 Frozen Consumer Addendum

Task 257B3 reuses exactly six `VariableReference`/`Value` terms and six
Task-252 lookup-selected references. In source order, three `x` occurrences
select outer quantifier binding 1, one `y` selects binding 2, and two `r`
occurrences select inner quantifier binding 3 rather than reserved binding 0.
Terms/references 0-1 use context 1 and rows 2-5 use context 3. Terms are
`VariableReference`/`Value`/`Normal` with source ordinals `0..5`; references
retain the variable role.
Scope paths and local identities are source-derived resolver-shaped preflight
facts, while use ordinals are authenticated Task-252 producer output. Formula
composition records owning-edge associations only; Task 252 keeps occurrence,
reference, spelling, lexical-scope, and lookup-winner ownership.

Task 257B3 now executes this six-row reciprocal consumer and verifies binding
ids `1,1,3,2,1,3` with use ordinals `2,2,4,4,4,4`; Task 252 retains
every occurrence and reference.

## Task 257C1 Frozen Consumer Addendum

Task 257C1 reuses exactly three Task-252 `Numeral`/`Value` primaries and
numeric requests, profile `3/0/3`, for source terms `1`, `2`, and `3`.
Primary 1 (`2`, `85..86`) is one occurrence: the new Task-256 shared-boundary
edge references it from both adjacent segment descriptions without duplicating
the term or request. Task 252 retains occurrence, spelling, range, arena, and
numeric-request ownership; predicate grouping and polarity remain Task 256.

Task 257C1 now exercises this frozen backlink in the active pass consumer.
The measured `3/0/3` profile and single middle-primary identity are preserved;
no Task-252 API or semantic numeric result changed.

## Task 255C1 Frozen Backlink

The exact conditioned-comprehension prerequisite builds one immutable
Task-252 `4/0/4` handoff. Primaries 0/1 are Task-253 mapper arguments;
primaries 2/3 are equality operands wholly inside the authenticated condition
range. The latter remain ordinary Task-252 occurrence/numeric-request rows
but have no Task-255 edge. This preserves the exact objects later Task 256
must target without granting formula or numeric semantics here.

## Task 255C1 Transport Result

The exact route now publishes that single `4/0/4` handoff. Copied, omitted,
or range-substituted condition primaries fail the complete Task-255
transaction, while both authentic condition operands remain available to
later Task-256 installation without a Task-255 edge.

## Task 257C2 Frozen Consumer Boundary

Task 257C2 reuses the same immutable `4/0/4` handoff. Task-256 equality edges
target primaries 2 and 3 directly; the Task-257C2 association targets only
the condition and formula IDs. No Task-252 row, request, parent, context,
fingerprint, debug byte, or numeric semantic meaning changes. The route is
gated on separate Task 256C1; Task 252 itself requires no compatibility edit.
