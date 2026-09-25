# Task STEP6A40-IMPORT-DIAGNOSTIC-REGISTRY: import diagnostic allocations
Canonical language: English; [Japanese pointer](../ja/STEP6A40-IMPORT-DIAGNOSTIC-REGISTRY.md).
Status: implemented. Tier: full. Owner: [diagnostics plan](../../mizar-diagnostics/en/00.crate_plan.md); consumer: future driver import continuation.
Authority: [specification 22.3.5/22.7](../../../spec/en/22.error_handling_and_diagnostics.md#2235-module-import-resolution); [registry owner](../../mizar-diagnostics/en/registry.md#import-allocations).
Gap: specified E0220–E0224 lack descriptors required by shared draft construction.
Scope: allocate those five codes with canonical names/summaries and the owner-defined metadata, preserving every existing descriptor and compatibility rule.
Tests: lock ordered allocation, compare metadata to specification, construct shared Resolver/ResolveError drafts and records, verify compatibility with previous descriptors and reject unallocated E0225.
Forbidden: spec, existing corpus/expectations/trace edits, E0205/E0225 allocation, resolver or driver conversion/emission, scheduling, public API expansion, proof or semantic acceptance.
Audit: chapter 22 stays partial; update registry readiness and the paired driver prerequisite without runtime credit.
Dependencies: existing resolver candidate classes and shared draft/record APIs suffice; source-bound runtime adoption remains deferred.
Require independent specification, test-sufficiency, implementation, volume/scope and consistency reviews; cargo fmt --check, cargo clippy --all-targets --all-features -- -D warnings, cargo test.
Exit: the five descriptors support shared records without claiming producer emission or corpus activation.
