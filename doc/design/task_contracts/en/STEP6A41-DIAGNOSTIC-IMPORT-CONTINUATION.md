# Task STEP6A41-DIAGNOSTIC-IMPORT-CONTINUATION: diagnostic-only import execution
Canonical language: English; [Japanese pointer](../ja/STEP6A41-DIAGNOSTIC-IMPORT-CONTINUATION.md).
Status: implemented. Tier: full. Owner: [driver plan](../../mizar-driver/en/00.crate_plan.md); consumes frontend, resolver and shared diagnostics.
Purpose: execute the restricted import continuation without changing failed frontend/build status.
Authority: [specification 22.3.5](../../../spec/en/22.error_handling_and_diagnostics.md#diagnostic-only-continuation-after-lexical-import-failure), [driver admission/transport owner](../../mizar-driver/en/frontend_adapter.md#diagnostic-only-import-continuation-design), [resolver candidates](../../mizar-resolve/en/imports.md#parsed-import-candidates).
Dependencies: existing live frontend service, AST candidate collector, captured module index, E0220–E0224 descriptors and shared diagnostic APIs are available.
Gap: source_drift/external_dependency_gap; the driver returns original diagnostics without the specified restricted semantic continuation.
Scope: existing FrontendService invocation admits only the authenticated E0022-only unrecovered AST/key, crosschecks source/prescan/AST, resolves typed path failures, builds an atomic separate Resolver batch and returns existing PhaseResult diagnostics with failed status and no outputs.
Structured detail spelling and lossless location policy belong to the driver owner; preserve original frontend batch on semantic rejection, suppress stale/cancelled reporting.
Tests: authentic source service fixtures for all five meanings, branch multiplicity and duplicate-alias peers, exact phase/detail/range provenance; malformed/mixed diagnostics and source binding rejection; existing clean/currentness/cancellation regressions and scheduler failed-dependent behavior.
Live-producer authenticity excludes forged AST/stub/class fixtures; reuse resolver adversarial and shared sink tests instead of adding an injection seam.
Reuse existing .miz intent where applicable; new Rust-loaded source tests exercise service integration without changing corpus expectations or trace credit.
Forbidden: spec/existing .miz/expectation/trace changes, E0225, graph activation/workspace-summary fabrication, normal semantic/proof/cache/artifact acceptance, failed-output storage, new public bridge/type or unused helper.
Affected: frontend_adapter.rs, existing source service/scheduler tests, paired driver owner/plan and current resolver/coverage adoption boundaries where they change.
Audit: chapter 22 remains partial; identify the bounded diagnostic continuation as adopted, with other resolver diagnostics and full-build dispatch deferred.
Require independent specification, test-sufficiency, implementation, volume/scope and consistency review; cargo fmt --check, cargo clippy --all-targets --all-features -- -D warnings, cargo test.
Exit: qualifying source failures co-report E0022 and allocated import diagnostics without success credit; invalid semantic input contributes no partial Resolver batch.
