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

## Native runtime-context definition

P25 は後続の native producer と LSP host の入力を定義する。**G14 承認は未取得であり、コード着手前に必要**。以下は内部設計の推奨であり、稼働中の既定値や host/API 実装ではない。既存の明示的呼出しと承認済み active-run cache binding の契約を維持する。

### Identity and same-request association

Driver は実際の bootstrap で既存の [running-image acquisition](../en/frontend_adapter.md#native-disk-cache-preparation) を使う。Linux の実行イメージ全体、参加コードの mapping evidence、既存の BLAKE3 derive-key domain が根拠となる。限定 native profile の `ToolchainInfo` は `mizar-driver/running-native-image/v1:` と当該 producer の小文字 hex digest の連結を推奨する。同じ値を生成 plan と capture の dependency inputs に渡す。package version、lock metadata、環境の主張、呼出し側の label からは作らない。未対応 platform/linkage や evidence 不足では当該 profile を利用できない。
これは稼働中の workspace rlib closure の識別のみである。記録された release SemVer、package constraint、選択 pin、外部 backend build、release authentication は Build の [toolchain boundary](../../mizar-build/en/planner.md#toolchain-constraint-boundary) に従う別の事実である。runtime identity の一致は pin の充足や W0306 の選択を意味しない。既存の cache compatibility operands を変更せず、異なる image の互換性も定義しない。

後続 producer は実際に検証された plan の verifier/build aggregates に既存の [workspace_config_content_hash](../../mizar-build/en/planner.md#current-package-configuration-content-fingerprint) を適用し、`VerifierConfigInput.hash` とする。これは package-content profile の明示的選択であり、万能な実効 verifier digest ではない。既存 encoding と package ごとの defaults/computation_replay を維持し、Proof fingerprint や raw TOML hash で代用しない。runtime policy、invocation override、backend、seed、budget は別の owner 入力であり、それらを使う後続 profile は完全な association を必要とする。config-to-policy mapping は追加しない。
明示的 native root の取得、検証済み plan、そこから生成した lock/config identity、実際の runtime identity、元の source capture tuples を 1 つの不変 request に対応させる。[workspace-only projection](../../mizar-build/en/planner.md#native-workspace-only-context-inputs) は planner が所有する。configuration/runtime/dependency selection の変更後は再構築し、foreign plan/capture は既存 guard で拒否する。digest は現在の file、原子的取得、live session/publication permission、後段 footprint の完全性を証明しない。

### Default LSP profile recommendation

`BuildProfile.name = "lsp"`、normal dependency selection、Frontend までの既存 `StageConfig::frontend` composition、外部 artifact roots なしを推奨する。発見した全 workspace members と実際の source inputs を使う。OpenBuffer overlay の capture/version/currentness 規則は維持する。profile 名は routing metadata であり、phase input を捏造する権限ではない。
既定 profile は source/frontend diagnostics を観測し、完全な検証済み build を主張しない。Proof/replay、artifact emission、persistent cache preparation、外部 backend は有効化しない。明示的な豊富な profile は別扱いとする。新しい diagnostic や成功扱いの fallback は定義しない。runtime/workspace input が取得不能なら当該 profile は利用不能のままとし、placeholder や空 context で置き換えない。後続 producer/host 契約で既存 unavailable/error transport と lifecycle tests を凍結する。

### G14 decisions and staged handoff

- **Q1 — snapshot runtime identity:** 上記の既存 image producer による versioned string を推奨する。代替は authenticated distribution/release identity を待つこと。推奨は承認済み running-image provenance の snapshot input への適用であり、どちらも release を認証せず Cache compatibility を変更しない。
- **Q2 — LSP default:** frontend-only `lsp`、normal dependencies、optional cache 無効を推奨する。代替は広い effective-policy/dependency producer が揃うまで profile の明示選択を要求すること。完全検証を既定には選ばない。
- **Q3 — initial dependency scope:** Build 所有の厳密な workspace-only projection を推奨する。代替は authenticated registry manifests/artifacts まで bootstrap を待つこと。registry の default metadata や未知の footprint は producer の代わりにならない。

G14 で owner 選択を記録した後に Muse が Wave 4 を dispatch する。先に staged producer の genuine positive/refusal inputs、厳密な同一 request association、既存 identity/config/lock matrix 再利用を凍結する。native executable-host activation は実 stdio/lifecycle evidence を伴う別の後続 unit とする。この定義はコード、host activation、policy integration、semantic coverage、build 完了の credit を与えない。

## Snapshot Capture

driver は、load 済み source version、dependency artifact reference、lockfile
identity、toolchain identity、verifier configuration identity から
`mizar-session::SnapshotInput` を組み立てる。snapshot identity と validation は
`mizar-session` が所有する。

従来の summary 捕捉は次を呼ぶ:

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

## Commented draft capture and retention
`BuildSession::capture_commented_module` は genuine metadata と sealed Frontend payload を protected native capture に結合し、[IR handoff](../../mizar-ir/ja/projection.md#commented-draft-handoff) を一度呼び、未公開の immutable draft と required handle を session clone 間で保持する。capture 失敗時は保持内容を変えない。
`commented_module` は Session/request と publisher の currentness・publication root・physical handle・typed Frontend の readability を確認して draft を借用し、cancelled session は拒否する。未登録と available-empty は区別する。session value は IR storage を pin せず draft を保持する。自動取得・source 再読/再構成・scheduler 起動・publication 資格の昇格・Doc attachment は供給しない。

## Commented artifact emission
`BuildSession::emit_commented_module` は captured loader の package root より厳密に下の既存 artifact root へ、安全な parent 検証後に未使用の `.mizir.json` path を atomic に予約し、同時 Driver emission による上書きを防いで保持 envelope を明示的に出力する。書込み前、transaction freshness check、readback 後に retained-read 検証と全 original handle の CurrentPackage 資格を確認し、OpenBuffer/InternalOnly は拒否する。これは point check であり、新たな currentness lease ではない。
Artifact 所有の commented codec/domain で完全 readback と outer hash を得る。既存 manifest transaction に導出済み package/source/provenance と witness reference を渡し、参照ファイルを検証して実 commit を返す。precommit failure は既存 manifest/main を保持し、late failure では orphan file や commit 済み manifest が残り得る。保持 draft は変更しない。Doc validated-publication consumption は未完了。

## Native source capture

`BuildSession::capture_source_load_failures` は成功済み native capture を変えず、実 Disk load を一度だけ試みた request/error を private に保持する。元の package/loader 束縛を要求し、非対応・重複・予想外の成功は原子的に拒否する。失敗 request に SourceVersion/text を作らず、初回の全件成功 capture と nonempty native currentness 条件は維持する。[診断専用 submit](driver.md#captured-source-load-failures) は保持 error を再読込せず消費する。


`BuildRequestDraft::capture_sources(allocator, registry, sources)` は保護された `BuildSession` または `DriverSubmitError` を返す。Native `(DiskSourceLoader, SourceInput)` が入力であり、summary があれば ID 前に draft を保持する `NativeSourceVersionsProvided { request }` で拒否する。同じ generic 型の異なる allocator instance は従来どおり許可する。
通常の request/session ID 取得後、実 `load_snapshot` を一度だけ呼ぶ。失敗は既存 capture error に元 pending を保持し、再試行用 descriptor clone は caller が保持する。成功は実 summary と元 loader/返された LoadedSource/SourceInput triple を CapturedSnapshot に保持する。source ID/map は registry allocator を使う。
Session の identity/request/capture/state と捕捉 snapshot/lease は crate 内だけで変更可能。公開 `id()`、`request()`、`captured()`、`state()`、`snapshot()`、`active_snapshot_lease()` は読み取り専用で、native_sources() は元 (DiskSourceLoader, LoadedSource, SourceInput) tuple または None を公開借用し、保存領域は private のまま。内部 lifecycle と watch 強制 supersession は維持する。clone は実 raw resource を保持し、終了時に lease/buffer を自動解放しない。
CapturedSnapshot は native tuple を private な共有 Arc slice に保持し、capture/Driver/result/error clone と LSP reader は元 tuple/map object を共有する。公開借用 accessor/signature と元 tuple の値の equality は変えない。
publisher owner が実 snapshot/work unit を承認してから[準備済み submit](driver.md#captured-native-submission)を呼ぶ。捕捉は publication 権限を付与しない。registry 検索や canonical ID 一致では個別の raw/version 捕捉を復元できない。

元 tuple の借用自体は live session/editor/publication 権限を持たず、外部 consumer が Driver 保存 session、live guard と現文書検査を使う。可変 access、新 ID、再読込、retain 操作は導入しない。

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
