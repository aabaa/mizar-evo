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
