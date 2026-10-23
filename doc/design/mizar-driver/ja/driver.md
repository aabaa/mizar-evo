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
- scheduler-selected registry dispatch 用の任意の `mizar-ir`
  `PhaseDispatchInputProvider<BuildTask>` bundle。bundle は IR 所有 phase input
  identity と validated sealed parent output handle を含み、scheduler-selected
  snapshot と一致しなければならない。欠落した bundle は、task が scheduler-selected
  dispatch callback に到達した場合にだけ報告される。invalid IR dispatch input は
  別に報告し、fail closed する。

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
   scheduler-selected task が owner-provided `mizar-ir` dispatch input bundle を欠く場合、
   registry-backed dispatcher は blocked scheduler outcome を返す。driver はその scheduler
   result から `BlockedByPhaseDispatchGap` と affected phases を記録する。driver は欠落した
   parent handle を生 hash で置き換えてはならない。供給された IR bundle が別 snapshot
   用であるか validation failure を報告する場合、dispatch は欠落 bundle として扱わず、
   invalid-input diagnostic で fail する。
9. 得られた `SchedulerInput` は registry-backed dispatcher 付きで
   `mizar-build::scheduler::run_scheduler_with_dispatcher` へ渡す。dispatcher は
   scheduler-selected execution point でだけ `PhaseRegistry` を呼び、selected task に listed
   された全 phase を、owning registry service span ごとに 1 回だけ task-graph phase order で
   invoke することで cover し、`PhaseStatus` を scheduler outcomes へ写す。driver は
   readiness、dependency ordering、resource admission、cache-decision consumption、
   cancellation semantics を複製してはならない。
10. driver は default completed output を含む scheduler synthetic output を real
    `mizar-ir` phase output と扱ってはならない。real `PhaseResult` output references は IR publisher で検証し、
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
driver scheduler summary（task state、scheduler event、scheduler diagnostic）と
`DriverSchedulerRun.phase_results` の実 `PhaseResult` を task id / 呼出順で公開する。
`SchedulerResult.output_refs` や `SyntheticOutputRef` は公開しない。
`CompilerDriver::with_output_publisher` は所有者の `Arc<PhaseOutputPublisher>` を共有する。
snapshot と work unit の登録権限は所有者に残る。返された handle は同じ publisher の
current package output / snapshot として検証する。publisher 不在で返された handle、foreign/stale handle、
異なる snapshot の診断 batch は dispatch を失敗させ、不正 result を保存しない。
Complete 以外は診断のみ保持し、output handle を公開しない。session publication guard が
obsolete と判定した場合（same-snapshot supersession を含む）は phase results 全体を抑制する。
公開前に IR snapshot/output validity も再検証する。保持した handle は内部結果であり、
artifact publication 権限ではない。consumer は公開時に再検証する。

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

## Current native Disk diagnostic index

`validate_current_disk_diagnostics(session, snapshots) -> Result<(), String>` は全 Disk 制約を保ち、[共有 native-session 条件](#current-native-diagnostic-index)へ委譲する。この Disk 専用 API は OpenBuffer/Generated capture を引き続き非対応とし、未知・非対応・obsolete は共有条件に従い Err とする。
`submit_captured_with_diagnostic_index(session, allocator, snapshots, input) -> Result<(BuildSubmission, Option<BuildDiagnosticIndex>), DriverSubmitError>` は既存 submit_captured をそのまま実行し、その元の実 phase-result batch だけを Diagnostics from_batches で集約する。集約の前後で guard と実 scheduler run を要求し、不成立や集約失敗は submission を変えず None、元の提出 error は保持する。実 Failed/Blocked phase の診断と clean run の空 index を許し、scheduler message から draft を推測せず意味・証明成功を要求しない。この Disk 専用 method は Open/Generated capture を除外し、overlay・current LSP 公開・後続 consumer は保留する。index は immutable snapshot data であり、後の受け渡し前に live guard を再検査し、file/version/map/routing 条件は別途満たす。

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

## Scheduled SourceLoad/Frontend prefix

registry が既存組込 SourceLoad と Frontend の両 descriptor identity だけ、またはその正確な二つと明示登録した [bounded resolver](frontend_adapter.md#scheduled-source-theorem-module-resolver) の正確な descriptor/具体型、またはその正確な三つと明示 [signature service](frontend_adapter.md#scheduled-source-theorem-signature-collection) の正確な descriptor と resolver/signature 両具体型、またはその正確な四つと明示 [type service](frontend_adapter.md#scheduled-source-theorem-type-checking) の正確なdescriptorおよびresolver/signature/typeの具体型、またはその正確な五つと明示 [registration service](frontend_adapter.md#scheduled-source-theorem-registration-resolution) の正確なdescriptorおよびresolver/signature/type/registrationの具体型を含み、呼出し元が捕捉 snapshot に対して current な publisher を供給するとき、後段 service 不足でも scheduler が選択した prefix を実行する。この前提がなければ従来の missing-service 事前停止を維持する。driver はこの経路で snapshot 登録や publication 権限付与を行わない。
供給済み dispatch provider は None/error/不正 bundle を含め優先し、fallback しない。provider 自体がない場合だけ、一意に一致する捕捉 SourceVersion の frontend adapter で共用する SourceUnitCacheKey、空の dependency/parent から SourceLoad 入力を作る。Frontend は同じ key、捕捉 index の summary hash のソート済み multiset、当該 task の依存元にある同一 work unit の完了 SourceLoad 出力1件と、その依存 task id の保持済み Complete Frontend/FrontendOutput を current IR publisher で seal して使う。PackageResolve は出力を供給しない。Complete または ImportsOnly の ImportSummary graph edge が workspace Frontend の順序を定め、frontend adapter が対応する workspace 親と実際の import を照合して、不足・余分な workspace 親を拒否する。実出力のない cache-hit task は親を供給できない。[bounded resolver](frontend_adapter.md#scheduled-source-theorem-module-resolver) は実際の一意な同 module Frontend dependency から既定入力を得る。直後 TypeChecking が missing の正確な CheckAndElaborate [signature prefix](frontend_adapter.md#scheduled-source-theorem-signature-collection) だけは実 ModuleResolve dependency から追加の既定入力を得る。正確な5-service [type prefix](frontend_adapter.md#scheduled-source-theorem-type-checking) は、直後RegistrationResolutionがmissingの場合に同じtaskの実Complete signature出力から新しいTypeChecking入力を構築する。正確な6-service [registration prefix](frontend_adapter.md#scheduled-source-theorem-registration-resolution) は、直後OverloadResolutionがmissingの場合に同taskの実Type出力をRegistrationResolutionへ追加で渡す。他phaseには作らない。
prefix の例外経路では service がない task の ValidatedHit を Unavailable に変え、通常の scheduler 検証用の task id は保持する。利用可能 service の cache 動作は維持する。scheduling 後も missing-service 分類と event を保持する。frontend/source の失敗は session Failed のままとし、正常 prefix も後段 service 不足なら Blocked とする。status は BlockedByMissingPhaseServices とし、選択された owner-input 不足がある場合は BlockedByPhaseDispatchGap を優先する。選択された missing service は既定入力の構築前に分類する。取消し、親検証、最終 lane/publisher 現行性を維持し、bounded resolved output、別途登録したsignature summary、明示対応のTypeChecking出力とregistration closureだけにphase成功creditを与え、grouped task は Blocked を維持する。prefix 完了から proof/cache/artifact や完全 semantic-service の成功は与えない。

## Import-discovered native diagnostic index

`submit_captured_with_import_discovery_and_native_diagnostic_index<A: SessionIdAllocator, L: SourceLayoutProvider>(&mut self, session: BuildSession, allocator: &A, snapshots: &SnapshotRegistry<A>, input: DriverSubmitInput<L>) -> Result<(BuildSubmission, Option<BuildDiagnosticIndex>), DriverSubmitError>` は既存 submit_captured_with_import_discovery を一度呼び、既存 native guard と元 run の Diagnostics 集約を使う。error/submission、実 ImportsOnly、省略可能な空 Unavailable、指定 overlay の優先順位、全 discovery 失敗時の fallback は不変で、edge 推測や意味的 Complete への昇格をしない。
private current_diagnostic_index は実 submission/registry を借用し、既存 Disk/native 検証関数を受ける。呼出し元は従来の通常 Disk 入口、通常 native 入口、この opt-in 入口の三つ。実 scheduler run を要求し、元 phase_results batch の収集前と、実 snapshot での既存 Diagnostics from_batches 後に検証する。None/error は元 submission を保持する。別 run/index の保存・複製や新表現を作らず、旧入口は従来の submit_captured、各 guard と集約順序を保つ。
既存 Fixture/capture/publisher/service を再利用して三つの実 family を追記する。Disk/Open/混在の import-aware 実行と既存 discovery submission 全体・元 aggregate・独立 record 条件・実 NoKey、架空の正 Complete を使わない discovery 失敗/指定 overlay 優先、成立する対照を伴う適格性/currentness/構造化 submit error。disk 編集後も source ID/raw map を保持し、既存 import/URI/map/Disk-only/共通 guard/error matrix を重複しない。新 setup 関数を使うなら三 family 全てで共有し、旧 test はそのまま保つ。
明示的接続だけであり、import 意味論・diagnostic identity/code/filter policy・source 再読込・cache/受理権限を加えない。opaque context/metadata hash に runtime 認証を与えない。現 editor tuple/lifetime/map と host/wire binding は LSP、完全な意味 import/runtime/current-build publication/広い gate は保留する。

## Pregraph import discovery

`CompilerDriver::submit_with_import_discovery` は plan/index 構築後、graph 構築前の disk source 事前検証を明示的に選択する。空の Unavailable overlay だけを置き換え、他の coverage や供給済み edge は優先する。通常の `submit` は意図的な Unavailable を含め全 overlay を維持する。既存 frontend adapter が source 取得と暫定 path 解決を担当し、driver core はその overlay だけを受け取る。事前検証は phase 実行ではなく、出力・診断を公開しない。allocator と source identity は局所的に保持する。
全 workspace module に一意な捕捉 disk version と workspace plan/index identity を要求し、安全な package/source path の workspace 内包含と、load 後の package/module/path/edition/hash/origin の捕捉値との一致を確認する。session loader、frontend preprocessing、resolver の暫定 import mapping を使う。読込不能・変更・未対応・重複/欠落・preprocess 診断・未解決があれば推定 edge 全体を破棄して Unavailable を維持し、既存 graph 検証により schedule を停止する。解決済み artifact target に workspace edge は不要。成功時は重複のない workspace ImportSummary edge を ImportsOnly として渡し、意味的 Complete coverage は付与しない。graph の self/cycle/reference 検証は維持する。
予定された SourceLoad は disk と捕捉値を再照合し、Frontend は実際の parsed import、保持した親と対応済み summary を publication 前に検証する。事前検証は reexport、意味的 import、公開診断、proof/cache/artifact 成功や open-buffer 取得を許可しない。事前検証後の変更を捕捉 key の下で公開できない。

## Published output authorization

[publisher 承認](../../mizar-ir/ja/publisher.md#published-output-authorization)は
実 OpenBuffer/InternalOnly の publication 来歴を確認できる。Driver の consumer は
Native 取得と入力束縛が供給されるまで current/package 検証を維持する。来歴だけで
summary のみ/Disk の結果を承認しない。物理 storage、現在 snapshot と lane の照合は
必須。canonical snapshot の一致だけで raw buffer/version の鮮度は証明しない。

## Captured native submission

`submit_captured(session, allocator, snapshots, input)` と `submit_captured_with_import_discovery` は実 Native 捕捉 session を消費する。非 Native/非 SnapshotCaptured、同 Driver に保存済みの session ID、registry request 非 current は lane 変更前に `InvalidCapturedSession { session }` で拒否する。準備済み clone は Driver ごとに一度だけ submit 可能。従来 submit の ID/捕捉経路は維持する。
caller の publisher owner が実 snapshot/work unit を事前承認する。Driver は再取得、ID 推定、snapshot 登録/復活や権限付与をしない。既存 planning/prefix status、lane/request guard、cancel と resource/error 処理を適用する。
既存の借用[サービス入力](frontend_adapter.md#native-captured-sources)で Native resource を運ぶ。実 Native dispatch だけが親束縛、結果/Frontend 親検証と最終保持に published 承認を採用し、従来の四つの current/package 検証は維持する。実 source/version/root の完全な束縛は必須。後続 service と LSP current 診断は未対応。

Native 前提拒否は watch の承認済み置換を作らない。CLI は両 error を既存 InternalError envelope の `invalid_native_submission` へ写し、phase 診断や event を作らない。

raw/version 変更は canonical snapshot ID を維持したまま storage 来歴を変え得る。既存 [IR identity](../../mizar-ir/ja/identity.md) の slot 競合拒否は維持し、その成功再実行には新しい owner publisher を使う。旧 handle の物理/source/lane guard は引き続き必要。

## Current native diagnostic index

`validate_current_native_diagnostics(session, snapshots) -> Result<(), String>` が共有 live session 適格性を所有する。Driver 自身の保存 record だけを読み、protected native capture、非空かつ全て Disk/OpenBuffer（混在可）の version、Finished(Succeeded/Failed/Blocked)、既存 Driver lane/request-snapshot の Current 判定、および当該 snapshot を認可する明示 publisher を要求する。未知・非対応・cancelled/非終端・obsolete・publisher 欠落は Err。Generated/将来 origin は非対応とし、loaded version/path の妥当性は protected capture の責務に残す。
`submit_captured_with_native_diagnostic_index(session, allocator, snapshots, input)` は Disk wrapper と同じ既存の引数/戻り型と元 run 集約契約を使い、既存 submit_captured を先に実行し、Diagnostics from_batches の前後で native guard を呼ぶ。実際の同じ呼出しの phase-result batch だけを入力とし、実 scheduler run を要求する。guard/集約失敗は元 submission のまま None、通常 submit error はそのまま伝える。実 Failed/Blocked 診断や clean 空 index に後続意味・証明成功を要求しない。
record identity/filter/dedup/sort は既存 Diagnostics に残す。
実 OpenBuffer/混在 native batch、実 Frontend failure/clean 空結果、通常 submission との一致、既存集約 oracle と独立 record/source/raw-map 条件、実 NoKey をテストする。既存 Disk/共通 live guard/error テストを変更せず、native origin 除外と live OpenBuffer supersession を追加し、URI/capture/map matrix は重複しない。file 編集/削除後も capture 済み raw text/map を保ち、native SourceLoad の本文再読込や error producer を捏造しない。
検証するのは current Driver session の適格性だけである。Server buffer、最新 editor URI/version/text、source-map/IR lifetime、file/config freshness、任意 batch/index の真正性、PublishedLspSnapshot は保証しない。後続 LSP consumer が元 raw descriptor/map を現文書へ対応付け、受け渡し前に再検査する。返す index は一時点の immutable data で、scheduling/cache/proof/artifact/publication や完全 native host の達成を意味しない。

## Explicit native Disk cached SourceLoad scheduling

`CompilerDriver::submit_captured_with_native_disk_cache<A: SessionIdAllocator, L: SourceLayoutProvider>(&mut self, session: BuildSession, allocator: &A, snapshots: &SnapshotRegistry<A>, input: DriverSubmitInput<L>, policy: &mizar_proof::policy::VerifierPolicy, store: &mizar_cache::cache_store::CacheStoreRoot) -> Result<BuildSubmission, DriverSubmitError>` は明示的な source-only 最適化である。通常の captured-session 検証、planning、graph、scheduler、最後の lane/publisher guard を使う。既存 entry point と両登録 service の実 NoKey は維持する。この経路は caller の cache decision を実 SourceLoad decision で置き換え、文字列参照を typed output として受理しない。
CacheSchedulingPolicy::Enabled、dispatch provider なし、正確な組込み SourceLoad/Frontend descriptor と具体的 service 型、空でない全 Disk native capture、current な supplied publisher、明示的 native lock/config content 比較の成功時だけ lookup する。他 profile は cache decision なしで通常実行する。既存 crate-local service lookup から登録 object の Any::type_id が SourceLoadService/FrontendService と一致することを要求し、自由に名乗れる descriptor だけで実装を認証しない。既存 entry point にこの新制約は加えない。Store と immutable policy は caller が選び、内容一致と明示的関連付けは完全な active configuration、released toolchain、file/dependency freshness、global runtime provenance を証明しない。
実 graph の構築後、正確な module と一意な captured version から既存 source_input_hash を使い、dependency/parent なしの SourceLoad input を作る。同じ supplied publisher の IrCacheAdapter と既存 guarded native Disk lookup を使う。実 Rehydrated SourceUnit の current physical 検証と IR retain に成功した場合だけ、実 handle、空 diagnostics、架空の query observation なしの source-only Complete PhaseResult と、その実 identity/hash を投影した scheduler decision を作る。miss/error、未対応 binding、retain 失敗では通常 SourceLoad 実行を可能に保ち、scheduler 前の phase service 実行や Store 書込みはしない。
準備した cached result は正確な task ID の下で private に保持する。通常 graph の dependency readiness が SourceLoad skip を受理してからだけ default Frontend が消費し、新 callback/readiness policy は追加しない。最終報告に残すのは該当 task が実際に CacheHit で終了した結果だけであり、disabled/cancelled/superseded/obsolete/未受理 skip に cached PhaseResult を出さない。最後の physical/publisher/lane guard は stale result 全体を抑止し続ける。後続・未登録 service の結果や prefix から build 成功を作らない。
実 cached source を active Driver/session/task ごとに異なる opaque IR owner で scheduler consumer と最終検証まで retain し、label に semantic identity の意味を持たせない。lookup 後の正常 success/error/cancellation/supersession return 全経路で全取得 owner を release する。返却後の handle lifetime は通常結果と同じ caller 管理であり、retain は currentness を復活させず、Session lease は IR slot retain ではない。既存 dispatcher result map と local ownership tuple を使い、新 type/field/adapter/stored state/source file を作らない。
テストは実 scheduled old SourceLoad、既存 encoder、実 Store insertion、新 capture、この経路と default Frontend input を使う。resident/blob の実 Source/Frontend payload/map/diagnostics を clean と比較し、実 CacheHit task/result/parent 関連を確認する。raw storage 変更/miss、未対応 profile fallback、cancellation/error release、stale result の viable control を加え、既存の実 NoKey と非 retain 文字列 hit 拒否テストを保持する。public descriptor lookup の実組込み descriptor を既存 Registry fixture に持たせ、その descriptor を変えても借用 PhaseService の Any TypeId が TypeId::of::<FixtureService>() のままとなることを実行検証する。cache 経路の positive は実組込みを使って具体型 guard を通す。外部 fixture は private 組込み型を名指しせずその TypeId も観測しない。この fixture identity 不変性と経路 conjunction の構造検査に、直接の型間比較や架空 producer による native cache 拒否全経路の実行 credit は与えない。実 storage collection で owner release を確認し、既存 lifetime/producer matrix は重複しない。架空 record/result/hit/受理を positive oracle にしない。
完全な自動 runtime/policy/root 関連付け、Frontend cache producer、後続 service、完全な cached equivalence、artifact、proof、Steps6–8 は完了しない。native source lookup と IR 受理は既存であり、この owner は実 scheduled skip に対する Driver 関連付けと範囲内 lifetime だけを追加する。

## Package-configured native Disk cache scheduling

`CompilerDriver::submit_captured_with_configured_native_disk_cache<A: SessionIdAllocator, L: SourceLayoutProvider>(&mut self, session: BuildSession, allocator: &A, snapshots: &SnapshotRegistry<A>, input: DriverSubmitInput<L>, policy: &mizar_proof::policy::VerifierPolicy, store: &mizar_cache::cache_store::CacheStoreRoot) -> Result<BuildSubmission, DriverSubmitError>` は package 設定を選択的に使い、直前の実 SourceLoad cache 経路の全 eligibility 検査、lookup、scheduler acceptance、既定 Frontend 消費、scoped retain/release を共有する。既存の caller-selected 経路と通常 entry は不変とし、既存 optional cache tuple の private な選択で分岐する。別 loop/runtime object は作らない。
各 SourceLoad module の lookup 前に、唯一の PackagePlan と Workspace source、その package の build_config.incremental、既存 crate-local safe_relative を通る非 dot の cache_dir を要求する。直接構築された config にも Build の既存 lexical path profile を保ち、parser を再実装しない。既存 guard の crate visibility だけを共有し、本体の拡張や public path API は行わない。
Store.root() は native Path equality により Path::new(build_plan.workspace_root.as_str()).join(package_workspace_root).join(cache_dir) と一致しなければならない。canonicalize、新規 filesystem access、directory 作成はしない。package 不在/重複、registry source、無効/disabled 設定、root 不一致では当該 task の cache 準備を保留し、通常実行と global caller cache policy を維持する。importer/先頭 aggregate/registry default の設定を代用しない。filesystem alias は fallback し得るが、storage freshness の証明や path syntax/symlink policy の変更ではない。対応 store が未作成なら通常 lookup/miss fallback に進む。
実 parsed package 設定と native capture、scheduled production/encoder/Store 挿入から、実 source skip と既定 Frontend parent/result を検証する。resident/blob、default/nested directory、global disable/miss/foreign store と有効な旧経路 control、package opt-out/直接構築の無効 path、複数 package の別設定/directory、入力順/worker を扱う。clean 実行の payload/map/診断と比較し、旧 tests は不変とする。inline を優先し、新 private setup helper は実三 consumer 系統で共有する一つまでとし、他の test type/adapter は追加しない。
これは保持した metadata に対する明示的な最適化 profile である。policy/Store は caller 入力に残り、完全 active configuration、選択 toolchain/dependency、current files、自動 host/runtime、Frontend cache expectation、後続 service/artifact と Steps6–8 は別途とする。

## Persistent native Disk SourceLoad cache

`CompilerDriver::submit_captured_with_persistent_native_disk_cache<A: SessionIdAllocator, L: SourceLayoutProvider>(&mut self, session: BuildSession, allocator: &A, snapshots: &SnapshotRegistry<A>, input: DriverSubmitInput<L>, policy: &mizar_proof::policy::VerifierPolicy, store: &mizar_cache::cache_store::CacheStoreRoot) -> Result<BuildSubmission, DriverSubmitError>` は configured lookup と best-effort 保存を明示的に選ぶ。既存 private cache tuple に write flag だけを加え、旧二 method は false、新 entry は configured=true/write=true とする。通常 method は不変とし、共通 configured eligibility と唯一の version/input 構築後、lookup 前に local な (実 TaskId, clone した PhaseInput) 候補を集める。独立 guard/scheduler/source 実行や保持 runtime state は作らない。
scheduler と既存の全 physical/publisher/lane result filtering 後、最終 task state に Cancelled が一つでもあれば保存を全て保留する。各候補は当該 source task が Completed、唯一の Complete result と唯一の output を必要とする。CacheHit/blocked/skipped/cancelled は cold producer ではない。下流 Frontend failure と有効な完了 SourceLoad は両立し、当該 source record の保存を妨げない。後続 service 不在は build 未完了のままである。
同じ実 capture/plan/index/allocator/publisher を SourceLoadInputs と IrCacheAdapter に渡し、実 typed SourceUnit を回収して encode_native_disk_cache_record を呼ぶ。既存の完全 native binding/current physical output/storage/side-table 検証を維持し、Encoded(record) だけを Store.insert へ渡す。各挿入の直前、encode 後に、新しい lane/request/snapshot currentness、publisher current-output authorization、physical handle validity を要求する。不成立なら後続保存を止め、encode 失敗/skipped では当該候補だけを省く。snapshot registration/grant、新しい key/codec/hash/schema、Frontend cache は導入しない。
Store 挿入は内部最適化の best-effort とし、error/rejected outcome で build を失敗させたり、診断/submission/status/policy/proof result を変更したりせず、成功を捏造しない。Store の complete-record validation、同一内容の再挿入、divergence/atomic write を維持する。currentness は handoff 時点の検査で、Store I/O と lane/publisher は atomic ではなく、既に完了した record は rollback しない。既存 SourceLoad の complete な zero-artifact-dependency footprint により artifact emission 前の内部保存が可能だが、artifact/witness publication ではない。
Store 不在から test-side insert なしで、実 cold 実行と native key による record 保存、fresh capture/local ID と実 warm skip/既定 Frontend を検証する。独立 clean の output/map/side table/診断と release、resident/blob、source/raw edit、Frontend failure prefix を比較する。cold の disabled/非対応 root/provider/context、取消と completed-obsolete の実 producer、currentness 抑止、Store 作成を実 file が妨げる error を有効 control と区別する。既存の広い共通経路 matrix を再利用し、旧 tests は不変。新 private setup helper は実三 family call site で共有する一つまでとし、他の test type/helper/adapter は追加しない。
別の明示的 opt-in として旧二 read-only 契約と absent-store 動作を保つ。完全 runtime identity/policy/toolchain/dependency provenance、Frontend 独立 expectation/lookup、後続 service、完全 cached equivalence、artifact/host/ENTRY と Steps6–8 は別途とする。

## Persistent native leaf Frontend cache

`CompilerDriver::submit_captured_with_persistent_native_prefix_cache` は既存 source-only persistence と同じ引数・戻り値で、設定済み SourceLoad と実行後の Frontend の保存を明示的に有効にする。既存 entry の Frontend write flag は false のまま、共通 configured guard、pending source task/input、scheduler、最終 physical/publisher/lane filtering を共有する。Frontend の事前 lookup や追加実行は行わない。
最終 Cancelled task があれば全保存を抑止する。pending source は Completed または CacheHit で、Complete result と実出力が各一つ必要。同じ work unit、単独 Frontend phase の task が一つ存在し、graph dependency は当該 source task と実際の Workspace/PackageResolve root（phase PackageResolve）の正確に二つを要求する。root と Frontend は Completed、Frontend の診断なし Complete result と出力は各一つとし、他 dependency は拒否する。pending input identity と実際の sealed published source parent から PhaseInput を構成し、他 task・文字列・oracle・obsolete 出力で代用しない。
同じ publisher から実 typed FrontendOutput を取得し、実 capture/plan/index/allocator と変更しない encode_native_disk_frontend_cache_record に渡す。current physical parent/output、lineage、raw source/maps、leaf/no-import/dependency/diagnostic、完全 payload の既存検証が権威を持つ。Encoded のみ Store.insert に渡し、直前に lane/request/snapshot と parent/output の current authorization・physical validity を再確認する。guard 喪失は以降の保存を停止し、encoding 不成立は候補を飛ばす。
挿入失敗・拒否は status、診断、proof 結果を変えない。内部 zero-artifact-dependency record は artifact 受理・公開を示さず、currentness と Store I/O は atomic でなく、先行保存も rollback しない。source-only entry と既存 read-only entry の保存範囲は維持する。
テストは空 Store から Driver の実保存を観測し、独立した current key と same-capture retained lookup を使う。positive の test-side insert は禁止。resident/blob、maps、side tables、parent、診断、実 NoKey を clean run と比較し、実 SourceLoad hit parent、source/raw 編集、package/order/workers を含める。disabled/ineligible/provider/context、cancelled/completed-obsolete、failed/imported Frontend、実 Store error は通常動作を維持する。旧テスト prefix を保ち、新規 setup helper は最大一つ・実三 family 共用とする。
cold Frontend expectations、retained scheduler skip、完全 runtime/toolchain/dependency identity、imports/Open cache、後続 phase、artifact/host/ENTRY、Steps6–8 完了は対象外。

## Retained native Frontend cache scheduling

`CompilerDriver::submit_captured_with_retained_native_frontend_cache<A: SessionIdAllocator, L: SourceLayoutProvider>(&mut self, session: BuildSession, allocator: &A, snapshots: &SnapshotRegistry<A>, input: DriverSubmitInput<L>, cache: (&mizar_proof::policy::VerifierPolicy, &mizar_cache::cache_store::CacheStoreRoot), oracle: (&crate::registry::PhaseInput, &mizar_ir::cache_adapter::IrCacheAdapter, &crate::registry::PhaseResult)) -> Result<BuildSubmission, DriverSubmitError>` は、同じ保護されたnative Disk captureと現在のSourceIdに対し、呼出側が供給する実際の保持済みFrontend oracleを明示的に対応付ける。policy/Storeは既存tupleへまとめ、既存private submission-options tupleのdiscover_imports/preparedに借用result oracleのOptionを追加し、旧入口では常にNoneとする。既存cache tupleの五要素は維持する。呼出側選択の既存read-only SourceLoad cache経路を用い、configured rootや永続化は変更しない。通常入口、登録済みNoKeyとquery observationは維持する。
既存native cache適格性、実組込みservice型、lock/config本文比較、SourceLoad lookupとscheduler設定を再利用する。実SourceLoad Rehydrated出力の物理/current検証とretain成功後だけFrontend lookupを試す。oracle work unitの実SourceLoad task、Workspace/PackageResolve/[PackageResolve] root、同unitの[Frontend] taskが各一つであり、Frontend依存がそのsource taskとrootの二つだけであることを要求する。曖昧性や他の依存形は対象外。source missや不適格時はFrontend準備を行わず通常実行とする。rootやphaseをschedulerより先に実行しない。
そのsource taskの実際の一意な捕捉version/source hashと、自身のseal済みpublished source handleからtarget Frontend PhaseInputを作り、供給された実PhaseResultがComplete、diagnosticなし、出力が正確に一件であることを要求する。oracle adapter storageでそのFrontend typed handleを取得し、既存rehydrate_native_disk_frontend_cacheへhandleとtarget adapterを渡す。phase型はFrontend adapter所有のままとし、Driverは既存PhaseResult protocolを消費し、既存永続化と同じく呼出先からhandle型を推論する。alias/reexport、lint抑制、core authority guardの緩和は行わない。capture/SourceId/dispatch identity/同一source-parent ID、recordに依存しないoracle全bytes/keys/side tables、currentな物理publisher、producer/decoder/leaf guardは変更しない。一般的published handleだけでは実executorを証明できず、実producerとoracleの並行寿命は呼出側の責任。同一Frontend出力を未格納の別storageで復元でき、既存duplicate-output失敗は準備を見送るだけとし、dedupやpermission/publicationの回避策を作らない。
物理/current検証と明示retainに成功した実Frontend Rehydrated handleだけが、実erased handle、空diagnostics、cache_observation=NoneのComplete private PhaseResultと、実identity/hashを投影したscheduler decisionを正確なFrontend TaskIdへ設定できる。miss/error/retain失敗時は通常実行を残し、呼出側の文字列decisionからhandleを受理しない。両payloadは既存local ownership tuple内の実erased handleで扱い、新type/field/adapterは作らない。各skipは通常のgraph readiness、開始前取消、publication freshnessを独立に通過する必要があり、準備からroot完了やsource受理を推定しない。
準備済み結果を報告へ残すのは、その正確なtaskが最終的にCacheHitとなった場合だけ。既存の全結果mapに対する物理/publisher/lane guardでobsolete結果を抑止する。target source/Frontendを個別のactive Driver/session/task ownerでconsumerと最終検証まで保持し、scheduler errorと通常return（取消・supersessionを含む）で全ownerをreleaseする。retainはcurrentnessを与えず、return後の寿命は通常どおり呼出側管理。旧read-only経路とこの入口はStoreへ書かない。
既存の実Fixture/capture_sources/組込みproducer/encoder/Storeと独立clean実行を用いるretained_native_frontend_cache_*三familyを追記し、新private setup helperは最大一つで全三familyから使う。resident/blobの全aggregate、map、全side/parent/diagnostics、実CacheHitだけのsource/Frontend TaskStarted不在、捕捉入力不変性、再実行・入力順・worker数を確認する。有効positiveを併設してsource/Frontend miss、foreign/stale/回収済みoracle、binding/context/provider/disableと依存形のfallback、実際の資源不足・取消・obsolete時のreleaseを実collectionで確認し、下位matrixとNoKey testを再利用する。既存scheduler-Err release loopは静的にレビューする。この有効graphの公開経路からは、その分岐に達する不正outcome/decisionやroot欠如を供給できない。oracle PhaseResultは実scheduled結果から借用し、positiveなComplete結果を再構築しない。不正status/diagnostic/出力件数のcontrolはそのadversarial derivativeで作る。実dataの改変はnegative controlに限る。
これは同一captureの明示保持スケジュール連携であり、coldな独立Frontend期待値、fresh capture/process再利用、atomic runtime寿命、完全active configuration/toolchain/dependency真正性、import/Open cache、semantic/proof/artifact service、STEP6C全体やSteps6–8の完了を与えない。oracle対応と既存leaf footprintの範囲で、これらを引き続き保留する。
