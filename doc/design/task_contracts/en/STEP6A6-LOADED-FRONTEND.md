# Task STEP6A6-LOADED-FRONTEND: consume loaded source
Canonical language: English; [Japanese pointer](../ja/STEP6A6-LOADED-FRONTEND.md).
Status: implemented. Tier: full. Owner: [frontend plan](../../mizar-frontend/en/00.crate_plan.md); dependency: STEP6A5; consumer: real Frontend phase service.
Authority: [architecture 02](../../architecture/en/02.source_and_frontend.md); [spec 23.4](../../../spec/en/23.package_management_and_build_system.md); [orchestration API and invariants](../../mizar-frontend/en/orchestration.md).
Gap: Frontend currently reloads source internally; expose its existing post-load pipeline for actual SourceUnit parents, without a new loader adapter or duplicated pipeline.
Scope: existing orchestration source/tests and paired owner docs; add `Frontend::run_loaded(SourceUnit)` and make `run` delegate after its unchanged load/error handling.
Preserve source identity/maps, parser/provider behavior, recovery, diagnostics and cache keys. Caller owns snapshot validation; this method neither loads files nor allocates source ids.
Test real disk-loaded sources with MizarParserSeam: ordinary/recovered output parity, deleted-file reuse, unchanged load failures, provider/span failures; reuse existing fixtures. No corpus/spec/expectation changes.
No publication/codec/cache integration credit, new phase registration, Step 7 or MVM; coverage audit stays partial. Later codec/diagnostic/provider seams remain prerequisites.
Require independent specification, test, implementation, volume/scope and consistency reviews; fmt, all-target/all-feature clippy -D warnings, cargo test. Exit: both entries share one real pipeline and preserve existing behavior.
