# Module: planner

> Canonical language: English. Japanese companion: [../ja/planner.md](../ja/planner.md).

## Purpose

`planner` implements pipeline phase 0 for `mizar-build`: it reads package,
workspace, and lockfile metadata and produces the deterministic `BuildPlan`
consumed by source loading, module indexing, resolver setup, and later
scheduler graph construction.

The planner owns package-level facts only. It validates manifests, resolves
package dependencies against the lockfile, records edition and verifier/build
configuration, and emits canonical package order. It does not read `.miz`
source text, assign source ids, create snapshots, parse imports, resolve names,
or run verification phases.

## Context

- [doc/spec/en/23.package_management_and_build_system.md](../../../spec/en/23.package_management_and_build_system.md)
- [doc/design/architecture/en/00.pipeline_overview.md](../../architecture/en/00.pipeline_overview.md)
- [doc/design/architecture/en/03.module_and_symbol_resolution.md](../../architecture/en/03.module_and_symbol_resolution.md)
- [doc/design/architecture/en/14.parallel_verification_and_scheduling.md](../../architecture/en/14.parallel_verification_and_scheduling.md)
- [doc/design/internal/en/01.compiler_driver_and_pipeline_scheduler.md](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md)
- [doc/design/mizar-session/en/source.md](../../mizar-session/en/source.md)
- [doc/design/mizar-session/en/snapshot.md](../../mizar-session/en/snapshot.md)

## Inputs

### Package Manifest: `mizar.pkg`

Every package has one TOML package manifest at its package root.

```toml
[package]
name = "algebra"
version = "1.2.3"
authors = ["Alice <alice@example.com>"]
license = "MIT"
description = "Algebraic structures"
edition = "2025"

[dependencies]
mml_core = "^1.0.0"
topology = { version = "^0.9.0", features = ["metric"] }

[dev-dependencies]
test_utils = "^0.2.0"

[verifier]
max_axioms = 128
atp_timeout = 30
default_solver = "auto"
require_kernel_certificates = true

[build]
incremental = true
cache_dir = ".mizar-cache"
artifact_dir = "build"
```

Required fields:

| Field | Rule |
|---|---|
| `package.name` | lowercase `snake_case`, matching `[a-z][a-z0-9]*(?:_[a-z0-9]+)*`; hyphenated spellings are rejected and never normalized |
| `package.version` | SemVer 2.0.0 version |

Optional package fields default as specified in chapter 23. Omitted
`package.edition` defaults to the current stable Mizar edition for the
toolchain; the initial supported stable edition is `2025`.

Dependency entries are keyed by package id and use either a version-constraint
string or an inline table with required `version` and optional `features`.
Dependency package ids use the same package-id spelling rule as
`package.name`. Feature strings are preserved as manifest data; feature
semantics belong to the future registry/feature resolver, not to phase 0.

`dev-dependencies` are parsed and validated like `dependencies`, but they are
marked with dependency kind `dev`. The `BuildPlan` always preserves both normal
and dev dependency records. The planner API takes a dependency selection from
the driver request (`normal` for ordinary verification, `normal + dev` for test
or development targets) and uses that selection to decide which edges are active
in the dependency graph for this plan.

Unknown top-level tables, unknown fields inside known tables, duplicate package
ids in dependency tables, absolute build paths, paths escaping the package
root, malformed SemVer versions, malformed version constraints, and invalid
solver names are manifest errors.

### Workspace Manifest: `mizar.workspace`

A multi-package workspace contains a TOML workspace manifest at the workspace
root.

```toml
[workspace]
members = ["algebra", "topology"]
```

`workspace.members` is required for a workspace manifest. Each member path is a
relative directory path from the workspace root. The special member path `.`
means the workspace root package. Other member paths must not be empty,
absolute, contain `.` or `..` components, escape the workspace root, or use
host-specific non-canonical spelling. Each member directory must contain a
`mizar.pkg`.

If `mizar.workspace` is absent, the workspace is a single-package workspace
whose root is the directory containing `mizar.pkg`. If both files are present
at the same root, the workspace manifest is authoritative for member discovery
and the root `mizar.pkg` is valid only when listed as a member by `members`.

Member packages share the workspace lockfile and cache root. Duplicate member
paths, duplicate package ids, and a member package whose manifest name differs
from the dependency key used for it are manifest errors.

### Lockfile: `mizar.lock`

The lockfile is TOML at the workspace root and records the exact dependency
assignment used for reproducible builds.

```toml
schema_version = 1

[[package]]
name = "algebra"
version = "1.2.3"
source = { kind = "workspace", path = "algebra" }
dependencies = [
  { name = "mml_core", version = "1.0.0" },
]

[[package]]
name = "mml_core"
version = "1.0.0"
source = { kind = "registry", registry = "default", checksum = "sha256:..." }
dependencies = []
```

`schema_version = 1` is the only supported lockfile schema for this planner.
Each package entry has:

| Field | Rule |
|---|---|
| `name` | package id spelling rule |
| `version` | exact SemVer version |
| `source.kind` | `workspace` or `registry` |
| `source.path` | required for workspace packages, relative to workspace root |
| `source.registry` | required for registry packages |
| `source.checksum` | required for registry packages |
| `dependencies` | array of exact `{ name, version }` entries |

Lockfile entries are unique by package id. A workspace member must appear in the
lockfile with matching exact version and `source.kind = "workspace"`. A
manifest dependency must appear in the lockfile with exactly one package entry
whose version satisfies the manifest constraint. A lockfile dependency edge must
refer to an existing locked package.

The planner validates an existing lockfile. It does not update or rewrite
`mizar.lock`; lockfile generation and registry solving are future work above
the same data model.

## BuildPlan Model

```rust
struct PlanRequest {
    workspace_root: WorkspaceRoot,
    dependency_selection: DependencySelection,
    toolchain: ToolchainInfo,
}

enum DependencySelection {
    Normal,
    NormalAndDev,
}

struct BuildPlan {
    workspace_root: WorkspaceRoot,
    packages: Vec<PackagePlan>,
    dependency_graph: DependencyGraph,
    lockfile: Lockfile,
    toolchain: ToolchainInfo,
    verifier_config: WorkspaceVerifierConfig,
    build_config: WorkspaceBuildConfig,
}

struct PackagePlan {
    package_id: PackageId,
    version: Version,
    source: PackagePlanSource,
    edition: Edition,
    dependencies: Vec<ResolvedPackageDependency>,
    verifier_config: VerifierConfig,
    build_config: BuildConfig,
}

enum PackagePlanSource {
    Workspace {
        root: String,
        source_root: String,
        manifest_path: String,
    },
    Registry {
        registry: String,
        checksum: String,
    },
}

struct ResolvedPackageDependency {
    package_id: String,
    requested: VersionConstraint,
    resolved: Version,
    kind: DependencyKind,
    features: Vec<String>,
}

enum DependencyKind {
    Normal,
    Dev,
}
```

`PackageId`, `Edition`, `WorkspaceRoot`, `ToolchainInfo`, and snapshot-facing
source identity wrappers come from `mizar-session`. The planner must provide
already validated package ids, editions, roots, and normalized paths to those
APIs. It must not manufacture `SourceId`, `BuildSnapshotId`, or source hashes.

For workspace packages, `source.source_root` is the normalized `src` directory
under the package root. This implementation slice exposes normalized plan paths
and registry metadata as `String` values; later source/snapshot APIs still
receive `mizar-session` identity wrappers at their own boundaries. The planner
records source-root paths but does not enumerate modules or require that the
directory already exists; source loading and module indexing report missing or
invalid source-tree diagnostics through their own error surfaces. Registry
packages use `PackagePlanSource::Registry` and are dependency-artifact inputs
rather than workspace source roots.

`packages` is sorted with dependencies before dependents. Ties are broken by
canonical package id. The root workspace members are preserved as package
plans; locked registry packages may appear as dependency package plans when
their artifacts are needed by module indexing or downstream phases.

`dependency_graph` records package-level correctness edges from dependent to
dependency. The `packages` vector is the reverse topological order of those
stored edges, so every dependency appears before every dependent. The graph
rejects cycles, duplicate package ids, duplicate dependency keys within a
package, missing lock entries, version constraints not satisfied by the
lockfile, and unsupported edition values. It never permits multiple
simultaneous versions of the same package id in one workspace. A lockfile
`source.kind = "workspace"` entry is valid only for a discovered workspace
member whose manifest was parsed and validated by this planner; non-member
dependencies must use registry source metadata in this implementation slice.

`WorkspaceVerifierConfig` and `WorkspaceBuildConfig` are deterministic
aggregates: they contain the package-plan-ordered list of each package's
effective config, while the adjacent `BuildPlan.toolchain` records the
toolchain identity used to interpret those configs. They are not cross-package
merges and do not erase package-specific settings. Snapshot/config hashes must
include these aggregate records or an equivalent canonical encoding of every
package's effective verifier and build config. Downstream code must use
package-local settings when a phase is package-scoped.

## Planning Algorithm

1. Discover workspace mode from `mizar.workspace` or `mizar.pkg`.
2. Parse TOML for the workspace manifest, every member package manifest, and
   the workspace lockfile.
3. Validate schema, package ids, SemVer versions, version constraints, enum
   values, config defaults, member paths, and build paths.
4. Build the package table keyed by package id and reject duplicates.
5. Validate that every workspace package has a matching lockfile entry and
   that every lockfile workspace-source entry corresponds to a discovered
   workspace package.
6. Validate each manifest dependency against the lockfile, collect exact
   resolved versions, and mark active edges according to the request's
   dependency selection.
7. Build the active package dependency graph and reject cycles.
8. Check supported editions and carry each package's effective verifier/build
   config into both the package plan and aggregate config records.
9. Produce `BuildPlan` with canonical ordering and deterministic diagnostics.

Phase 0 may stop after manifest/lockfile validation when errors are blocking.
It must still return diagnostics in canonical order so repeated runs report the
same error list.

## Determinism

All externally visible planner output is canonical:

- package ids use manifest spelling exactly, with no hyphen normalization;
- package plans are sorted dependencies-before-dependents, then by package id;
- dependencies are sorted by dependency kind, package id, resolved version, and
  feature list;
- duplicate features are manifest diagnostics, and accepted feature lists are
  sorted for the plan;
- diagnostics are sorted by normalized path, manifest key path, source range
  when available, diagnostic category, and rejected value;
- lockfile package entries are interpreted by package id, not by TOML order;
- path output uses normalized `/` separators and never host-specific absolute
  paths in published plan data.

No planner result may depend on hash-map iteration order, filesystem traversal
order, wall-clock time, session-local ids, task ids, or registry network
timing.

## Diagnostics

Planner diagnostics are build/manifest diagnostics. They include at least:

| Category | Examples |
|---|---|
| manifest syntax | invalid TOML, duplicate TOML keys |
| manifest schema | missing required tables or fields, unknown fields |
| package identity | invalid package id, duplicate package id |
| versioning | invalid SemVer, invalid constraint, lock version mismatch |
| workspace layout | missing member manifest, duplicate member, member outside root |
| lockfile | unsupported schema, missing package, unknown locked dependency |
| graph | dependency cycle, incompatible diamond dependency |
| config | unsupported edition, invalid solver, invalid build path |

Diagnostics should carry the manifest path, key path, offending value, and a
stable category. When TOML spans are available, they should be attached as
primary spans; when spans are unavailable, path and key path remain the stable
location.

The current external diagnostic-code specification does not reserve a dedicated
manifest range. Until that range is allocated, `mizar-build` keeps structured
planner diagnostics internally and maps them to user-facing build diagnostics
at the driver boundary.

## Tests

The planner test suite covers:

- valid single-package and multi-package workspace manifests;
- invalid package-id spellings, including hyphenated names;
- missing required package/workspace/lockfile fields;
- invalid TOML and deterministic multiple-error ordering;
- dependency entries in both string and inline-table forms;
- lockfile missing package and version mismatch diagnostics;
- dependency cycles and incompatible duplicate version assignments;
- deterministic `BuildPlan` equality for identical inputs and for inputs whose
  TOML table order differs.

## Constraints and Non-Goals

- The planner does not perform registry network resolution in this slice.
- The planner validates an existing lockfile but does not rewrite it.
- The planner does not discover `.miz` modules; module discovery is specified
  by `module_index.md`.
- The planner does not allocate source or snapshot identities; those remain
  owned by `mizar-session`.
- The planner does not depend on `mizar-driver`; the driver depends on
  `mizar-build`.
- Package dependency resolution is package-level only. Import graph resolution
  and symbol visibility are resolver responsibilities.
