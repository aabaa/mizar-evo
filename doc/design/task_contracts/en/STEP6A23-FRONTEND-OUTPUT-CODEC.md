# Task STEP6A23-FRONTEND-OUTPUT-CODEC: retained disk frontend output storage
Canonical language: English; [Japanese pointer](../ja/STEP6A23-FRONTEND-OUTPUT-CODEC.md).
Status: frozen. Tier: full. Owner: [frontend plan](../../mizar-frontend/en/00.crate_plan.md).
Authority: [aggregate architecture](../../architecture/en/02.source_and_frontend.md#frontendoutput), [diagnostic source spans](../../../spec/en/22.error_handling_and_diagnostics.md#2212-source-span-and-context-display); [storage owner](../../mizar-frontend/en/orchestration.md#disk-frontendoutput-storage).
Gap: derived storage dependency; existing source/AST/preprocessing/token/cache-key codecs omit aggregate composition and merged diagnostics. Ready under current language behavior.
Scope: two methods on FrontendOutput<SurfaceAst>, inline orchestration.rs implementation/tests, reuse existing phase and span codecs; paired orchestration owners, contracts, plan indexes, global todo.
Contract: complete bounded disk aggregate transport with exact framing and diagnostic vocabulary, preserved order/fields and caller source rebinding per owner. Source-load locations, non-disk origins and generic AST types are out of format.
Tests: existing real_parser_frontend_returns_ast_and_merges_diagnostics_by_phase and run_loaded regressions supply producer fixtures; add real output/fresh-ID/absent-AST round trips, fixed diagnostic typed oracles, retained nested bytes, corrupt framing/records, foreign-ID/path/origin rejection, no-file-I/O proof and size boundary controls.
Forbidden: new public types/fields/files/dependencies, producer or lower-owner API changes, recomputed diagnostics/keys, cache-hit or proof acceptance, publication or driver integration, spec/corpus/expectation/trace/audit changes, artifact task17 and Step7/MVM.
Cross-artifact provenance/bounds/producer consistency and reuse policy remain driver-owned; existing nested validation remains intact. Helpers require three real call sites.
Require independent specification, test-sufficiency, implementation, volume/scope and consistency reviews; cargo fmt --check, cargo clippy --all-targets --all-features -- -D warnings, cargo test.
Exit: complete retained disk-source SurfaceAst frontend outputs round-trip without executing producers; acceptance and publication remain subsequent work.
