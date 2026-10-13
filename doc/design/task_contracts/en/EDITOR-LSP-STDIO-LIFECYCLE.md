# Task EDITOR-LSP-STDIO-LIFECYCLE: executable lifecycle prerequisite
Canonical language: English; [Japanese pointer](../ja/EDITOR-LSP-STDIO-LIFECYCLE.md).
Status: implemented. Tier: full. Owners: [LSP plan](../../mizar-lsp/en/00.crate_plan.md) (31, executable host), [Driver plan](../../mizar-driver/en/00.crate_plan.md) (28, CLI); consumers: editor clients.
Dependencies: implemented LSP tasks 4–5 and existing Driver CLI parser. Readiness: lifecycle/framing can run without semantic or artifact producers.
Authority: specification §§1/23.6; internal [crate ownership](../../internal/en/07.crate_module_layout.md#crate-ownership); [CLI host boundary](../../mizar-driver/en/cli.md#stdio-lifecycle-host); [server lifecycle](../../mizar-lsp/en/server.md).
Gap: executable/process `source_gap` resolved by this lifecycle boundary; full editor entry still needs genuine workspace/settings, build and current-diagnostic bindings.

Scope: compose the existing owners in a real `mizar` executable hosted by LSP with a Driver dependency; implement the CLI host boundary without changing library APIs or protocol behavior.
Affected sources: LSP Cargo manifest/lock, necessary `src/main.rs` executable root and reusable `tests/stdio.rs` process suite; update paired CLI/plan/TODO owners only as required.
Forbidden: new crates, public Rust seams, single-use helper files/types/adapters, copied protocol logic, fake build inputs/statuses/diagnostics, semantic capabilities, network transport or protected specification/test/acceptance changes.
Deferral: full EDITOR-LSP-ENTRY, Driver23/LSP26, LSP6–11 and Step8 remain open; executable lifecycle is no current-build diagnostic or verification claim. §23.6 stays partial; no coverage owner changes.

Tests: actual Cargo executable and OS pipes under a real workspace, interactive response flush before closing stdin, lifecycle/full-sync/cancel notifications, shutdown/early exit, malformed/truncated/EOF input, invalid/non-Unicode arguments and unavailable build bootstrap; assert framed-only stdout and bounded process cleanup.
Reviews: independent specification, test sufficiency, implementation, volume/scope and consistency. Parent verification: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
Exit: the real process follows the existing lifecycle and CLI host boundary with genuine stdin/stdout and no invented producer outputs.
