# Task STEP6A12-FRONTEND-LEXICAL-BINDING: frontend-bound lexical correspondence
Canonical language: English; [Japanese pointer](../ja/STEP6A12-FRONTEND-LEXICAL-BINDING.md).
Status: implemented. Tier: full. Owner: [resolver plan](../../mizar-resolve/en/00.crate_plan.md).
Authority: [spec 11.4](../../../spec/en/11.symbol_management.md#114-public-and-private-symbols), [resolver boundary](../../architecture/en/03.module_and_symbol_resolution.md#resolver-is-the-first-semantic-owner), [frontend orchestration](../../mizar-frontend/en/orchestration.md), [source mapping](../../mizar-frontend/en/span_bridge.md).
Gap: external dependency/design gap; A11 leaves actual frontend source/map and parser-backed collection binding to callers.
Scope: existing SymbolCollectionResult method, resolver frontend dependency, paired [symbols owner](../../mizar-resolve/en/symbols.md#frontend-bound-correspondence), existing frontend integration tests, contract and index links. No new type, field, adapter or code file.
Contract and test intent: the linked symbols owner defines trusted-input obligations, equality checks, map reconstruction, rejection behavior and real frontend integration cases.
Forbidden: complete export-summary/source-authentication claims, alias/operator export policy, artifact identity/publication, fake production providers, public diagnostics, spec/corpus/expectation/trace changes, task17 and Step7/MVM. No semantic coverage credit or audit change.
Require independent specification, test-sufficiency, implementation, volume/scope and consistency reviews; cargo fmt --check, cargo clippy --all-targets --all-features -- -D warnings, cargo test.
Exit: actual frontend locals and maps bind to the exact source-derived collection or fail closed; full export production remains subsequent work.
