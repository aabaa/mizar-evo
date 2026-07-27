# Source Functor-Application Transport

> Canonical language: English. Japanese companion:
> [../ja/source_application.md](../ja/source_application.md).

## Scope

Checker Task 253 owns a syntax-free, immutable description of source
functor-application occurrences. It transports source shape and unresolved
dependencies only. It does not decide candidate applicability, completeness,
viability, ranking, or a winner; infer semantic signatures or result types;
implement functor definitions or inline substitution; or create facts,
proofs, CoreIr, ControlFlowIr, or verification conditions.

The canonical language requirements are Chapters 10, 13 section 13.2, 15
section 15.2.3, and 19. Task 252 owns primary-term occurrences, binding
references, and numeric requests. Task 253 links to those dense IDs without
copying their rows. Tasks 270, 277, and 278 retain inline semantics, direct
template transport, and candidate collection/selection respectively.

## Public Transaction

`SourceFunctorApplicationProducer::build` consumes
`SourceFunctorApplicationHandoffInput`, `SymbolEnv`, `BindingEnv`,
`SourcePrimaryTermHandoff`, and `TypedArena`. The input has five source-ordered
vectors:

- applications;
- transparent application wrappers;
- individually authenticated resolver functor references;
- ordered argument edges to a Task-252 primary or later Task-253 application;
- unresolved candidate-signature and application-result type requests.

The producer publishes the corresponding five dense immutable tables only
after the complete transaction validates. Each public ID is a zero-based row
index with `new` and `index`; each table exposes `get`, source-ordered `iter`,
`len`, and `is_empty`. Rows expose read-only validated fields.

Application kinds are `Symbolic` and `Inline`. Source forms are `Bare`,
`Prefix`, `Infix`, `Postfix`, `Bracket`, and `Functional`. Inline rows admit
only the functional form and have no candidate or request rows. Symbolic rows
have one or more individually authenticated candidates, one signature request
per candidate, and one final application-result request.

## Public Enum Policy

| Public enum | Compatibility policy |
|---|---|
| `SourceFunctorApplicationKind` | `#[non_exhaustive]`; callers must tolerate later application-shape classes. |
| `SourceFunctorApplicationRecovery` | `#[non_exhaustive]`; callers must tolerate later recovery classes. |
| `SourceFunctorApplicationForm` | `#[non_exhaustive]`; callers must tolerate later written source forms. |
| `SourceFunctorHeadSite` | `#[non_exhaustive]`; callers must tolerate later head-site shapes. |
| `SourceFunctorArgumentTarget` | `#[non_exhaustive]`; callers must tolerate later frozen cross-family targets. |
| `SourceFunctorTypeRequestKind` | `#[non_exhaustive]`; callers must tolerate later unresolved request kinds. |
| `SourceFunctorApplicationError` | `#[non_exhaustive]`; callers must not exhaustively match validation failures. |

No exhaustive public enum exceptions are owned by this module.

## Validation And Ownership

The producer authenticates source/module identity, dense pre-order, grouping
and ordinals, contexts, nonempty ranges, exact typed-arena anchors and recovery
state, canonical token spelling, head position, delimiters, form/cardinality,
wrapper nesting, argument order and non-overlap, and single incoming ownership
for nested applications.

A primary argument must be an existing Task-252 root row in the same context
and inside its application. Inner primary descendants, duplicate primary
ownership, missing or partial argument lists, and cross-family targets without
a frozen owner fail atomically. Parentheses around an application are
Task-253 wrappers, outer-to-inner; they never become detached Task-252
parenthesized rows.

Candidate input contains only the application, ordinal, symbol, and source
contribution. The producer clones origin, visibility, export status, and the
optional resolver signature shell. It requires a non-recovered functor entry
and cross-index provenance. Same-module candidates additionally require a
normal conflict-free preceding functor definition; imported candidates
require public exported or re-exported provenance. Missing, pending, and
opaque signature shells remain unresolved provenance. Malformed signatures
are rejected.

## Derived Dependency Fingerprint

The output derives `primary_term_fingerprint` from the exact Task-252
`debug_text()` used during the build. `TypedAst::with_source_application`
requires the Task-252 handoff to be installed first, compares that exact
fingerprint, and revalidates every primary target. Replacement and
non-equivalent same-source/module substitution fail atomically; an equivalent
clone is accepted. If Task 254 is already installed, the same transaction
revalidates its structure handoff before publishing Task 253. This rejects
Task-253 argument ownership of a Task-254 primary target, reverse containment
or partial overlap with a Task-254 term, and any contained application not
owned by the closest Task-254 term, independent of installation order.
If Task 255 is already installed, the transaction also revalidates its
application fingerprint, root-only target, and nearest-family range
partition before publishing Task 253. Thus a later application cannot
contain, overlap, or retarget an installed Task-255 occurrence.

`ResolvedTypedAst` revalidates the same association and clone-preserves the
handoff. It never rebuilds or retargets dense IDs. Both AST debug renderings
include the handoff only when present.

## Private Source Consumer

Raw `SurfaceAst`, source node IDs, and syntax kinds remain in
`mizar-test::runner::type_elaboration::source_application`. Production selects
exactly two cases:

1. the imported `1 ++ 2` application inside `1 divides (1 ++ 2)`;
2. `task253_local_source(x)` in the second definiens of the frozen local
   two-functor definition block.

The aggregate application/wrapper/candidate/argument/request oracle is
2/1/2/3/4. The co-installed Task-252 primary/reference/numeric-request oracle
is 3/1/2. The local actual is the Task-248 definition parameter:
`BindingId(1)`, `BindingContextId(1)`, use ordinal 2. The imported
parentheses are one Task-253 wrapper and no Task-252 parenthesized row.

The imported case preserves its existing outcome, detail keys, and public
diagnostics. The local case validates Task 253 and then remains at the
Task-260 definition-declaration payload gap with stable detail
`type_elaboration.external_dependency.ast_payload_extraction` and no public
diagnostic.

## Verification Boundary

Checker tests cover dense tables, every form and cardinality rule, inline
schema, degraded recovery, wrapper ownership, root-only primary ownership,
dependency fingerprint substitution, nested applications, candidate
provenance/signature policy, requests, corruption, determinism, and atomic
failure. Runner tests cover both exact selectors, their aggregate oracle,
local binding coordinates, wrapper ownership, corruption isolation,
deterministic replay, final clone preservation, exclusion of every other
active type-elaboration case, and the complete ordinary/inline/nested/
parenthesized/wrapped/degraded/candidate-subset/template-and-mixed synthetic
matrix through the private extractor and public producer.

The bounded trace row is
`spec.en.checker.type_elaboration.source_functor_application_payload`. Task
253 changes MC-G017/MC-G020 executable coverage but leaves both gaps partial:
semantic term/formula/definition behavior, overload selection, later
cross-family terms, accepted facts/proofs, and Steps 6/7 remain unimplemented.
