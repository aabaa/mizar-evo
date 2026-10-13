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

open は未登録の非空 URI key・string languageId/text・i32 version を要求する。
change は open key と増加する i32 version を要求し、全 event を先に検査して full text を順に適用する。
range/rangeLength は拒否し、空 event list は text を保って version を進める。close は key を削除する。
不正・重複・未知・古い notification は buffer を変えない。client text を正確に保持し、compiler snapshot/artifact にはしない。

ASCII HTTP-field CRLF header、一意の Content-Length、UTF-8 JSON byte を要求する。
確保前に header 8 KiB・body 16 MiB に制限する。任意の Content-Type は
application/vscode-jsonrpc と utf-8/旧 utf8 charset を許す。完全 frame の未対応 type/charset・
不正 JSON は -32700/null を返して継続する。framing 不正・切断・早期 EOF・I/O 失敗は io error。
応答長は byte 数で、flush し log text を混ぜない。
