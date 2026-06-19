# Module: imports

> Canonical language: English. Japanese companion: [../ja/imports.md](../ja/imports.md).

Status: task R-008 specification. Import graph, cycle, alias, relative-prefix,
and unresolved-import implementation starts in tasks R-009 and R-010. Export
validation grows with the later import, name, label, and symbol tasks that
populate the required resolver tables.

## Purpose

This module specifies resolver-owned semantic import and export resolution. It
turns parsed import/export directive shapes and the phase-0 module index into a
deterministic import graph, `ResolvedImport` entries, `ResolvedExport` entries,
and recoverable unresolved-import records.

The resolver does not pre-scan source text, tokenize, parse, discover packages,
load source files, or construct the build-side module index. Frontend
preprocessing may shallowly pre-scan candidate import stubs so tokenization can
load provisional lexical summaries, but those stubs are not authoritative.
Semantic import resolution revalidates every import from `SurfaceAst` before it
publishes resolver output.

## Inputs

- `SurfaceAst` import and export directive nodes, including source ranges,
  source order, and recovered syntax markers.
- The current module's canonical `ModuleId`.
- `ModuleIndexInput` from task R-007, backed by the build-side
  `ModuleIndexProvider` contract.
- Source-backed summaries for current-workspace modules and dependency summary
  projections as later resolver tasks provide them. This specification does not
  assume a `mizar-artifact` `ModuleSummary` schema before task R-024.

Malformed or recovered syntax remains visible to this phase when the parser can
produce a `SurfaceAst` node for it. If the parser cannot produce a directive
node, the resolver does not invent a semantic directive.

## Outputs

The import phase produces:

- one import-graph node for each module participating in semantic import
  resolution;
- import edges from the current module to successfully resolved dependency
  modules;
- a deterministic topological order for resolved acyclic graph nodes;
- `ResolvedImport` records for syntactic import candidates, including source
  range, source-order ordinal, canonical target module when available, optional
  local alias, and resolution status;
- `ResolvedExport` records for syntactic export candidates, including source
  range, source-order ordinal, canonical target when available, and resolution
  status;
- unresolved-import records that preserve the failed path spelling, source
  range, failure class, and any partial namespace/package candidate useful to
  later diagnostics.

Unresolved imports do not abort the module. The resolver records the failure,
omits the unavailable graph edge and imported export surface, and continues
with independent imports, exports, and local declarations.

The current `ResolvedAst` data shape contains a minimal unresolved-import record
with spelling, range, and failure class. The richer unresolved fields described
below are the design target for tasks R-009 and R-010; implementation may land
them incrementally without changing the semantic contract.

## Two-Pass Contract

### Pass A: Candidate Collection

Candidate collection walks parsed module-level import and export directive
nodes in source order. It records only syntax that the parser has represented in
`SurfaceAst`.

For each import declaration, Pass A records:

- source range and source-order ordinal;
- raw module path components;
- relative prefix (`.`, `..`, or none);
- optional alias spelling and alias range;
- recovery state if the directive is malformed but represented.

Branch imports expand into one candidate per branch member. Each expanded
candidate retains both the branch member span and the shared base path
provenance so diagnostics can point at the precise member without losing the
source context.

Pass A does not decide module existence, package identity, alias legality,
visibility, export validity, or cycles.

### Pass B: Semantic Validation

Semantic validation resolves collected candidates against `ModuleIndexInput`.
It:

- maps absolute imports through namespace roots and namespace bindings from the
  build-side module index;
- maps relative imports from the current module identity;
- binds aliases as local namespace spellings;
- validates export targets and private-item restrictions as far as currently
  available symbol/export summaries permit;
- builds import graph edges for resolved module imports;
- rejects import cycles;
- emits resolved and unresolved records in deterministic order.

Frontend import stubs and provisional lexical summaries may explain why a token
was classified during parsing, but they do not validate semantic import
legality. If frontend and resolver disagree, resolver output is authoritative
for later semantic phases.

When semantic validation rejects an import that contributed provisional
lexicon entries during frontend processing, resolver output must preserve enough
provenance for downstream consumers to mark dependent token classifications as
tainted. Batch verification suppresses semantic commitments that depend on
tainted lexicon provenance; LSP recovery may still use the tokenization for
navigation and follow-on diagnostics.

## Module Path Resolution

Import path resolution follows this order:

1. A path with `.` or `..` is relative to the current module and package.
2. A path whose first component matches a namespace root or package-name
   binding in the build-side module index is cross-package and resolves through
   that binding.
3. An unprefixed path that does not match a namespace binding is package-local
   and resolves against the current package.

Cross-package imports ask `ModuleIndexInput` for the matching package, then
resolve the remaining path components to a canonical module identity. Local
import aliases and source spelling are never part of the canonical `ModuleId`.

Package-local and relative imports use only the current module's package and
path:

- `.` resolves from the current module's containing module directory.
- `..` resolves from the parent of the current module's containing module
  directory.
- an unprefixed package-local path resolves from the current package root;
- Escaping the package root is invalid.

Branch import members inherit the base path's absolute or relative context.
The grammar currently provides only `.` and `..`; resolver behavior for deeper
relative prefixes is out of scope until parser syntax changes.

If an unprefixed first component could be interpreted both as a package-local
module component and as a namespace/package binding, the namespace binding wins
for cross-package import. A package-local module with the same first component
remains reachable through an explicit relative import.

## Alias Binding

An alias is a local namespace spelling for the imported module. It never changes
canonical module identity, exported module identity, graph order, or artifact
identity.

Alias binding rules:

- an import without `as` is visible through its canonical final module path
  component;
- an import with `as Alias` is visible through `Alias` inside the importing
  module;
- duplicate import declarations that resolve to the same canonical module are
  preserved as source records, while downstream import closure uses one
  canonical graph edge;
- duplicate aliases that point to different canonical modules are rejected
  deterministically;
- aliases that collide with reserved namespace roots or already-bound imported
  namespace spellings are rejected deterministically.

The resolver may keep crate-local failure classes for alias conflicts, but it
must not invent public diagnostic codes until the resolver diagnostic-code gap
is closed.

## Export Resolution

`ResolvedExport` represents export directives after semantic validation.

The resolver validates that:

- an exported module path resolves to a known module;
- an exported import alias names an import that was successfully bound;
- re-exported modules and symbols are public according to available summaries;
- private items are not copied into an export surface.

Detailed symbol and label export validation grows with the later name, label,
and symbol tasks. Until those tables exist, this phase records unresolved or
pending export targets rather than manufacturing checker-owned facts.
Export failure records include unresolved export targets and illegal private
re-exports as crate-local failure classes until public resolver diagnostic
codes are specified.

## Cycle Policy

Import graph cycles are forbidden. After resolving module import edges, the
resolver detects strongly connected components. Any component with more than
one module, or a self-edge, is rejected as cyclic.

Cycle records are deterministic:

- modules in a cycle are ordered by canonical `ModuleId`;
- edges are ordered by source range and then canonical target module;
- equal source positions are ordered by crate-local failure class and candidate
  key.

Cyclic imports make the affected graph edges unavailable to later import and
name resolution. Acyclic modules outside the rejected cycle remain available.

The topological order contains resolved acyclic modules only. Unresolved module
imports and rejected cyclic components are omitted from that order and retained
as unresolved/cycle records with source provenance. When several modules are
ready at the same time, the order is by canonical `ModuleId`.

## Unresolved Imports

Unresolved imports are first-class resolver output, not missing entries. Each
record preserves:

- the original source range and path spelling;
- normalized path components and relative prefix, when parseable;
- source-order ordinal;
- failure class;
- any partial package, namespace, or module candidate that was found before the
  failure;
- recovery state inherited from the parser when applicable.

Failure classes are crate-local until public resolver diagnostic codes are
specified. Required classes include unknown namespace/package, unknown module,
relative import escaping the package root, malformed recovered directive,
duplicate alias, alias/root conflict, unavailable dependency summary, illegal
import candidate state, and import cycle.

## Determinism

Resolution is deterministic for equivalent source, module-index input, and
available summaries.

- Source-order candidates are used for conflict checks and user-facing
  provenance.
- Canonical graph edges are deduplicated and sorted by target `ModuleId` after
  source-order conflict checks.
- `ResolvedImport` and `ResolvedExport` records preserve source-order ordinals
  and expose deterministic iteration.
- Unresolved and cycle records are sorted by source range, failure class, and
  stable candidate key.

## Boundary Notes

- Parser and syntax crates own directive syntax and recovery shape.
- Frontend and lexer crates own shallow pre-scan, tokenization, and provisional
  lexical summaries.
- `mizar-build` owns package planning, module discovery, namespace bindings,
  and the build-side module-index provider.
- The resolver owns semantic import/export validation, graph edges, alias
  bindings, cycle rejection, and unresolved-import representation.
- Checker, type, proof, and artifact crates own later type facts, proof facts,
  and persistent artifact schemas.
