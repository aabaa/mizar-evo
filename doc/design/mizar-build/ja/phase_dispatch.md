# mizar-build scheduler-selected phase dispatch

> 正本は英語です。英語版:
> [../en/phase_dispatch.md](../en/phase_dispatch.md)。

## 目的

この文書は、scheduler が選択した task を外部 owner に実行させるための
`mizar-build` seam を定義する。

この seam は DRIVER-G-011 の build-side 部分を閉じる。`mizar-build` は task
readiness、dependency ordering、resource admission、cancellation checkpoints、
cache-aware decision consumption、deterministic collation を引き続き所有する。
`mizar-driver` は registry-backed dispatcher を渡してこの seam を消費してよいが、
driver が scheduler semantics を再実装してはならない。

## Context

- [scheduler.md](./scheduler.md)
- [cache_seam.md](./cache_seam.md)
- [task_graph.md](./task_graph.md)
- [architecture 14](../../architecture/ja/14.parallel_verification_and_scheduling.md)
- [internal 01](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)
- [architecture 22](../../architecture/ja/22.incremental_verification_contract.md)
- [mizar-driver driver.md](../../mizar-driver/ja/driver.md)
- [mizar-driver registry.md](../../mizar-driver/ja/registry.md)

## 所有境界

`mizar-build` が所有するもの:

- canonical task graph から ready task を選択すること。
- execution 前に validated cache decisions を消費すること。
- worker/resource limits の下で selected task を admit すること。
- start 前、running checkpoint、publication 前の cancellation を適用すること。
- scheduler が task を execution 用に選択した後にだけ dispatch callback を呼ぶこと。
- callback の scheduler outcome を task state、failure/block records、resource
  release、deterministic event/result collation へ写すこと。

`mizar-build` が所有しないもの:

- phase semantics、type checking、name resolution、overload resolution、VC
  generation、ATP policy、kernel acceptance、proof acceptance、trusted status、
  proof reuse。
- driver build requests、sessions、event streams、phase-service registry、`salsa`
  query storage。
- `mizar-cache` cache-key construction、dependency fingerprints、cache-store
  compatibility、proof-reuse validation。
- `mizar-ir` sealed output storage、producer payload schemas、snapshot
  rehydration。
- artifact serialization、manifest publication tokens、producer-owned artifact
  projection、LSP protocol conversion。

callback が返すのは scheduler outcome だけである。callback は phase semantics を
`mizar-build` へ移さず、`mizar-build` が output handle、artifact publication token、
cache compatibility decision、proof authority を mint することも許さない。

## Gap classification

| ID | Class | Evidence | Action |
|---|---|---|---|
| DISPATCH-G001 | `source_drift` / `test_gap` | DRIVER-G-011 は、execution-selection point を scheduling が所有するにもかかわらず、`mizar-build` が scheduler-selected real phase dispatch callback を公開していないことを記録していた。 | この task は `mizar-driver` dependency を追加せず callback seam と focused tests を追加する。 |
| DISPATCH-G002 | `external_dependency_gap` | real phase services は owner-supplied phase input identities、parent output handles、diagnostics sinks、output publishers を必要とする。これらは build-owned data ではない。 | driver は scheduler-selected dispatch point で seam を消費する。selected task に owner inputs が欠ける場合、registry-backed dispatcher が owner gap を記録し、inputs を fabricate しない。 |
| DISPATCH-G003 | `external_dependency_gap` | real producer outputs と artifact publication tokens は `mizar-build` の外に残る。 | dispatch completion は scheduler completion だけである。artifact publication は owning producer/artifact seam を待つ。 |
| DISPATCH-G004 | `deferred` | real semantic、proof、artifact、cache、LSP integrations 上の full clean/incremental/parallel equivalence は、この task より多くの owner seams を要求する。 | 既存の implemented-seam equivalence tests を維持し、focused dispatch-seam tests を追加する。 |

## API contract

Rust surface は `SchedulerInput` から意図的に分離する。これにより input は
clone/comparable な deterministic build data のままである:

```rust
pub trait SchedulerTaskDispatcher {
    fn dispatch(&mut self, task: SchedulerDispatchTask<'_>) -> SchedulerDispatchOutcome;
}

pub struct SchedulerDispatchTask<'a> {
    pub task: &'a BuildTask,
    pub snapshot: BuildSnapshotId,
    pub cancellation: Option<CancellationToken>,
}

pub struct SchedulerDispatchOutcome {
    pub status: SchedulerDispatchStatus,
    pub diagnostics: Vec<SchedulerDiagnosticRef>,
}

#[non_exhaustive]
pub enum SchedulerDispatchStatus {
    Complete,
    Failed,
    Blocked,
    Skipped,
    Cancelled,
}

pub fn run_scheduler_with_dispatcher<D: SchedulerTaskDispatcher>(
    input: SchedulerInput,
    dispatcher: &mut D,
) -> Result<SchedulerRun, SchedulerDiagnostics>;
```

`run_scheduler` は modeled/synthetic scheduler fixtures のために残る。real phase
services は実行しない。

external crates は `SchedulerDispatchTask` の public fields を通じて selected task を
消費する。`SchedulerDispatchOutcome` は public fields と constructor helpers
(`complete`, `failed`, `blocked`, `skipped`, `cancelled`) を公開し、driver/registry
adapter が fake producer outputs や artifact publication tokens を作らず scheduler
states を返せるようにする。

callback は scheduler readiness、cache fallback、cancellation-before-start、
resource admission を通過した task に対してのみ呼ばれる。validated cache hit は引き続き
callback execution を skip する。callback は build-owned task identity、snapshot、
optional cooperative cancellation token を受け取る。mutable scheduler internals は
受け取らない。

callback は以下を返してよい:

- `Complete`: scheduler は `Completed` を記録する。
- `Failed`: scheduler は `Failed` と deterministic failure records を記録する。
- `Blocked`: scheduler は direct dispatch block を記録し、correctness dependents を
  block する。
- `Skipped`: scheduler は `Skipped` を記録する。既存の conditional subgraph rules は、
  dependent callback が走る前に、どの dependents を unblock または skip できるかを
  引き続き決める。
- `Cancelled`: scheduler は `Cancelled` を記録する。

callback outcome を publish する前に、scheduler は completed-before-publication freshness
checkpoint を引き続き適用する。resource releases と event ordering は scheduler-owned で
deterministic である。

## Driver consumption contract

`mizar-driver` は、scheduler-selected task に対して phase registry query boundary を呼ぶ
dispatcher を渡してこの seam を消費する。driver は phase inputs、diagnostics sinks、
cancellation resources、output publisher handles を準備してよい。これらは
driver/owner boundaries だからである。driver は以下をしてはならない:

- ready queues や dependency ordering を計算する。
- scheduler dispatch 前に phase services を実行する。
- cache compatibility や proof-reuse decisions を複製する。
- `SchedulerResult.output_refs` や synthetic outputs を real `mizar-ir` handles として扱う。
- artifact owner なしで artifact-boundary events を publish する。
- registry boundary で diagnostics を CLI/LSP presentation へ変換する。

real phase input identity、producer output、artifact token、diagnostics bridge、LSP
bridge が利用できない場合、driver は `external_dependency_gap` または `deferred` owner
gap を記録する。fake phase adapter、stub producer output、provisional artifact token、
fake cache decision、proof authority、LSP payload を作ってはならない。

## Tests

この task は focused Rust tests を追加する:

- scheduler callback が scheduler-selected dispatch 後にだけ、deterministic task order で
  呼ばれること。
- callback order が simulated worker completion order に依存しないこと。
- cache hits が callback を skip しつつ dependents を unblock すること。
- resource admission failures が callback execution を防ぐこと。
- dispatcher mode が既存の skipped-dependency scheduler semantics を保つこと。
- callback `Blocked` outcomes が synthetic outputs なしで dependents を block すること。
- running-checkpoint cancellation が callback execution を防ぐこと。
- driver が `PhaseRegistry` 経由でこの seam を消費し、multi-phase task の service-span
  dispatch、registry status mapping、cache-hit task が owner dispatch inputs を要求しないことを
  cover する。
- `mizar-driver` が test-local registry fixture を通じて seam を消費し、scheduler
  readiness、dependency ordering、cache decisions、result collation を再実装しないこと。
- source guards が、この seam によって driver、IR、cache-key、proof-authority、
  artifact-token、producer-output、LSP authority が `mizar-build` に追加されないことを
  示すこと。

test-local fixture phase services は scheduler と driver integration contract を証明する
場合に限り許可される。real adapters として export または documentation してはならない。

## 公開 enum policy

この module が所有する exhaustive public enum exception はない。

| Enum | Policy |
|---|---|
| `SchedulerDispatchStatus` | `#[non_exhaustive]`; downstream callers は wildcard match arms を含めなければならない。 |
