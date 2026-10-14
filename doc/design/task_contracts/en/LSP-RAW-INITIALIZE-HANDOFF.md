# Task LSP-RAW-INITIALIZE-HANDOFF: raw client initialize handoff
Canonical language: English; [Japanese pointer](../ja/LSP-RAW-INITIALIZE-HANDOFF.md).
Status: implemented for raw client input transport. Tier: full. Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md), task 32; consumer: Driver/native editor host. Dependency: existing lifecycle server.
Authority: spec §§1/23.6, architecture12/internal03 and existing lifecycle tests. Gap: source_gap; accepted initialize params are discarded before host input binding.
Scope: [sole lifecycle owner](../../mizar-lsp/en/server.md) defines immutable raw params transport by reusing the existing initialization state; no additional carrier, field, module or dependency.
Tests: append lifecycle/framed consumers in existing server.rs; exact null/string root, arbitrary options/capabilities/unknown data, failed/duplicate/post-terminal initialize, raw documents and unchanged responses/errors.
Forbidden: protocol acceptance/validation/diagnostic changes, URI/package/root selection, effective settings/configuration interpretation, filesystem or compiler calls, fake snapshots/currentness/diagnostics, protected spec/.miz/expectation/trace/ledger edits.
Audit/exit: chapter23 stays partial; borrow exact accepted params after initialization without mutation or runtime authority. ENTRY, workspace/settings/build/current diagnostics and Steps6–8 remain open.
Reviews: independent specification/test/code/volume/consistency and scoped gates; parent cargo fmt --check, full Clippy -D warnings and cargo test.
