# Server lifecycle
正本: [英語版](../en/server.md)。
権威: architecture 12/internal 03 と [LSP 3.17](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/)。

既存 serde_json で decoding する最小 JSON-RPC layer とし、framing・lifecycle・buffer 更新は LSP の責務、追加依存は不要である。
`Server: Default` は1接続を所有する。`handle_message(Value)` は optional response、
`document(uri)` は借用 `(i32 version, text)`、`serve(BufRead, Write)` は framed traffic と exit code を返す。

initialize は1回で、params に processId (i32/null)、rootUri (string/null)、object capabilities を要求する。
未知の capability/option は無視し、UTF-16 と openClose/full text sync だけを応答する。
初期化前の request は -32002、notification は exit 以外無視する。重複 initialize と
shutdown 後の request は -32600、notification は exit 以外無視する。shutdown/exit の params は省略/null。
shutdown は null 応答のみで終了せず、exit は shutdown 後0、その他1。
未知 request は -32601、envelope/ID 不正は -32600、request params 不正は -32602。
整数 ID は i32、文字列 ID は保持し、有効 notification に応答しない。未対応 response と
$/cancelRequest notification は無視し、$/ request は -32601 とする。request 専用 method の
notification は無視し、notification 専用 method の request は -32600 とする。
`initialization_params(&self) -> Option<&serde_json::Value>` は、boolean の代わりに既存の初期化状態に保持した受理済 initialize params 全体を借用する。初期化成功前は None とし、既存の全検査後だけ保持し、重複・不正・shutdown/exit 後の traffic でも最初の受理値を保つ。rootUri・capabilities・options・未知 field の JSON 値を正確に保持する。protocol 応答・検査・buffer 受理・未知 option の解釈を変えず、URI 変換・filesystem access・設定選択・compiler/currentness/publication 操作を行わない。

open は未登録の非空 URI key・string languageId/text・i32 version を要求する。
change は open key と増加する i32 version を要求し、全 event を先に検査して full text を順に適用する。
range/rangeLength は拒否し、空 event list は text を保って version を進める。close は key を削除する。
不正・重複・未知・古い notification は buffer を変えない。client text を正確に保持し、compiler snapshot/artifact にはしない。

ASCII HTTP-field CRLF header、一意の Content-Length、UTF-8 JSON byte を要求する。
確保前に header 8 KiB・body 16 MiB に制限する。任意の Content-Type は
application/vscode-jsonrpc と utf-8/旧 utf8 charset を許す。完全 frame の未対応 type/charset・
不正 JSON は -32700/null を返して継続する。framing 不正・切断・早期 EOF・I/O 失敗は io error。
応答長は byte 数で、flush し log text を混ぜない。

## Native initialize workspace inputs

`Server::native_workspace_inputs(&self) -> Result<Option<(WorkspaceRoot, Vec<WorkspacePackage>, Lockfile)>, String>` は最初に受理した initialize rootUri から、呼出側が明示的に native metadata を取得する。初期化前または null は Ok(None) とし、cwd・親・rootPath・workspaceFolders・settings へ fallback しない。terminal traffic 後も protocol 検査/応答、保持 params、文書 buffer を変更しない。
文字列は Session の既存 file_path_from_document_uri で decode する。metadata query 前に標準 Path components を使い、decode 結果が絶対 path かつ通常 directory であることを要求した後 canonicalize し、返す WorkspaceRoot が UTF-8 であることを確認する。非対応 URI/path、取得不能、通常 directory でない root は String Err。Build read_native_workspace_inputs を再利用して実 member manifest と必須 lock を parse し、既存 tuple 全体だけを返す。取得/parser error は部分成功や metadata 補完なしで伝える。
明示呼出しごとに metadata を再取得し、root/context を保存せず、source 本文読取りや ID 割当を行わない。一時点の可変 record は atomic provenance、実効設定、選択済 toolchain/dependency、snapshot、compiler work、current publication を証明しない。完全 native host binding は別の責務に残す。
実 initialize と実 root/入れ子 member metadata のテストで既存 parse 座標、raw params/文書と応答の保持、null/初期化前、不正/非 file URI、欠落/file/symlink root、native reader 失敗時の無 fallback を確認する。前回呼出し後の metadata 編集/削除を反映し、既存 decoder/parser/protocol matrix は重複しない。

## Native frontend diagnostic notifications

`Server::submit_native_frontend_diagnostics<A: SessionIdAllocator, L: SourceLayoutProvider>(&self, driver: &mut CompilerDriver, session: BuildSession, allocator: &A, snapshots: &SnapshotRegistry<A>, input: DriverSubmitInput<L>) -> Result<(BuildSubmission, Option<Vec<serde_json::Value>>), DriverSubmitError>` は既存 submit_captured_with_import_discovery_and_native_diagnostic_index を先に呼び、通知が不適格でも通常 error/submission を保持する。capture 由来 discovery と指定済み overlay の扱いは Driver が所有し、notification・originating・writer はこの producer を共有する。外部 index/batch/origin/map は受け取らない。明示的な全 OpenBuffer の lightweight Frontend 出力であり、完全 build publication ではない。
初期化済みで shutdown/exit 前の Server、obsolete draft のない実 Some index、変換前後の live native Driver guard を要求する。Driver 自身の保存 session/capture の native_sources を読み、registry.get や caller の差し替えを使わない。非空の全 Open 元 tuple、重複しない SourceId/URI、現 Server の URI/version/raw text 完全一致を要求する。expected/actual version と loaded map は既存 native/mapper 規則に従い、新 version 方針や本文再読込を加えない。
各元 tuple に対し、実 submission の kind Frontend・phase list [Frontend]・同 package/module work unit の task が唯一存在し、その task の元 PhaseResult が一つあることを要求する。Complete/Recoverable/Fatal のみ対応し、当該 SourceId の primary 診断 span が index に無ければ clean とする前に Complete を要求する。graph/run/task/result 欠落、曖昧な束縛、Blocking/Cancelled、診断なし非 Complete は通知全体を None とする。他 source の非空 index は当該 source の検査実行を保証しない。
全 record は実 snapshot の Current、Frontend phase、唯一の capture source に属する primary span を要求し、不適合なら通知全体を None とする。元 loaded LineMap/LoadingMap と SourceOriginInput で既存 lsp_range_from_open_buffer を呼び、URI 内は Diagnostics の record 順を保つ。独自 sort/dedup/identity 割当はしない。Disk/Generated/混在、Resolver/他 phase、非対応 location/freshness/map/severity、欠落/close/stale 文書、guard 失敗はいずれも部分通知なしの None。
URI 順に、各 capture URI へ owned JSON-RPC 2.0 textDocument/publishDiagnostics を一つ返す。params は uri、capture の i32 version、diagnostics で、clean 文書は空配列。各診断は変換 range、元 code 表示文字列/message と Error=1/Warning=2/Info=3 severity を持つ。optional related-information/data/tag や compiler status は捏造しない。[LSP 3.17 notification](https://github.com/microsoft/language-server-protocol/blob/gh-pages/_specifications/lsp/3.17/language/publishDiagnostics.md) と [Diagnostic](https://github.com/microsoft/language-server-protocol/blob/gh-pages/_specifications/lsp/3.17/types/diagnostic.md) 形状を使い、code/message/identity/集約は既存 Diagnostics に残す。
実 initialize/open/change/close、既存 native index 対応付けと protected capture、実 SourceLoad/Frontend と元 index を接続する。discovery 付き submission 全体、元診断 field、独立した raw UTF-16 range、clean clearing、実複数 Open source を比較し、既存の実 dispatch NoKey control を再利用する。import は元 editor text と矛盾／利用不能な disk 本文で検査し、実 lexical 依存順序、完全 semantic coverage ではない ImportsOnly、指定済み overlay の保持、discovery 失敗 control、notification/originating/writer の配送と元 incarnation の抑止を確認する。同本文の新版、正規化等価な raw 変更、LoadingMap がない URI 別名、stale Driver、実非対応 origin/phase を他条件が有効な control で区別し、既存 protocol/stdio/producer/mapper テストを保つ。他条件が current な実 SourceLoad publication 拒否で未実行 Frontend を clean clearing にせず、他 source の実診断があっても clean source の Frontend 拒否なら通知全体を保留する。
返す message は呼出し中の current raw tuple 一致のみを示す。close/reopen の lifecycle identity、IR retain、完全 settings/toolchain/dependency/file freshness、VerifiedArtifact、PublishedLspSnapshot は示さない。ここでは送信せず、後の wire handoff が再検査する。実行 host/settings/build scheduling/cancel/完全 current diagnostics は別途とし、この notification 操作は lifecycle／文書 state を変更せず、[open 寿命 handle](#open-document-lifetimes) は Server lifecycle が所有する。

## Native frontend diagnostic wire

`Server::submit_native_frontend_diagnostics_to_writer<A: SessionIdAllocator, L: SourceLayoutProvider>(&self, driver: &mut CompilerDriver, session: BuildSession, allocator: &A, snapshots: &SnapshotRegistry<A>, input: DriverSubmitInput<L>, output: &mut impl Write) -> Result<(BuildSubmission, io::Result<usize>), DriverSubmitError>` は既存 notification producer を一度だけ呼ぶ。通常の submission と構造化 Driver error を保持し、outer error は書込みなし、通知対象外の None は実 submission と inner Ok(0) を返す。代替 messages/diagnostics/capture は受け取らない。
生成から即時書込みまで同じ immutable Server borrow を保持するため、producer の初期化/終了状態と元 URI/version/raw text 検査は有効なままであり重複実装しない。各 frame header の直前、serialize 後に、同じ保存 session と registry の validate_current_native_diagnostics を要求する。guard 不成立なら後続を止め、inner Ok(完全 flush 済み件数) を返す。0 は送信 frame なしを意味する。registry.get で元 raw capture を再構成せず、値の置換や phase 再実行もしない。
元通知を UTF-8 JSON にし、byte 数の Content-Length と CRLF/CRLF、body 全体、flush の順に書いてから件数を増やす。順序と元 field を保ち、既存 framing 操作を inline に使う。production 二箇所だけの helper は作らない。最初の serialize/header/body/flush error は実 submission を保持した inner Err とし、直ちに止め、部分 bytes の retry/rollback は行わない。Write callback が共有 publisher/registry を無効化しても既に開始した bytes は取り消せず、次 frame は新しい guard を通す。これは handoff 時点の検査であり atomic currentness や PublishedLspSnapshot ではない。
三つの実 native consumer 系統で、error/clean frame と独立した byte/range/version 期待値、正常 control と送信抑止/Driver error、I/O/部分送信/currentness を検証する。private setup helper/Write adapter は各三系統の実 call site を必要とする。既存通知/protocol/stdio/producer tests は不変とし、条件付き lightweight Diagnostics consumer の範囲も変えない。自動 host/serve/main 接続、完全な runtime/settings/toolchain/dependency、scheduler cancellation、IR retention、完全な current-build 診断は保留する。

## Open document lifetimes

`Server::document_lifetime(&self, uri: &str) -> Option<std::sync::Weak<()>>` は生存中の接続で現在 open の正確な URI に対するプロセス内 handle を返す。未登録／close 済み／shutdown・exit 後は None とする。既存 private 文書 tuple の第3要素に標準 Arc<()> を持ち、既存 didOpen 検証の成功時だけ生成して downgrade を返す。状態を変更せず、document(uri) の version/text を保つ。
同じ open は空変更を含む有効 didChange で同じ allocation を保ち、不正／重複／古い／無視する message でも置換しない。didClose は項目を削除し、同じ URI/version/text の再 open でも新しい allocation とする。別文書・Server は異なる allocation を持つ。保持 handle と新しい query の比較には Weak::ptr_eq だけを使い、内容 equality・pointer 数値・参照数・upgrade 成功から現在性を導かない。直列化／数値／hash identity や caller handle の採用は作らない。
handle が示すのは Server の open 寿命だけで、現在の版／text・compiler capture・snapshot・診断資格・runtime・artifact は示さない。既存 notification/wire・protocol/stdio 動作を保ち、実 capture と寿命の結合は[明示的な別経路](#originating-native-open-buffer-capture)で行う。既存 message helper と実装前の独立関係 trace により、反復／変更・複数文書、同一 tuple の close/reopen・Server 分離、不正／無視 traffic・shutdown/exit と raw state 保持の3系列を確認する。全 protocol/native matrix の複製や test adapter／型／helper は加えない。

## Originating native open-buffer capture

`NativeOpenBufferCapture` は元の BuildSession と `(String URI, Weak<()>)` の列だけを private に持つ LSP の envelope とする。public constructor/field/Clone/accessor/conversion は持たず、以下の実3操作で関係を作成・消費する。source/map/診断の複製や Server state の追加は行わない。
`Server::capture_native_open_buffers<A: SessionIdAllocator>(&self, draft: BuildRequestDraft, allocator: &A, snapshots: &SnapshotRegistry<A>, index: &ModuleIndex, sources: Vec<(DiskSourceLoader, PackageId, String)>) -> Result<NativeOpenBufferCapture, String>` は初期化済み生存 Server と、重複のない正確な URI key の非空列を要求する。同じ immutable borrow で各現在文書 entry を一度読み、未登録/close 済みや負の i32 version は protocol 受理を変えず拒否する。その entry の URI/raw text と非負を確認した i64 version（expected/actual 両方）から OpenBuffer origin を作り、同じ操作で元 Arc を downgrade する。draft.workspace_root と明示 package/loader により既存 index.native_open_buffer_input を呼び、全座標の検査後に draft.capture_sources を一度だけ呼ぶ。association の String error を伝え、capture error は Debug で String にし、部分成功や source/context の補完をしない。結果の元 session と元 URI/Weak 関係だけを保持し、古い capture や caller の lifetime/origin を受け取らない。実 index/plan、context、allocator/registry、publisher は caller の責務である。
`submit_originating_native_frontend_diagnostics` は既存通知 signature の BuildSession を NativeOpenBufferCapture に置き換える。全保存 handle を、同じ Server の正確な URI の新しい document_lifetime と Weak::ptr_eq で比較する。元 session で既存 notification producer を一度呼び、実 submission と構造化 Driver error を保持し、lifetime 不一致/不在なら通知全体を抑止する。既存の現在 raw version/text、native capture/index/range、Driver guard は不変かつ必須であり、upgrade/参照数/内容一致に権限を与えない。
`submit_originating_native_frontend_diagnostics_to_writer` も既存 wire の session 引数を envelope に置き換える。同じ immutable Server borrow で lifetime が有効なら元 wire に一度委譲し、不適格なら元 notification を一度呼んで submission/error だけを保持し、通知を捨て、writer に触れず inner Ok(0) を返す。既存の各 frame の Driver guard、byte/順序/flush、部分件数と I/O 動作を保持し、retry/rollback や framing の複製をしない。同じ open の change は incarnation を保つが元 raw tuple 検査も必要であり、同一 tuple での reopen や別 Server は元の関係を引き継がない。
実受理メッセージ/native 座標から実 capture/SourceLoad/Frontend を使い、独立した origin/version/raw UTF-16 の期待値と通常 submission 全体を比較する。同じ incarnation の成功と、他条件が同じ reopen/別 Server の抑止を区別し、その正常 control で旧 raw-tuple API は成功することを示す。版/raw/close/terminal、不正取得、Driver guard/error、writer 失敗も保持する。既存 native setup/writer を再利用し、新 setup helper は実3系列で共有する。全旧テストと production 関数を不変にし、envelope/capture/診断/cache Hit/受理を捏造しない。
保証するのは実明示 capture の元 open incarnation だけである。完全な設定/toolchain/dependency/file freshness、coherent LspSnapshot、runtime/host scheduling/cancel、IR retain、外部 Write 中の atomic currentness は別途とし、ENTRY/SCA-003/roadmap gate の完了を示さない。
