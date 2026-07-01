# mizar-driver ビルドイベント

> 正本は英語です。英語版: [../en/events.md](../en/events.md)。

## 目的

`mizar-driver` は build session 用の protocol-agnostic な build event stream を所有する。
この stream により、batch、watch、LSP 向け caller は進捗、readiness、blocking gap、
終端状態を観測できる。ただし driver が phase semantics、diagnostic identity、artifact
publication、LSP protocol conversion の権限を持つわけではない。

この仕様は task D-010 用の event contract を定義する。task D-009 は documentation-only であり、
Rust source module は追加しない。

## 所有権境界

event stream は次を報告してよい:

- `BuildSessionId` に対する session lifecycle change;
- snapshot capture と request freshness decision;
- `mizar-build` の task graph / scheduler identity を使った task / phase progress;
- registry が分類した phase-service availability gap;
- DRIVER-G-011 のような scheduler-to-registry dispatch gap;
- owner seam が提供する場合の `mizar-diagnostics` owner record または batch への参照による
  diagnostics readiness;
- artifact owner が real committed result を報告した場合に限る artifact-boundary readiness;
- session completion、cancellation、supersession、stale-publication suppression。

event stream は次をしてはならない:

- diagnostic code や diagnostic identity を allocate する;
- diagnostic record を deduplicate したり CLI diagnostic text を render したりする;
- LSP JSON-RPC payload、range、severity、code action、document version を emit する;
- artifact publication token を mint したり、artifact を serialize したり、scheduler completion を
  artifact publication と扱ったりする;
- `SyntheticOutputRef`、`SchedulerResult.output_refs`、fake phase output handle を公開する;
- cache compatibility、proof acceptance、trusted status、kernel acceptance を決定する;
- artifact、diagnostic、phase result を worker completion order で並べ替える。

## 公開 enum の互換性

この module の public enum はすべて downstream-facing な event boundary type であり、
`#[non_exhaustive]` を付ける。D-017 では以下について exhaustive exception を記録しない:

- `BuildEventKind`;
- `PlanningEventStatus`;
- `EventOwner`;
- `OwnerGapClassification`;
- `BuildEventError`。

Downstream crate はこれらの enum を match するとき wildcard arm を持たなければならない。
将来の event kind、owner、gap classification、validation error は、driver を diagnostics
identity、artifact publication、proof / cache decision、LSP protocol conversion の owner に
せずに追加できる。

## イベント形状

具体的な Rust 名は task D-010 で変わり得るが、すべての build event は次の共通 identity を
持たなければならない:

- `session`: event を所有する `BuildSessionId`;
- `lane` と `generation`: accepted request の driver lane / generation;
- `snapshot`: snapshot-scoped result を参照する場合の captured `BuildSnapshotId`;
- `publication`: current result として見える可能性がある event に対する request-layer
  `PublicationDecision`。

event kind は次のグループに分かれる:

| 種別 | 意味 | 権限 |
|---|---|---|
| `SessionAccepted` | request id / session id が allocate され、snapshot capture が始まる | `mizar-driver` + `mizar-session` |
| `SnapshotCaptured` | immutable snapshot lease が session 用に capture された | `mizar-session` |
| `PlanningReady` | phase-0 plan / index / task graph data、または structured planning / index / graph error が存在する | `mizar-build` |
| `TaskProgress` | scheduler / task state が変化した、または task が blocked / cancelled になった | `mizar-build` |
| `PhaseServiceGap` | required phase owner seam が missing / deferred / unavailable である | `PhaseRegistry` |
| `DispatchGap` | scheduler-to-registry owner seam が未利用のため task graph の work を dispatch できない | DRIVER-G-011 に対する `mizar-driver` の分類 |
| `OwnerReadinessGap` | readiness event に必要な diagnostics、artifact、producer、bridge owner seam が missing / deferred / unavailable である | owner crate closeout / gap classification |
| `PhaseReady` | real phase owner が completed / recoverable / blocking / fatal / cancelled phase result を報告した | registry 経由の phase owner |
| `DiagnosticsReady` | diagnostics owner が consumer 用 record または batch を用意した | `mizar-diagnostics` |
| `ArtifactBoundary` | artifact owner が actual committed artifact / projection handoff を報告した | artifact owner |
| `PublicationSuppressed` | obsolete session の result が current になれない | request publication guard |
| `SessionFinished` | session が terminal outcome に到達した | `mizar-driver` session lifecycle |

`PhaseReady`、`DiagnosticsReady`、`ArtifactBoundary` は readiness signal であり、ownership
transfer ではない。payload は owner-provided record、index、committed result を参照しなければ
ならない。該当 owner seam がない場合、driver は placeholder payload ではなく classified gap
event を emit する。`PhaseServiceGap` は missing phase adapter 用、`DispatchGap` は
scheduler-to-registry dispatch seam 用、`OwnerReadinessGap` は missing diagnostics、artifact、
producer-output、protocol bridge seam 用である。

internal architecture sketch には `SnapshotPublished(BuildSnapshotId)` event が含まれる。
`mizar-driver` では、positive snapshot publication は combined request guard 通過後に
`publication` field が `Current` である event として表現される。editor snapshot publication と
protocol conversion は LSP bridge の所有であり、driver event stream の所有ではない。D-010 の
implementation が named `SnapshotPublished` event を公開する場合、それは独立した freshness
authority ではなく、protocol-agnostic な `Current` publication decision の alias でなければならない。

## Freshness と Suppression

event を current diagnostics、current build status、current artifact-boundary handoff、
latest watch/LSP state として公開する前に、driver は [request.md](request.md) の combined
request publication guard を呼ばなければならない。

guard が `Current` を返す場合、event は current consumer に見えてよい。guard が `Suppressed`
を返す場合、event stream は obsolete session の subscriber 向けに protocol-agnostic な
`PublicationSuppressed` event を emit してよい。ただし obsolete diagnostics、phase output、
cache decision、artifact record を current として relabel してはならない。

batch session は通常 supersede されないが、同じ guard を使う。watch / LSP session は、newer
generation が accepted された後に older generation が current result を publish しないよう、
snapshot currentness に加えて lane / generation currentness を使わなければならない。

## 決定的順序

event ordering は決定的であり、worker completion order に依存しない。task D-010 は次の構成要素を
この順で持つ stable ordering key を定義しなければならない:

1. session-local lifecycle rank;
2. event が task を参照する場合の `mizar-build` 由来の canonical scheduler / task order;
3. `mizar-build::task_graph::PipelinePhase` の pipeline phase order;
4. package、module、VC descriptor、backend attempt、evidence candidate などの canonical
   work-unit identity;
5. event が owner record を参照する場合の owner-provided diagnostic / artifact order;
6. stable event-kind tie-breaker。

ordering は scheduler state を反映する progress event を含んでよいが、diagnostic readiness と
artifact-boundary readiness は canonical owner order で collate しなければならない。event は、
最初に終了した worker が semantic winner、artifact order、diagnostic order であると示唆しては
ならない。

terminal な `SessionFinished` は、publication または suppression が accepted されたすべての
in-session event の後に現れる。cancellation と supersession event も同じ ordering key を使い、
repeated delivery でも deterministic でなければならない。

## Diagnostics イベント

`DiagnosticsReady` event は `mizar-diagnostics` の record、index、batch、または将来の
owner-provided handle を参照する。diagnostics owner が提供する count、package / module identity、
severity / category summary、freshness information を含んでよい。

rendered message text を identity として使ったり、diagnostic id を allocate したり、record を
deduplicate したり、CLI / LSP presentation へ変換したりしてはならない。CLI rendering は
`mizar-diagnostics` を通じた CLI entry point の責務である。LSP diagnostic conversion は LSP
bridge の責務である。

planning、module-index、task-graph、scheduler、driver lifecycle error は、structured owner /
driver record として表現された後にだけ diagnostics-readiness event を生成してよい。diagnostics
bridge が存在するまで、D-010 は diagnostic record を発明する代わりに classified readiness / gap
event を emit してよい。

## Artifact イベント

`ArtifactBoundary` event は、artifact owner が real committed result または projection output を
報告した後に限って許される。event は owner-provided artifact identity、content hash、package /
module identity、manifest transaction status を参照してよい。

driver は次から `ArtifactBoundary` event を生成してはならない:

- `TaskState::Completed` だけ;
- scheduler synthetic output;
- artifact / proof / kernel owner acceptance のない retained IR handle;
- driver が発明した provisional publication token。

artifact owner seam が利用できない間、task D-010 は fake commit event ではなく
`external_dependency_gap` または `deferred` event として artifact-boundary readiness を報告する。

## Consumer ルール

CLI consumer は event stream を購読して progress を render し、batch exit path を選んでよい。
CLI は diagnostics rendering に `mizar-diagnostics` を使わなければならず、event text を diagnostic
identity と扱ってはならない。

LSP / watch consumer は同じ protocol-agnostic stream を購読してよいが、LSP bridge は JSON-RPC id、
document URI、range、diagnostic severity、code action、progress token、editor snapshot publication
の責務を持ち続ける。bridge は `PublicationSuppressed` event を stale-session notification として
ignore または translate しなければならず、current diagnostics として扱ってはならない。

consumer が reconnect するか遅れて subscribe した場合、retained session event の deterministic
replay を受け取ってよい。replay は live delivery と同じ ordering と freshness decision を保持しなければ
ならない。

## テスト要件

task D-010 のテストは次を cover しなければならない:

- scheduler / worker completion が shuffle されても event ordering が同一である;
- すべての event が既知の `BuildSessionId` を参照し、該当する場合は captured `BuildSnapshotId`
  も参照する;
- stale watch / LSP generation は current diagnostics / artifact event ではなく suppression を
  emit する;
- missing phase service と missing artifact publication seam は placeholder phase output や
  publication token ではなく classified gap event を生成する;
- DRIVER-G-011 dispatch-gap blocking は classified `DispatchGap` event を生成し、non-phase-0
  synthetic scheduler output を current progress として submit しない;
- missing diagnostics、artifact、producer-output、bridge readiness seam は fake readiness payload
  ではなく `OwnerReadinessGap` event を生成する;
- scheduler `TaskState::Completed`、scheduler synthetic output、artifact / proof / kernel
  owner acceptance のない retained IR handle は `ArtifactBoundary` event を生成しない;
- diagnostics readiness は structured owner identity を持ち、rendered message text を identity として
  使わない;
- driver-owned event は diagnostic id / code を allocate せず、diagnostic record を deduplicate しない;
- CLI / LSP protocol term は driver-owned event payload に現れない;
- event replay は deterministic ordering と publication decision を保持する。
