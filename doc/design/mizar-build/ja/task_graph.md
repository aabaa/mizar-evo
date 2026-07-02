# mizar-build Task Graph

> 正本は英語です。英語版:
> [../en/task_graph.md](../en/task_graph.md)。

## 目的

この文書は、`mizar-build` が生成し所有する versioned verification task graph
を仕様化する。

task graph は parallel scheduling と incremental invalidation の共有基盤である。
correctness dependencies、canonical task identity、後続の
scheduler/resource/cancellation/failure modules が消費する unit を記録する。
phase semantics、proof acceptance、cache-key construction、artifact
publication trust は決定しない。

## 文脈

- [architecture 14](../../architecture/ja/14.parallel_verification_and_scheduling.md)
- [architecture 22](../../architecture/ja/22.incremental_verification_contract.md)
- [internal 01](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)
- [internal 07](../../internal/ja/07.crate_module_layout.md)
- [planner.md](./planner.md)
- [module_index.md](./module_index.md)

## 範囲

`task_graph` が所有する:

- 検証済み `BuildPlan` と `ModuleIndex` から build task への決定的展開。
- scheduling と result collation のための versioned `TaskId` 構築。
- package、module、VC、backend、kernel、artifact、documentation work の
  task kind と work unit。
- correctness dependency edges と dependency coverage metadata。
- wave B の初期 task granularity decision。

`task_graph` が所有しない:

- build request、session、watch/LSP entry point、phase registry、event、
  `salsa` query。これらは `mizar-driver` の仕事である。
- source loading、import parsing、name resolution、type checking、VC
  generation、ATP execution、kernel checking、proof policy、artifact schema
  projection。
- `mizar-cache` の `CacheKey`、dependency fingerprint、proof-reuse validation、
  cache-store lookup。
- `mizar-ir` output storage、sealed handle、snapshot rehydration。
- proof authority や cache/artifact record の trusted status への昇格。

## Gap classification

| ID | Class | Evidence | Action |
|---|---|---|---|
| TG-G001 | `design_drift` | `todo.md` は `task_graph.md` を要求していたが、task 7 の前には module spec が存在しなかった。 | task 7 でこの spec と Japanese companion を追加する。 |
| TG-G002 | resolved `source_drift` / `test_gap` | `src/task_graph.rs` と focused task-graph tests は task 8 の前には存在しなかった。 | task 8 でこの spec に沿って source/tests を実装する。 |
| TG-G003 | `external_dependency_gap` | `mizar-driver` は存在するが、driver request/session/registry/salsa authority は `mizar-build` の外側にある。 | graph model では caller-supplied snapshot token を使い、driver dependency、session model、placeholder driver API を追加しない。 |
| TG-G004 | `external_dependency_gap` | real IR output handles と storage adapters は build-owned seam 経由では利用できない。 | output requirements は opaque phase-output requirements としてのみ model し、IR storage API を創作しない。 |
| TG-G005 | `external_dependency_gap` / `deferred` | real VC descriptor は後続の `mizar-vc` integration が生成する。 | tests では explicit/synthetic VC descriptors を支援し、source text から VC を捏造しない。 |
| TG-G006 | `deferred` | cache-aware scheduling は task 18 である。 | task graph identity を cache key から分離し、この module で cache lookup を追加しない。 |

## 初期 granularity decision

初期 wave B graph は、VC generation までは **module-level phase tasks** を使い、
explicit VC descriptors が利用可能になった後だけ **VC-level proof tasks** を
作成する。

つまり:

- package planning は build graph ごとに 1 つの completed `PackageResolve`
  root task として表現する。
- workspace source files は module ごとの `SourceLoad`、`Frontend`、
  `ModuleResolve`、`CheckAndElaborate`、`VcGenerate` tasks を生成する。
- dependency-summary-backed registry modules は inputs であり、source tasks
  ではない。
- `VcDischarge`、`AtpSolve`、`BackendRun`、`KernelCheck` は explicit VC
  descriptor subgraph data から宣言し、source files や deterministic-discharge
  results から推測しない。
- `ArtifactCommit` は module ごとであり、canonical commit ordering は worker
  completion order から分離される。
- `DocumentationExtract` は build profile または後続 driver integration が
  documentation/extraction work を要求しない限り無効である。

graph は最終的な理想より粗くてよいが、correctness edge を省略してはならない。
false-positive waiting は許容される。false-negative parallelism は soundness bug
である。

## Data model

以下の shape は契約を定義するものであり、最終 Rust 名とは限らない:

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

`BuildSnapshotId` は `mizar-session` が所有し、caller から供給される。
これは freshness/content-state token であり、proof authority ではない。
graph は snapshot id を生成したり再解釈したりしてはならない。

### TaskId

`TaskId` は決定的な scheduling identity である。以下から構築される:

- task-graph schema version。
- snapshot id。
- task kind。
- canonical work-unit identity。
- phase group。
- VC/backend/kernel subgraph tasks に descriptor がある場合は descriptor
  identity。

以下は除外する:

- worker id。
- wall-clock time。
- queue priority。
- completion order。
- cache hit/miss timing。
- backend runtime duration。
- artifact write order。
- proof acceptance status。

`TaskId` は cache key、dependency fingerprint、proof witness hash、
`ObligationAnchor`、kernel evidence identity ではない。task result が別 snapshot
で再利用される場合は normal cache validation を通す必要があり、`TaskId` の一致
だけでは再利用してはならない。

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

`CheckAndElaborate` は phases 5-10 の初期 coarse semantic grouping である:
signature collection、type checking、registration/cluster resolution、overload
resolution、elaboration、algorithm verification preparation を含む。後で split
しても task graph contract を変えずに output identity と diagnostics を保てる
よう、対象 phases を記録しなければならない。

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

`VcTaskDescriptorId`、`BackendProfileId`、`EvidenceCandidateId` は
VC/ATP/kernel-facing services から供給される descriptor ids である。
`task_graph` はそれらを保存し sort するが、proof meaning は導出しない。

### DependencyCoverage

```rust
enum DependencyCoverage {
    Complete,
    PackageConservative,
    MissingModuleDependencyOverlay,
    MissingVcDescriptors,
}
```

`Complete` は task kind に必要なすべての dependencies が graph に表現されている
ことを意味する。`PackageConservative` は、package-level dependencies は既知だが
より細かい module import edges が利用できない場合に許される。これは作業を遅らせ
てよいが unsound run を許してはならない。`MissingModuleDependencyOverlay` は
module import edges を必要とする semantic tasks を block する。
`MissingVcDescriptors` は、その module について細粒度 proof tasks をまだ構築
できないことを意味する。

## Dependency inputs

### BuildPlan edges

`BuildPlan` の package dependency graph は correctness input である。
workspace package dependency ごとに、dependent package の semantic tasks は
dependency package の published summaries または completed module artifacts を
待たなければならない。Registry/dependency-artifact packages は、その module
summaries が `ModuleIndex` に存在するとき ready inputs である。

graph は `BuildPlan` の package order を保つ: dependencies before dependents、
ties by canonical package id。

### ModuleIndex entries

Workspace-file-backed module entries は source tasks と module tasks を作る。
Dependency-summary-backed module entries は immutable inputs であり、workspace
source files として schedule してはならない。

canonical module order は `(package_id, module_path, location key)` であり、
`ModuleIndex` がすでに保証する順序を使う。

### ModuleDependencyOverlay

`mizar-build` は imports を parse しない。精密な module import edges は、後続の
source/frontend/resolver-owned overlay から供給される:

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

`ModuleDependencyCoverage` は edge coverage が semantic tasks を実行できる
精度かどうかを定義する:

```rust
enum ModuleDependencyCoverage {
    Complete,
    CoveredModules(Vec<ModuleId>),
    PackageOnly,
    Unavailable,
}
```

- `Complete` は `ModuleIndex` 内のすべての workspace module が明示的な
  module-dependency coverage を持つことを意味する。
- `CoveredModules` は listed modules だけが precise edges を持つことを意味する。
  それ以外の modules の semantic tasks は `MissingModuleDependencyOverlay` として
  mark するか conservatively gate する。
- `PackageOnly` は graph が package dependency edges を使ってよいが、precise
  module edges が届くまで semantic module tasks を `PackageConservative` として
  mark しなければならないことを意味する。
- `Unavailable` は frontend/source-load tasks は schedule してよいが、import
  edges を必要とする semantic tasks は `MissingModuleDependencyOverlay` として
  mark することを意味する。

overlay が module を cover している場合、その module の semantic tasks は必要な
dependency module summaries に依存する。overlay が欠けている場合でも
source-load と frontend tasks は存在してよいが、final semantic tasks は missing
coverage として mark するか conservatively gate しなければならない。graph は
package names、aliases、source paths、local heuristics から import edges を
創作してはならない。

### VC descriptors

VC-level tasks は explicit descriptors からだけ作成する:

```rust
struct VcTaskDescriptor {
    id: VcTaskDescriptorId,
    module: ModuleId,
    vc_order_key: VcOrderKey,
    backend_profiles: Vec<BackendProfileId>,
    evidence_candidates: Vec<EvidenceCandidateId>,
}
```

`vc_order_key` は 1 snapshot 内の `mizar-vc` canonical `VcId` ordering に従う。
`ObligationAnchor` は diagnostics と後続 cache/reuse candidates のために
descriptor metadata に存在してよいが、`VcId` ordering を置き換えてはならず、
proof evidence でもない。

descriptor は、`VcDischarge` 完了後に conditionally skipped される backend
profiles や evidence candidates を宣言してよいが、graph construction は proof
results を検査しない。conditional skip/block state は scheduler と
failure-state modules の責務である。

## Edge rules

graph は acyclic である。edge order は canonical: `(dependent task id,
dependency task id)`。

Base edges:

1. `PackageResolve` はすべての package、module、VC、commit tasks に先行する。
2. 各 workspace module について:
   `SourceLoad -> Frontend -> ModuleResolve -> CheckAndElaborate -> VcGenerate`。
3. `ModuleResolve` は、dependency overlay が module を cover する場合に module
   dependency summaries を待つ。coverage が欠けている場合、semantic task は
   block するか conservatively gate する。
4. `CheckAndElaborate` は completed `ModuleResolve` と、package/module
   dependency edges が要求するすべての visible summary/registration inputs を
   待つ。
5. `VcGenerate` は `CheckAndElaborate` を待つ。
6. 各 `VcDischarge` は対象 module の `VcGenerate` と explicit VC descriptor を
   待つ。
7. descriptor-declared `AtpSolve` は対応する `VcDischarge` を待つ。後で
   deterministic discharge が VC を閉じた場合、scheduler は静的 graph を変更せず、
   ATP subgraph を skipped または blocked として記録する。
8. descriptor-declared `BackendRun` は親 `AtpSolve` と backend profile
   availability を待つ。
9. 各 `KernelCheck` は candidate backend または deterministic evidence input、
   immutable kernel context、dependency proof references を待つ。
10. `ArtifactCommit` はその module の diagnostics、proof statuses、artifact
    inputs を待つ。canonical commit order を記録するが、manifest writes は
    `task_graph` では実行しない。
11. `DocumentationExtract` は有効な場合、committed verified artifacts を待つ。

これらの edges は correctness edges である。performance のために削除することは
soundness bug である。upstream seam が欠けている、または実装が粗い granularity
から始まる場合に conservative edge を追加することは許される。

## Deterministic ordering

graph は以下を sort しなければならない:

- packages は `BuildPlan.packages` order。
- modules は `ModuleIndex.modules` canonical order。
- task kinds は上記 pipeline phase order。
- VC descriptors は `vc_order_key`。
- backend profiles は configured backend priority。
- evidence candidates は deterministic candidate id。
- edges は `(dependent, dependency)`。
- diagnostics は package id、module path、task kind、stable diagnostic code。

Worker completion order、queue priority、cache lookup timing、backend runtime
duration は graph ordering に参加しない。

task 8 の model では、各 `VcTaskDescriptor` の caller-supplied
`backend_profiles` vector が configured backend-priority list である。
`task_graph` はその priority order を保持する。重複する generated task identity
は通常の graph validation で拒否される。profile id に solver semantics を
付与しない。

## Construction algorithm

1. graph schema version を検証する。
2. `BuildPlan.packages` と `ModuleIndex` を canonical order で読む。
3. workspace graph に 1 つの completed-root `PackageResolve` task を作成する。
4. workspace-file-backed module ごとに workspace-module tasks を作成する。
5. dependency-summary-backed modules を immutable inputs として扱う。
6. `BuildPlan.dependency_graph` から package-level dependency edges を追加する。
7. coverage がある場合は `ModuleDependencyOverlay` から module dependency edges
   を追加し、ない場合は missing coverage を明示的に mark する。
8. discharge や proof results を参照せず、`vc_descriptors` から explicit
   VC/backend/kernel subgraph tasks を追加する。
9. schedulable outputs を持つ modules に `ArtifactCommit` tasks を追加する。
10. graph profile が要求する場合だけ `DocumentationExtract` tasks を追加する。
11. tasks と edges を canonically sort する。
12. duplicate task ids、unknown module/package references、self edges、
    dependency cycles を拒否する。

constructor は complete graph または deterministic diagnostics を返さなければ
ならない。unknown dependencies や unsupported descriptor families を silently drop
してはならない。

## Diagnostics

Task graph diagnostics は structural build diagnostics であり、phase semantic
diagnostics ではない。

想定する diagnostic kinds:

- duplicate task identity。
- unknown package または module reference。
- dependency cycle。
- missing module dependency overlay。
- missing VC descriptors。
- unsupported task graph schema version。
- graph identity に cache key または proof-acceptance field が現れるなどの
  boundary violation。

diagnostic wording は改善してよいが、kind と stable ordering は決定的なままに
する。

## Tests

Task 7 は documentation-only である。Task 8 は focused Rust tests を追加し、
以下を検証する:

- deterministic task id construction。
- `BuildPlan` / `ModuleIndex` からの package と module ordering。
- workspace modules が source/frontend/semantic tasks を作ること。
- dependency-summary-backed modules が inputs のままで source tasks にならない
  こと。
- package dependency edges が downstream semantic tasks を gate すること。
- module dependency overlay edges と missing-coverage diagnostics。
- explicit VC descriptors が `VcId` order で VC-level tasks を作ること。
- duplicate task id と cycle rejection。
- cache-key、dependency-fingerprint、proof-reuse、driver、IR placeholder
  behavior が存在しないこと。
- task graph identity、descriptors、diagnostics に proof-authority、
  proof-acceptance、proof-trust、trusted-status placeholder fields/APIs が
  存在しないこと。

## Non-authority rules

- task が ready、completed、skipped、cached であることは semantic acceptance
  ではない。
- `TaskId` は `CacheKey`、dependency fingerprint、proof witness hash、proof
  reuse validation ではない。
- `CacheHit` は `mizar-cache` による validation 後の task scheduling state であり、
  proof evidence ではない。
- artifact records と commit ordering は proof trust を昇格しない。
- backend completion order は accepted proof candidate を選ばない。

## 公開 enum policy

この module が所有する exhaustive public enum exception はない。

| Enum | Policy |
|---|---|
| `TaskKind` | `#[non_exhaustive]`; downstream callers は wildcard match arms を含めなければならない。 |
| `PipelinePhase` | `#[non_exhaustive]`; downstream callers は wildcard match arms を含めなければならない。 |
| `WorkUnit` | `#[non_exhaustive]`; downstream callers は wildcard match arms を含めなければならない。 |
| `DependencyCoverage` | `#[non_exhaustive]`; downstream callers は wildcard match arms を含めなければならない。 |
| `ResourceClass` | `#[non_exhaustive]`; downstream callers は wildcard match arms を含めなければならない。 |
| `PriorityClass` | `#[non_exhaustive]`; downstream callers は wildcard match arms を含めなければならない。 |
| `ModuleDependencyCoverage` | `#[non_exhaustive]`; downstream callers は wildcard match arms を含めなければならない。 |
| `ModuleDependencyKind` | `#[non_exhaustive]`; downstream callers は wildcard match arms を含めなければならない。 |
| `DocumentationProfile` | `#[non_exhaustive]`; downstream callers は wildcard match arms を含めなければならない。 |
| `VcDescriptorPolicy` | `#[non_exhaustive]`; downstream callers は wildcard match arms を含めなければならない。 |
| `TaskGraphDiagnosticKind` | `#[non_exhaustive]`; downstream callers は wildcard match arms を含めなければならない。 |
