# Task WEB-PLAYGROUND: hosted web playground
Canonical language: English; [Japanese pointer](../ja/WEB-PLAYGROUND.md).
Status: planned. Tier: full. Owners: [lsp todo](../../mizar-lsp/en/todo.md) (editor protocol) and [driver plan](../../mizar-driver/en/00.crate_plan.md) (server sessions); the service lives outside the Rust workspace.
Dependencies: EDITOR-LSP-ENTRY; DIST-TOOLCHAIN-MANAGER for server images; a published standard library.
Authority: specification §23.6 (LSP); [architecture 12](../../architecture/en/12.diagnostics_and_lsp.md).
Gap: `design_drift`; tutorials need a zero-install environment.

## Scope

Following the Lean 4 web editor model: a browser editor that speaks LSP over
WebSocket to a per-session, sandboxed `mizar lsp` with the standard library
preinstalled; limits on CPU, memory, wall time, and ATP time per session;
code shared through the URL; an example selector for tutorial material.
Browser-side WebAssembly is out of scope.

## Forbidden

Semantics in the web layer; unbounded server resources; persisting user code
without consent; a separate verification path.

## Tests and exit

End-to-end editing with diagnostics, resource-limit enforcement, sandbox
escape attempts, URL sharing, and concurrent sessions. Require
specification, test-sufficiency, implementation, volume/scope, and
consistency reviews. Exit: a tutorial example opens from a URL and verifies
in the browser within the session limits.
