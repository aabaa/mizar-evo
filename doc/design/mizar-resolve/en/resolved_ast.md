# mizar-resolve: ResolvedAst

> Canonical language: English. Japanese companion:
> [../ja/resolved_ast.md](../ja/resolved_ast.md).

## Purpose

`ResolvedAst` is the resolver-owned, source-shaped semantic snapshot for one
module. It preserves the `SurfaceAst` shape that later diagnostics and editor
features need, while attaching stable module, name, label, import, and origin
information that downstream phases can consume without redoing name resolution.

This document refines:

- [architecture 01](../../architecture/en/01.ir_layers.md) `ResolvedAst`;
- [architecture 03](../../architecture/en/03.module_and_symbol_resolution.md)
  interface definitions and recoverability policy.

## Boundary

`ResolvedAst` owns:

- canonical module identity for the resolved source unit;
- semantic import/export resolution results;
- namespace and symbol reference results;
- label reference results;
- explicit unresolved and ambiguous reference representation;
- recovered semantic shells for recoverable syntax;
- normalized semantic origins and provenance.

`ResolvedAst` does not own:

- parsing, parser recovery, or syntax vocabulary changes;
- type inference, selector type checking, or overload winner selection;
- cluster firing, coercion insertion, or registration activation;
- proof obligation generation or proof validity;
- artifact schema emission.

Known gap: resolver-specific public diagnostic code ranges are not yet reserved
in the external diagnostic specification. `ResolvedAst` may store structured
diagnostic anchors or crate-local diagnostic handles, but this spec does not
invent public user-facing resolver codes.

## Top-Level Shape

The top-level shape is:

```rust
struct ResolvedAst {
    source_id: SourceId,
    module_id: ModuleId,
    nodes: ResolvedArena,
    name_refs: NameRefTable,
    label_refs: LabelRefTable,
    imports: ResolvedImports,
}
```

`source_id` is the session-owned source identity for source-map lookup.
`module_id` is the canonical module identity from the resolver's module-index
input. `nodes` is an arena of source-shaped resolved nodes. The reference tables
record semantic decisions separately from the structural node arena so later
phases can inspect either source shape or reference outcomes directly.

## Stable Identity

### `ModuleId`

`ModuleId` is canonical and alias-independent. It consists of package identity
plus a normalized module path. Local import aliases, relative import spelling,
and source file spelling are not part of `ModuleId`.

`ModuleId` must not contain `SourceId`, absolute host paths, session-local
allocation counters, or display-only aliases.

### `SymbolId`

`SymbolId` is stable and fully qualified. It consists of:

- the declaring `ModuleId`;
- a resolver-owned local symbol identity derived from declaration kind and
  deterministic declaration position within the module;
- a fully qualified name projection used for artifacts, deterministic debug
  rendering, and candidate ordering.

When multiple declarations share a surface spelling, the local symbol identity
must include a deterministic overload slot, relation ordinal, or declaration
ordinal as specified by `symbols.md`. It must not depend on hash iteration
order, memory addresses, `SourceId`, or local import aliases.

`SymbolId` is assigned only to declarations that the resolver can represent as
semantic declarations. An unresolved or ambiguous reference must not receive a
fabricated `SymbolId`.

## Node Arena

`ResolvedArena` stores `ResolvedNode`s with stable `ResolvedNodeId`s.

Required node data:

- source-shaped node kind corresponding to the originating `SurfaceAst` shape;
- source range or generated/recovered anchor;
- zero or more child `ResolvedNodeId`s in source order;
- a `RecoveryState` flag;
- a `NodeResolutionState` that distinguishes resolved, unresolved, ambiguous,
  deferred, and not-applicable nodes when the node itself carries a resolver
  outcome;
- stable keys into `NameRefTable`, `LabelRefTable`, or `ResolvedImports` for
  node-local reference/import outcomes;
- a normalized `SemanticOrigin`;
- optional node-local payload for resolver-owned facts.

Arena invariants:

- every child id refers to a node allocated in the same arena;
- the root node belongs to `module_id`;
- parent/child edges are acyclic;
- child order is deterministic and source-shaped;
- repeated resolution of equivalent inputs produces the same ids and ordering;
- unknown or unsupported recovered syntax is represented as a recovered shell
  instead of being silently dropped when the parser produced a recoverable
  surface node.

`NodeResolutionState` is required for traversal semantics: a later phase walking
the arena must be able to see that a node is degraded even before it consults a
reference table. The detailed candidates and failure classes remain in the
tables so they have one canonical storage location.

The arena must not store inferred expression types, checker facts, final
overload results, or proof obligations.

## Name Reference Table

`NameRefTable` maps resolver-attempted name-use sites to `NameResolution`
results. A name-use site may be a whole node, a token inside a node, or a
resolver-created reference anchor, but its key must be stable inside the
`ResolvedAst`.

Required result variants:

- `Resolved(SymbolRef)` for a resolved declaration;
- `ResolvedBuiltin(BuiltinRef)` for built-ins whose identity is not a normal
  source declaration;
- `DeferredSelector(DeferredSelectorRef)` when dotted syntax has a term base and
  the remaining selector decision requires type information;
- `Ambiguous(AmbiguousNameRef)` with a deterministic candidate list;
- `Unresolved(UnresolvedNameRef)` with the attempted spelling and failing
  lookup class.

`SymbolRef` records the target `SymbolId`, the use-site range, and optional
import/provenance information. It may include the local spelling used at the
site for diagnostics, but the identity is the `SymbolId`.

Ambiguous candidate lists are sorted by canonical fully qualified name, then
module id, then source range. An unresolved or ambiguous root must be explicit
so later phases can skip or degrade dependent nodes without cascading fabricated
semantic identities.

## Label Reference Table

`LabelRefTable` is separate from `NameRefTable` because labels are scoped
separately from ordinary symbols.

Required result variants:

- `Resolved(LabelRef)` for a resolved theorem, definition, proof-step, or
  registration label;
- `Ambiguous(AmbiguousLabelRef)` with deterministic candidates;
- `Unresolved(UnresolvedLabelRef)` with the attempted label spelling and
  expected scope family. The expected family may be a concrete label kind or a
  mixed citation family such as proof-step-or-theorem for `by` references.

`LabelRef` records the normalized label-origin path and the use-site range.
The label-origin path must be stable enough for downstream `ObligationAnchor`
construction, but it must not imply that the resolver generates obligations.

The detailed label scope rules are specified in `labels.md`; this document
defines only the storage shape and invariants.

## Resolved Imports

`ResolvedImports` stores resolver outcomes for module import and export
directives.

Required contents:

- all import directives in source order;
- all export directives in source order;
- the owning `ResolvedNodeId` for each import/export directive outcome;
- canonical module targets for resolved imports/exports;
- local alias spelling when one was present;
- unresolved import/export entries with source spelling, range, and failure
  class;
- provenance links from name and label references back to the import edge that
  made a candidate visible, when applicable.

The canonical dependency projection may be exposed in deterministic
`ModuleId` order, but source-order records must remain available for diagnostics.
An unresolved import is represented explicitly and does not abort resolution of
the rest of the module.
Node-local import/export keys must point at an entry whose owner is the same
arena node.

The detailed alias, relative-prefix, and cycle rules are specified in
`imports.md`; this document defines only the storage shape and recoverability
requirements.

## Recovered Shells

When the parser marks a recoverable subtree, the resolver should preserve a
semantic shell if the subtree still has enough source shape to identify an item,
reference, label, import, or export position.

Recovered-shell rules:

- mark the corresponding node or table entry as recovered;
- keep source ranges and parser recovery anchors;
- record unresolved or ambiguous references explicitly;
- do not allocate `SymbolId`s for declarations whose identity cannot be
  represented deterministically;
- do not drop a recoverable shell merely because a later child is malformed;
- do not hide parser diagnostics or convert syntax recovery into semantic
  validity.

Later phases must treat recovered shells as degraded input and may skip facts
that depend on them.

## Semantic Origin And Provenance

Every resolved node, reference result, import/export result, and declaration
shell must carry normalized provenance sufficient for diagnostics, navigation,
incremental invalidation, and downstream anchor construction.

Required origin fields:

- `source_id` for source-map lookup;
- `module_id` for canonical module ownership;
- source range or generated/recovered anchor;
- source-shaped structural path or deterministic ordinal within the module;
- optional import edge id for facts introduced through imports;
- recovery marker when the origin came from recovered syntax.

Origins must be independent of absolute paths, memory addresses, hash-map
iteration order, and local import aliases. Source ranges are for diagnostics and
navigation; canonical identity must come from `ModuleId`, `SymbolId`, label
origin paths, or deterministic structural ordinals as appropriate.

Downstream `ObligationAnchor` construction may consume these origin fields, but
`ResolvedAst` does not create obligations.

## Determinism

All ids, table iteration, ambiguous candidate ordering, unresolved entry
ordering, and debug rendering inputs must be deterministic across runs and
platforms for equivalent source, module-index input, and dependency summaries.

Implementations must not expose raw `HashMap` or `HashSet` iteration order in
public renderings, snapshots, diagnostics, or serialized projections.

The human-readable debug rendering used for resolver snapshot baselines is a
versioned debug format, not a published artifact schema. It uses LF line
endings, locale-independent decimal formatting, deterministic string escaping,
and hand-written variant names rather than unstable implementation `Debug`
output.

## Public Enum Forward-Compatibility

Task R-026 applies the frontend task-25 public-enum decision procedure to this
module. All public resolver-owned enums in `resolved_ast` are forward-compatible
API surfaces and must remain `#[non_exhaustive]`:

- `RecoveryState`
- `NodeResolutionState`
- `NodeReferenceKey`
- `ResolvedArenaError`
- `NameLookupClass`
- `NameResolution`
- `LabelKind`
- `LabelExpectation`
- `LabelResolution`
- `ImportResolution`
- `ImportFailureClass`
- `ExportFailureClass`
- `ExportTarget`
- `ResolvedAstError`

No exhaustive public enum exceptions are owned by this module. Downstream
consumers must keep wildcard or fallback arms; resolver-internal matches may
remain exhaustive over the currently represented variants when implementing the
specified behavior.

## Planned Data-Shape Tests

Task R-004 must add focused unit tests for:

- deterministic `ModuleId`, `SymbolId`, and `ResolvedNodeId` allocation;
- arena child-id validation and cycle rejection;
- `NameRefTable` round-trips for resolved, unresolved, ambiguous, builtin, and
  deferred-selector results;
- `LabelRefTable` round-trips for resolved, unresolved, and ambiguous results;
- `ResolvedImports` source-order records and canonical target projections;
- `NodeResolutionState` preservation for unresolved, ambiguous, deferred, and
  recovered nodes during arena traversal;
- stable node-to-table and node-to-import keys that survive repeated resolution
  of equivalent inputs;
- recovered-shell flags and origin preservation;
- deterministic ordering for candidate lists and table iteration.
