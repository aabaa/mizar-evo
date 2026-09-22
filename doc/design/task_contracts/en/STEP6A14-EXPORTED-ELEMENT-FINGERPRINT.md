# Task STEP6A14-EXPORTED-ELEMENT-FINGERPRINT: canonical exported element hashing
Canonical language: English; [Japanese pointer](../ja/STEP6A14-EXPORTED-ELEMENT-FINGERPRINT.md).
Status: implemented. Tier: full. Owner: [artifact plan](../../mizar-artifact/en/00.crate_plan.md); consumers: subsequent real export producers.
Authority: [architecture 18 stable inputs](../../architecture/en/18.dependency_fingerprint.md#stable-fingerprint-inputs), [artifact exported symbols](../../mizar-artifact/en/module_summary.md#exported-symbols), [spec 23.5](../../../spec/en/23.package_management_and_build_system.md#235-verifier-artifacts-and-build-output).
Gap: source/design gap; module hashing exists but element fingerprint calculation has no artifact-owned entry. Source signature and proof-status production remain separate prerequisites.
Scope: one method on existing ExportedSymbolSummary, existing module_summary tests and paired owner docs; contract/index links only. No new type, field, carrier or code file.
Contract/test intent: [element fingerprint owner](../../mizar-artifact/en/module_summary.md#exported-element-fingerprint) specifies exact projection, validation, exclusions and compatibility. Test field sensitivity, source-range/stored-fingerprint exclusion, determinism, invalid inputs and unchanged writer/reader acceptance.
Forbidden: fabricated semantic signatures/status, complete exported-row or summary/provider/publication claims, schema changes, new reader enforcement, spec/corpus/expectation/trace/audit changes, task 17 or Step 7/MVM.
Require independent specification, test-sufficiency, implementation, volume/scope and consistency reviews; cargo fmt --check, cargo clippy --all-targets --all-features -- -D warnings, cargo test.
Exit: artifact owners can compute the canonical element value from supplied projection fields without changing existing artifact compatibility; actual semantic producers remain subsequent work.
