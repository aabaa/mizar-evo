# Task STEP6A37-DIAGNOSTIC-IMPORT-CONTINUATION: diagnostic-only import continuation
Canonical language: English; [Japanese pointer](../ja/STEP6A37-DIAGNOSTIC-IMPORT-CONTINUATION.md).
Status: implemented (design only). Tier: full. Owner: [driver plan](../../mizar-driver/en/00.crate_plan.md); frontend, IR and resolver retain their existing authorities.
Purpose: define restricted import diagnostics from authentic E0022-only frontend input while preserving failed build and semantic acceptance boundaries.
Authority: [spec 22.2.3/22.3.5](../../../spec/en/22.error_handling_and_diagnostics.md), spec 23.4; architecture 14/19 failure propagation; existing [frontend service](../../mizar-driver/en/frontend_adapter.md#disk-frontend-service) and [parsed candidates](../../mizar-resolve/en/imports.md#parsed-import-candidates).
Gap: `spec_gap` for diagnostic-only admission/co-reporting/continuation policy; runtime transport remains an `external_dependency_gap`.
Scope: paired spec 22, paired architecture 14/19 pointers, paired driver continuation ownership/test design, driver plan links, todo and chapter-22 audit.
Details and future test matrix: [driver owner](../../mizar-driver/en/frontend_adapter.md#diagnostic-only-import-continuation-design); existing real fixture: [source_load.rs](../../../../crates/mizar-driver/tests/source_load.rs).
Forbidden: Rust/API/registry, .miz/expectation/trace changes, recovery expansion, normal semantic dispatch or acceptance, cache/artifact credit, fabricated workspace summaries and E0225 continuation. No runtime coverage activation.
Audit: chapter 22 remains partial; admission policy is specified, implementation and complete graph/summary inputs remain deferred.
Require independent specification/EN-JA, test-intent, design implementation, volume/scope and consistency reviews; local links, cargo fmt --check, cargo clippy --all-targets --all-features -- -D warnings, cargo test.
Exit: bounded admission, retained diagnostics and failed-build semantics are specified consistently; implementation is a separate task.
