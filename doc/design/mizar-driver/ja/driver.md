# Module: driver

> 正本は英語です。英語版: [../en/driver.md](../en/driver.md)。

状態: task D-007 で仕様化。source implementation は task D-008。

## 目的

`driver` module は build submission、cancellation、event subscription の
`CompilerDriver` front door を定義する。

driver は request/session state、phase-service registry、`mizar-build` の
planning / task-graph / scheduler authority、diagnostics sink、IR output
publication、後続の event stream を結線する。所有するのは orchestration である。
phase semantics、scheduler semantics、cache compatibility、proof acceptance、
artifact serialization、LSP protocol conversion は所有しない。

## 所有境界

`driver` が所有するもの:

- CLI/watch/LSP-facing caller から `BuildRequestDraft` を受け取り、
  `BuildSession` を allocate / capture する;
- session を driver lane の current として mark し、request publication guard に
  よって obsolete watch/LSP session を suppress する;
- `mizar-build` planner と module index API を通じて phase-0 build bootstrap を
  呼び出す;
- `mizar-build::task_graph::TaskGraphInput` を構築し、
  `mizar-build::task_graph::build_task_graph` へ渡す;
- `mizar-build::scheduler::SchedulerInput` を構築し、
  `mizar-build::scheduler::run_scheduler` result を消費する;
- registered phase service を `PhaseRegistry` 経由でのみ dispatch する;
- owner-provided phase result、diagnostic batch、sealed output handle を
  driver / scheduler / session outcome へ変換する;
- 後続の `events` module のため、protocol-agnostic event stream handle を公開する。

`driver` が所有しないもの:

- manifest syntax rule、dependency solving、task graph validation、ready-queue
  semantics、resource admission、cache scheduling policy、cancellation checkpoint
  decision、scheduler result collation。これらは `mizar-build` が所有する;
- source loading、parsing、name resolution、type checking、overload resolution、
  VC generation、ATP policy、proof acceptance、kernel trust、documentation
  extraction。これらは phase crate が所有する;
- `mizar-cache` の cache-key construction、dependency fingerprint、cache-store
  compatibility、proof-reuse validation、cache promotion;
- `mizar-diagnostics` の diagnostic-code allocation、diagnostic identity、
  aggregation、rendering、explanation、fix semantics;
- `mizar-ir` の storage internals、producer payload schema、artifact projection
  authority;
- `mizar-artifact` の manifest transaction、artifact serialization、
  publication-token issuance;
- `mizar-lsp` の JSON-RPC payload、document-version handling、range conversion、
  code action、editor command。

## Gap Classification

| Gap | Classification | Driver disposition |
|---|---|---|
| `SourceFrontend` はまだ canonical frontend payload や diagnostics draft を publish できない。 | `external_dependency_gap` | [frontend_adapter.md](frontend_adapter.md) の registry missing-service classification を維持する。frontend output を synthesize しない。 |
| semantic / proof / artifact / doc phase adapter がすべて利用可能ではない。 | `external_dependency_gap` / `deferred` | submit call は missing owner seam を blocked または unavailable として報告してよい。phase complete と mark してはならない。 |
| real cache lookup / compatibility はまだ `mizar-cache` 経由で結線されていない。 | `external_dependency_gap` | real cache decision が owner seam から供給されるまで、disabled / unavailable cache scheduling を使う。 |
| real artifact publication token と phase-15 producer emission が利用できない。 | `external_dependency_gap` | driver-owned code から committed-artifact event や manifest publication record を emit しない。 |
| 現在の `mizar-build` scheduler は public registry-dispatch callback ではなく precomputed modeled outcome を受け取る。 | `external_dependency_gap` | D-008 は scheduler submission と result consumption を validate してよいが、real scheduler-driven phase execution は owner dispatch seam を待つ。 |
| event stream source module は task D-010。 | `events.md` / `src/events.rs` が着地するまで `deferred` | `driver.md` は subscription boundary だけを仕様化する。event payload は後続で定義する。 |

## Public API

public API は概念上の形である。task D-008 は既存の request / registry module に合う
具体的な ownership と borrowing details を選んでよい。

```rust
struct CompilerDriver {
    sessions: DriverSessionStore,
    lanes: DriverLanes,
    registry: PhaseRegistry,
}

impl CompilerDriver {
    fn submit(&mut self, request: BuildRequestDraft, input: DriverSubmitInput)
        -> Result<BuildSubmission, DriverSubmitError>;

    fn cancel(&mut self, session: BuildSessionId, reason: DriverCancelReason)
        -> DriverCancelOutcome;

    fn events(&self, session: BuildSessionId) -> BuildEventStream;
}
```

`submit` は driver-owned build session を開始する唯一の entry である。caller は
CLI、watch mode、`mizar-lsp` のいずれでもよいが、protocol-agnostic driver input を
渡さなければならない。この API は LSP range、JSON-RPC id、editor command payload を
受け付けない。

`cancel` は driver / session の cancellation request を記録し、
`mizar-build::cancel::CancellationPolicy` を通じて表現する。worker thread を kill せず、
cancellation を phase failure と再解釈しない。

`events` は session-scoped かつ protocol-agnostic な event stream を返す。
`events.md` が着地するまで、task D-008 は test に必要な最小 in-memory /
subscription boundary だけを公開してよいが、LSP payload を公開してはならない。

## Submit Input

`BuildRequestDraft` は allocator-issued request identity が存在する前の request
metadata を持つ: lane / generation、source snapshot inputs、dependency artifacts、
lockfile / toolchain identity、verifier-config identity、target、profile、origin。
`BuildRequestId` は draft が `PendingBuildRequest` として受理されるときだけ allocate される。

`DriverSubmitInput` は build bootstrap に必要な owner input を持つ:

- request / session id と snapshot capture のための `SessionIdAllocator` と
  `SnapshotRegistry`;
- `produce_build_plan` 用の `mizar-build::planner::PlanRequest`、workspace package
  manifest、lockfile data;
- `build_module_index` 用の `mizar-build::module_index::SourceLayoutProvider` と
  dependency artifact index;
- real owner seam から供給される module dependency overlay と VC descriptor、または
  seam がない場合の classified unavailable coverage;
- task graph profile、scheduler mode、priority hint、resource budget、
  cancellation policy、cache scheduling plan / policy;
- real phase service が要求する場合に owner crate から供給される diagnostics と IR
  publisher handle。

driver はこれらの input が存在し適切に分類されていることを validate してよい。
local rule で manifest を parse したり、semantics を推測して module dependency を導出したり、
cache key を構築したり、output handle を mint したり、publication token を発明してはならない。

## Submit Control Flow

1. supplied session id allocator を使い、`BuildRequestDraft` を
   `PendingBuildRequest` へ allocate する。
2. `PendingBuildRequest::capture_snapshot` で immutable snapshot を capture し、
   返された active snapshot lease を `BuildSession` に保持する。
3. `DriverLanes::mark_current` で session を lane-current にする。lane が old /
   conflicting session として拒否した場合、その session を `Superseded` として finish し、
   current output は publish しない。
4. `mizar-build` owner API で phase 0 を実行する:
   `planner::produce_build_plan`、`module_index::build_module_index`、
   `task_graph::build_task_graph`。
5. planning、module-index、task-graph diagnostic が返った場合、structured kind / value data を
   diagnostics integration のために保持する。diagnostic identity を rendered message text に
   落としてはならない。
6. graph task ごとに `PhaseRegistry` を確認する。missing phase service は classified
   missing-service または external-dependency outcome を生む。synthetic output handle は
   決して作らない。
7. `TaskGraph` と owner-provided scheduling controls から `SchedulerInput` を構築する。
   cache decision は `Disabled`、`Unavailable`、または owner-provided real plan である。
   driver は compatibility を決定してはならない。
8. Task D-008 では scheduler submission の前に phase service を実行しない。現在の
   `mizar-build` scheduler は modeled / synthetic scheduler input surface であり、
   precomputed `SyntheticTaskOutcome` を受け取るが、scheduler dispatch 時に driver へ
   `PhaseService` 実行を依頼する public callback を持たない。したがって real
   scheduler-driven phase execution は `external_dependency_gap` である。
9. 得られた `SchedulerInput` は scheduler submission validation と authoritative scheduler
   state の消費のためだけに `mizar-build::scheduler::run_scheduler` へ渡す。driver は、
   default completed output を含む scheduler synthetic output を real `mizar-ir` phase output
   と扱ってはならない。
10. 将来 `mizar-build` owner seam が real scheduler dispatch を公開したら、driver は
    scheduler-selected execution point でのみ registry query boundary 経由の phase service を
    実行し、real `PhaseResult` の output reference だけを session outcome へ変換する。
11. result、diagnostic、artifact-boundary handoff を current として公開する前に、
    `request.md` の combined request publication guard を呼ぶ。
12. structured scheduler / session outcome から、session を succeeded、failed、blocked、
    cancelled、superseded のいずれかとして finish する。

required real owner seam がない場合、driver は fail-fast を選んでよい。test が明示的に
scheduler fixture として scope する場合だけ、modeled または test-local outcome 上の scheduler
validation を実行してよい。存在しない phase が output を生成したように見せるために、
`mizar-build` synthetic output type を使ってはならない。

## Scheduler Boundary

`mizar-build` は task graph と scheduler semantics を所有する。driver は次を消費する:

- canonical dependency / phase-order plan としての `TaskGraph`;
- 唯一の scheduler submission format としての `SchedulerInput`;
- authoritative scheduler output としての `SchedulerRun`、`TaskStateRecord`、
  `SchedulerResult`、`SchedulerEvent`、failure record、blocked record、
  resource telemetry、scheduler diagnostic。

driver は scheduler record を build event と session status へ map してよい。次は禁止:

- ready queue の再計算;
- worker completion time による scheduler result の並べ替え;
- modeled outcome を埋めるために scheduler dispatch より前に phase service を実行すること;
- `CacheHit` を proof evidence と再解釈すること;
- `Skipped`、`Cancelled`、`Blocked` を semantic acceptance に変えること;
- `SyntheticOutputRef` を real sealed `AnyPhaseOutputRef` として扱うこと;
- artifact commit を completion order で publish すること。

## Cancellation

`cancel(session, reason)` は冪等である。

session が unknown または terminal なら、driver は no-op outcome を返す。session が active なら、
driver は state を `Cancelling` に移し、`mizar-build::cancel::CancellationPolicy` を使って
session の cancellation policy を更新し、scheduler / phase service が owner token と safe
checkpoint で cancellation を観測できるようにする。

watch/LSP supersession では、新しい session が lane-current になり、古い session は cancel
されるか `Superseded` として finish する。obsolete completed result は、event や diagnostic が
current になる前に combined publication guard を通過しなければならない。

real artifact owner が atomic commit 開始を報告した後、cancellation は artifact manifest
transaction を中断してはならない。driver はその transaction state を所有しない。存在する場合に
owner-provided commit event を消費するだけである。

## Artifact Boundary

task graph には phase 15 が含まれるため、driver は `ArtifactCommit` task を schedule してよい。
artifact owner seam が real committed result または projection output を返す場合にだけ、
artifact-boundary handoff を公開してよい。

driver は次をしてはならない:

- artifact を自分で serialize する;
- publication token を mint する;
- scheduler `Completed` を artifact publication と扱う;
- stale / obsolete session の phase-15 event を publish する;
- artifact / proof / kernel owner seam なしに cached または retained IR handle を
  verified artifact に変える。

## Diagnostics Boundary

phase service は `mizar-diagnostics` producer sink を通じて diagnostic を emit するか、
`DiagnosticBatch` を返す。planning、graph、scheduler diagnostic は、diagnostics owner が
提供する場合に structured diagnostic code / category bridge を通じて変換しなければならない。

driver は diagnostic batch と readiness event を transport してよい。diagnostic code を allocate
したり、record を deduplicate したり、CLI text を render したり、LSP diagnostic payload を publish
したり、message string を identity として使ってはならない。

## Testing Requirements

Task D-008 の source test は次を cover しなければならない:

- submit は session snapshot を capture し、`mizar-build` bootstrap が成功した後だけ
  submitted / running と mark する;
- phase-0 bootstrap は `mizar-build` planner / module-index / task-graph API を使う;
- missing real phase service は synthetic output なしで classified gap として報告される;
- scheduler submission は ready-queue semantics を複製せず `SchedulerInput` /
  `SchedulerRun` を消費する;
- D-008 は現在の `mizar-build` synthetic outcome を modeled scheduler fixture としてのみ扱い、
  real phase output handle としては扱わない;
- stale または superseded session は current diagnostic や artifact-boundary event を
  publish できない;
- source / lint guard は、D-008 が diagnostic code allocation、diagnostic record
  deduplication、CLI / LSP diagnostic rendering、artifact serialization、
  publication token minting、scheduler completion の artifact publication 扱いを追加しないことを
  証明する;
- cancellation は冪等であり、`mizar-build` cancellation policy / token を通じて表現される。

test が phase service を必要とする場合、test 対象の実装済み挙動に限って test-local fixture を
使ってよい。fixture service は export してはならず、real adapter として文書化してもならない。
