# Task STEP6A16-PROVISIONAL-IMPORT-CANDIDATES: frontend import correspondence
Canonical language: English; [Japanese pointer](../ja/STEP6A16-PROVISIONAL-IMPORT-CANDIDATES.md).
Status: implemented. Tier: full. Owner: [resolver plan](../../mizar-resolve/en/00.crate_plan.md).
Authority: [spec 12.3](../../../spec/en/12.modules_and_namespaces.md#123-import-statements), [provisional boundary](../../mizar-resolve/en/imports.md#provisional-frontend-candidates), existing parser import-items and resolver branch-import corpus tests.
Gap: source/design integration gap; real frontend stubs have no mapping to the existing resolver path-candidate seam.
Scope: one method on existing ImportPathCandidate, existing resolver import tests, paired imports owners/plan indexes, contracts and global todo. No new carrier, field, code file or provider.
Contract: the owner defines request provenance checks and faithful provisional mapping; existing path resolution remains unchanged and formal semantic publication still requires AST revalidation.
Tests: actual disk loading/preprocessing with BOM, CRLF and comments; absolute/current/parent paths, aliases and branches into existing path resolution; empty input, inconsistent provenance/cardinality and partial malformed pre-scan input.
Forbidden: fake producer/provider, semantic acceptance from stubs, new diagnostics/recovery rules, artifact reads/publication, complete summaries, spec/corpus/expectation/trace/audit changes, task17 and Step7/MVM.
Require independent specification, test-sufficiency, implementation, volume/scope and consistency reviews; cargo fmt --check, cargo clippy --all-targets --all-features -- -D warnings, cargo test.
Exit: actual frontend request stubs map faithfully into provisional resolver candidates or reject inconsistent carriers; actual provider resources and semantic producer publication remain subsequent work.
