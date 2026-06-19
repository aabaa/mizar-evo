# mizar-resolve: SymbolEnv

> Canonical language: English. Japanese companion:
> [../ja/env.md](../ja/env.md).

## Purpose

`SymbolEnv` is the resolver-owned indexed signature environment visible from a
module. It is built after declaration shells and signature collection have
assigned stable semantic identities, and it is the boundary object consumed by
type checking, proof preparation, downstream module resolution, and incremental
invalidation.

This document refines:

- [architecture 01](../../architecture/en/01.ir_layers.md) `SymbolEnv`;
- [architecture 03](../../architecture/en/03.module_and_symbol_resolution.md)
  symbol environment and signature collection;
- [spec chapter 11](../../../spec/en/11.symbol_management.md) scope,
  visibility, imports, and public/private symbols.

## Boundary

`SymbolEnv` owns:

- indexes of local, imported, re-exported, and visible symbols;
- declaration definitions and opaque resolver-level signature shells;
- visible label projections and label contribution provenance;
- overload candidate groups before winner selection;
- resolver-side registration declarations before checker activation;
- namespace, import, export, alias, and re-export visibility graph edges;
- declaration dependency edges discovered without type checking;
- source contribution tracking for deterministic invalidation.

`SymbolEnv` does not own:

- parser or syntax recovery behavior;
- type inference or expression type facts;
- overload winner selection;
- selector type checking;
- cluster firing or registration activation for a term;
- proof obligation generation, ATP premise selection, or proof status;
- build planning, source loading, or artifact storage.

Known gap: resolver-specific public diagnostic code ranges are not yet reserved.
`SymbolEnv` may preserve structured diagnostic anchors and crate-local failure
classes, but this spec does not invent public user-facing resolver codes.

## Top-Level Shape

The top-level shape is:

```rust
struct SymbolEnv {
    module_id: ModuleId,
    imports: ResolvedImportIndex,
    exports: ResolvedExportIndex,
    symbols: SymbolIndex,
    labels: LabelIndex,
    definitions: DefinitionIndex,
    overloads: OverloadIndex,
    registrations: RegistrationIndex,
    namespace_graph: NamespaceGraph,
    declaration_dependencies: DeclarationDependencyIndex,
    contributions: SourceContributionIndex,
    module_summaries: ModuleSummaryIndex,
}
```

`module_id` is the canonical module whose environment is represented.
`imports` and `exports` are resolver-owned projections from `ResolvedAst`.
The index family is deterministic and query-oriented: later phases must be able
to look up by `SymbolId`, fully qualified name, visible spelling, namespace, or
source contribution without observing raw map iteration order.

`module_summaries` is the in-memory dependency-facing summary index available
to the resolver/checker boundary. Artifact-backed summary reuse is specified by
task R-024 and must not be invented by this data shape.

The `RegistrationIndex` in this document is the resolver-side declaration index
for registrations before checker validation and obligation acceptance. It is
distinct from the checker-side active registration index described in
[architecture 04](../../architecture/en/04.type_and_registration_resolution.md).

## SymbolIndex

`SymbolIndex` stores all symbol identities known to the environment:

- local declarations in the current module;
- imported public symbols visible through resolved imports;
- legal re-exports visible through facade modules;
- built-in prelude symbols enabled by the edition, when represented as
  resolver-visible candidates.

Required lookup projections:

- `SymbolId` to symbol entry;
- fully qualified name to `SymbolId`;
- visible spelling and namespace to deterministic candidate list;
- defining `ModuleId` to exported entries;
- source contribution id to contributed entries.

Each symbol entry records:

- `SymbolId`;
- kind;
- visibility and export status;
- primary spelling and optional notation spelling;
- defining origin and contribution id;
- optional opaque signature shell populated by signature-collection tasks;
- relation metadata for synonyms, antonyms, and redefinitions when those are
  resolver-owned declaration facts.

`SymbolIndex` must not store inferred expression types, selected overload
winners, activated registration facts, or proof validity.

## LabelIndex

`LabelIndex` stores label declarations and visible label projections separately
from ordinary symbols.

Required projections:

- local labels declared by the current module;
- exported theorem/lemma labels visible to importers;
- imported public labels visible through resolved imports;
- label origin path to label entry;
- visible label spelling and namespace to deterministic candidate list;
- source contribution id to contributed labels.

Each label entry records:

- stable label identity or origin path;
- label kind;
- visibility and export status;
- primary spelling;
- defining origin and contribution id;
- recovery state when the label shell came from recovered syntax.

`LabelIndex` does not resolve proof validity or generate obligation anchors. It
preserves the label provenance consumed later by proof and VC phases.

## DefinitionIndex

`DefinitionIndex` stores declaration definitions keyed by `SymbolId`.

It records resolver-owned declaration facts:

- declaration kind and visibility;
- parameter and binder shell ids;
- syntactic arity and notation shape;
- source doc/comment attachment ids when available;
- normalized semantic origin;
- duplicate/conflict classification produced by declaration collection;
- syntactic dependency references discovered during declaration collection;
- opaque per-kind signature payloads once `symbols.md` defines them.

`DefinitionIndex` is a declaration-pass index. It may record that a signature is
malformed or recovered, but it must not decide whether a type expression is
well-typed or whether a proof obligation is satisfied.

## OverloadIndex

`OverloadIndex` groups candidate declarations that share a resolver-visible
spelling or notation slot.

Required grouping keys:

- namespace or module visibility context;
- surface spelling or symbolic notation;
- symbol kind family;
- arity or syntactic shape when available without type checking.

Candidate order is deterministic: canonical fully qualified name, symbol kind,
source range, and declaration ordinal are used as tie breakers.

`OverloadIndex` records candidate availability and illegal grouping diagnostics.
It does not select the winning overload, rank candidates by inferred types, or
insert coercions.

## RegistrationIndex

`RegistrationIndex` stores registration declarations before checker activation.

Required contents:

- registration `SymbolId` or declaration id;
- registration kind and syntactic target shell;
- visibility/export status where applicable;
- normalized origin and contribution id;
- dependency references to declarations mentioned syntactically;
- recovery state for malformed registration shells.

The index may expose deterministic candidate lists to the checker. It must not
fire registrations, compute cluster closure, or decide applicability for a
specific term.

## DeclarationDependencyIndex

`DeclarationDependencyIndex` records resolver-visible declaration dependency
edges discovered without type checking.

Required edge data:

- source endpoint: declaration `SymbolId`, import/export entry id, namespace
  edge id, label origin path, or unresolved reference key;
- target `SymbolId`, `ModuleId`, label origin path, or unresolved reference key;
- dependency kind such as import edge, re-export edge, signature mention,
  synonym target, antonym target, redefinition target, registration mention, or
  label citation;
- source range or recovered anchor;
- source contribution id.

The index is used for deterministic invalidation and dependency diagnostics. It
must not encode type-derived dependencies, cluster firing traces, selected
overload winners, or proof-obligation dependencies.

Import, export, and namespace facade structure remains owned by
`NamespaceGraph` and the import/export indexes. `DeclarationDependencyIndex`
stores only the dependency edge needed to explain or invalidate a declaration,
label, or projection that depends on that structure.

## NamespaceGraph

`NamespaceGraph` models the resolver-visible namespace and module visibility
relationships.

Required node and edge kinds:

- canonical module nodes keyed by `ModuleId`;
- local alias nodes from imports;
- export and re-export facade edges;
- namespace segment edges used for qualified lookup;
- built-in prelude root edges when edition-enabled;
- unresolved or recovered edges represented explicitly when recovery continues.

Each edge records:

- source range or generated/recovered anchor;
- source contribution id;
- visibility;
- canonical target identity when resolved;
- local spelling when it differs from canonical identity.

The graph must not discover packages, load sources, or construct the build-side
module index. It consumes the resolver-side module-index input defined by task
R-007.

## Source Contribution Tracking

`SourceContributionIndex` records which source unit or dependency summary
contributed each environment entry.

Contribution records must include:

- contribution id stable within the environment;
- contributing `ModuleId`;
- `SourceId` for workspace source contributions when available;
- dependency summary identity/hash for summary-backed contributions when
  available;
- source range, generated anchor, or recovered anchor;
- produced symbol ids, definition ids, overload group ids, registration ids,
  label ids, namespace edges, declaration dependency edges, import/export
  entries, and diagnostics anchors.

Canonical symbol identity must not depend on `SourceId`; contribution tracking
uses `SourceId` only to map back to session source maps and invalidation inputs.

When a source unit is reparsed or a dependency summary changes, the contribution
index identifies the entries to remove, replace, or revalidate. Downstream
queries must be able to distinguish:

- local current-module contributions;
- imported source-backed dependency contributions;
- artifact/summary-backed dependency contributions;
- built-in/prelude contributions.

## Invalidation Notes

`SymbolEnv` invalidation is deterministic and contribution-based.

A current module edit invalidates:

- that module's local `SymbolEnv`;
- downstream `ResolvedAst` and `SymbolEnv` instances that import changed
  exported symbols, labels, namespace edges, or re-export projections;
- checker/type/proof phases that consume changed symbol or registration
  entries.

An imported module summary edit invalidates:

- visible symbol and label projections derived from that summary;
- overload groups containing changed candidates;
- label projections and declaration dependency edges derived from that summary;
- namespace edges and re-export projections derived from that summary;
- downstream modules whose contribution records reference the changed summary.

Changes that affect comments or formatting only must not change stable ids or
ordering unless they change a recorded origin range, doc/comment attachment, or
source hash that is explicitly part of the consuming cache key.

The environment cache key includes the resolver schema version, `ResolvedAst`
identity, module-index input identity, dependency summary identities, language
edition, and contribution fingerprints. The exact cache-key structure is owned
by the future incremental build/cache layer; this spec defines only the
resolver data needed to compute it.

## Determinism

All indexes expose deterministic iteration order. Implementations must sort or
use ordered maps before exposing:

- symbol entries;
- label entries;
- definition entries;
- overload candidates;
- registration entries;
- namespace graph nodes and edges;
- declaration dependency edges;
- contribution records;
- diagnostics and failure anchors.

Ordering must not depend on raw `HashMap`/`HashSet` iteration order, memory
addresses, filesystem traversal order, or local import alias spelling when a
canonical identity is available.

The human-readable `SymbolEnv` debug rendering used for snapshot baselines is a
versioned debug format, not a published artifact schema. It must render every
index family and contribution effect in a fixed section order, use LF line
endings, and avoid absolute paths, opaque source-id debug output, and derived
`Debug` text for externally visible variants.

## Planned Data-Shape Tests

Task R-005 must add focused unit tests for:

- insert/lookup round-trips for every index family;
- deterministic ordering for symbols, overload candidates, registrations,
  labels, namespace edges, declaration dependency edges, and contribution
  records;
- per-source contribution tracking for local source, imported source-backed
  dependencies, summary-backed dependencies, and built-ins;
- invalidation lookup from changed contribution to affected symbols, overload
  groups, labels, namespace edges, declaration dependencies, and registrations;
- no checker-owned facts in `SymbolEnv` entries;
- stable behavior across repeated construction from equivalent inputs.
