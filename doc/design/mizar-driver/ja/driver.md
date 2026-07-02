# Module: driver

> 正本は英語です。英語版: [../en/driver.md](../en/driver.md)。

状態: task D-007 で仕様化。source implementation は task D-008。

## 目的

`driver` module は build submission、cancellation、event subscription の
`CompilerDriver` front door を定義する。

driver は request/session state、phase-service registry、`mizar-build` の
planning / task-graph / scheduler authority、diagnostics sink、IR output
publication、protocol-agnostic event stream を結線する。所有するのは orchestration である。
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
  `mizar-build::scheduler::run_scheduler_with_dispatcher` / `run_scheduler`
  result を消費する;
- registered phase service を `PhaseRegistry` 経由でのみ dispatch する;
- owner-provided phase result、diagnostic batch、sealed output handle を
  driver / scheduler / session outcome へ変換する;
- `events` module の replayable protocol-agnostic event stream を公開する。

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

## 公開 enum の互換性

この module の public enum はすべて downstream-facing な driver / session boundary
type であり、`#[non_exhaustive]` を付ける。D-017 では以下について exhaustive
exception を記録しない:

- `WatchSnapshotReplacementStatus`;
- `WatchModeGapOwner`;
- `WatchOwnerSeam`;
- `WatchSubmitError`;
- `DriverSubmissionStatus`;
- `DriverCancelReason`;
- `DriverSubmitError`。

Downstream crate はこれらの enum を match するとき wildcard arm を持たなければならない。
将来の watch owner seam、cancellation reason、submission state、structured submit error
は、driver に phase semantics、proof、cache、artifact、LSP authority を与えずに追加できる。

## Gap Classification

| Gap | Classification | Driver disposition |
|---|---|---|
| `SourceFrontend` はまだ canonical frontend payload や diagnostics draft を publish できない。 | `external_dependency_gap` | [frontend_adapter.md](frontend_adapter.md) の registry missing-service classification を維持する。frontend output を synthesize しない。 |
| semantic / proof / artifact / doc phase adapter がすべて利用可能ではない。 | `external_dependency_gap` / `deferred` | submit call は missing owner seam を blocked または unavailable として報告してよい。phase complete と mark してはならない。 |
| real cache lookup / compatibility はまだ `mizar-cache` 経由で結線されていない。 | `external_dependency_gap` | real cache decision が owner seam から供給されるまで、disabled / unavailable cache scheduling を使う。 |
| real artifact publication token と phase-15 producer emission が利用できない。 | `external_dependency_gap` | driver-owned code から committed-artifact event や manifest publication record を emit しない。 |
| owner-provided phase input identities、parent output handles、diagnostics/output publisher handles、producer publication tokens が real scheduler-selected dispatch に常に利用できるわけではない。 | `external_dependency_gap` / `deferred` | scheduler-selected dispatch が owner inputs のない task に到達した場合は `BlockedByPhaseDispatchGap` で block する。供給された場合は `mizar-build` の scheduler-selected dispatcher seam を消費し、phase service は `PhaseRegistry` 経由でのみ実行する。 |
| external file watcher / coalescing owner と LSP build bridge が利用できない。 | `external_dependency_gap` / `deferred` | D-014 は owner-provided changed path と snapshot input を受け取ってよいが、fake file watcher API、source-loading rule、LSP document payload、editor snapshot conversion は作らない。 |
| `mizar-ir` snapshot replacement は `PhaseOutputPublisher` 経由でのみ利用できる。 | real owner seam が供給された場合のみ real、それ以外は `external_dependency_gap` | D-014 は最初の watch snapshot を current として register し、real publisher が供給され snapshot id が変わる場合だけ `replace_current_snapshot(old, new)` を呼ぶ。same-snapshot replacement は no-op。publisher input がない場合は provisional output handle ではなく分類済み gap。 |
| CLI rendering や LSP protocol conversion などの後続 event consumer はまだ実装されていない。 | entry-point task へ `deferred` | `events.md` / `src/events.rs` は protocol-agnostic event payload と replay を定義する。consumer は CLI / LSP authority を driver へ持ち込んではならない。 |

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

    fn submit_watch_change(
        &mut self,
        request: BuildRequestDraft,
        input: DriverSubmitInput,
        control: WatchSubmitControl,
    )
        -> Result<WatchSubmission, WatchSubmitError>;

    fn cancel(
        &mut self,
        session: BuildSessionId,
        reason: DriverCancelReason,
        snapshots: &SnapshotRegistry,
    )
        -> DriverCancelOutcome;

    fn events(&self, session: BuildSessionId) -> BuildEventStream;
}
```

`submit` は driver-owned build session を開始する canonical primitive である。caller は
CLI、watch mode、`mizar-lsp` のいずれでもよいが、protocol-agnostic driver input を
渡さなければならない。`submit_watch_change` は watch-facing wrapper であり、session
creation を `submit` に委譲したうえで lane-current supersession、replay suppression、
任意の IR snapshot replacement を処理する。どちらの API も LSP range、JSON-RPC id、
editor command payload を受け付けない。

`cancel` は driver / session の cancellation request と terminal outcome を記録する。
渡された snapshot registry と driver lane table で combined publication guard を適用してから
terminal event を追加する。`mizar-build::cancel::CancellationPolicy` は snapshot supersession
など owner seam が対応する場合だけ使う。worker thread を kill せず、cancellation を phase
failure と再解釈しない。

`events` は [events.md](events.md) / `src/events.rs` が実装する session-scoped かつ
protocol-agnostic な event stream を返す。driver は replay 可能な session event を保存するが、
CLI-rendered diagnostics、LSP payload、artifact publication token、scheduler synthetic output を
event payload として公開してはならない。

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
- scheduler-selected registry dispatch 用の任意の `PhaseDispatchInputProvider`
  identities。欠落した identities は、task が scheduler-selected dispatch callback に到達した
  場合にだけ報告される。

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
8. scheduler submission の前に phase service を実行せず、scheduler readiness、cache hit、
   cancellation、resource admission logic を dispatch preflight として replay しない。
   scheduler-selected task が owner-provided dispatch inputs を欠く場合、
   registry-backed dispatcher は blocked scheduler outcome を返す。driver はその scheduler
   result から `BlockedByPhaseDispatchGap` と affected phases を記録する。
9. 得られた `SchedulerInput` は registry-backed dispatcher 付きで
   `mizar-build::scheduler::run_scheduler_with_dispatcher` へ渡す。dispatcher は
   scheduler-selected execution point でだけ `PhaseRegistry` を呼び、selected task に listed
   された全 phase を、owning registry service span ごとに 1 回だけ task-graph phase order で
   invoke することで cover し、`PhaseStatus` を scheduler outcomes へ写す。driver は
   readiness、dependency ordering、resource admission、cache-decision consumption、
   cancellation semantics を複製してはならない。
10. driver は default completed output を含む scheduler synthetic output を real
    `mizar-ir` phase output と扱ってはならない。real `PhaseResult` output references、
    producer outputs、artifact tokens は owner seams のままである。
11. result、diagnostic、artifact-boundary handoff を current として公開する前に、
    `request.md` の combined request publication guard を呼ぶ。
12. structured scheduler / session outcome から、session を succeeded、failed、blocked、
    cancelled、superseded のいずれかとして finish する。

required real owner seam がない場合、driver は fail-fast を選んでよい。phase input
identities や publisher/producer seams がない場合は classified owner gaps のままである。
存在しない phase が output を生成したように見せるために、`mizar-build` synthetic output
type を使ってはならない。

## Scheduler Boundary

`mizar-build` は task graph と scheduler semantics を所有する。driver は次を消費する:

- canonical dependency / phase-order plan としての `TaskGraph`;
- 唯一の scheduler submission format としての `SchedulerInput`;
- authoritative scheduler output としての `SchedulerRun`、`TaskStateRecord`、
  `SchedulerResult`、`SchedulerEvent`、failure record、blocked record、
  resource telemetry、scheduler diagnostic。

D-008 は raw `SchedulerRun` を内部で消費するが、public な `BuildSubmission` surface には
output-free な driver scheduler summary（task state、scheduler event、scheduler diagnostic）
だけを公開する。`SchedulerResult.output_refs` や `SyntheticOutputRef` は公開しない。

driver は scheduler record を build event と session status へ map してよい。次は禁止:

- ready queue の再計算;
- worker completion time による scheduler result の並べ替え;
- modeled outcome を埋めるために scheduler dispatch より前に phase service を実行すること;
- `CacheHit` を proof evidence と再解釈すること;
- `Skipped`、`Cancelled`、`Blocked` を semantic acceptance に変えること;
- `SyntheticOutputRef` を real sealed `AnyPhaseOutputRef` として扱うこと;
- artifact commit を completion order で publish すること。

## Cancellation

`cancel(session, reason, snapshots)` は冪等である。

session が unknown または terminal なら、driver は no-op outcome を返す。session が active なら、
D-011 は state を `Cancelling` 経由で terminal session state へ移し、対応する terminal build event を
追加する。explicit request と shutdown は `Cancelled` として finish する。supersession は
`Superseded` として finish する。combined lane/snapshot publication guard が `Suppressed` を
返した場合に suppressed-publication event を emit する。

現在の `mizar-build::CancellationPolicy` は snapshot supersession と task-scoped explicit
cancellation を表現できるが、driver-owned snapshot-wide explicit-request / shutdown mutator を
公開していない。したがって D-011 は `DriverCancelReason::Superseded` だけを
`supersede_snapshot` で伝播し、explicit-request / shutdown は false supersession reason を
発明せず build policy には入れない。snapshot-wide explicit / shutdown policy propagation は
`external_dependency_gap` として記録する。

watch/LSP supersession では、新しい session が lane-current になり、古い session は cancel
されるか `Superseded` として finish する。obsolete completed result は、event や diagnostic が
current になる前に combined publication guard を通過しなければならない。

task D-014 は同じ `submit` boundary の上に watch-facing submission helper を追加する。
この helper は、changed path と source snapshot input がすでに external owner から供給
された `BuildRequestOrigin::Watch` draft を受け取る。filesystem watching、change
debounce、source text loading、module target inference、LSP protocol data conversion は
所有しない。新しい watch generation が accept された後、この helper は previous session
がすでに terminal state に達していた場合でも stored replay を更新し、replayed event が
`Current` status を主張し続けるのではなく、suppressed publication decision と terminal
`Superseded` outcome を運ぶようにする。

新しい watch request が request-id allocation や snapshot validation failure など
snapshot capture より前に失敗した場合、helper は previous session を current のままにし、
snapshot replacement も記録しない。snapshot capture が成功し、その後の planning、
module indexing、task graph construction、registry checking、scheduler submission が
失敗した場合、captured newer session は review 対象の lane generation になっている。
そのため previous session は supersede され replay は suppress され、供給された publisher
seam には successful submission と同じ規則で captured snapshot の register / replace を
依頼する。

real `mizar-ir::publisher::PhaseOutputPublisher` が供給された場合、最初の watch session は
snapshot を current として register する。以降の watch submission は snapshot id が変わる
場合に `replace_current_snapshot(old, new)` を呼ぶ。新しい generation が同じ snapshot id
を捕捉した場合、request / session lane guard が supersession を処理するため replacement は
no-op である。publisher が供給されない、または owner seam が replacement を拒否した場合、
result は分類済み gap / error を記録する。driver はこの seam の処理中も output handle、
artifact publication token、cache / proof reuse decision を mint しない。

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

Task D-008 と D-011 の source test は次を cover しなければならない:

- submit は session snapshot を capture し、`mizar-build` bootstrap が成功した後だけ
  submitted / running と mark する;
- phase-0 bootstrap は `mizar-build` planner / module-index / task-graph API を使う;
- missing real phase service は synthetic output なしで classified gap として報告される;
- scheduler submission は ready-queue semantics を複製せず `SchedulerInput` /
  `SchedulerRun` を消費する;
- scheduler-selected registry dispatch は `run_scheduler_with_dispatcher` を通じてだけ
  消費され、missing owner dispatch inputs は classified dispatch gaps のままである;
- stale または superseded session は current diagnostic や artifact-boundary event を
  publish できない;
- source / lint guard は、D-008 が diagnostic code allocation、diagnostic record
  deduplication、CLI / LSP diagnostic rendering、artifact serialization、
  publication token minting、scheduler completion の artifact publication 扱いを追加しないことを
  証明する;
- cancellation は冪等であり、terminal な `Cancelled` または `Superseded`
  session outcome に到達し、terminal replay event を追加する;
- supersession は `mizar-build::CancellationPolicy::supersede_snapshot` を通じて表現される;
- explicit-request / shutdown cancellation は未対応の snapshot-wide policy token を発明せず、
  その propagation を `external_dependency_gap` として保つ。

test が phase service を必要とする場合、test 対象の実装済み挙動に限って test-local fixture を
使ってよい。fixture service は export してはならず、real adapter として文書化してもならない。
