# ソース／仕様対応監査

> 正本は英語です。英語版:
> [../en/source_spec_correspondence.md](../en/source_spec_correspondence.md)。

状態: task D-018 で完了。

## 範囲

この監査は、実装済みの `mizar-driver` source を英語正本の module spec に照合する:

- [request.md](request.md)
- [registry.md](registry.md)
- [driver.md](driver.md)
- [events.md](events.md)
- [cli.md](cli.md)

public API と約束された挙動を Rust source と test に trace する。これは軽量な
source/spec/test 対応表であり、実行可能 test の代替ではない。欠落実装、古い仕様、
欠落 test が見つかった場合は、drift を黙認せず、分類済み follow-up として記録する。
Medium finding は修正するか、理由付きで明示的に deferred としなければならず、generic
follow-up note として扱わない。

## 結果

- 未解決の blocking、high、medium source/spec drift は見つからなかった。
- `src/request.rs`、`src/registry.rs`、`src/driver.rs`、`src/events.rs`、
  `src/cli.rs` が公開する現在の public API は、module spec または task D-017 の
  public enum policy section で cover されている。
- 現在の public enum はすべて `#[non_exhaustive]` であり、exhaustive exception は
  記録していない。
- 実装と test は driver ownership boundary を保っている。orchestration、
  request/session lifecycle、registry/query boundary、scheduler submission、
  protocol-agnostic event、CLI batch entry point、watch orchestration は実装済みである。
  phase semantics、proof / cache / artifact authority、diagnostics identity、LSP
  protocol conversion は driver の外に残る。
- 既存 owner gap は、この監査で修復せず意図的に分類済みのままにする。semantic /
  proof / artifact phase adapter は `external_dependency_gap`、documentation extraction は
  `deferred`、producer / cache / artifact / proof seam を持つ real clean / incremental /
  parallel equivalence は deferred、欠落している `mizar-artifact` closeout report は
  report-only `repo_metadata_conflict` のままである。

## 公開 API 対応

| 仕様 | 確認した public API | Source | Test evidence |
|---|---|---|---|
| [request.md](request.md) | request / session envelope: `BuildRequestDraft`, `BuildRequest`, `PendingBuildRequest`, `CaptureSnapshotError`, `CapturedSnapshot`, `BuildSession`, `DriverLanes`, `LaneCurrentSession`, `BuildLaneId`, `BuildRequestGeneration`, `BuildProfile`, `BuildTargets`, `SourceInputSet`, `DependencyInputSet`, `VerifierConfigInput`, `BatchRequest`, `BatchInvocation`, `WatchRequest`, `LspRequest`, `LspFocus`, `ObsoletePublication`; enum `BuildRequestOrigin`, `LspPriority`, `BuildSessionState`, `BuildSessionOutcome`, `PublicationDecision`; id allocation、snapshot input projection、snapshot capture、lifecycle transition、lane currentness、publication decision の method。 | `crates/mizar-driver/src/request.rs` | `tests/request.rs` が batch/watch/LSP request shape、`mizar-session` 経由の snapshot capture、同一 snapshot input、lane/generation currentness、same-snapshot supersession rejection、publication suppression、capture error、lifecycle/cancellation idempotence を cover する。`tests/watch.rs`、`tests/driver.rs`、`tests/events.rs`、`tests/determinism.rs` も driver submit、watch supersession、event publication、stale-output rejection 経由で同じ境界を確認する。 |
| [registry.md](registry.md) | phase registry と query boundary: `PhaseDescriptor`, `PhaseRequirement`, `PhaseInput`, `PhaseInputIdentities`, `PhaseContext`, `PhaseCacheContext`, `PhaseExecutionContext`, `PhaseExecutionResources`, `PhaseResult`, `PhaseCacheObservation`, `PhaseCacheQueryResult`, `PhaseExecutionQueryResult`, `PhaseRegistryBuilder`, `PhaseRegistry`, `DriverQueryBoundary`, `DriverQueryDatabase`, `PhaseService`, `required_phase_services`; enum `PhaseOwner`, `PhaseServiceAvailability`, `PhaseStatus`, `PhaseCacheIntent`, `PhaseRegistryError`。 | `crates/mizar-driver/src/registry.rs` | `tests/registry.rs` が phase service table、deterministic registration、duplicate rejection、descriptor normalization、missing owner seam、cache-key purity、dependency/parent query identity、salsa/query-boundary execution、semantic/proof/cache/artifact/LSP authority を持たない source guard を cover する。`tests/lint_policy.rs` は phase owner crate が driver または salsa dependency を得ないことを確認する。 |
| [driver.md](driver.md) | driver front door と watch orchestration: `BuildSubmission`, `DriverSubmitInput`, `CompilerDriver`, `WatchSubmission`, `WatchSubmitControl`, `WatchSupersededSession`, `WatchSnapshotReplacement`, `WatchModeGap`, `WatchSubmitFailure`, `DriverSchedulerRun`, `DriverMissingPhaseService`, `DriverCancelOutcome`; enum `WatchSnapshotReplacementStatus`, `WatchModeGapOwner`, `WatchOwnerSeam`, `WatchSubmitError`, `DriverSubmissionStatus`, `DriverCancelReason`, `DriverSubmitError`; `CompilerDriver::{new, registry, session, cancellation_policy, submit_watch_change, submit, cancel, events}`。 | `crates/mizar-driver/src/driver.rs` | `tests/driver.rs` が phase-0 bootstrap、`mizar-build` planner / index / task graph の利用、scheduler submission / result consumption、missing phase-service gap、synthetic output なしの dispatch-gap blocking、stale same-lane suppression、failed module-index session storage、cancellation、non-owner authority を持たない source guard を cover する。`tests/watch.rs` は watch replacement、superseded replay、missing watcher/LSP/publisher seam、publisher replacement failure、previous-session validation を cover する。 |
| [events.md](events.md) | event stream boundary: `BuildEventStream`, `BuildEvent`, `BuildEventIdentity`, `BuildEventOrderKey`, `TaskEventRef`, `OwnerRecordRef`, `BuildEventLog`, `BuildEventIdentityKey`, `diagnostics_gap_event`; enum `BuildEventKind`, `PlanningEventStatus`, `EventOwner`, `OwnerGapClassification`, `BuildEventError`。 | `crates/mizar-driver/src/events.rs` | `tests/events.rs` が deterministic event sorting、stream/session/snapshot validation、publication suppression、stale diagnostics/artifact rejection、failure isolation、gap event、task-progress non-artifact behavior、owner ref、replay order、diagnostics/artifact/scheduler/LSP authority を持たない source guard を cover する。`tests/watch.rs` と `tests/determinism.rs` は real driver session からの suppressed replay を cover する。 |
| [cli.md](cli.md) | batch CLI surface: `CliInvocation`, `CliSnapshotInputs`, `CliBatchInput`, `CliOutput`, `CliUsageError`; enum `CliCommand`, `CliBuildProfile`, `CliMessageFormat`, `CliExitCode`; `CliInvocation::{parse, request_draft}`, `CliSnapshotInputs::new`, `CliBatchInput::new`, `CliOutput::process_code`, `CliBuildProfile::as_str`, `CliExitCode::process_code`, `run_batch`, `run_batch_with_driver`, `run_invocation_with_driver`。 | `crates/mizar-driver/src/cli.rs` | `tests/cli.rs` が argument parsing、request/profile/target/scheduler control、success/progress rendering、JSON event output、manifest/module-index diagnostics owner gap、usage/internal error、owner-unavailable path、source snapshot/layout guard、cancellation mapping、quiet output、LSP/artifact/proof/cache/phase-semantics authority を持たない source guard を cover する。`tests/determinism.rs` は human/JSON CLI output と exit code の repeated-run / worker-count byte stability を cover する。 |
| [todo.md](todo.md) task D-017 | すべての public driver enum に対する public enum compatibility policy。 | `crates/mizar-driver/src/*.rs`, `crates/mizar-driver/tests/lint_policy.rs` | `tests/lint_policy.rs::public_driver_enums_are_forward_compatible` は crate target files を scan し、隣接する `#[non_exhaustive]` を持たない public enum を失敗させる。D-017 の module-spec section は exhaustive exception がないことを記録する。 |

## 公開メソッド surface 対応

上の型単位の表を、public constructor、accessor、query entry point、free
function の粒度へ展開する。private helper と private trait adapter は、この public
API 監査の対象外である。

- `request.rs`: `BuildLaneId::{new, get}`,
  `BuildRequestGeneration::{new, get}`, `BuildProfile::new`,
  `DependencyInputSet::new`, `VerifierConfigInput::new`,
  `BuildRequestDraft::allocate`, `BuildRequest::snapshot_input`,
  `PendingBuildRequest::capture_snapshot`,
  `BuildSession::{lane_current_session, mark_submitted, mark_running, cancel,
  finish, is_terminal}`,
  `DriverLanes::{mark_current, current, is_current_session,
  is_session_current, publication_decision}` は、request spec の snapshot
  boundary、lane/generation currentness、lifecycle、publication decision rule を
  `tests/request.rs` に対応させる。`tests/driver.rs`、`tests/watch.rs`、
  `tests/events.rs`、`tests/determinism.rs` も downstream 経路で同じ surface を確認する。
- `registry.rs`: `PhaseDescriptor::new`, `PhaseInput::new`,
  `PhaseInputIdentities::new`, `PhaseResult::complete`,
  `PhaseRegistryBuilder::{new, register, register_arc, build}`,
  `PhaseRegistry::{empty, descriptors, query_boundary, descriptor_for_phase,
  cache_key_for_phase, execute_phase, execute_phase_with_resources}`,
  `DriverQueryBoundary::{cache_key, execute}`,
  `PhaseService::{phase, cache_key, execute}`, `required_phase_services` は、
  registry spec の deterministic registration、duplicate rejection、
  missing-service classification、pure cache-key projection、driver-owned salsa query
  boundary を `tests/registry.rs` と `tests/lint_policy.rs` の owner-boundary guard に
  対応させる。
- `driver.rs`: `CompilerDriver::{new, registry, session,
  cancellation_policy, submit_watch_change, submit, cancel, events}` と
  `DriverSubmitInput::new` は、driver front door、scheduler submission、
  cancellation、session storage、event replay、watch orchestration rule を
  `tests/driver.rs`、`tests/watch.rs`、`tests/events.rs`、`tests/determinism.rs` に
  対応させる。
- `events.rs`: `BuildEventStream::{empty, from_events, events, replay}`,
  `BuildEvent::new`, `BuildEventOrderKey::{new, with_scheduler_order,
  with_phase, with_work_unit, with_owner_order}`, `TaskEventRef::new`,
  `OwnerRecordRef::new`, `BuildEventLog::{new, push, extend, into_stream}`,
  `diagnostics_gap_event` は、event spec の identity validation、deterministic order、
  replay、stale-publication rejection、owner-record reference、diagnostics-owner-gap
  event construction を `tests/events.rs` に対応させる。watch と determinism の replay
  coverage は `tests/watch.rs` と `tests/determinism.rs` が担う。
- `cli.rs`: `CliInvocation::{parse, request_draft}`,
  `CliSnapshotInputs::new`, `CliBatchInput::new`, `CliOutput::process_code`,
  `CliBuildProfile::as_str`, `CliExitCode::process_code`, `run_batch`,
  `run_batch_with_driver`, `run_invocation_with_driver` は、CLI spec の batch
  parsing、request construction、driver invocation、deterministic output、exit-code
  contract を `tests/cli.rs` と `tests/determinism.rs` に対応させる。

## 約束された挙動の対応

| 約束された挙動 | Source status | Test evidence |
|---|---|---|
| Driver は orchestration を所有し、phase semantics、proof acceptance、cache compatibility、artifact serialization、diagnostics identity、LSP protocol conversion は所有しない。 | ownership boundary は各 module に文書化し、source では narrow API と forbidden authority term の不在で守る。 | `tests/driver.rs`、`tests/registry.rs`、`tests/events.rs`、`tests/cli.rs`、`tests/watch.rs`、`tests/lint_policy.rs` の source guard。 |
| Build request / session は 1 つの current source/dependency snapshot を capture し、obsolete watch/LSP publication を拒否する。 | `request.rs` は lane/generation metadata を保持し、`mizar-session` 経由で snapshot を capture し、current lane を追跡し、`PublicationDecision` を計算する。 | `tests/request.rs`、`tests/watch.rs`、`tests/events.rs`、`tests/determinism.rs`。 |
| Phase registry registration は deterministic、duplicate coverage は拒否、cache-key/query-boundary input は pure。 | `registry.rs` は descriptor を normalize し、registration を sort し、requirement を記録し、driver salsa query boundary を所有し、missing owner seam を報告する。 | `tests/registry.rs`、`tests/lint_policy.rs`。 |
| Driver は `mizar-build` の planning、task graph、scheduler、cancellation authority を消費し、scheduler semantics を複製しない。 | `driver.rs` は `mizar-build` planner、module index、task graph builder、scheduler を呼ぶ。non-phase-zero work は real dispatch がない間 scheduler 前に block される。 | `tests/driver.rs`、`tests/determinism.rs`、D-016 で実行した `cargo test -p mizar-build` coverage。 |
| Event stream は protocol-agnostic、deterministic order、suppressed session の current diagnostics/artifact payload を拒否する。 | `events.rs` は event identity/publication を validate し、stable order key で sort する。 | `tests/events.rs`、`tests/watch.rs`、`tests/determinism.rs`。 |
| CLI は command input を driver request に map し、event stream から progress を render し、fake record なしで diagnostics owner gap を報告し、exit code を deterministic に map する。 | `cli.rs` は current batch command を parse し、`BuildRequestDraft` / `DriverSubmitInput` を構築し、`CompilerDriver::submit` を呼び、human/JSON progress を render する。 | `tests/cli.rs`、`tests/determinism.rs`。 |
| Watch mode は owner-provided changed path / snapshot input と任意の real `mizar-ir` snapshot replacement を消費し、missing watcher/LSP/publisher seam を分類する。 | `driver.rs::submit_watch_change` は previous session を validate し、submit に委譲し、replay を supersede し、任意で real `PhaseOutputPublisher` を呼び、watch gap を記録する。 | `tests/watch.rs`、`tests/determinism.rs`。 |
| End-to-end determinism は実装済み seam で cover し、full real cache/producer/artifact/proof equivalence は deferred のままにする。 | D-016 は crate-local deterministic projection を追加し、full-system deferral を文書化した。 | `tests/determinism.rs`; `todo.md` と `00.crate_plan.md` の D-016 記録。 |

## Gap と follow-up 記録

この監査では新しい blocking/high/medium source/spec drift は見つからなかった。

既存の分類済み follow-up は残る:

- `DRIVER-G-001`: 欠落している `mizar-artifact` closeout report は report-only
  `repo_metadata_conflict` のまま。
- `DRIVER-G-013`: semantic / proof / artifact phase adapter は、owner crate が
  driver-callable input、canonical `mizar-ir` producer output、diagnostics bridge、
  non-driver proof / cache / artifact / LSP authority contract を公開するまで
  `external_dependency_gap` のまま。
- `DRIVER-G-014`: `DocExtractionService` は documentation / extraction owner crate と
  service surface が存在するまで `deferred` のまま。
- real cache hit、producer output、artifact commit、proof reuse、multi-task driver phase
  dispatch を含む full clean / incremental / parallel equivalence は、それらの owner seam が
  存在するまで deferred のまま。
- D-019 は bilingual documentation sync audit を完了した。この D-018 audit は
  source/spec correspondence を確認したものであり、翻訳品質全体の監査ではない。

## 検証

この監査の関連検証は documentation review と、trace した挙動を固定する既存 crate test
である。D-018 自体は、後続 review が blocking/high drift、または source で修正すべき
medium finding を見つけない限り design document だけを変更する。

D-018 change は docs-only であるため、必要な local check は `git diff --check` と、
stage 後の `git diff --cached --check` である。後続 review が Rust source change を
要求する場合は、Rust verification path として `cargo fmt --check`、
`cargo clippy -p mizar-driver --all-targets -- -D warnings`、
`cargo test -p mizar-driver` を実行する。
