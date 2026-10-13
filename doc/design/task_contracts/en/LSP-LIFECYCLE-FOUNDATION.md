# Task LSP-LIFECYCLE-FOUNDATION: range and server lifecycle foundation
Canonical language: English; [Japanese pointer](../ja/LSP-LIFECYCLE-FOUNDATION.md).
Status: complete. Tier: full. Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md).
Scope: existing LSP tasks 1–5; consumers: editor bridge and driver stdio entry.
Authority: specification §§1/23.6; [architecture 12](../../architecture/en/12.diagnostics_and_lsp.md), [internal 03](../../internal/en/03.diagnostics_model_and_lsp_bridge.md), session source-map ownership and LSP 3.17.
Dependencies: existing session/lexer range APIs and workspace serde_json; no semantic or proof producer is required for lifecycle messages.
Resolved gaps: lint/owners/lifecycle and range uinteger drift. Later snapshots/diagnostics/build stay independent.

Add paired plans/range/server owners and workspace lint opt-in/guard. Preserve
range APIs/errors, enforce uinteger and test independent boundaries/fixtures/loaded CRLF/LF.
Implement [server](../../mizar-lsp/en/server.md): initialize, full sync, shutdown/exit and framed I/O over serde_json.
Public surface: `Server: Default`, `handle_message(Value) -> Option<Value>`,
`document(&str) -> Option<(i32, &str)>` and
`serve(&mut impl BufRead, &mut impl Write) -> io::Result<i32>`.
Use tuple/string/JSON types and owner buffer/capability rules, without separate document/request/response types.

No inverse/raw-editor/acquisition policy, invented compiler identity/snapshot,
semantic/proof/artifact authority, network endpoint or CLI. Tasks 6–11 and
EDITOR-LSP-ENTRY retain their producer dependencies.
Test lifecycle/IDs, actual full-sync buffers and invalid/duplicate/stale messages;
Unicode framing, fragmentation/truncation/limits, malformed messages, unsupported
methods/charset, silent notifications and I/O failure. Preserve `.miz`, expectations,
trace and proof policy; verify accepted range-boundary properties.
Audit §23.6 stays partial with range/server design added; SCA-003 remains open.
Require independent specification, test-sufficiency, implementation,
volume/scope and consistency reviews; cargo fmt --check;
cargo clippy --all-targets --all-features -- -D warnings; cargo test.
Exit: tasks 1–5 hold; current-build diagnostics, semantic snapshots and editor/Step 8 gates remain open.
