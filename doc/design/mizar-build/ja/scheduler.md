# mizar-build Scheduler

> Canonical language: English. Canonical document:
> [../en/scheduler.md](../en/scheduler.md).

## Purpose

この文書は `mizar-build` が所有する scheduler contract を定義する。

scheduler は `task_graph` が生成した決定的な `TaskGraph` を消費し、dependency
constraints の下で tasks を実行または skip し、task states を記録し、task
results と scheduler events を canonical order で collate する。これは execution
order component であり、semantic authority ではない。

## Context

- [architecture 14](../../architecture/ja/14.parallel_verification_and_scheduling.md)
- [architecture 22](../../architecture/ja/22.incremental_verification_contract.md)
- [internal 01](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)
- [cache_seam.md](./cache_seam.md)
- [task_graph.md](./task_graph.md)

## Scope

`scheduler` が所有するもの:

- validated `TaskGraph` 上の ready-state computation。
- pending、ready、running、terminal、skipped、blocked work の task state
  transitions。
- batch、watch、LSP-oriented runs の work-queue selection と priority keys。
- 任意の worker completion order の下での deterministic result/event collation。
- 後続の resource、cancellation、failure-state、cache、commit modules が消費する
  build-side seam。

`scheduler` が所有しないもの:

- `mizar-driver` build requests、sessions、phase registries、live progress
  transports、LSP bridges。
- phase semantics、proof acceptance、ATP policy、kernel trust、artifact schema
  projection、diagnostic rendering。
- `mizar-cache` `CacheKey`、dependency fingerprint、proof-reuse validation、
  cache-store lookup。
- 専用 task の前の resource accounting internals、cancellation-token storage、
  failure taxonomy、artifact publication tokens、manifest writes。
- cache hits、artifact records、skipped tasks を trusted proof status へ昇格すること。

## Gap Classification

| ID | Class | Evidence | Action |
|---|---|---|---|
| SCH-G001 | `design_drift` | `todo.md` は `scheduler.md` を要求していたが、task 9 の前には module spec が存在しなかった。 | task 9 でこの spec と Japanese companion を追加する。 |
| SCH-G002 | `source_drift` / `test_gap` | task 10 の前には `src/scheduler.rs` と scheduler tests が存在しなかった。task 10 は synthetic scheduler core と focused unit tests を提供している。 | 後続の resource/cancel/failure/cache tasks が module を拡張するとき、この spec、Rust surface、tests を同期し続ける。 |
| SCH-G003 | `external_dependency_gap` | この checkout には `mizar-driver` request/session/registry/event integration が存在しない。 | 後続 source では caller-supplied graph/session-like inputs を受け取り、driver dependency や placeholder driver API を追加しない。 |
| SCH-G004 | `external_dependency_gap` | `mizar-ir` sealed output handles と storage adapters が存在しない。 | scheduler tests では synthetic in-memory task outputs を使い、IR storage API を創作しない。 |
| SCH-G005 | `source_drift` / `test_gap` | tasks 11-12 の前は resource-budget accounting が architecture notes にしか存在しなかった。task 12 が `src/resource.rs`、scheduler admission integration、focused tests を追加する。 | budget scopes が発展するとき、この spec、`resource.md`、scheduler/resource Rust surface を同期し続ける。 |
| SCH-G006 | `source_drift` / `test_gap` | cancellation state は tasks 13-14 以前には存在しなかった。task 14 が `src/cancel.rs`、scheduler checkpoint integration、focused tests を追加する。 | scheduler cancellation checkpoint を `cancel.md` と同期し続ける。snapshot-token ownership はそこに属する。 |
| SCH-G007 | `source_drift` / `test_gap` | failure-state propagation は task 16 以前には存在しなかった。task 16 が deterministic `failure_records` / `blocked_records`、bounded propagation、scheduler integration、focused tests を追加する。 | scheduler record surface を `failure_state.md` と同期し続ける。詳細な taxonomy はそこに属する。 |
| SCH-G008 | `source_drift` / `test_gap` | task 18 以前の cache-aware scheduling には disabled/miss placeholder しかなく、`mizar-cache` はすでに validation を所有していた。 | task 18 は cache hit を validated execution-skip outcome としてのみ model し、cache key や proof-reuse decision をここで構築しない。 |
| SCH-G009 | `external_dependency_gap` | real producer artifact publication token は `mizar-build` が利用できない。 | commit tasks を決定的に order し、caller-supplied manifest entries を `mizar-artifact` へ渡すが、publication authority を作らない。 |

## Data Model

以下の形は scheduler contract を定義するもので、最終的な Rust 名を固定しない:

```rust
struct SchedulerInput {
    graph: TaskGraph,
    mode: SchedulerMode,
    priority_hints: PriorityHints,
    cache: CacheSchedulingPolicy,
    cache_decisions: CacheSchedulingPlan,
    resource_budget: ResourceBudget,
    cancellation: CancellationPolicy,
    task_outcomes: Vec<SyntheticTaskOutcome>,
    worker_count: usize,
    completion_order: CompletionOrder,
}

struct SchedulerRun {
    graph_version: TaskGraphVersion,
    snapshot: BuildSnapshotId,
    task_states: Vec<TaskStateRecord>,
    results: Vec<SchedulerResult>,
    failure_records: Vec<BuildFailureRecord>,
    blocked_records: Vec<BlockedTaskRecord>,
    events: Vec<SchedulerEvent>,
    resource_telemetry: Vec<ResourceTelemetry>,
    diagnostics: Vec<SchedulerDiagnostic>,
}

struct TaskStateRecord {
    task_id: TaskId,
    state: TaskState,
    dependencies: Vec<TaskId>,
    blocked_by: Vec<TaskId>,
    queue: SchedulerQueue,
    dependency_coverage: DependencyCoverage,
}
```

`SchedulerInput` は build-side data である。将来の driver はこれを sessions と
live event streams で包んでよいが、`mizar-build` は `mizar-driver` に依存せず
利用可能でなければならない。`worker_count` は scheduler tests が使う synthetic
executor dispatch cap のままであり、`resource_budget` はその下にある modeled
queue-admission layer である。ready task が running になるには、両方の limit を
満たさなければならない。

### TaskState

```rust
enum TaskState {
    Pending,
    Ready,
    Running,
    Completed,
    CacheHit,
    Skipped,
    Failed,
    Blocked,
    Cancelled,
}
```

状態の意味:

- `Pending`: dependencies がまだ terminal ではない。
- `Ready`: すべての correctness dependencies が terminal で、task は execution または
  cache probing の前に deterministic dispatch batch で待ってよい。
- `Running`: worker が task execution attempt を所有している。
- `Completed`: execution が終了し、scheduler-visible outputs を生成した。
- `CacheHit`: external cache validation が成功した後に execution を skip した。
- `Skipped`: 静的に宣言された conditional subgraph が不要になった。例:
  deterministic discharge が VC を閉じた後の ATP/backend work。
- `Failed`: task が実行され、failing task outcome を生成した。
- `Blocked`: required dependency が failed/cancelled、または required coverage が
  欠けており、この run で valid result を生成できない。
- `Cancelled`: task は superseded または明示的に cancelled された snapshot に属する。

`Completed`、`CacheHit`、`Skipped`、`Failed`、`Blocked`、`Cancelled` は
scheduling terminal states である。`Completed` と validated `CacheHit` は
successful/unblocking terminal states である。`Skipped` は、parent task が child を
不要にした conditional subgraph に対してのみ unblock する。`Failed`、`Blocked`、
`Cancelled` は、後続 failure-state rule が degraded non-semantic work を明示的に
許可しない限り、correctness dependents にとって blocking terminal states である。
これらの state は、それ自体では semantic acceptance、trusted proof status、artifact
publication を意味しない。

### SchedulerResult

```rust
struct SchedulerResult {
    task_id: TaskId,
    state: TaskState,
    canonical_order: SchedulerOrderKey,
    output_refs: Vec<SyntheticOutputRef>,
    diagnostics: Vec<SchedulerDiagnosticRef>,
}
```

task 10 は synthetic output references を使ってよい。real phase output handles は
`mizar-ir` または driver phase registry が提供するまで external dependency gap の
ままである。

### SchedulerEvent

```rust
enum SchedulerEventKind {
    TaskBecameReady,
    TaskStarted,
    TaskFinished,
    TaskSkipped,
    TaskBlocked,
    RunFinished,
}

struct SchedulerEvent {
    kind: SchedulerEventKind,
    task_id: Option<TaskId>,
    order: SchedulerOrderKey,
}
```

Events は後続 driver integration のための progress records である。`mizar-build`
が生成する canonical event stream は `SchedulerOrderKey` で sort される。live worker
telemetry が `mizar-driver` で追加されても、それは artifact、diagnostic、proof
ordering rule ではない。

events が同じ scheduler order と task を共有する場合、lifecycle rank は
`TaskBecameReady`、`TaskStarted`、`TaskSkipped`、`TaskBlocked`、`TaskFinished`、
`RunFinished` の順である。run-finished event は同じ run のすべての task events の後に
sort される。

### ResourceTelemetry

```rust
struct ResourceTelemetry {
    task_id: TaskId,
    queue: SchedulerQueue,
    status: ResourceAdmissionStatus,
    requested_units: ResourceRequestUnits,
    blocking_scope: Option<ResourceScope>,
    admission_order: usize,
}
```

Resource telemetry は admission、delay、impossible requests、release を説明する。
これは debugging と後続 driver events のため deterministic に sort されるが、
diagnostic ordering key、artifact manifest input、cache validation input、proof
evidence、trusted-status authority ではない。

## Readiness and State Transitions

初期状態:

1. root 以外のすべての task は `Pending` から始まる。
2. 現在の task-graph contract では、single `PackageResolve` root は `Completed` から
   始まる。phase 0 が scheduling の前にすでに `BuildPlan` を生成しているためである。
   将来の scheduler-owned phase-0 mode は別の状態から始めてよいが、それは task 10 の
   範囲外である。
3. すべての graph dependencies が、その task の dependency coverage を満たす
   terminal states になったとき、task は `Ready` になる。
4. cache scheduling が enabled の場合、scheduler は ready task について
   caller-supplied validated cache decision を worker/resource admission の前に消費してよい。

Terminal dependency handling:

- `Completed` と validated `CacheHit` は dependents を unblock する。
- `Skipped` は、conditional proof subgraph の skip を許容する task kind の
  dependents だけを unblock する。
- `Failed`、`Blocked`、`Cancelled` は、後続 failure-state rule が degraded
  non-semantic work を明示的に許可しない限り、correctness dependents を block する。

Execution transition:

```text
Pending -> Ready -> Running -> Completed
Pending -> Ready -> CacheHit
Pending -> Blocked
Ready   -> Blocked
Running -> Failed
Pending/Ready/Running -> Cancelled
Ready/Running -> Skipped
```

task 12 は core deterministic scheduler の下に modeled resource admission を重ねる。
`worker_count` は synthetic dispatch batches を引き続き制限し、resource budgets は
ready task が該当する workspace、package、module、obligation、backend、commit units を
すべて reserve できるかを決める。temporary resource denial は task を ready/queued に
残し、impossible request は stable resource diagnostic で block する。task 14 は
cancellation checkpoint を追加し、task 16 は deterministic failure/block records を
追加する。task 18 は build-side cache decision seam を追加する。cache validation は
`mizar-cache` が所有し続ける。scheduler record が cache hit、skipped work、
artifact record を proof authority へ昇格することはない。

## Work Queues

scheduler は ready tasks を scheduler queue ごとに分ける。Queue selection は
task graph resource class から導出され、1 つの resource class が coordination tasks と
worker-process tasks の両方を含む場合は `TaskKind` でさらに精緻化してよい:

| Queue | Task classes |
|---|---|
| coordinator | root と graph bookkeeping |
| source/local CPU | `SourceLoad`, `Frontend`, `ModuleResolve`, `CheckAndElaborate`, `VcGenerate` |
| deterministic proof | `VcDischarge` |
| ATP portfolio | `AtpSolve` coordination と child-subgraph collation |
| ATP process | `BackendRun` child-process work |
| kernel | `KernelCheck` |
| I/O commit | `ArtifactCommit` |
| documentation | `DocumentationExtract` |

Queue choice は correctness を変えない。resource module は queue admission を遅延して
よいが、graph dependencies を削除したり completion order を semantic にしたりしては
ならない。

task 8 graph では、`AtpSolve` と `BackendRun` はどちらも `AtpProcess` resource class
を使う。scheduler は task kind により `AtpSolve` を portfolio coordination へ、
`BackendRun` を external backend process slots へ route する。これにより、後続の
resource-budget tasks が distinct portfolio resource class の要否を決めるまで、
Task 8 の resource-class API を安定させる。

## Priority Policy

default priority key は stable tuple である:

1. build mode priority class。
2. open/requested-file hint rank。
3. downstream fanout。downstream を unblock する task を leaf work より先にする。
4. `task_graph.md` の task kind rank。
5. package/module/VC/backend/evidence canonical work-unit order。
6. `TaskId`。

Batch mode は dependency summaries と downstream tasks を解放する lower phase work を
優先する。Watch と LSP mode は open/requested files の source/frontend work を引き上げて
よいが、それは priority rank に限られる。Priority は latency だけに影響し、diagnostics
order、artifact order、proof selection、cache publication には影響しない。

Backend profile priority は `TaskGraph` descriptor ordering から来る。後続の ATP portfolio
service は宣言する backend work を選んでよいが、scheduler は worker completion time で
accepted evidence を選んではならない。

## Deterministic Collation

scheduler は ready tasks を任意の worker order で実行してよいが、externally visible
records は canonical keys で collate する:

- task states は task graph order と `TaskId`。
- scheduler results は package、module、task kind、descriptor order、`TaskId`。
- diagnostics は architecture diagnostic ordering。worker finish time ではない。
- artifact commit attempts は canonical module と manifest order。
- canonical scheduler events は `SchedulerOrderKey`。

同じ `TaskGraph`、resource configuration、synthetic task behavior、cache seam responses を
使う 2 つの実行は、canonical serialization 後の `SchedulerRun` records が byte-for-byte
equivalent でなければならない。Resource admission と release telemetry も同じ canonical
collation rule を保ち、scheduler result、diagnostic、artifact、cache、proof authority を
変えてはならない。

## Batch, Watch, and LSP Modes

### Batch

Batch mode は obsolete でないすべての task を terminal state へ進めようとする。
independent work が useful diagnostics を生成できる場合、recoverable task failures の後も
続行する。final build status は、すべての ready work が completed、skipped、failed、
blocked、または cancelled になった後に計算する。

### Watch

Watch mode は同じ graph と state transitions を使うが、新しい snapshot が pending または
running work を supersede してよい。scheduler tests は driver watch APIs ではなく
explicit synthetic cancellation seam responses を使う。この checkout では driver watch
integration が利用できないためである。

### LSP

LSP mode は open files と syntax/local feedback を優先してよい。incompatible snapshots の
diagnostics を混ぜたり、stale results を current として publish したりしてはならない。
Snapshot publication と editor protocol conversion は `mizar-driver`/LSP bridge の責務で
ある。

## Cache-Aware Scheduling Seam

Cache-aware scheduling は optimization seam である。詳細な seam contract は
[cache_seam.md](./cache_seam.md) が定義する。

ready task を実行する前に、scheduler は caller-supplied cache seam outcome を消費してよい:

```rust
enum CacheSchedulingOutcome {
    ValidatedHit(ValidatedCacheHit),
    Miss(CacheFallbackReason),
    NoKey(CacheFallbackReason),
    Unavailable(CacheFallbackReason),
    ErrorAsMiss(CacheFallbackReason),
}
```

Rules:

- `scheduler` は `CacheKey` を構築しない。
- `scheduler` は dependency fingerprint を計算しない。
- `scheduler` は proof reuse を検証しない。
- `ValidatedHit` は、caller がすでに `mizar-cache` public contract で record と
  output handles を検証したという caller assertion である。task を `CacheHit` に遷移させ、
  execution を skip してよい。
- `Miss`、`NoKey`、`Unavailable` は task を通常どおり実行する。
- `ErrorAsMiss` は、後続の explicit cache-required mode が cache diagnostic を出すと
  定めない限り、`Miss` として扱う。
- `CacheHit` は completed scheduling dependency であり、proof evidence ではない。
- cache miss timing と lookup order は、proof selection、diagnostics、artifact order、
  trusted status に影響しない。

task 18 は validated-hit input surface を公開しつつ、default は uncached execution の
まま保つ。scheduler は cache decision shape を input boundary で検証するが、disabled
cache scheduling は valid caller-supplied hit decisions を無視する。validated hits は
modeled worker/resource admission の前に考慮されるため、cache-hit task は `TaskStarted`
event も execution resource telemetry も持たない。

Rust surface は scheduler decisions と synthetic output references だけを公開する。
cache key を公開せず、dependency fingerprint を計算せず、cache records を読まず、
proof-reuse validation もしない。

## Failure, Cancellation, and Commit Seams

Failure propagation は bounded である。failed task は、その outputs を必要とする
correctness dependents だけを block する。independent tasks は継続してよい。
`SchedulerRun` は決定的な `failure_records` と `blocked_records` を持つ。詳細な
failure classes、blocked-state diagnostics、nearest-blocker propagation、
degraded-mode permissions は `failure_state.md` が定義する。

Cancellation は snapshot で versioned である。cancelled task は current diagnostics や
artifacts を publish しない。実際の cancellation-token storage、child-process termination、
snapshot supersession は tasks 13-14 の `cancel.md` が定義する。

Blocked tasks と cancelled tasks は synthetic outcome outputs や diagnostics を
`SchedulerResult` に copy しない。failed tasks は failure diagnostics を持ってよいが、
dependents へ output references を publish しない。

Artifact commit は scheduler layer と manifest-transaction layer を持つ。この spec は
canonical `ArtifactCommit` task ordering と outcome を定義し、build-side manifest consumer は
`artifact_commit.md` が定義する。その consumer は caller-supplied manifest entries を
`mizar-artifact` へ渡してよいが、producer artifacts を書かず、publication token を mint せず、
artifact records を proof authority として扱わない。

## Tests

Task 9 は documentation-only である。Task 10 は focused Rust tests を追加する:

- shuffled worker completion order の下での deterministic execution。
- worker-count variation が同一 result/event collation を生成すること。
- immutable synthetic output publication と shared mutable semantic output がないこと。
- Task 8 graph 上の readiness transitions。`PackageConservative`、
  `MissingModuleDependencyOverlay`、`MissingVcDescriptors` coverage を含む。
- coordinator、source/local CPU、deterministic proof、ATP portfolio、ATP process、
  kernel、I/O commit、documentation queues。
- priority hints が execution latency にだけ影響し、canonical result order に影響しないこと。
- reversed `BackendRun` と `KernelCheck` completion orders が descriptor/backend order で
  collate され、proof-acceptance authority を公開しないこと。
- `BuildTask.dependencies` と scheduler state records の整合。
- disabled cache seam behavior: `Miss`、`Unavailable`、error-as-miss は tasks を通常実行し、
  `CacheHit` を生成せず、cache keys を構築しないこと。
- task-18 cache seam behavior: caller-validated hits が execution を skip し、
  canonical scheduler-visible output references を publish し、dependents を unblock し、
  proof authority を作らないこと。duplicate cache decisions と graph に存在しない task
  id への decisions が scheduler input boundary で失敗すること。
- missing/failed dependencies が bounded `Blocked` states を生成すること。
- synthetic cancellation が current publication を防ぐこと。
- `mizar-driver`、`mizar-ir`、cache-key construction、dependency-fingerprint
  construction、proof-reuse validation、publication token、proof-authority placeholders が
  存在しないこと。

## Non-Authority Rules

- Scheduling readiness は semantic acceptance ではない。
- `CacheHit` は proof evidence ではない。
- `Skipped` ATP/backend work は accepted proof status ではない。
- Worker completion order は proof candidate、diagnostic order、artifact order、
  cache publication order を決して選ばない。
- Artifact commit records は trusted status を昇格しない。
