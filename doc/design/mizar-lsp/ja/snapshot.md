# Editor snapshot と freshness の境界
正本: [英語版](../en/snapshot.md)。

## Responsibility

[architecture12](../../architecture/ja/12.diagnostics_and_lsp.md#lsp-は安定したスナップショットを読む) と [internal03](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md#lsp-スナップショット) に従い、LSP は immutable な editor read と freshness を所有する。source 取得/map/lease は Session、sealed output の有効性と retention は IR、build scheduling/currentness は Driver、manifest と公開 metadata は Artifact が所有する。compiler や proof の権限を LSP に移さない。

## Source association

実際の [originating capture](server.md#originating-native-open-buffer-capture) を使い、元 source/map object と URI/version/raw text/open incarnation の関係を保つ。BuildSnapshotId は正規化済み build input の identity であり、文書版/raw loading map/open incarnation/publication generation の identity ではない。有効な版のみの変更と正規化等価な raw 編集でも、build id が等しいまま旧 editor association を無効化する。同一 tuple の reopen も新しい incarnation である。

各 read は終了まで一つの immutable association を保持する。置換後の read は新 association を選び、実行中 read は完了まで元の値と retention を保つ。新旧の map/診断/version/metadata を混ぜない。current 配送前には、既存 native guard に加え、正確な現在文書 tuple と元 incarnation を検証する。保存 id や retained handle でこの検査を代替しない。

## Retained native sources

`Server::capture_native_open_buffers<A: SessionIdAllocator + Send + Sync + 'static>(&mut self, draft: BuildRequestDraft, allocator: &A, snapshots: &Arc<SnapshotRegistry<A>>, index: &ModuleIndex, sources: Vec<(DiskSourceLoader, PackageId, String)>) -> Result<NativeOpenBufferCapture, String>` は Server を非 generic のまま保ち、初期化済み生存接続と重複のない非空の正確な URI key 列を要求する。各現在文書 entry を一度読み、未登録/close 済みと負の i32 version を protocol 受理を変えず拒否する。entry の正確な URI/raw text と非負を確認した i64 version（expected/actual 両方）から OpenBuffer origin を作り、同じ操作で元 open Arc を downgrade する。明示 draft root/package/loader で既存 index.native_open_buffer_input を使い、全座標検査後に draft.capture_sources を一度だけ呼ぶ。association の String error と Debug 形式の capture error を伝え、部分成功や入力補完をしない。caller の capture/lifetime/origin は受け取らず、index/plan/context と publisher 権限は既存 owner に残す。
`NativeOpenBufferSnapshot` は server.rs の public 型で、全 field は private、公開操作は `captured(&self) -> &CapturedSnapshot` のみとする。public constructor/Clone/conversion/mutation は持たない。元 CapturedSnapshot clone、元 URI/Weak 関係、所有 release closure を保持する。元 native tuple/map は [Driver capture storage](../../mizar-driver/ja/request.md#native-source-capture) を共有し、再読込や等しい build id/Session 検索からの再構成は行わない。
`Server::native_open_buffer_snapshot(&self) -> Option<Arc<NativeOpenBufferSnapshot>>` は初期化済み生存接続で全ての元 URI/version/raw text が現在も一致し、元 Weak と新しい document_lifetime が Weak::ptr_eq で一致するときだけ、一つの完全な captured cohort を返す。source read だけを公開し、完全 artifact/pending work/cleanup 権限を与えない。借用 ActiveBuild lease 値は所有権を移さない。

Server は private に `Option<Arc<NativeOpenBufferSnapshot>>` を保持し、内部可変性を持たせない。実際の独立した OpenBufferOverlay lease 取得後、capture 成功時に pointer を atomic に置換し、取得/capture 失敗は旧選択を保つ。置換/無効化は排他的 Server borrow を要求し、originating consumer の immutable Server borrow は生成や外部 Write 中の recapture を防ぐ。所有 reader Arc は後の編集/置換をまたいで使える。cohort URI の有効な新 didChange（空変更/版のみを含む）または didClose で clear し、無視/不正/古い/無関係 traffic は保持する。shutdown/exit で clear しても raw 文書保持方針は変えず、reopen で復活させない。reader は置換/無効化をまたいで元 cohort/map/overlay lease 全体を保つ。この保持 source read は stale_artifact や current 配送ではない。
`NativeOpenBufferCapture` は move-only とし、public constructor/Clone/accessor/conversion を持たない。private に Option<BuildSession>、association Arc、任意の ActiveBuild cleanup closure を所有する。Drop は handoff 前だけその正確な ActiveBuild lease を解放する。各 originating consumer は元 session を取り出し、一度の既存 Driver 呼出し直前に cleanup を disarm する。handoff 後は error を含め、既存の lease/deferred retirement は Driver の責務である。
overlay allocation 失敗時は未 handoff の ActiveBuild lease を解放してから String Err を返し、取得前 validation 失敗では解放対象がない。Snapshot Drop は association/reader/ticket の最後の参照で自身の overlay lease だけを一度解放する。closure は実 registry Arc と正確な lease を所有し、解放 error は不変条件を明示した expect で観測可能にする。Session production 変更や public lease wrapper は不要である。
originating notification/writer は現在の association Arc を選び、実生成/変換/書込みまで保持し、ticket の association と Arc::ptr_eq が成立するときだけ配送する。元 incarnation/raw/native/Driver guard と、不適格時の実 submission/error 経路を保つ。raw text/version/build id が等しくても新 capture は旧 ticket の配送を抑止し、保持 read は publication 権限や IR publisher replacement を与えない。

## Freshness and replacement

current/stale_artifact/pending は [architecture12](../../architecture/ja/12.diagnostics_and_lsp.md#開いているバッファはディスク上のソースを上書きする) の既存の意味に従う。current は現在 editor source から計算した応答を表し、proof acceptance ではない。Pending は実際に scheduled/running な verification を必要とする。producer input 不在は unavailable のままとし、予定済み作業や空の成功 snapshot を示さない。

current な syntax-only overlay は完全 artifact より前に存在できるが、semantic metadata や coherent な完全 LspSnapshot を与えない。change/close/reopen で旧 overlay の配送を直ちに抑止し、close は open association を除去する。shutdown/exit 後に新しい current 配送を行わない。新しい lifetime handle の query で旧 overlay を付け替えない。

保持された artifact read は編集後も stale_artifact として残せる。元の source association と明示的 stale status を保持し、unsafe edit と、target を検証できない current-buffer 配送を抑止する。read-only な stale range は元の retained source と明示的 stale status に結び付けたまま使え、current-buffer range として付け替えない。producer work や current map が無いだけで診断を clear しない。publication/conversion の詳細は task8、scheduling/cancellation は task10 が所有する。

完全 LspSnapshot は [internal03](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md#lsp-スナップショットの公開) の coherent completed build 境界と元 manifest-backed metadata を必要とする。manifest を先に読み、一つの完全 immutable set を保持してから handler-visible snapshot を atomic に置換する。ArtifactCommitted のみ、Frontend 完了、診断 index ではこの境界を確立しない。現 Driver event enum に coherent SnapshotPublished producer はなく、完全 snapshot の構築/起動は担当 owner の真正な境界と payload がそろうまで保留する。

## Currentness and retention

[Driver lane/request publication](../../mizar-driver/ja/request.md) と [IR publisher currentness](../../mizar-ir/ja/publisher.md#snapshot-and-work-unit-validation) を区別する。SnapshotHandleRegistry は identity/lineage を登録し、currentness を持たない。build id が異なる場合は、owner が許可した publisher replacement で旧 id を obsolete にできる。id が等しい editor edit でも editor association を無効化し、等しい id に IR replace_current_snapshot を呼んだり obsolete id を復活させたりしない。

LSP 所有の Session lease は SnapshotRegistry により、実際の PublishedLspSnapshot/OpenBufferOverlay retention reason で取得/解放する。ActiveBuild lease は captured build owner が所有し、値の clone は cleanup 権限を移さない。read に必要な実 sealed IR output のみ IrStorageService retain/release で保持する。Session lease だけで IR storage は pin されない。published/overlay association または実行中 reader が必要とする間は retention を保ち、両方が終了してから所有 retention を解放する。collection は上流 storage 規則に従う。retention は stale read を許し、current publication や artifact 権限を与えない。

## Implementation boundary

task7 はまず上記の retained native source association だけを実装する。完全な current/stale artifact registry、IR retention、coherent LspSnapshot は別途とし、task8–11 は診断/scheduling/lifecycle cleanup を担当する。仮の完全 snapshot、metadata index、manifest、event producer、runtime identity は作らず、この前提作業で task7、ENTRY/SCA-003、Steps6–8 を完了としない。

## Test design

実受理 open/change/close/reopen/terminal message と protected native capture を使う。同じ build id を共有する版のみの編集と正規化等価な raw 編集、同一 tuple の reopen、別 Server を検査する。いずれも旧 current association を抑止しつつ、有効な retained read を保つ。置換をまたぐ reader は完全な新旧どちらか一つの association を観測する。Session lease と IR retain/release/collection は別々に実 API で確認し、stale-readable だが publisher-obsolete な output と等しい id の置換拒否を含める。syntax-only 成功から完全 metadata/current-build completion を作らず、producer work の不在/部分実行から診断 clear や pending work を作らない。既存 language/producer/mapper/lifecycle matrix は不変とする。

retained native source は実 originating capture/notification/writer test を拡張し、独立した raw/version/map 期待値で同じ normalized id の二 capture、置換をまたぐ旧 reader、Driver clone/submission を通じた native tuple/map pointer 一致を確認する。同一 tuple の reopen、別 Server、close/terminal で配送を抑止しても旧 reader は読めること、capture 失敗が旧選択を保ち、無視/不正/無関係 traffic が退避させないことを確認する。
実 observing Session allocator の lease ID と実 release 結果により、保存 association/reader/ticket のいずれかが残る間の overlay 保持と、最後の参照で一度だけ解放することを示す。ticket drop/overlay 失敗での未 handoff ActiveBuild cleanup と handoff 後の Driver 所有を別に確認する。全既存 assertion、native/raw reopen/writer/error matrix、submission 経路を保ち、registry Arc/capture 可変 borrow/private field の機械的適応と、既存 discovery 各 route で native file 削除前に capture し、route 間に fixture の不正 disk byte だけを復元する順序調整を許可する。disk 不在/current 配送の全 assertion は維持する。producer/cache Hit/Accepted の捏造、任意 timing、unsafe test access は使わない。
