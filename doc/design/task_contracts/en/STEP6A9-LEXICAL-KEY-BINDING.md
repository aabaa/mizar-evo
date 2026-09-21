# Task STEP6A9-LEXICAL-KEY-BINDING: unique lexical contribution pairing
Canonical language: English; [Japanese pointer](../ja/STEP6A9-LEXICAL-KEY-BINDING.md).
Status: implemented. Tier: full. Owner: [resolver plan](../../mizar-resolve/en/00.crate_plan.md); consumer: artifact-backed resolver projection and later Step 6 provider work.
Dependencies: existing artifact summary reader and resolver projection; A8 payload storage does not define identity binding.
Authority: [spec 11.2](../../../spec/en/11.symbol_management.md#112-scope-and-visibility), [spec 23.4](../../../spec/en/23.package_management_and_build_system.md#234-build-lifecycle-and-reproducibility), [artifact ordering](../../mizar-artifact/en/module_summary.md#canonical-ordering).
Gap: source drift; origin-id and fully-qualified-name aliases currently overwrite earlier exported rows, even though artifact uniqueness only excludes duplicate pairs.
Scope: existing resolver module_summary_reuse source/tests, paired owner documents and plan links, global todo and this contract; no public API or new code files.
Binding and failure rules: [resolver projection](../../mizar-resolve/en/module_summary_reuse.md#summary-backed-projection).
Tests: `crates/mizar-resolve/src/module_summary_reuse.rs::tests::lexical_keys_require_one_exported_row` and existing inline tests cover unique origin/name, equal aliases within one row, duplicate origins/names, cross-alias collisions, persistent ambiguity, private-row exclusion and canonical order independence through the real artifact reader.
Forbidden: new key/schema/identity conventions, payload interpretation, lexer provider or producer synthesis, export discovery, public diagnostics, spec/corpus/expectation changes, task17, Step7/MVM. Existing internal fallback reason is reused; no coverage credit changes.
Require independent specification, test-sufficiency, implementation, volume/scope and consistency reviews; cargo fmt --check, cargo clippy --all-targets --all-features -- -D warnings, cargo test.
Exit: lexical keys select exactly one exported row or retain the existing unpaired fallback; authenticated lexer identity conversion and real publication remain subsequent owner tasks.
