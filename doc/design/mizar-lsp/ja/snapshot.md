# Editor snapshot と freshness の境界
正本: [英語版](../en/snapshot.md)。

## Responsibility

[architecture12](../../architecture/ja/12.diagnostics_and_lsp.md#lsp-は安定したスナップショットを読む) と [internal03](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md#lsp-スナップショット) に従い、LSP は immutable な editor read と freshness を所有する。source 取得/map/lease は Session、sealed output の有効性と retention は IR、build scheduling/currentness は Driver、manifest と公開 metadata は Artifact が所有する。compiler や proof の権限を LSP に移さない。

## Source association

実際の [originating capture](server.md#originating-native-open-buffer-capture) を使い、元 source/map object と URI/version/raw text/open incarnation の関係を保つ。BuildSnapshotId は正規化済み build input の identity であり、文書版/raw loading map/open incarnation/publication generation の identity ではない。有効な版のみの変更と正規化等価な raw 編集でも、build id が等しいまま旧 editor association を無効化する。同一 tuple の reopen も新しい incarnation である。

各 read は終了まで一つの immutable association を保持する。置換後の read は新 association を選び、実行中 read は完了まで元の値と retention を保つ。新旧の map/診断/version/metadata を混ぜない。current 配送前には、既存 native guard に加え、正確な現在文書 tuple と元 incarnation を検証する。保存 id や retained handle でこの検査を代替しない。

## Freshness and replacement

current/stale_artifact/pending は [architecture12](../../architecture/ja/12.diagnostics_and_lsp.md#開いているバッファはディスク上のソースを上書きする) の既存の意味に従う。current は現在 editor source から計算した応答を表し、proof acceptance ではない。Pending は実際に scheduled/running な verification を必要とする。producer input 不在は unavailable のままとし、予定済み作業や空の成功 snapshot を示さない。

current な syntax-only overlay は完全 artifact より前に存在できるが、semantic metadata や coherent な完全 LspSnapshot を与えない。change/close/reopen で旧 overlay の配送を直ちに抑止し、close は open association を除去する。shutdown/exit 後に新しい current 配送を行わない。新しい lifetime handle の query で旧 overlay を付け替えない。

保持された artifact read は編集後も stale_artifact として残せる。元の source association と明示的 stale status を保持し、unsafe edit と、target を検証できない current-buffer 配送を抑止する。read-only な stale range は元の retained source と明示的 stale status に結び付けたまま使え、current-buffer range として付け替えない。producer work や current map が無いだけで診断を clear しない。publication/conversion の詳細は task8、scheduling/cancellation は task10 が所有する。

完全 LspSnapshot は [internal03](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md#lsp-スナップショットの公開) の coherent completed build 境界と元 manifest-backed metadata を必要とする。manifest を先に読み、一つの完全 immutable set を保持してから handler-visible snapshot を atomic に置換する。ArtifactCommitted のみ、Frontend 完了、診断 index ではこの境界を確立しない。現 Driver event enum に coherent SnapshotPublished producer はなく、完全 snapshot の構築/起動は担当 owner の真正な境界と payload がそろうまで保留する。

## Currentness and retention

[Driver lane/request publication](../../mizar-driver/ja/request.md) と [IR publisher currentness](../../mizar-ir/ja/publisher.md#snapshot-and-work-unit-validation) を区別する。SnapshotHandleRegistry は identity/lineage を登録し、currentness を持たない。build id が異なる場合は、owner が許可した publisher replacement で旧 id を obsolete にできる。id が等しい editor edit でも editor association を無効化し、等しい id に IR replace_current_snapshot を呼んだり obsolete id を復活させたりしない。

LSP 所有の Session lease は SnapshotRegistry により、実際の PublishedLspSnapshot/OpenBufferOverlay retention reason で取得/解放する。ActiveBuild lease は captured build owner が所有し、値の clone は cleanup 権限を移さない。read に必要な実 sealed IR output のみ IrStorageService retain/release で保持する。Session lease だけで IR storage は pin されない。published/overlay association または実行中 reader が必要とする間は retention を保ち、両方が終了してから所有 retention を解放する。collection は上流 storage 規則に従う。retention は stale read を許し、current publication や artifact 権限を与えない。

## Implementation boundary

task6 はこの設計のみを固定する。task7 はこの association に基づく buffer/version registry、task8–11 は診断/scheduling/lifecycle cleanup を実装する。各契約は実上流 input に基づく具体 API を選び、仮の完全 snapshot、metadata index、manifest、event producer、runtime identity を作らない。ここでは新 public type/signature を選定しない。

## Test design

実受理 open/change/close/reopen/terminal message と protected native capture を使う。同じ build id を共有する版のみの編集と正規化等価な raw 編集、同一 tuple の reopen、別 Server を検査する。いずれも旧 current association を抑止しつつ、有効な retained read を保つ。置換をまたぐ reader は完全な新旧どちらか一つの association を観測する。Session lease と IR retain/release/collection は別々に実 API で確認し、stale-readable だが publisher-obsolete な output と等しい id の置換拒否を含める。syntax-only 成功から完全 metadata/current-build completion を作らず、producer work の不在/部分実行から診断 clear や pending work を作らない。既存 language/producer/mapper/lifecycle matrix は不変とする。
