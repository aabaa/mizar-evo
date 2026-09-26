# Task EDITOR-LSP-ENTRY: `mizar lsp` stdio entry point
Canonical language: English; [Japanese pointer](../ja/EDITOR-LSP-ENTRY.md).
Status: planned. Tier: full. Owners: [driver plan](../../mizar-driver/en/00.crate_plan.md) (CLI) and [lsp todo](../../mizar-lsp/en/todo.md) (server); consumers: editor clients.
Dependencies: mizar-lsp tasks 4-5 (server lifecycle and the transport decision).
Authority: specification §1 (LSP support for editors), §23.6 (LSP reads build artifacts); [architecture 12](../../architecture/en/12.diagnostics_and_lsp.md); [driver CLI](../../mizar-driver/en/cli.md#command-surface).
Gap: `design_drift`. The LSP server is planned, but no task defines how an editor starts it; the CLI implements only `mizar build`.

## Scope

Add a `mizar lsp` subcommand that runs the mizar-lsp server over standard input
and output. The workspace root and settings come from the LSP `initialize`
request; the subcommand reuses the existing driver session and request API and
owns no language semantics. Document the subcommand in the CLI owner document.

## Forbidden

Network transports; LSP feature logic in the driver; a second build pipeline;
editor-specific configuration; specification changes.

## Tests and exit

An initialize, open, diagnostics, and shutdown round trip over stdio against a
real workspace; malformed input, early exit, and cancellation. Require
independent specification, test-sufficiency, implementation, volume/scope, and
consistency reviews; `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
Exit: an LSP client can start the server with `mizar lsp` and receive the
diagnostics of the current build.
