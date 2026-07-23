# mizar-checker: Source Type Application Projection

> Canonical language: English. Japanese companion:
> [../ja/source_type.md](../ja/source_type.md).

## Purpose And Authority

`source_type` implements the Task 249 source type-head/application/argument
producer frozen in [`00.crate_plan.md`](./00.crate_plan.md). Its canonical
authority is Chapters 03 §§3.2-3.3, 05 §§5.2/5.6, 07 §§7.2-7.3/7.6, 08
§§8.1/8.3, 12 §§12.3/12.5/12.6.1/12.7, 18 §§18.1/18.2.2, and Appendix A.
The bounded audit owners are MC-G014, MC-G016, and MC-G020.

## Boundary And Model

The module accepts no `SurfaceAst`, `SurfaceNodeId`, or `SyntaxKind`. One
syntax-free `SourceTypeHandoffInput` contains dense outer-application,
expression/head, and ordered argument vectors. Applications link authenticated
reserve or definition bindings to root expressions. Expressions retain written
and head sites, ranges, spellings, recovery, form, and builtin or
resolver-authenticated mode/structure heads. Arguments retain exactly
`TermSite`, recursive `TypeSite`, or `QuaSite` input; term and `qua` sites carry
`SemanticOrigin` but no selected `BindingId`.

`SourceTypeProducer` authenticates the input against the actual `BindingEnv`,
`SymbolEnv`, and `TypedArena` before publishing
`SourceTypeApplicationHandoff`. The legacy reserve bridge exposes
`prepare_binding_env` as an input-only path; it validates symbol heads and
builds the real binding environment without declaration checking or type
normalization. Definition-parameter applications require an actual resolver
`DeclarationShell` owner; a generated context is never authenticated as a
declaration.

## Validation And Atomicity

Validation rejects cross-source/module data, stale binding identity/order/type
sites, unsupported head kinds, stale contribution provenance, local heads not
active before their use, invisible imported heads, missing or out-of-closure
import edges/targets, invalid or duplicate typed sites, empty spellings,
range/recovery mismatches, and invalid `SemanticOrigin`. Term/`qua` provenance
must use the exact identifier range, current source/module, no import edge,
matching recovery, and deterministic
`[parent-expression, argument-ordinal]` structural path.

The flat graph rejects dangling, cyclic, multiply parented, forward-parent,
duplicate-child, wrong-form, unreachable, non-contained, and overlapping
sibling/top-level relationships. Cycle and reachability checks use iterative
worklists, so public flat input does not consume the call stack. Validation
never sorts or repairs input. Failure publishes no partial handoff.

Every expression, head, term, and `qua` site is checked against its actual
typed-arena node both during production and during `TypedAst` installation.
The owning node must have a same-source range containing the narrower row range
and exactly matching recovery. This permits distinct role sites on the
existing Task-248 item nodes without changing that arena.

## Ownership, Consumers, And Exclusions

`TypedAst` owns the optional immutable handoff. `ResolvedTypedAst` can only
clone it from that typed AST; no separately replaceable resolved input exists.
Conditional debug rendering keeps legacy bytes unchanged when the handoff is
absent.

The broad real consumer traverses exactly ten reserve written types and
publishes 10 applications, 13 expressions/heads, and 6 arguments. The
Task-248 route separately co-installs two `Bare`/builtin-`set` rows and no
arguments using its actual checker-owned binding environment. Expansion,
normalization, inhabitation, subtyping, evidence, term or `qua` binding
selection, facts, declaration/proof acceptance, and Core/CFG/VC production are
outside Task 249.

## Public Enum Policy

| Public enum | Compatibility policy |
|---|---|
| `SourceTypeApplicationForm` | `#[non_exhaustive]`; callers must tolerate later source-written forms. |
| `SourceTypeHead` | `#[non_exhaustive]`; callers must tolerate later authenticated head kinds. |
| `SourceTypeArgument` | `#[non_exhaustive]`; callers must tolerate later syntax-free argument shapes. |
| `SourceTypeError` | `#[non_exhaustive]`; callers must not exhaustively match validation failures. |

No exhaustive public enum exceptions are owned by this module.

## Task 249 Classification

| Class | Result |
|---|---|
| `test_gap` | Closed only for the exact Task-249 handoff and Task-248 dependency consumer. |
| `source_drift` | Repaired for complete type-head/application/argument and final-handoff transport, import-closure authentication, and real `DeclarationShell` ownership. |
| `design_drift` | Repaired by the paired component, plan, todo, audit, and runner documents. |
| `boundary_violation` | Recursive public-input graph traversal found in implementation review was replaced by iterative worklists; syntax remains runner-owned and semantic result fabrication is forbidden. |
| `spec_gap` | None for this bounded input-handoff slice. |
| `repo_metadata_conflict` | None observed. |
