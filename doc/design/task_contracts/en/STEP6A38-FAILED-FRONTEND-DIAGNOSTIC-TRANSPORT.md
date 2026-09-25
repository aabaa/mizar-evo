# Task STEP6A38-FAILED-FRONTEND-DIAGNOSTIC-TRANSPORT: failed Frontend diagnostic transport
Canonical language: English; [Japanese pointer](../ja/STEP6A38-FAILED-FRONTEND-DIAGNOSTIC-TRANSPORT.md).
Status: frozen design; runtime deferred. Tier: full. Owner: [driver plan](../../mizar-driver/en/00.crate_plan.md).
Purpose: bind live E0022-only Frontend input to restricted path diagnostics while preserving failed scheduling.
Authority: [spec 22.3.5](../../../spec/en/22.error_handling_and_diagnostics.md#diagnostic-only-continuation-after-lexical-import-failure), [driver owner](../../mizar-driver/en/frontend_adapter.md#diagnostic-only-import-continuation-design), [resolver imports](../../mizar-resolve/en/imports.md#parsed-import-candidates).
Gap: this design closes the unspecified transport gap; E0220–E0224 registry adoption, implementation and full graph dispatch remain `external_dependency_gap`.
Scope: paired driver owner transport/invocation design, this paired contract and driver plan links; no source or test changes.
Forbidden: durable failed-output publication, general Recoverable dispatch, synthetic summaries, cycles/E0225, normal semantic/cache/proof/artifact credit, public converter or new registry type.
Tests: [source service fixtures](../../../../crates/mizar-driver/tests/source_load.rs) and [scheduler fixtures](../../../../crates/mizar-driver/src/driver/tests.rs) remain future obligations; no new coverage is claimed.
Require independent document, test-intent, implementation-feasibility, scope and consistency review; run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings` and `cargo test`.
Exit: one bounded owner design with retained frontend batch, optional atomic Resolver batch and failed scheduler status; implementation remains separate.
