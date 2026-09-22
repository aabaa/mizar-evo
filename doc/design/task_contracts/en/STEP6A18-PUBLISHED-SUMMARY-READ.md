# Task STEP6A18-PUBLISHED-SUMMARY-READ: indexed summary file consumption
Canonical language: English; [Japanese pointer](../ja/STEP6A18-PUBLISHED-SUMMARY-READ.md).
Status: implemented. Tier: full. Owner: [build plan](../../mizar-build/en/00.crate_plan.md).
Authority: [build read boundary](../../mizar-build/en/module_index.md#indexed-summary-file-read), [artifact store](../../mizar-artifact/en/store.md), [module summary reader](../../mizar-artifact/en/module_summary.md).
Gap: external dependency/design gap; A17 supplies real manifest metadata but no build-owned indexed summary read connects it to canonical JSON consumers.
Scope: one method on existing DependencyModuleSummaryRef, existing module_index tests, paired owner/plan indexes, contracts and global todo. No new type, field, code file or dependency.
Contract: caller supplies the artifact root; delegate path safety, canonical JSON and current-schema artifact-hash verification to the store, then summary validation to the artifact reader; require current schema and exact known ModuleId package ID and module path identity before returning JSON.
Tests: actual summary publication/read, exact bytes/value, wrong artifact-versus-interface hash, missing/corrupt files, unsafe path, inconsistent internal hash, unsupported schema, and reference package/module mismatches; no dependency source fallback.
Forbidden: root discovery, manifest/current-build lock or expected interface binding claims, cache/proof credit, new diagnostics, source producer/provider stand-ins, service publication, spec/corpus/expectation/trace/audit changes, artifact task17 and Step7/MVM.
Require independent specification, test-sufficiency, implementation, volume/scope and consistency reviews; cargo fmt --check, cargo clippy --all-targets --all-features -- -D warnings, cargo test.
Exit: an indexed current summary yields validated canonical JSON or no result; provider identity/resource integration and complete source producers remain subsequent work.
