# Server lifecycle
Canonical language: English; [Japanese companion](../ja/server.md).
Authority: architecture 12/internal 03 and [LSP 3.17](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/).

Existing serde_json decodes minimal JSON-RPC; LSP owns framing/lifecycle/buffers, with no new protocol dependency.
`Server: Default` owns one connection. `handle_message(Value)` returns an
optional response; `document(uri)` borrows its `(i32 version, text)` buffer;
`serve(BufRead, Write)` runs framed traffic and returns the exit code.

Initialize once with object params containing processId (i32/null), rootUri
(string/null) and object capabilities. Unknown capabilities/options are ignored.
Reply with UTF-16 position encoding and openClose/full text sync only. Before
initialization, requests get -32002 and notifications are ignored except exit.
Duplicate initialize and requests after shutdown get -32600; notifications
after shutdown are ignored except exit. Shutdown/exit accept absent/null params;
shutdown replies null without exiting; exit returns 0 after shutdown and 1 otherwise. Unknown
requests get -32601. Invalid envelopes/IDs get -32600, invalid request params
-32602; integer IDs fit i32, string IDs are retained, and valid notifications
never receive responses. Unmatched responses and $/cancelRequest notifications
are ignored; $/ requests get -32601. Request-only methods sent as notifications
are ignored; notification-only methods sent as requests get -32600.
`initialization_params(&self) -> Option<&serde_json::Value>` borrows the complete accepted initialize params, retained in the existing initialization state instead of a boolean. Before successful initialization return None; commit only after all existing validation, preserve the first accepted value through duplicate/failed/post-shutdown/post-exit traffic, and retain exact JSON values for rootUri, capabilities, options and unknown fields. Raw retention changes no protocol response, validation, buffer acceptance or unknown-option interpretation, and performs no URI conversion, filesystem access, settings selection or compiler/currentness/publication operation.

Open requires a new nonempty URI key, string languageId/text and i32 version.
Change requires an open key and a strictly newer i32 version; validate all
events before mutation, apply full texts in order, reject range/rangeLength,
and let an empty event list retain text while advancing version. Close removes
the key. Invalid, duplicate, unknown or stale notifications leave buffers intact.
Buffers retain exact client text; they are not compiler snapshots or artifacts.

Framing requires ASCII HTTP-field CRLF headers, one Content-Length and UTF-8 JSON bytes.
Bound headers to 8 KiB and bodies to 16 MiB before allocation. Content-Type is
optional; accept application/vscode-jsonrpc with utf-8 or legacy utf8 charset.
Complete frames with unsupported type/charset or invalid JSON get -32700/null
and processing continues. Invalid framing, truncation, early EOF and I/O failure
return io errors. Responses use byte lengths and flush without log text.

## Native initialize workspace inputs

`Server::native_workspace_inputs(&self) -> Result<Option<(WorkspaceRoot, Vec<WorkspacePackage>, Lockfile)>, String>` performs explicit caller-invoked metadata acquisition from the first accepted initialize rootUri. Before initialization or for null return Ok(None); never select cwd, parents, rootPath, workspaceFolders or settings as fallback. Protocol validation/responses, retained params and document buffers stay unchanged, including after terminal traffic.
Decode a present string with Session's existing file_path_from_document_uri. Require a decoded absolute path and ordinary directory using standard Path components before the metadata query, then canonicalize and require UTF-8 for the returned WorkspaceRoot. Unsupported URI/path, unavailable or nonordinary root is String Err. Reuse Build read_native_workspace_inputs for the actual parsed member manifests and mandatory lock, returning only the complete existing tuple; propagate acquisition/parser errors without partial success or synthesized metadata.
Each explicit call reacquires metadata; store no root/context and do not read source bodies or allocate IDs. These point-in-time mutable records establish no atomic provenance, effective settings, selected toolchain/dependencies, snapshot, compiler work or current publication. Full native host binding remains separate.
Tests use actual accepted initialize values and real root/nested-member metadata, compare existing parsed coordinates, preserve raw params/documents and unchanged responses, and distinguish null/uninitialized, malformed/non-file URI, missing/file/symlink roots and native-reader failure without fallback. Metadata edits/deletion after a prior call must be observed; do not repeat the existing decoder/parser/protocol matrices.
