# Module: request

> 正本は英語です。英語版:
> [../en/request.md](../en/request.md)。

## 目的

この module は driver-owned な build request と build session の境界を定義する。

`mizar-driver` は batch、watch、LSP 由来 build request を受け取り、submit された
各 session について検証済みの `mizar-session` snapshot を 1 つ捕捉し、置き換え
られた watch/LSP session が current として公開されることを防ぐ current-session
publication guard を公開する。この module は source loading semantics、phase
semantics、diagnostic identity、artifact serialization、LSP protocol conversion を
所有しない。

## 所有境界

`request` が所有するもの:

- batch、watch、LSP 由来 build の request 形状;
- `mizar-session::BuildRequestId` を再定義せず、watch/LSP generation の置き換えで
  session を更新する driver-level currentness lane;
- 1 つの `BuildSessionId`、1 つの request lane、1 つの `BuildSnapshot`、
  `SnapshotRegistry::create_snapshot` が返す active snapshot lease を結び付ける
  `BuildSession` record;
- scheduler submission、cancellation、event publication gate が使う lifecycle
  state transition;
- driver-owned lane-current session と `mizar-session` request-generation current
  snapshot の両方を確認してから、diagnostics、build event、artifact-boundary handoff
  を current として公開する publication guard。

`request` が所有しないもの:

- source text の parse、import resolution、type checking、VC generation、proof
  acceptance、trusted status、kernel acceptance;
- cache compatibility、cache-key validation、proof-reuse validation;
- artifact manifest transaction、artifact serialization、publication-token issuance;
- LSP document-version protocol conversion、range conversion、code action、
  editor command。

## 公開 enum の互換性

この module の public enum はすべて downstream-facing な driver envelope であり、
`#[non_exhaustive]` を付ける。D-017 では以下について exhaustive exception を記録しない:

- `BuildRequestOrigin`;
- `LspPriority`;
- `BuildSessionState`;
- `BuildSessionOutcome`;
- `PublicationDecision`。

Downstream crate はこれらの enum を match するとき wildcard arm を持たなければならない。
Crate-local code は、新しい variant を request/session lifecycle とともに review すべき
場所では、既知 variant への exhaustive match を内部に保ってよい。

## データモデル

### Request Identity と Currentness Lane

`mizar-session::BuildRequestId` は 1 つの batch/watch/LSP request generation を識別
する。driver は submit される generation ごとに新しい `BuildRequestId` を割り当て、
それを `SnapshotRegistry::create_snapshot` と
`SnapshotRegistry::is_current_for_request` に渡さなければならない。

supersession は driver-owned であり、別個の `BuildLaneId` を使う。batch build では、
lane は通常 1 つの generation だけを持つ。watch と LSP build では、2 つの generation
が同じ `BuildSnapshotId` を生む場合でも、新しい session が古い session を atomically
に置き換えられるよう、複数の edit generation が 1 つの lane を共有できる。lane と
generation は `BuildRequest` 上の driver-owned metadata であり、どちらも
`BuildSnapshotId` の一部ではなく、cache や artifact compatibility authority として
使ってはならない。

### BuildRequest

```rust
struct BuildRequest {
    id: BuildRequestId,
    lane: BuildLaneId,
    origin: BuildRequestOrigin,
    generation: BuildRequestGeneration,
    workspace_root: WorkspaceRoot,
    profile: BuildProfile,
    targets: BuildTargets,
    source_inputs: SourceInputSet,
    dependency_inputs: DependencyInputSet,
    verifier_config: VerifierConfigInput,
}

enum BuildRequestOrigin {
    Batch(BatchRequest),
    Watch(WatchRequest),
    Lsp(LspRequest),
}
```

`BuildRequest` は driver の input envelope である。何を build するか、どの
source/dependency state を snapshot に捕捉するかを指定する。mutable phase output、
cache decision、artifact handle、LSP protocol payload、render 済み diagnostic text
を含めてはならない。

batch request は CLI arguments と package defaults から作られる。watch request は、
owner-provided file-change / coalescing layer が changed path と source snapshot
input を正規化した後、watch-facing orchestrator が作る。driver は OS file
watching、debouncing、source loading、file-to-module discovery rule を所有しない。
LSP request は `mizar-lsp` bridge が protocol conversion を終えた後に作る。driver
は protocol-agnostic な open-buffer source input と priority hint だけを受け取る。

### Request Origins

```rust
struct BatchRequest {
    invocation: BatchInvocation,
}

struct WatchRequest {
    watch_root: WorkspaceRoot,
    changed_paths: Vec<NormalizedPath>,
}

struct LspRequest {
    focus: Option<LspFocus>,
    priority: LspPriority,
}
```

batch request は別の session を置き換えずに terminal status へ進める。watch
request は、次の coalesced generation が accept されたとき、同じ watch lane の以前の
session を置き換える。これは同じ `BuildSnapshotId` を捕捉した場合も含む。
same-snapshot replacement は IR publisher の no-op であり、古い session を current のまま
保つ理由ではない。LSP request は、新しい open-buffer input がその lane の古い diagnostics
を置き換えるべきとき、同じ LSP lane の以前の session を置き換える。

`LspRequest` は protocol-agnostic のままでなければならない。`mizar-session` により
正規化済みの source input と、`mizar-lsp` が選んだ priority 情報は参照できるが、
LSP range、protocol payload としての document URI、JSON-RPC id、code-action edit、
render 済み diagnostic object を含めてはならない。

### BuildSession

```rust
struct BuildSession {
    id: BuildSessionId,
    request: BuildRequest,
    captured: CapturedSnapshot,
    state: BuildSessionState,
}

struct PendingBuildRequest {
    session_id: BuildSessionId,
    request: BuildRequest,
}

struct CapturedSnapshot {
    snapshot: BuildSnapshot,
    active_snapshot_lease: SnapshotLease,
}

enum BuildSessionState {
    SnapshotCaptured,
    Submitted,
    Running,
    Cancelling,
    Finished(BuildSessionOutcome),
}

enum BuildSessionOutcome {
    Succeeded,
    Failed,
    Blocked,
    Cancelled,
    Superseded,
}
```

session は 1 つの immutable snapshot 上の 1 回の scheduler run を表す。session id は
allocator-issued で、driver/scheduler bookkeeping の local identity である。snapshot
id は content-derived で、source/dependency/toolchain/verifier input state を識別する。
task result は session snapshot に属し、session id には属さない。別 session で再利用
できるのは、target snapshot に対する owner-provided cache validation を通った場合
だけである。

`PendingBuildRequest` は accepted request の pre-snapshot record である。完全な
`BuildSession` は snapshot capture の後にだけ存在する。`SnapshotCaptured` は
`SnapshotRegistry::create_snapshot` が `BuildSnapshot` と `ActiveBuild` lease を返した
状態である。`Submitted` は driver が planned graph を scheduler へ渡した状態である。
`Running` は scheduler-owned task がまだ session の event を生成し得る状態である。
`Cancelling` は cancellation が要求され、in-flight task が safe checkpoint で停止する
ことを期待される状態である。`Finished` は terminal。

`Superseded` は、lane がより新しい current session を指す watch/LSP session の terminal
state である。superseded session は stale diagnostic data や retained explanation handle
を持ち続け得るが、それらを current として公開してはならない。

## Snapshot Capture

driver は、load 済み source version、dependency artifact reference、lockfile
identity、toolchain identity、verifier configuration identity から
`mizar-session::SnapshotInput` を組み立てる。snapshot identity と validation は
`mizar-session` が所有する。

registry snapshot の唯一の public creation path は次である:

```rust
SnapshotRegistry::create_snapshot(request.id, snapshot_input)
```

返された `BuildSnapshot` と `SnapshotLease` は `BuildSession` の `CapturedSnapshot`
部分になる。lease reason は `RetentionReason::ActiveBuild` でなければならない。後続の
retention handoff が必要になったとき、driver はこの lease を `RetentionManager` に
bridge してよい。ただし snapshot の retain は currentness を作らない。request-generation
currentness は引き続き `SnapshotRegistry::is_current_for_request` で確認し、watch/LSP
supersession currentness は driver-owned lane table で確認する。

同一の canonical snapshot input は、別 session に現れても同一の `BuildSnapshotId`
を生まなければならない。`BuildSnapshotId` の再利用は、allocator-issued な session、
request、source、source-map、lease id が再利用可能であることを意味しない。

## Current-Session Publication Boundary

driver が session output を current として公開する前に、必ず次を確認する:

```rust
driver_lanes.is_current_session(
    session.request.lane,
    session.request.generation,
    session.id,
    session.captured.snapshot.id,
) &&
snapshot_registry.is_current_for_request(session.captured.snapshot.id, session.request.id)
```

両方の確認が必要である。driver-owned lane check は、古い generation と新しい generation
が同じ `BuildSnapshotId` を持つ場合も含め、obsolete な watch/LSP session を拒否する。
`mizar-session` registry check は、captured snapshot がそれを作成した request generation
の current snapshot であり続けていることを確認する。

この combined check は次で必須である:

- current diagnostics readiness;
- current build-event stream publication;
- watch/LSP consumer へ報告する final session status;
- real artifact owner seam が committed result を報告した場合の artifact-boundary
  handoff event;
- "latest" build state を公開する将来の driver API。

どちらかの確認が失敗した場合、request layer は suppressed publication decision を
返さなければならない。後続の event layer は、その decision を、古い session をまだ
subscribe している observer 向けの protocol-agnostic な stale/suppressed event に変換
してよい。driver は stale diagnostics、phase outputs、cache decisions、artifact
records を current として付け替えてはならない。

batch session は通常 superseding generation を持たないが、driver の publication path
が 1 つの freshness rule を共有するよう、同じ guard を使う。

## Supersession

watch と LSP orchestration は、同じ driver `BuildLaneId` に fresh な
`BuildRequestId` を持つ新しい request generation を作り、その新しい session を
driver-owned lane table で current と mark することで session を supersede する。
`SnapshotRegistry::create_snapshot` は新しい request generation の current snapshot
だけを更新し、watch/LSP lane supersession authority ではない。lane table は
`BuildRequestGeneration` について monotonic でなければならない。新しい generation が
current と mark された後に、古い generation が再び current になってはならず、同じ
generation の繰り返しは同じ session/snapshot tuple に限り有効である。driver lane
table が更新された後:

- lane 内の古い session は、より新しい generation と同じ `BuildSnapshotId` を持つ
  場合でも current publication に対して obsolete になる;
- 古い in-flight task は scheduler/cancellation seam を通じて cancel される;
- 完了済みの古い結果は、owner-provided cache validation path が新しい snapshot に
  適用可能であることを証明しない限り discard される;
- diagnostic、explanation、IR retention lease は、currentness を変えずに古い
  snapshot resources を生存させてよい。

driver は supersession を freshness boundary として扱い、新しい build が成功した
証拠として扱ってはならない。失敗した新しい session も lane の current session で
あり得る。その場合、古い successful output に戻すのではなく、新しい session 自身の
diagnostics/status を公開する。

## Error Handling

snapshot creation error は、phase diagnostics を発明せず driver request/session
failure として報告する。request layer は snapshot creation error を返すとき pending
request/session context を保持し、caller が submitted request に対して failure を報告
できるようにしなければならない。driver は event delivery のために owner error を wrap
してよいが、structured identity は owner-provided error または diagnostic record のまま
でなければならない。message text は presentation であり diagnostic identity として
使ってはならない。

cancellation は driver boundary で冪等である。すでに terminal な session を cancel
しても、その session は revive されず、無関係な snapshot は release されず、stale
output は publish されない。

## Tests

task D-003 implementation は次を cover する Rust tests を追加しなければならない:

- protocol payload を漏らさない batch、watch、LSP request construction;
- `SnapshotRegistry::create_snapshot` を通じた snapshot capture;
- 同一 canonical snapshot input が session をまたいで同一 `BuildSnapshotId` を生む;
- submitted generation ごとに fresh な `BuildRequestId` を割り当てつつ、複数の
  watch/LSP generation が 1 つの driver `BuildLaneId` を共有すること;
- watch/LSP supersession が lane-current snapshot だけでなく lane-current session を
  置き換えること;
- 古い generation を current に戻そうとする stale lane-current update の拒否と、
  同じ generation を異なる session で置き換えることの拒否;
- より新しい lane-current session と同じ snapshot id を持つ古い session を含む、
  superseded watch/LSP session の obsolete publication rejection;
- lane-current-session と `SnapshotRegistry::is_current_for_request` を組み合わせた
  publication guard;
- current diagnostics/artifact publication を伴わない stale/suppressed publication
  decision;
- snapshot creation error が pending request/session context を返すこと;
- current session と既に superseded された session の idempotent cancellation。

この module は orchestration state を定義し、language behavior を変えないため、
`.miz` test は不要である。
