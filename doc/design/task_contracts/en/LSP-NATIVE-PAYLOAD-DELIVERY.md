# Task LSP-NATIVE-PAYLOAD-DELIVERY: native feature payload delivery evidence
Canonical language: English; [Japanese pointer](../ja/LSP-NATIVE-PAYLOAD-DELIVERY.md).
Status: frozen implementation contract. Tier: full; independent Audit G4 evidence unit, not a slice of a registered pattern family.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md); consumers: Tasks17/19/38. Dependency: implemented [native producers](../../mizar-lsp/en/native_payload_producers.md) and [ranged host input](../../mizar-lsp/en/ranged_diagnostic_host.md).

## Authority and scope

Authority: spec [fix suggestions](../../../spec/en/22.error_handling_and_diagnostics.md#2213-fix-suggestions-and-help-annotations), [frontend adoption](../../../spec/en/22.error_handling_and_diagnostics.md#2223-frontend-diagnostic-adoption), [editor features](../../../spec/en/23.package_management_and_build_system.md#2361-hover-code-actions-and-diagnostics); owner [action tests](../../mizar-lsp/en/code_action.md#test-design), [explanation tests](../../mizar-lsp/en/explain.md#test-design), [validators](../../mizar-lsp/en/code_action_validators.md).
Existing tests: `server.rs::native_payload_producer_tests` genuine export delivery, stale retained reads, replacement and independent refusal; `diagnostics.rs::code_actions`, `explain_tests.rs` and `deferred_action_tests.rs` remain typed consumer fixtures.
Gaps: `test_gap` in combined evidence: each stale transition must first prove successful edit resolution, explanation and command execution on that same genuine publication; compare complete retained payload/source locations against independent raw-source expectations and require exact stale errors. `design_drift`: test-design and G4 correspondence text still describe native delivery as unavailable.
Writes: LSP native host tests, this contract pair, paired action/explanation test-design sections, G4 source/spec correspondence, plan links and Task39 checkbox/link only. Prefer strengthening existing tests over duplicate fixtures.
Forbidden: producer implementation, production behavior/API changes, spec/.miz/expectation changes, Tasks40/41, owner-gated scope, AGENTS/protocol edits, semantic coverage credit or SCA-003 status changes. Missing producer prerequisites stop the unit.

## Test matrix and exit

Cap three grouped cases; extra cases require a cited authority clause or .miz intent:
1. Real OpenBuffer export error with BOM/CRLF/Unicode and preceding lines reaches Driver/Diagnostics/ranged host. Before each version-only, normalized-equal, shifted-line and close/reopen transition, resolve its actual insertion and execute its reconstructed explanation command; query content and original raw UTF-16 range/URI/version are current and agree. After the transition, offers are empty, both locators and execution return ContentModified, and the same-selected retained query preserves the complete original payload, snapshot and locations as stale_artifact; shifted live coordinates differ.
2. Reuse genuine equal-id and changed-id replacement controls: publication identity/store association changes, old action/command cannot revive, old backing remains independently retained, and obsolete snapshot queries return stale_handle.
3. Reuse genuine missing/corrupt backing, obsolete capture, cancellation and missing-host controls; retain diagnostic/independent local edit where specified. Typed conversion/validator fixtures provide no native evidence.
Reviews: independent specification/test-intent before freeze; test sufficiency, precision, volume/scope and source/doc consistency with final full-tier assessment over the complete unit.
Verification: fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, complete mizar-lsp owner group including framed stdio, and explicit native payload tests as the short source-to-host end-to-end check.
No producer/hash/cache/evidence obligations change; their heavy matrices and full cargo test defer to named cadence/integration triggers.
Exit: freeze then implementation locally committed as one changed task; no push; Task39 checked off with typed/native boundaries explicit and all required reviews/checks passing.
