# Task STEP6A26-DIAGNOSTIC-ANCHORS: lossless shared diagnostic anchors
Canonical language: English; [Japanese pointer](../ja/STEP6A26-DIAGNOSTIC-ANCHORS.md).
Status: frozen. Tier: full. Owner: [diagnostics plan](../../mizar-diagnostics/en/00.crate_plan.md), failure records; dependency: existing mizar-session SourceAnchor. Consumer: future frontend diagnostic bridge.
Authority: [specification 22.1.2](../../../spec/en/22.error_handling_and_diagnostics.md#2212-source-span-and-context-display); [anchor owner](../../mizar-diagnostics/en/failure_record.md#frontend-anchor-adoption).
Gap: source_drift in shared spans, which cannot retain Point/Generated shape or unspecified zero-width intent required for frontend adoption.
Scope: retain SourceAnchor as the single span payload, expose anchor construction/access and geometric range projection, preserve existing range-constructor validation; validate structural bounds and explicit intent compatibility.
Keep exact generated reason, optional label, freshness and ordered repeated secondary spans through draft/record creation. Preserve existing range debug output and aggregation identity/representative policy; new anchor debug output must distinguish shape and reason.
Source text binding, bounds against text, UTF-8 validation and unknown frontend categories belong to the future bridge; constructors do not establish publication authority. Unknown anchor variants must fail explicitly.
Forbidden: specification or existing corpus/expectation changes, registry activation, frontend bridge/service publication, parser recovery or proof/acceptance changes, new duplicate span types or parallel payloads.
Tests: existing constructor regressions; Range/Point/Generated Range/Generated Point and zero-range distinctions, exact reasons and explicit/unspecified intent; malformed ranges and nonzero intent; repeated secondary roundtrip; point rendering without invented extent. Existing source-load tests remain authoritative.
Coverage audit: chapter22 stays partial; shared retention is implemented while registry and source-bound frontend conversion remain deferred. No new executable language or producer-emission coverage is claimed.
Require independent specification, test-sufficiency, implementation, volume/scope and consistency reviews; cargo fmt --check, cargo clippy --all-targets --all-features -- -D warnings, cargo test.
Exit: shared records retain approved anchors without fabricating intent, with existing APIs and aggregation behavior preserved.
