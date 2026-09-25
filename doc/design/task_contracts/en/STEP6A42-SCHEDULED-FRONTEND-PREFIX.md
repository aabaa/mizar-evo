# Task STEP6A42-SCHEDULED-FRONTEND-PREFIX: scheduled source/frontend execution
Canonical language: English; [Japanese pointer](../ja/STEP6A42-SCHEDULED-FRONTEND-PREFIX.md).
Status: frozen. Tier: full. Owner: [driver plan](../../mizar-driver/en/00.crate_plan.md).
Purpose: make the real source/frontend services reachable through ordinary submit without granting later-phase success.
Authority: [spec 22.3.5](../../../spec/en/22.error_handling_and_diagnostics.md#diagnostic-only-continuation-after-lexical-import-failure), [spec 23.4](../../../spec/en/23.package_management_and_build_system.md), [driver owner](../../mizar-driver/en/driver.md#scheduled-sourceloadfrontend-prefix), [source services](../../mizar-driver/en/frontend_adapter.md).
Dependencies: real SourceLoad/Frontend publication services, A41 diagnostics, existing scheduler results and IR sealed-parent dispatch bundles.
Gap: source_drift/external_dependency_gap; missing later services prevent scheduling available source/frontend work, and real parent outputs lack an automatic dispatch handoff.
Scope: restricted preflight exception for a registry containing only the two built-in descriptor identities plus a supplied current publisher; private captured-source/summary identity construction with source-key derivation owned by the frontend adapter and shared by the two services and dispatcher and exactly-one completed dependency SourceLoad parent for Frontend; retain external provider precedence and downstream owner gaps; mark ValidatedHit decisions for missing-service tasks unavailable without discarding task-id validation.
No provider: construct bundles only for the exact existing SourceLoad/Frontend descriptors. Supplied provider None/error/invalid bundle remains authoritative and never falls back.
Tests: ordinary submit with real services for clean and E0022/E022x failed sources, missing publisher/service/currentness/rights and supplied provider negatives; preserve failed dependents and absent parent on cache-hit gap. Exercise sequential/parallel prefix where existing scheduler supports it.
Forbidden: new public type/adapter, publisher authority creation, dynamic generic semantic input synthesis, ModuleResolver/E0225/graph/summary activation, cache/proof/artifact acceptance, spec or existing .miz/expectation/trace changes.
Affected: existing driver.rs/scheduler.rs/frontend_adapter.rs and source/scheduler tests; paired driver owner/registry/plan and readiness statements whose state changes.
Audit: chapters 22/23 remain partial; bounded real frontend execution is reachable through submit while later adapters and full-build completion remain deferred.
Require independent specification, test-sufficiency, implementation, volume/scope and consistency reviews; cargo fmt --check, cargo clippy --all-targets --all-features -- -D warnings, cargo test.
Exit: failed Frontend submission retains diagnostics and fails; clean prefix publishes only its existing IR outputs then blocks on later owner gaps.
