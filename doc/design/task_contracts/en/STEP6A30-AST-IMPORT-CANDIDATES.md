# Task STEP6A30-AST-IMPORT-CANDIDATES: resolver-owned parsed import collection
Canonical language: English; [Japanese pointer](../ja/STEP6A30-AST-IMPORT-CANDIDATES.md).
Status: frozen. Tier: full. Owner: [resolver plan](../../mizar-resolve/en/00.crate_plan.md); consumer: existing declaration-symbol runner.
Authority: [spec 12.3](../../../spec/en/12.modules_and_namespaces.md#123-import-statements), [two-pass import contract](../../mizar-resolve/en/imports.md#two-pass-contract), existing branch/alias/recovery source fixtures and runner assertions.
Gap: boundary/design drift; real AST candidate production resides in the test runner rather than the resolver owner. Existing provisional frontend candidates do not replace parsed imports.
Scope: method on existing ImportPathCandidate, existing imports source/tests, remove runner walker and route real runner/fixture assertions to owner method; paired import owner, plan and contract links, affected audit ownership. No new type, code file, adapter or dependency.
Frozen API, supported parser shapes, source/range provenance, ordering and rejection policy: [parsed candidates](../../mizar-resolve/en/imports.md#parsed-import-candidates).
Tests retain existing branch/alias/recovered fixture intent and verify actual parser inputs plus structural/source mutation boundaries, including valid no-import units, top-level-only collection, rejection of real skipped late imports/top-level recovery, and real parser malformed terminator/alias/comma/branch cases, including variants without recovery flags. Existing semantic resolver and active assertion outcomes remain unchanged.
Forbidden: spec/corpus/expectation/trace changes, parser/prepass changes, full recovered-directive or export validation, public diagnostic allocation, complete resolver service/output/summary/publication or semantic/proof/cache acceptance claims.
Coverage audit changes owner of parsed candidate production only; wider resolver/publication gaps remain. Do not upgrade language coverage from this migration.
Require independent specification, test-sufficiency, implementation, volume/scope and consistency reviews; cargo fmt --check, cargo clippy --all-targets --all-features -- -D warnings, cargo test.
Exit: real runner uses resolver-owned parsed candidates with preserved source intent and no duplicate walker; unsupported inputs fail the entire projection.
