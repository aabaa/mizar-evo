# mizar-build Task Graph

> Canonical language: English. Japanese companion:
> [../ja/task_graph.md](../ja/task_graph.md).

## Purpose

This document specifies the versioned verification task graph produced and
owned by `mizar-build`.

The task graph is the shared substrate for parallel scheduling and incremental
invalidation. It records correctness dependencies, canonical task identity, and
the units that later scheduler/resource/cancellation/failure modules consume.
It does not decide phase semantics, proof acceptance, cache-key construction,
or artifact publication trust.

## Context

- [architecture 14](../../architecture/en/14.parallel_verification_and_scheduling.md)
- [architecture 22](../../architecture/en/22.incremental_verification_contract.md)
- [internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md)
- [internal 07](../../internal/en/07.crate_module_layout.md)
- [planner.md](./planner.md)
- [module_index.md](./module_index.md)

## Scope

`task_graph` owns:

- deterministic expansion of a validated `BuildPlan` and `ModuleIndex` into
  build tasks;
- versioned `TaskId` construction for scheduling and result collation;
- task kinds and work units for package, module, VC, backend, kernel, artifact,
  and documentation work;
- correctness dependency edges and dependency coverage metadata;
- the initial task granularity decision for wave B.

`task_graph` does not own:

- build requests, sessions, watch/LSP entry points, phase registry, events, or
  `salsa` queries, which remain `mizar-driver` work;
- source loading, import parsing, name resolution, type checking, VC
  generation, ATP execution, kernel checking, proof policy, or artifact schema
  projection;
- `mizar-cache` `CacheKey`, dependency fingerprint, proof-reuse validation, or
  cache-store lookup;
- `mizar-ir` output storage, sealed handles, or snapshot rehydration;
- proof authority or promotion of cache/artifact records to trusted status.

## Gap Classification

| ID | Class | Evidence | Action |
|---|---|---|---|
| TG-G001 | `design_drift` | `todo.md` required `task_graph.md`, but no module spec existed before task 7. | Task 7 adds this spec and its Japanese companion. |
| TG-G002 | `source_drift` / `test_gap` | `src/task_graph.rs` and task-graph tests do not exist yet. | Task 8 implements source/tests against this spec. |
| TG-G003 | `external_dependency_gap` | `mizar-driver` request/session/registry/salsa work is open and the crate is absent. | Use caller-supplied snapshot tokens in the graph model; do not add a driver dependency, session model, or placeholder driver APIs. |
| TG-G004 | `external_dependency_gap` | `mizar-ir` output handles and storage adapters are absent. | Model output requirements as opaque phase-output requirements only; do not invent IR storage APIs. |
| TG-G005 | `external_dependency_gap` / `deferred` | Real VC descriptors are produced by later `mizar-vc` integration. | Support explicit/synthetic VC descriptors for tests; do not fabricate VCs from source text. |
| TG-G006 | `deferred` | Cache-aware scheduling is task 18. | Keep task graph identity separate from cache keys and do not add cache lookup here. |

## Initial Granularity Decision

The initial wave B graph uses **module-level phase tasks** through VC
generation, and **VC-level proof tasks** only after explicit VC descriptors are
available.

This means:

- package planning remains represented as one completed `PackageResolve` root
  task per build graph;
- workspace source files produce per-module `SourceLoad`, `Frontend`,
  `ModuleResolve`, `CheckAndElaborate`, and `VcGenerate` tasks;
- dependency-summary-backed registry modules are inputs, not source tasks;
- `VcDischarge`, `AtpSolve`, `BackendRun`, and `KernelCheck` are declared from
  explicit VC descriptor subgraph data, not guessed from source files or
  deterministic-discharge results;
- `ArtifactCommit` is per module, with canonical commit ordering separate from
  worker completion order;
- `DocumentationExtract` is disabled unless the build profile or later driver
  integration requests documentation/extraction work.

The graph may be coarser than the eventual ideal, but it must never omit a
correctness edge. False-positive waiting is acceptable. False-negative
parallelism is a soundness bug.

## Data Model

The following shapes define the contract, not necessarily the final Rust names:

```rust
struct TaskGraphInput {
    graph_version: TaskGraphVersion,
    snapshot: BuildSnapshotId,
    build_plan: BuildPlan,
    module_index: ModuleIndex,
    dependency_overlay: ModuleDependencyOverlay,
    vc_descriptors: Vec<VcTaskDescriptor>,
    profile: TaskGraphProfile,
}

struct TaskGraph {
    version: TaskGraphVersion,
    snapshot: BuildSnapshotId,
    tasks: Vec<BuildTask>,
    edges: Vec<TaskEdge>,
    diagnostics: Vec<TaskGraphDiagnostic>,
}

struct BuildTask {
    id: TaskId,
    kind: TaskKind,
    unit: WorkUnit,
    phases: Vec<PipelinePhase>,
    dependencies: Vec<TaskId>,
    dependency_coverage: DependencyCoverage,
    resource_class: ResourceClass,
    priority_class: PriorityClass,
}
```

`BuildSnapshotId` is owned by `mizar-session` and supplied by the caller. It is
a freshness/content-state token, not proof authority. The graph must not create
or reinterpret snapshot ids.

### TaskId

`TaskId` is a deterministic scheduling identity. It is built from:

- task-graph schema version;
- snapshot id;
- task kind;
- canonical work-unit identity;
- phase group;
- descriptor identity for VC/backend/kernel subgraph tasks when present.

It excludes:

- worker id;
- wall-clock time;
- queue priority;
- completion order;
- cache hit/miss timing;
- backend runtime duration;
- artifact write order;
- proof acceptance status.

`TaskId` is not a cache key, dependency fingerprint, proof witness hash,
`ObligationAnchor`, or kernel evidence identity. A task result may be reused in
another snapshot only through normal cache validation, never by matching
`TaskId` alone.

### TaskKind

```rust
enum TaskKind {
    PackageResolve,
    SourceLoad,
    Frontend,
    ModuleResolve,
    CheckAndElaborate,
    VcGenerate,
    VcDischarge,
    AtpSolve,
    BackendRun,
    KernelCheck,
    ArtifactCommit,
    DocumentationExtract,
}
```

`CheckAndElaborate` is the initial coarse semantic grouping for phases 5-10:
signature collection, type checking, registration/cluster resolution, overload
resolution, elaboration, and algorithm verification preparation. It must record
the phases it covers so a later split can preserve output identity and
diagnostics without changing the task graph contract.

### WorkUnit

```rust
enum WorkUnit {
    Workspace,
    Package { package_id: PackageId },
    Module { module: ModuleId },
    Vc { module: ModuleId, descriptor: VcTaskDescriptorId },
    BackendAttempt {
        module: ModuleId,
        descriptor: VcTaskDescriptorId,
        backend_profile: BackendProfileId,
    },
    EvidenceCandidate {
        module: ModuleId,
        descriptor: VcTaskDescriptorId,
        evidence_candidate: EvidenceCandidateId,
    },
}
```

`VcTaskDescriptorId`, `BackendProfileId`, and `EvidenceCandidateId` are
descriptor ids supplied by VC/ATP/kernel-facing services. `task_graph` preserves
and sorts them; it does not derive proof meaning from them.

### DependencyCoverage

```rust
enum DependencyCoverage {
    Complete,
    PackageConservative,
    MissingModuleDependencyOverlay,
    MissingVcDescriptors,
}
```

`Complete` means all dependencies required for the task kind are represented in
the graph. `PackageConservative` is allowed when package-level dependencies are
known but finer module import edges are unavailable; it may delay work but must
not permit an unsound run. `MissingModuleDependencyOverlay` blocks semantic
tasks that require module import edges. `MissingVcDescriptors` means no
fine-grained proof tasks can be constructed for that module yet.

## Dependency Inputs

### BuildPlan Edges

The package dependency graph from `BuildPlan` is a correctness input. For every
workspace package dependency, dependent package semantic tasks must wait for the
dependency package's published summaries or completed module artifacts.
Registry/dependency-artifact packages are ready inputs when their module
summaries are present in `ModuleIndex`.

The graph must preserve the package order from `BuildPlan`: dependencies before
dependents, ties by canonical package id.

### ModuleIndex Entries

Workspace-file-backed module entries create source and module tasks.
Dependency-summary-backed module entries are immutable inputs and must not be
scheduled as workspace source files.

The canonical module order is `(package_id, module_path, location key)`, using
the order already guaranteed by `ModuleIndex`.

### ModuleDependencyOverlay

`mizar-build` does not parse imports. Precise module import edges are supplied
by a later source/frontend/resolver-owned overlay:

```rust
struct ModuleDependencyOverlay {
    coverage: ModuleDependencyCoverage,
    edges: Vec<ModuleDependencyEdge>,
}

struct ModuleDependencyEdge {
    dependent: ModuleId,
    dependency: ModuleId,
    kind: ModuleDependencyKind,
}
```

`ModuleDependencyCoverage` defines whether edge coverage is precise enough to
run semantic tasks:

```rust
enum ModuleDependencyCoverage {
    Complete,
    CoveredModules(Vec<ModuleId>),
    PackageOnly,
    Unavailable,
}
```

- `Complete` means every workspace module in the `ModuleIndex` has explicit
  module-dependency coverage.
- `CoveredModules` means only the listed modules have precise edges; semantic
  tasks for other modules are marked `MissingModuleDependencyOverlay` or gated
  conservatively.
- `PackageOnly` means the graph may use package dependency edges but must mark
  semantic module tasks `PackageConservative` until precise module edges arrive.
- `Unavailable` means frontend/source-load tasks may be scheduled, but semantic
  tasks requiring import edges are marked `MissingModuleDependencyOverlay`.

When the overlay covers a module, semantic tasks for that module depend on the
required dependency module summaries. When the overlay is missing, source-load
and frontend tasks may still exist, but final semantic tasks must be marked
with missing coverage or conservatively gated. The graph must not invent import
edges from package names, aliases, source paths, or local heuristics.

### VC Descriptors

VC-level tasks are created only from explicit descriptors:

```rust
struct VcTaskDescriptor {
    id: VcTaskDescriptorId,
    module: ModuleId,
    vc_order_key: VcOrderKey,
    backend_profiles: Vec<BackendProfileId>,
    evidence_candidates: Vec<EvidenceCandidateId>,
}
```

`vc_order_key` follows `mizar-vc` canonical `VcId` ordering within one
snapshot. `ObligationAnchor` may be present in descriptor metadata for
diagnostics and later cache/reuse candidates, but it must never replace `VcId`
ordering and is not proof evidence.

Descriptors may declare backend profiles or evidence candidates that are
conditionally skipped after `VcDischarge` completes, but graph construction
does not inspect proof results. Conditional skip/block state belongs to the
scheduler and failure-state modules.

## Edge Rules

The graph is acyclic. Edge order is canonical: `(dependent task id,
dependency task id)`.

Base edges:

1. `PackageResolve` precedes all package, module, VC, and commit tasks.
2. For each workspace module:
   `SourceLoad -> Frontend -> ModuleResolve -> CheckAndElaborate -> VcGenerate`.
3. `ModuleResolve` waits for module dependency summaries when the dependency
   overlay covers the module. Missing coverage blocks or conservatively gates
   the semantic task.
4. `CheckAndElaborate` waits for completed `ModuleResolve` and all visible
   summary/registration inputs required by package/module dependency edges.
5. `VcGenerate` waits for `CheckAndElaborate`.
6. Each `VcDischarge` waits for its module's `VcGenerate` and explicit VC
   descriptor.
7. Each descriptor-declared `AtpSolve` waits for the corresponding
   `VcDischarge`. If deterministic discharge later closes the VC, the scheduler
   records the ATP subgraph as skipped or blocked without changing the static
   graph.
8. Each descriptor-declared `BackendRun` waits for its `AtpSolve` parent and
   backend profile availability.
9. Each `KernelCheck` waits for its candidate backend or deterministic evidence
   input, immutable kernel context, and dependency proof references.
10. `ArtifactCommit` waits for module diagnostics, proof statuses, and
    artifact inputs for that module. It records canonical commit order but does
    not perform manifest writes in `task_graph`.
11. `DocumentationExtract` waits for committed verified artifacts when enabled.

These edges are correctness edges. Removing one for performance is a soundness
bug. Adding a conservative edge is allowed when an upstream seam is missing or
the implementation starts with coarser granularity.

## Deterministic Ordering

The graph must sort:

- packages by `BuildPlan.packages` order;
- modules by `ModuleIndex.modules` canonical order;
- task kinds by the pipeline phase order above;
- VC descriptors by `vc_order_key`;
- backend profiles by configured backend priority, then profile id;
- evidence candidates by deterministic candidate id;
- edges by `(dependent, dependency)`;
- diagnostics by package id, module path, task kind, and stable diagnostic code.

Worker completion order, queue priority, cache lookup timing, and backend
runtime duration never participate in graph ordering.

## Construction Algorithm

1. Validate the graph schema version.
2. Read `BuildPlan.packages` and `ModuleIndex` in canonical order.
3. Create one completed-root `PackageResolve` task for the workspace graph.
4. Create workspace-module tasks for every workspace-file-backed module.
5. Treat dependency-summary-backed modules as immutable inputs.
6. Add package-level dependency edges from `BuildPlan.dependency_graph`.
7. Add module dependency edges from `ModuleDependencyOverlay` where coverage is
   available; mark missing coverage explicitly where it is not.
8. Add explicit VC/backend/kernel subgraph tasks from `vc_descriptors` without
   consulting discharge or proof results.
9. Add module `ArtifactCommit` tasks for modules with schedulable outputs.
10. Add `DocumentationExtract` tasks only when requested by the graph profile.
11. Sort tasks and edges canonically.
12. Reject duplicate task ids, unknown module/package references, self edges,
    and dependency cycles.

The constructor must return either the complete graph or deterministic
diagnostics. It must not silently drop unknown dependencies or unsupported
descriptor families.

## Diagnostics

Task graph diagnostics are structural build diagnostics, not phase semantic
diagnostics.

Expected diagnostic kinds include:

- duplicate task identity;
- unknown package or module reference;
- dependency cycle;
- missing module dependency overlay;
- missing VC descriptors;
- unsupported task graph schema version;
- boundary violation, such as a cache key or proof-acceptance field appearing
  in graph identity.

Diagnostic wording may improve, but the kind and stable ordering must remain
deterministic.

## Tests

Task 7 is documentation-only. Task 8 must add focused Rust tests for:

- deterministic task id construction;
- package and module ordering from `BuildPlan`/`ModuleIndex`;
- workspace modules creating source/frontend/semantic tasks;
- dependency-summary-backed modules staying inputs, not source tasks;
- package dependency edges gating downstream semantic tasks;
- module dependency overlay edges and missing-coverage diagnostics;
- explicit VC descriptors creating VC-level tasks in `VcId` order;
- duplicate task id and cycle rejection;
- absence of cache-key, dependency-fingerprint, proof-reuse, driver, and IR
  placeholder behavior;
- absence of proof-authority, proof-acceptance, proof-trust, or trusted-status
  placeholder fields/APIs in task graph identity, descriptors, or diagnostics.

## Non-Authority Rules

- A task being ready, completed, skipped, or cached is not semantic acceptance.
- `TaskId` is not `CacheKey`, dependency fingerprint, proof witness hash, or
  proof reuse validation.
- `CacheHit` is task scheduling state after validation by `mizar-cache`, not
  proof evidence.
- Artifact records and commit ordering do not promote proof trust.
- Backend completion order never selects the accepted proof candidate.
