# mizar-build Resources

> Canonical language: English. Canonical document:
> [../en/resource.md](../en/resource.md).

## Purpose

この文書は `mizar-build` が所有する resource-budget と queue-admission contract を定義する。

Resource budgeting は task correctness、proof acceptance、diagnostic order、
artifact order、cache behavior を変えずに、parallelism と external process pressure を
制限する。これは `task_graph.md` と `scheduler.md` が定義する `TaskGraph` と
scheduler queues を消費するが、新しい semantic authority を作らない。

## Context

- [architecture 14](../../architecture/ja/14.parallel_verification_and_scheduling.md)
- [architecture 22](../../architecture/ja/22.incremental_verification_contract.md)
- [internal 01](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)
- [scheduler.md](./scheduler.md)
- [task_graph.md](./task_graph.md)

## Scope

`resource` が所有するもの:

- workspace、package、module、VC obligation、backend process、commit work の
  hierarchical budget scopes。
- ready tasks の deterministic queue-admission decisions。
- modeled worker slots、memory units、ATP process slots、timeout budgets、
  capture limits、I/O commit permits の accounting。
- local semantic work と expensive ATP/backend work の starvation prevention。
- ready work が admit または delay された理由を説明する telemetry records。

`resource` が所有しないもの:

- task graph construction または correctness dependency edges。
- phase semantics、proof selection、kernel trust、ATP policy decisions。
- OS process spawning、child termination、stdout/stderr capture implementation、
  backend protocol details。
- cache-key construction、dependency fingerprints、proof-reuse validation、
  cache hit authority。
- artifact publication tokens、manifest writes、trusted-status promotion。
- driver session ownership、watch/LSP event streams、`salsa` query storage。

## Gap Classification

| ID | Class | Evidence | Action |
|---|---|---|---|
| RES-G001 | `design_drift` | `todo.md` は `resource.md` を要求していたが、task 11 の前には module spec が存在しなかった。 | task 11 でこの spec と Japanese companion を追加する。 |
| RES-G002 | `source_drift` / `test_gap` | `src/resource.rs` と resource-budget tests はまだ存在しない。 | task 12 でこの spec に沿って accounting/admission source と focused tests を実装する。 |
| RES-G003 | `external_dependency_gap` | real ATP backend process managers と capture adapters は `mizar-build` の外にある。 | backend limits と handoff values だけを model し、ここで OS processes を spawn/supervise しない。 |
| RES-G004 | `external_dependency_gap` | この checkout には `mizar-driver` request/session/watch integration が存在しない。 | budgets を input-driven かつ session-agnostic に保ち、driver dependency や placeholder driver API を追加しない。 |
| RES-G005 | `deferred` | cache-aware scheduling は task 18 のままであり、`mizar-cache` は validation を所有する。 | Resource decisions は cache-ready work を優先してよいが、cache key を構築せず cache hits を昇格しない。 |
| RES-G006 | `deferred` | cancellation と failure-state policies は tasks 13-16 である。 | terminal states での resource release だけをここで仕様化し、cancellation tokens と failure categories はそれぞれの modules に残す。 |

## Budget Model

Budgets は hierarchical である。task は、該当するすべての ancestor budgets が必要 units を
reserve できる場合だけ start してよい。

| Budget | Applies To | Examples |
|---|---|---|
| workspace | whole scheduler run | total local workers、total modeled memory、total ATP processes、total I/O commits |
| package | one package | package build concurrency、package-local semantic workers |
| module | one workspace module | active module-phase workers、module からの active VC obligations |
| obligation | one VC descriptor | ATP timeout budget、axiom/search policy budget、backend fanout limit |
| backend | one backend attempt | process slot、wall-clock timeout、memory ceiling、stdout/stderr capture limit |
| commit | artifact publication boundary | manifest transaction permit、artifact write permit |

Budgets は conservative である。upstream integration が precise unit を記述できない場合、
overcommit ではなく最も近い安全な coarse unit を要求しなければならない。

task 12 は integral units から始めてよい:

```rust
struct ResourceBudget {
    workspace_workers: usize,
    source_workers: usize,
    proof_workers: usize,
    atp_processes: usize,
    kernel_workers: usize,
    io_commits: usize,
    memory_units: usize,
}

struct TaskResourceRequest {
    queue: SchedulerQueue,
    package: Option<PackageId>,
    module: Option<ModuleId>,
    vc: Option<VcTaskDescriptorId>,
    worker_units: usize,
    memory_units: usize,
    external_process_slots: usize,
}
```

この形は illustrative である。不変条件は、admitted task がすべて bounded request を持ち、
terminal task が reservation を正確に一度だけ release することである。

## Queue Admission

Ready tasks はまず `scheduler.md` が選んだ scheduler queue に入る。次に resource admission が、
ready task を running worker slot へ移してよいか決定する。

Admission は次を満たさなければならない:

1. `TaskGraph` の correctness dependency edges を保つ。
2. 該当するすべての ancestor budgets を atomic に reserve する。
3. 同じ budget を争う tasks には deterministic tie-breakers を使う。
4. temporary exhaustion だけが理由の denial では task を failed にせず ready または queued に残す。
5. canonical diagnostics や artifact ordering を変えずに stable admission telemetry を出す。

temporary exhaustion による admission denial は semantic failure ではない。ready だが budget が
足りない task は、resources が release されたとき再び schedulable になる。

task が configured hard limit では絶対に満たせない resource を要求する場合、task 12 は
indefinite retry ではなく stable scheduler/resource diagnostic を報告して task を block する。
その diagnostic は proof evidence ではなく、それ自体で trusted status を昇格/降格しない。

## Deterministic Ordering

Resource availability は latency にだけ影響する。

複数の ready tasks が budget を争う場合、admission order は次の順である:

1. `scheduler.md` の scheduler priority key。
2. resource queue rank。
3. `TaskGraph` の package/module/VC/backend canonical work-unit order。
4. `TaskId`。

Worker completion order は proof candidates、diagnostic order、artifact order、
cache publication order、manifest commit order を選んではならない。

同じ `TaskGraph`、resource configuration、synthetic task behavior、cache seam responses、
cancellation inputs を使う 2 つの実行は、run 内の resource release timing が異なっても、
collation 後の canonical `SchedulerRun` が同じでなければならない。

## Worker Pools

Worker pools は resource classes と queue permits によって model する:

| Pool | Scheduler queues | Typical work |
|---|---|---|
| coordinator | coordinator | root bookkeeping と graph-level orchestration |
| source/local CPU | source/local CPU | source loading、frontend、module resolution、checking、VC generation |
| deterministic proof | deterministic proof | deterministic discharge と bounded computation |
| ATP portfolio | ATP portfolio | portfolio coordination と child-subgraph accounting |
| ATP process | ATP process | external backend process attempts |
| kernel | kernel | kernel/SAT evidence validation |
| I/O commit | I/O commit | artifact/manifest commit attempts |
| documentation | documentation | verified artifacts 後の documentation extraction |

この table order は、scheduler priority の次に使う default resource queue rank である。
task 12 は queue 内 aging や fairness を追加してよいが、canonical result collation を変えない
deterministic state で行わなければならない。

ATP portfolio pool は coordination work である。concrete `BackendRun` を開始しない限り、
external backend process slot を消費してはならない。

ATP process pool は backend runner へ渡す external-process limit を表す。`mizar-build` は
modeled slot を記録し強制する。実際の process creation、termination、output capture は
backend manager が所有する。

## ATP and Backend Limits

1 つの VC portfolio について:

- `AtpSolve` は proof/portfolio coordination permit を消費する。
- 各 `BackendRun` は 1 つの backend process slot と、該当する obligation/backend budget を消費する。
- Backend fanout は obligation fanout limit と global ATP process pool の小さい方で上限を持つ。
- Backend timeout、memory、capture limits は constraints として ATP/backend runner へ渡され、
  `mizar-build` はそれらを proof outcome として解釈しない。
- backend slot の release は accepted evidence を選ばない。proof selection は引き続き
  policy/kernel work である。

後続の early proof policy が、pending backend が current winner を覆せないと判断する場合、
余剰 backend work の cancellation は resource accounting だけではなく cancellation と ATP
modules が調整する。

## Release and Accounting

Resource reservation は、admitted task が terminal scheduling state に到達したとき release する:

- `Completed`
- `CacheHit`
- `Skipped`
- `Failed`
- `Blocked`
- `Cancelled`

resources の release は outputs を publish しない。Publication は scheduler、artifact、proof、
cancellation boundaries に従う。

Resource telemetry は次を記録してよい:

- task id と queue。
- requested units。
- admitted または delayed status。
- blocking budget scope。
- deterministic admission attempt order。

Telemetry は progress/debugging と後続 driver events のためのものである。これは diagnostic
ordering key、artifact manifest input、cache validation input、proof authority ではない。

## Tests

Task 11 は documentation-only である。Task 12 は focused Rust tests を追加しなければならない:

- budget exhaustion が overcommit せず ready work を queue すること。
- reservations が workspace/package/module/obligation/backend/commit budgets を
  compose すること。
- resource release がすべての terminal state で正確に一度だけ起きること。
- worker-count changes が canonical result/event collation を変えないこと。
- ATP portfolio coordination が backend process slots を消費しないこと。
- backend fanout が obligation と global ATP process budgets の両方で制限されること。
- I/O commit permits が publication authority を作らず modeled commit work を
  serialize すること。
- shuffled ready/completion order の下でも deterministic admission ordering が安定すること。
- impossible requests が infinite retry ではなく stable resource diagnostics を生成すること。
- resource telemetry が proof status、diagnostics、artifact order、cache identity、trusted status に
  影響しないこと。
- `mizar-driver`、`mizar-cache`、ATP OS-process、artifact publication token、
  proof-authority placeholder を導入しないこと。

## Non-Authority Rules

- Resource admission は semantic acceptance ではない。
- budget があることは proof candidate の acceptance を意味しない。
- temporary budget が不足していることは proof failure ではない。
- Backend process completion order は proof evidence を決して選ばない。
- Resource telemetry は cache validation や trusted status に影響しない。
- Artifact commit permits は publication authority を mint しない。
