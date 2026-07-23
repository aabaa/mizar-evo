# mizar-checker: Source and Binding Context Projection

> Canonical language: English. Japanese companion:
> [../ja/source_context.md](../ja/source_context.md).

## Purpose And Authority

`source_context` implements the Task 248 source/binding-context producer frozen
in [`00.crate_plan.md`](./00.crate_plan.md). Its language authority is limited
to Chapters 04 §4.3 and §4.6, 11 §11.2, 12 §12.3 and §12.7, and 15 §15.10.
It preserves source-item order, resolver shell provenance, distinct reserve and
definition-parameter identities, local shadowing, and checker context links.

## Boundary

The module accepts syntax-free projections. Opaque `DeclarationShellId` values
must come from the resolver's real `DeclarationShellSet`; the checker neither
constructs shell identities nor imports `mizar-syntax`. `mizar-test` owns the
bounded `SurfaceAst` walk and supplies source ranges, typed sites, lexical
scope, source order, and resolver-shaped `LocalTermBinding` provenance.

Task 248 admits exactly the named real-consumer transaction: one module-level
`reserve x for set;` item followed by one `definition` block with one local
`set` parameter named `x`. The Vec-based input and table shapes preserve order
and future extension seams, but no other cardinality or role combination is
accepted by this task. Additional reserve items, including canonical
distinct-name multiple-reserve input, are valid language shapes but are
rejected as `UnsupportedTaskShape` because they are outside this exact
transaction. Only the replacement or duplicate rule for re-reserving the same
identifier is undefined by the cited canonical specification; that nonblocking
`spec_gap` requires later human-reviewed authority before it can gain meaning.

The module does not normalize types, resolve use sites, evaluate RHS terms,
build facts or obligations, verify formulas/proofs, or implement Tasks 249+
or 269+. Steps 6/7 remain deferred.

## Projection Model

- `SourceBindingContextInput` carries source/module identity, the module typed
  site, ordered item shells, and ordered binding sites.
- Complete construction produces checker-owned source-item and declaration
  tables, one `BindingEnv`, exact binding-to-local-context links, and one
  immutable `SourceBindingContextHandoff` that owns its local-context table.
- `TypedAst` installs the handoff only when source/module identity, the entire
  local-context table, item/declaration sites, context links, and the module
  root owner agree. `ResolvedTypedAst` can only clone that installed handoff.
- Reserve and local parameter bindings retain distinct checker ids; the local
  row records the module reserve as its structural shadow predecessor.

## Validation, Recovery, And Atomicity

Validation rejects missing/duplicate/reordered rows, stale ordinals, source,
module or range mismatches, invalid parent/context/site links, unsupported
visibility, stale local provenance, wrong roles, duplicate local binders, and
partial payloads before publishing a complete handoff. The exact transaction
also requires both items to be top level and the definition parameter to have
the reserve spelling, so the structural shadow link cannot disappear.

A recovered definition shell is supported only when it claims no binding. The
producer then returns `SourceBindingContextIncomplete` with an empty recovered
context and one deterministic internal diagnostic. A recovered shell with a
binding is rejected. Incomplete or inconsistent data never installs any
source-context table in `TypedAst` or `ResolvedTypedAst`.

## Determinism And Coverage

Dense ids follow validated source order. Identical input yields equal tables
and byte-identical nonempty debug text; reordered input is rejected rather than
sorted. The legacy `TypedAst` path with no source-context handoff retains an
exact full-string debug oracle.

The real fixture
`pass_type_elaboration_source_binding_context_shadowing_001.miz` traverses the
frontend, resolver shells, producer, `TypedAst`, and `ResolvedTypedAst`. Its
runner test reconstructs corruption inputs only from those real opaque shell
ids, covers the frozen corruption/recovery/atomicity matrix, and keeps every
later type, fact, obligation, formula, statement, and proof payload empty.

## Public Enum Policy

| Public enum | Compatibility policy |
|---|---|
| `SourceItemRole` | `#[non_exhaustive]`; callers must tolerate later source-item roles. |
| `SourceItemVisibility` | `#[non_exhaustive]`; Task 248 accepts only `Unspecified`. |
| `SourceItemRecovery` | `#[non_exhaustive]`; callers must handle later recovery states. |
| `SourceBindingContextOwner` | `#[non_exhaustive]`; callers must tolerate later owner forms. |
| `SourceBindingSiteRole` | `#[non_exhaustive]`; callers must tolerate later binding roles. |
| `SourceBindingContextBuild` | `#[non_exhaustive]`; callers must distinguish complete and incomplete results. |
| `SourceContextError` | `#[non_exhaustive]`; callers must not exhaustively match validation failures. |

No exhaustive public enum exceptions are owned by this module.

## Task 248 Classification

| Class | Result |
|---|---|
| `test_gap` | Closed only for the exact named source/binding-context slice; broader canonical producer shapes remain with existing MC-G011/MC-G016 follow-up owners. |
| `source_drift` | Repaired by the real shell-to-checker handoff and immutable final projection. |
| `design_drift` | This module spec, paired audits, plan, todo, and harness records are synchronized. |
| `boundary_violation` | No current violation; shell fabrication and syntax imports are forbidden. |
| `spec_gap` | Only same-identifier re-reservation replacement/duplicate semantics remain undefined; this nonblocking gap does not authorize implementation. |
| `repo_metadata_conflict` | None observed. |
