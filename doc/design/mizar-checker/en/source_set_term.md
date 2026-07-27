# Source Set/Choice/Qua-Term Transport

> Canonical language: English. Japanese companion:
> [../ja/source_set_term.md](../ja/source_set_term.md).

## Scope

Checker Task 255 owns a syntax-free immutable description of source set
enumerations, condition-free independent set comprehensions, choice terms,
and `qua` terms. It transports source shape, transparent wrappers, written
comprehension generators, bare builtin target-type sites, ordered child
edges, and unresolved request intent only. It does not bind comprehension
variables, resolve capture or conditions, decide sethood or nonemptiness,
select a choice witness, establish `qua` reachability, compute result types,
publish facts, accept definitions, or lower proofs and IR.

The canonical language requirements are Chapter 13 Sections 13.4-13.6,
Chapter 7 Section 7.8.1, Chapter 8 Section 8.2.2, and their Chapter 17/21
semantic dependencies. Task 252 owns primary children, Task 253 owns
application children, Task 254 owns structure-family children, and Task 255
links to their dense root IDs without copying rows. Task 257 retains
comprehension binding/capture; Tasks 256-257 retain conditioned formula
ownership; later semantic owners retain all request resolution.

## Public Transaction

`SourceSetTermProducer::build` consumes `SourceSetTermHandoffInput`,
`BindingEnv`, `SourcePrimaryTermHandoff`, optional
`SourceFunctorApplicationHandoff` and `SourceStructureHandoff` dependencies,
and `TypedArena`. The input has six source-ordered vectors:

- set/choice/`qua` terms;
- transparent set-term wrappers;
- written comprehension generators;
- term- or generator-owned bare target-type sites;
- ordered enumeration-element, comprehension-mapper, and `qua`-base edges;
- unresolved result-type, generator-sethood, choice-nonempty, and
  `qua`-widening requests.

The producer publishes six dense immutable tables only after the entire
transaction validates. Public IDs expose zero-based `new` and `index`;
tables expose `get`, source-ordered `iter`, `len`, and `is_empty`; validated
rows expose only the read-only accessors frozen in the crate plan.

Term kinds are `Enumeration`, `Comprehension`, `Choice`, and `Qua`. Recovery
is `Normal` or `Degraded`. Type heads are bare `BuiltinSet` or
`BuiltinObject`. Targets are a Task-252 `Primary`, Task-253 root
`Application`, Task-254 root `Structure`, or later nested Task-255
`SetTerm`.

## Public Enum Policy

| Public enum | Compatibility policy |
|---|---|
| `SourceSetTermKind` | `#[non_exhaustive]`; callers must tolerate later frozen set-family source kinds. |
| `SourceSetTermRecovery` | `#[non_exhaustive]`; callers must tolerate later recovery classes. |
| `SourceSetTypeOwner` | `#[non_exhaustive]`; callers must tolerate later target-site owners. |
| `SourceSetTypeRole` | `#[non_exhaustive]`; callers must tolerate later term-owned target roles. |
| `SourceSetTypeHead` | `#[non_exhaustive]`; callers must tolerate later frozen bare builtin heads. |
| `SourceSetEdgeRole` | `#[non_exhaustive]`; callers must tolerate later child-edge roles. |
| `SourceSetTarget` | `#[non_exhaustive]`; callers must tolerate later frozen cross-family targets. |
| `SourceSetRequestKind` | `#[non_exhaustive]`; callers must tolerate later unresolved request kinds. |
| `SourceSetTermError` | `#[non_exhaustive]`; callers must not exhaustively match validation failures. |

No exhaustive public enum exceptions are owned by this module.

## Validation And Ownership

The producer authenticates source/module identity, dense source preorder,
context, ranges, recovery, exact typed-arena anchors, grouping, ordinals,
canonical token spelling, and single ownership. Arena keys are
`source.term.set.enumeration`, `.comprehension`, `.choice`, `.qua`,
`.parenthesized`, `.comprehension-generator`, `.target-type`, and
`.target-type-head`.

Canonical spelling is reconstructed recursively from authenticated rows.
Enumeration elements join with ` , ` inside `{ }`; a comprehension joins its
mapper, ` where `, and `identifier is type` generator fragments; choice is
`the type`; `qua` is `base qua type`; and every wrapper is
`format!("( {} )", contained_spelling)`. Generator spelling is one lexer
identifier. A bare type expression and head both spell exactly `set` or
`object`.

An enumeration owns zero or more element edges and one final result request.
A comprehension owns one or more generators, one bare type site and sethood
request per generator, one mapper edge, and one final result request. A
choice owns one target type site, one nonempty request, and one final result
request. A `qua` term owns one target type site, one base edge, one widening
request, and one final result request.

For every written child slot, validation computes the maximal effective
Task-252/253/254/255 occurrence after removing descendants. Exactly one
remaining occurrence must cover the complete child slot. A primary already
owned by Task 253 or 254 and an application already owned by Task 254 cannot
be targeted again. Nested Task-253, Task-254, and Task-255 descendants remain
with their nearest family owner. Reverse Task-253/254 parents containing a
Task-255 child, conditioned comprehensions, generator-referencing
comprehensions, non-bare targets, and all other frozen exclusions fail closed
without detached descendants.

## Derived Dependency Fingerprints

The output always derives `primary_term_fingerprint` from the exact Task-252
`debug_text()`. `application_fingerprint` and `structure_fingerprint` are the
exact dependency `debug_text()` values and are `Some` only when an edge
targets that family. Unrelated installed optional handoffs coexist with
`None` only when their effective occurrences are range-disjoint from all
Task-255 terms, wrappers, and targets.

`TypedAst::with_source_set_term` is one-shot and requires every targeted
dependency first. `with_source_application` and `with_source_structure`
revalidate an already installed Task-255 handoff, so installation order
cannot bypass ownership or fingerprint checks. `ResolvedTypedAst` revalidates
and clone-preserves the same association without rebuilding or renumbering
rows. Typed and resolved debug renderings include the handoff only when
present.

## Private Source Consumer

Raw `SurfaceAst`, source node IDs, and syntax kinds remain in
`mizar-test::runner::type_elaboration::source_set_term`. Production selects
only the four functor definientia in
`fail_type_elaboration_local_set_choice_qua_term_gap_001`. The leaf reuses
Task 248's real binding-context transaction and Task 252's primary producer;
it fabricates no comprehension `BindingId`.

The exact Task-255 term/wrapper/generator/type-site/edge/request oracle is
4/0/1/3/4/7. The shared arena contains the Task-252
primary/reference/numeric-request slice 4/0/4. The real route has no Task-253
or Task-254 row or fingerprint. After transport validation it retains the
Task-260 `type_elaboration.external_dependency.ast_payload_extraction`
boundary with no public diagnostic.

## Verification Boundary

Checker tests cover every table and enum, all arena keys, canonical spelling,
wrapper nesting, per-kind cardinality and request association, cross-family
nearest ownership, optional dependency fingerprints, installation orders,
corruption, determinism, clone preservation, and atomic failure. Runner tests
cover the exact consumer/oracle, real lower-stage shape, zero/many
enumerations, independent multiple/nested comprehensions, choice, `qua`,
wrappers, degraded transport, cross-family children, exclusions, mutation
isolation, deterministic replay, final ownership, and isolation from every
other active type-elaboration case.

The bounded trace row is
`spec.en.checker.type_elaboration.source_set_choice_qua_term_payload`.
Task 255 changes only executable source-transport coverage; generator/capture,
formula, typing, evidence, facts, proof, and Steps 6/7 semantics remain
unimplemented.
