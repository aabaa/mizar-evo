# mizar-build Module Index

> Canonical language: English. Japanese companion: [../ja/module_index.md](../ja/module_index.md).

## Purpose

This document specifies the phase-0 module index produced by `mizar-build`
after `planner.md` has produced a deterministic `BuildPlan`.

It refines architecture 03, "Step 1: Build Package and Module Indexes". The
module index maps planned packages to canonical module identities and source or
artifact locations. The resolver consumes this index as a provider input; it
does not rediscover workspace roots or invent package identity on its own.

## Scope

`mizar-build` owns:

- package index entries derived from `BuildPlan.packages`;
- namespace bindings that map package names and any artifact-declared language
  namespace roots to package entries;
- discovery of current-workspace `.miz` source files below each workspace
  package source root;
- canonical `ModuleId` construction from package identity and package-relative
  module paths;
- loading or referencing dependency module summaries supplied by dependency
  artifacts;
- deterministic diagnostics for duplicate or invalid module entries.

`mizar-build` does not own:

- parsing imports, exports, declarations, or aliases;
- resolving relative import paths or validating visibility;
- assigning `SourceId`, `SourceMapId`, `BuildSnapshotId`, or source hashes;
- creating synthetic dependency summaries when artifacts are unavailable;
- choosing overloads, types, proof obligations, or checker inputs.

Those responsibilities remain with `mizar-frontend`, `mizar-resolve`,
`mizar-session`, and later semantic phases.

## Inputs

Module-index construction takes:

- a validated `BuildPlan`;
- a source-layout reader for workspace package source roots;
- dependency artifact metadata for registry or already-built package inputs;
- the same toolchain and edition identities recorded in the plan.

The source-layout reader may walk the filesystem, a virtual test fixture, or a
driver-provided snapshot view, but the resulting index must be byte-identical
for equivalent source layouts.

## Identity Model

Canonical module identity is package-scoped:

```rust
struct ModuleId {
    package: PackageId,
    path: ModulePath,
}

struct ModulePath {
    components: Vec<String>,
}
```

`PackageId` comes from the validated package manifest and lockfile. `ModulePath`
is derived from `source_relative_path`, the source-root-relative file path:

- `lib.miz` maps to `ModulePath("lib")`;
- `groups/basic.miz` maps to `ModulePath("groups.basic")`;
- the `.miz` extension is removed;
- path separators become `.`;
- every path component must be a valid language identifier;
- path spelling uses normalized `/` separators before module-path conversion.

Local import aliases are never part of `ModuleId`. For example, importing
`algebra.groups.basic as Grp` still resolves to the same `ModuleId` as importing
`algebra.groups.basic` without an alias. Alias binding is resolver-owned.

Within a package, a relative or unprefixed import path is interpreted by the
resolver against the current module and package. Cross-package imports use the
package namespace exposed by the package index.

The module index does not parse a surface import path, but it must expose the
namespace bindings the resolver needs to interpret one. Current `BuildPlan`
data always yields a package-name binding whose prefix is the validated package
id spelling, matching spec chapter 23 cross-package imports such as
`algebra.groups.basic`. When dependency artifacts or registry metadata declare
language roots from spec chapter 12 (`std`, `pub`, `pkg`, `dev`, `ext`), those
bindings are carried as additional `NamespaceIndexEntry` entries. The indexer must
not guess a language root that is absent from the plan or dependency artifact.
The resolver chooses which binding matches a surface path and reports unresolved
namespace roots.

## Data Model

The following shapes define the contract, not necessarily the exact Rust names:

```rust
struct ModuleIndex {
    packages: Vec<PackageIndexEntry>,
    namespace_bindings: Vec<NamespaceIndexEntry>,
    modules: Vec<ModuleIndexEntry>,
    dependency_summaries: Vec<DependencyModuleSummaryRef>,
}

struct PackageIndexEntry {
    package_id: PackageId,
    version: Version,
    edition: Edition,
    source: PackageIndexSource,
    dependencies: Vec<PackageId>,
}

struct NamespaceIndexEntry {
    root: NamespaceRoot,
    prefix: Vec<String>,
    package_id: PackageId,
}

enum NamespaceRoot {
    PackageName,
    Std,
    Pub,
    Pkg,
    Dev,
    Ext,
}

enum PackageIndexSource {
    Workspace {
        package_root: String,
        source_root: String,
        manifest_path: String,
    },
    RegistryArtifact {
        registry: String,
        checksum: String,
    },
}

struct ModuleIndexEntry {
    module: ModuleId,
    package_id: PackageId,
    module_path: ModulePath,
    location: ModuleIndexLocation,
    edition: Edition,
}

enum ModuleIndexLocation {
    WorkspaceFile {
        source_root: String,
        normalized_path: String,
        source_relative_path: String,
    },
    DependencySummary {
        artifact: String,
        content_hash: Hash,
    },
}

struct DependencyModuleSummaryRef {
    module: ModuleId,
    artifact: String,
    content_hash: Hash,
}
```

`package_root`, `source_root`, `manifest_path`,
`ModuleIndexLocation::WorkspaceFile.normalized_path`, and
`ModuleIndexLocation::WorkspaceFile.source_relative_path` are normalized plan
paths, not session source identities. For workspace modules:

- `normalized_path` is package-root-relative and includes the required `src/`
  boundary, for example `src/groups/basic.miz`;
- `source_relative_path` is source-root-relative and omits `src/`, for example
  `groups/basic.miz`;
- `ModulePath` is derived from `source_relative_path`;
- dependency-summary-backed modules do not carry source paths and must not be
  passed to source loading as if they were workspace files.

When a later phase loads source text, it must pass the package root and the
package-root-relative `normalized_path` through the `mizar-session`
source-loading boundary. `mizar-session` remains responsible for accepting the
path as a `NormalizedPath`, allocating `SourceId`, computing source hashes, and
recording snapshot membership.

Registry packages normally contribute dependency-summary-backed module entries,
not workspace source files. Every dependency module summary accepted from an
artifact produces both:

- one `DependencyModuleSummaryRef`, so dependency summary content and hashes can
  be requested by canonical `ModuleId`; and
- one `ModuleIndexEntry` with `ModuleIndexLocation::DependencySummary`, so
  `provider.module()` works uniformly for workspace and dependency modules.

If dependency artifacts are missing or contain no valid module summaries for a
planned dependency package, construction must report `MissingDependencySummary`
instead of fabricating module content. Provider `UnavailableDependencySummary`
errors are reserved for constructed indexes when callers ask for dependency
summary content for a module that is present but not dependency-summary-backed.

## Construction Algorithm

1. Start from `BuildPlan.packages` in deterministic package-plan order.
2. Create one `PackageIndexEntry` for every package plan.
3. Create canonical `NamespaceIndexEntry` records for each package-name binding
   and any artifact-declared language namespace bindings; reject duplicate
   `(root, prefix)` bindings that point at different packages.
4. For each workspace package, walk its `source_root` recursively and collect
   `.miz` files only.
5. Normalize each discovered path both relative to the package root
   (`normalized_path`, including `src/`) and relative to the package source root
   (`source_relative_path`, excluding `src/`).
6. Convert the normalized source-root-relative path to a package-scoped
   `ModulePath`.
7. Create one workspace-file-backed `ModuleIndexEntry` per source file.
8. Load or reference dependency module summary records supplied by dependency
   artifacts, keyed by canonical `ModuleId`, and create matching
   dependency-summary-backed module entries.
9. Sort namespace bindings by `(root, prefix, package_id)`.
10. Sort modules by `(package_id, module_path, location key)`.
11. Reject duplicate `(package_id, module_path)` entries.
12. Return either the complete `ModuleIndex` or deterministic diagnostics.

The indexer does not read source text for syntax, import, or declaration
content. It may need filesystem metadata to discover files, but semantic content
is loaded by later source/frontend phases.

## Resolver Provider Contract

The resolver consumes the module index through an immutable provider. The
provider is a phase-service boundary, not a driver entry point.

Conceptually, it must support:

```rust
trait ModuleIndexProvider {
    fn packages(&self) -> &[PackageIndexEntry];
    fn namespace_bindings(&self) -> &[NamespaceIndexEntry];
    fn package(&self, package: &PackageId) -> Result<&PackageIndexEntry, ModuleIndexProviderError>;
    fn package_for_namespace(&self, root: &NamespaceRoot, prefix: &[String]) -> Result<&PackageIndexEntry, ModuleIndexProviderError>;
    fn module(&self, module: &ModuleId) -> Result<&ModuleIndexEntry, ModuleIndexProviderError>;
    fn modules_for_package(&self, package: &PackageId) -> Result<&[ModuleIndexEntry], ModuleIndexProviderError>;
    fn dependency_summary(&self, module: &ModuleId) -> Result<&DependencyModuleSummaryRef, ModuleIndexProviderError>;
}
```

Provider invariants:

- lookups are keyed by canonical `PackageId` and `ModuleId`;
- local aliases, import spelling, and import order never change lookup results;
- all returned slices are in canonical order;
- repeated calls over the same index return equal values;
- provider errors are deterministic and side-effect free;
- provider errors must not allocate replacement identities or fall back to
  workspace stubs once a real `ModuleIndex` is supplied.
- `module()` must work for workspace modules and dependency-summary-backed
  modules; `dependency_summary()` succeeds only for modules whose location is
  `DependencySummary`.

`mizar-resolve` task 7 has landed the resolver-side seam and its test
workspace-stub provider. `mizar-build` defines and constructs the index, but it
does not own resolver fixtures or a resolver crate-local compatibility layer on
behalf of `mizar-resolve`.

## Diagnostics

Module-index diagnostics are phase-0 build diagnostics. Required categories:

| Category | Examples |
|---|---|
| source root | missing source root, unreadable source root |
| source path | file outside source root, unsupported path encoding |
| module path | invalid module component, empty module path |
| duplicate module | two files map to the same `(PackageId, ModulePath)` |
| namespace binding | duplicate prefix, unsupported namespace root, invalid prefix component |
| dependency artifact | missing module summary, malformed summary identity |
| provider | unknown package, unknown module, unavailable dependency summary |

Diagnostics must include the package id and stable diagnostic category. When
available, they also include the normalized path or module id and rejected
value. They are sorted by package id, module path, normalized path, category,
then value.

## Determinism

The module index must not depend on filesystem traversal order, hash-map
iteration order, platform path separators, local import aliases, current time,
or session-local source ids. Equivalent source trees and dependency artifacts
must produce byte-identical indexes and diagnostics.

Stable ordering rules:

- package entries follow `BuildPlan.packages`;
- namespace bindings sort by `(root, prefix, package_id)`, where
  `NamespaceRoot` variants sort in the enum order shown above;
- module entries sort by canonical `(PackageId, ModulePath, location key)`;
- a workspace-file location key is its `normalized_path`; a dependency-summary
  location key is `(artifact, content_hash)` using the canonical artifact path
  and content-hash encoding;
- dependency summaries sort by canonical `ModuleId`;
- duplicate diagnostics use the first canonical path as the kept entry and
  report later canonical paths in sorted order;
- provider results use the same canonical order as the stored index.

## Task 6 Exit Criteria

The implementation task that cites this spec must:

- build `ModuleIndex` from `BuildPlan` and workspace source layout;
- expose the build-side provider/accessor contract without allocating
  `SourceId` or snapshot ids;
- cover multi-package fixtures and alias-independent module identity;
- preserve deterministic output under shuffled source discovery;
- confirm `mizar-resolve` task 7 status before claiming provider parity with
  resolver stub fixtures.

If `mizar-resolve` task 7 is still open, task 6 must classify resolver stub
replacement and resolver-fixture parity as an external dependency gap, complete
only the `mizar-build` index and build-side provider slice, and leave
resolver-owned compatibility work to `mizar-resolve`.
