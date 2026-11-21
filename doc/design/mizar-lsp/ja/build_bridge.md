# Build bridge
正本: [英語版](../en/build_bridge.md)。英語版と同期した日本語訳。

## Responsibility

Task10 は Server 接続ごとの単一 `BuildLaneId`、origin `BuildRequestOrigin::Lsp` の scheduling と event 消費を定義する。以下の所有者決定方針は [internal01 の Watch and LSP Build](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md#watch-and-lsp-build) を具体化する。Task11 が実装し、source/freshness は [snapshot](snapshot.md)、公開は [diagnostics](diagnostics.md) が引き続き所有する。

## Admission and deadlines

accepted event は [Server 同期規則](server.md) が受理し状態を変える didOpen、didChange、didClose であり、空 change list による version 更新も含む。無視・不正・重複・stale 通知は含まない。didSave は trigger ではない。
LSP lane は admitted かつ未 flush の slot を最大1つ持つ。text、version、generation、freshness label を持たず、`pending` ではない。admission は Driver entry point を呼ばず capture しない。初回は t_first/t_last を現在時刻に設定し、以後は同じ lane の全 document/package を通じて t_last だけを更新する。
注入された単調時計を用い、各 accepted event の直後と次の deadline 到達時に trigger を評価する。仕様定数は quiet_period = 300 ms、max_wait = 2 s、quiet_period <= max_wait とし、editor 設定面は定義しない。
now - t_last >= quiet_period、または now - t_first >= max_wait AND lane に running session がない、という条件を最初に満たす瞬間に flush する。submitted request は generation と Driver session を持ち、その session が非終端の間を running とする。quiet 満了は running 中も flush でき、cap 満了だけではできない。
coalesce するのは admitted request のみ。submitted request 同士や slot とは coalesce せず、lane/origin を跨ぐ coalesce も行わない。中間の editor version は build しない。

## Capture and submission

他の accepted event を挟まず、flush は slot を消去し、lane 内の過去の全割当より厳密に大きい generation を1つ割り当て、その時点で開いている全 document と現 version を1つの cohort として capture し、成功時に1回だけ submit する。open document がなければ capture/submit せず slot を破棄し、document 数ゼロで build を行わない。
[Server の native capture](snapshot.md#retained-native-sources) と既存の captured-session Driver submission（`submit_captured` または既存の import-discovery/native-index entry point）を使い、`submit_watch_change` は使わない。bridge が現在の open-document coordinate 全体を渡し、Driver は Server buffer を列挙しない。元の capture/ticket handoff と lease 所有を保ち、source map を再構成せず、呼出側が capture を捏造しない。
capture は admission 時でなく flush 時に行う。capture 失敗は slot を消費して自動再試行せず、割当済 generation を再利用しない。従前の lane-current session を変更も cancel もせず、診断を消去せず、pending 状態を捏造しない。失敗の利用者通知は未規定である。
capture 成功後に submission が失敗しても、新 generation は lane の generation に留まり旧 generation に戻さない。既存の構造化 Driver submission error と cleanup 所有は変えない。

## Public enum policy

`BuildBridgeError` は意図的に exhaustive を維持する。Capture/Submit は bridge の二つの失敗段階を区別し、Driver error は Submit 内に保持する。新たな段階の追加時には compile time に caller の失敗処理の再検討を要求する。

## Supersede and terminal lifecycle

編集の継続は t_last のみを更新し、Driver request を行わず、running generation g を cancel しない。置換 capture が成功し Driver が置換を lane-current として受理した後、g に属する session が非終端なら `cancel(session_g, DriverCancelReason::Superseded, snapshots)` をその session に正確に1回要求し、既に終端なら何もしない。capture 失敗では g を cancel しない。
shutdown request または exit は slot を flush せず破棄し、running session に `cancel(session, DriverCancelReason::Shutdown, snapshots)` を1回要求する。終端 lifecycle traffic 後は admission/capture/submission しない。cancel は既存 [request guard](../../mizar-driver/en/request.md) の下で協調的に行い、Driver が所有する。

## Build event consumption

所有する submitted session の既存 `CompilerDriver::events(session)` 履歴を読む。known-session 状態と実 event identity（session、lane、generation、optional snapshot、publication）で submission と元の captured cohort に対応付け、BuildSnapshotId の等値から editor version/incarnation を推定しない。これは replay 取得であり、新 subscription API ではない。
自分の LSP lane の event のみ消費する。Driver 順序と publication guard を保ち、readiness の replay は current 権限を復活させない。progress、gap、suppression、SessionFinished は既存 [BuildEventKind の意味](../../mizar-driver/en/events.md) を保ち、下流の non-exhaustive match は wildcard を要する。event kind/payload field、公開 diagnostic code/name/alias、producer behavior を追加しない。
真正で coherent な `SnapshotPublished { snapshot }` payload だけを task7 の registry/atomic activation と task9 の診断公開境界へ渡す。ArtifactBoundary/ArtifactCommitted、Frontend 完了、DiagnosticsReady、index 単独は coherent full snapshot ではない。ここで event 消費が registry を activate したり診断を送信したりせず、current 配送と stale read は元の source association とその所有者に従う。

## Watch-mode interplay

bridge は自分の LSP lane のみ submit/cancel/consume し、Watch-origin request を作らず、watch lane に submit/cancel/subscribe しない。watch と LSP は異なる BuildLaneId を使い、各 lane は1つの origin を持つ。arbitration、lane 間の generation 比較、supersede、cancel、coalesce はない。
各 lane の admission、flush、capture 失敗、cancel は他 lane の admitted/submitted/running request を変更しない。Disk file 変更は LSP lane を trigger せず、editor buffer 編集は watch lane を trigger しない。
独立性は Driver lane table の lane 分離を条件とする。IR `replace_current_snapshot` の `old` は同じ lane の直前 snapshot のみに限り、LSP と watch は `PhaseOutputPublisher` instance を共有しない。publisher currentness は [editor currentness](snapshot.md#currentness-and-retention) と別である。
共有 Disk artifact/cache は各所有者の manifest transaction と cache validation 規則だけに従い、bridge は規則を加えない。registry/publication へは所有 LSP event のみ渡す。Disk-committed artifact の read は bridge 外の snapshot/metadata `stale_artifact` 規則に従う。これらの境界は例外なく全 bridge operation/target に適用する。
したがって Disk 側の変更は次の accepted event まで LSP 結果に反映されず、この制限は [DRIVER-G-012](../../mizar-driver/en/00.crate_plan.md#known-gaps-and-drift) に紐付けて deferred とする。

## Implementation boundary

承認済み[非終端実行 seam](execution_seam.md) が bridge cancellation test に実際の captured execution を供給する。
実装済みの Driver 所有 [checkpoint 観測 seam](../../mizar-driver/en/driver.md#native-checkpoint-observations) が native 受領証拠を保持し、unit test の bridge start は同じ captured route の `start_captured_observed` を使用する。
`src/build_bridge.rs` は `Server::with_build_bridge`、`drive_build_bridge`、`next_build_deadline`、`build_bridge_events` により task11 を実装する。host は lane 専用 Driver と対応する publisher、単調時計を渡し、受信 traffic と返された deadline で駆動し、flush 時だけ全 open URI の座標を解決する。文書 traffic の受理前に bridge を構成する。[native executable host](server.md#native-executable-host) が承認済み frontend-only runtime を構成し、Watch 統合は DRIVER-G-012 に残る。
`with_nonterminal_build_bridge` は構成済み host を `start_captured` に切り替え、`progress_build_bridge` は所有する各 live execution を一段進めて排出済み submission を返す。protocol traffic を処理してから進行する。`next_build_deadline` は Shutdown/exit 後も work が残れば即時進行を要求する。terminal lifecycle は current session を一度 cancel し、admission を止め、排出用 progress/replay を維持する。`with_build_bridge` は同期動作を維持する。
event replay は元の capture を保持し、registry activation や診断送信は行わない。これらは task7 と task9 の境界に残す。OpenBuffer provenance、artifact storage、Session SourceId encoding は保留中の Driver 単位に属し、本 scheduling 方針は依存も決定もせず、OpenBuffer artifact-production 能力を付与しない。SCA-003 credit は変更しない。

## Test design

task11 の fixture は注入時計で accepted-event admission、編集 burst、quiet/cap deadline、terminal discard を検証し、実際の native capture と Driver submission で失敗時の消費、元の event association、lane 分離を検証する。同期 control は terminal submission を維持し、opt-in host の前提 test は実 native task の進行と元の terminal replay、および step 間の protocol traffic を検証する。
Task40 test は実際に admitted された native Frontend task で停止し、編集と capture 失敗では実行を維持し、同一 snapshot の置換受理で Superseded、Shutdown/exit で admission 破棄と Shutdown を要求する。progress 前の Cancelling、drain 後の保持された `FrontendServiceEntry` 受領証拠と正確な session/task/input snapshot/cancellation token、単一終端 event、元の capture/replay、obsolete publication の不在を検証し、非 cancel の置換と別 lane は独立した実行・publisher 権限を維持する。受領証拠の provenance と checkpoint は Driver が所有し、running predicate の timing fixture は別の証拠とする。
