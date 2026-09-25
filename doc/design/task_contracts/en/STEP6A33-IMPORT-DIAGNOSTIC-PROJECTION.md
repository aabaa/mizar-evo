# Task STEP6A33-IMPORT-DIAGNOSTIC-PROJECTION: semantic import reporting policy
Canonical language: English; [Japanese pointer](../ja/STEP6A33-IMPORT-DIAGNOSTIC-PROJECTION.md).
Status: implemented (design only). Tier: full. Primary owner: [resolver plan](../../mizar-resolve/en/00.crate_plan.md). Consumers: future diagnostics registry and driver resolver service.
Purpose: specify E0220–E0225 draft cardinality, retained-source locations and fail-closed conversion before public adoption.
Authority: [spec §12.3](../../../spec/en/12.modules_and_namespaces.md#123-import-statements), [spec §22.3.5](../../../spec/en/22.error_handling_and_diagnostics.md#2235-module-import-resolution), and [resolver imports owner](../../mizar-resolve/en/imports.md#public-diagnostic-projection-prerequisites).
Readiness: typed import outcomes exist; complete workspace lexical summaries, authentic semantic input transport for negative ASTs, registry descriptors and a public bridge do not. Spec-root `mml` differs from the current `std` resolver root.
Gaps: `spec_gap` for reporting policy closes here; `source_drift` for the root seam and `external_dependency_gap` for real public emission remain.
Scope: paired spec 22 reporting policy, paired imports owner source/test prerequisites, paired architecture pointer, resolver plan links, todo and chapter-22 coverage audit. No new language meaning beyond the A32 allocation.
Tests: existing [typed import tests](../../../../crates/mizar-resolve/src/imports/tests.rs) and E0221/E0223 source fixtures inform the design; future real parser/index fixtures must cover all six, branches, aliases, SCC/self-cycles, provenance and invalid batches. Lower-stage fixtures do not establish public emission.
Forbidden: Rust/registry/API, `.miz` or expectation changes, frontend recovery/publication changes, synthetic workspace completion, and public diagnostic coverage claims.
Coverage audit: chapter 22 remains partial; reporting design does not activate producer or bridge coverage.
Require independent specification/EN-JA, test-intent, design implementation, volume/scope and consistency reviews; local link/allocation checks, `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test` before finalization.
Exit: six reporting policies and source/test prerequisites are documented consistently; emission remains deferred.
