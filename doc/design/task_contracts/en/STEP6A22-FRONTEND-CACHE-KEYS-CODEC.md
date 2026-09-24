# Task STEP6A22-FRONTEND-CACHE-KEYS-CODEC: retained cache-key bundle storage
Canonical language: English; [Japanese pointer](../ja/STEP6A22-FRONTEND-CACHE-KEYS-CODEC.md).
Status: frozen. Tier: full. Owner: [frontend plan](../../mizar-frontend/en/00.crate_plan.md).
Authority: [frontend incrementality](../../architecture/en/02.source_and_frontend.md#incrementality), [aggregate output](../../architecture/en/02.source_and_frontend.md#frontendoutput); [cache-key storage owner](../../mizar-frontend/en/cache_key.md#retained-cache-key-storage).
Gap: derived storage dependency; FrontendOutput retains cache-key components without a complete codec. Ready after retained token storage; no new language requirement.
Scope: two methods on FrontendCacheKeys, inline cache_key.rs tests, crate-private sharing of lexing context codecs; paired cache-key owners, contracts, plan indexes and global todo.
Preserve all fields, versions, order and optional AST key; exact schema, bounds and path binding follow the owner. Decoder does not access filesystem or rebuild any producer output.
Tests: existing cache_key.rs source/lexical/parser invalidation tests stay unchanged; new fixed-wire typed oracles, round trips and stable-hash preservation, context vocabulary, malformed/noncanonical/oversized inputs, path binding and numeric/range controls.
Forbidden: new public types/files/dependencies, stable-hash or key-version changes, lower-owner API changes, source/AST/diagnostic codecs, aggregate publication, cache/proof acceptance, spec/corpus/expectation/trace/audit edits, artifact task17 or Step7/MVM.
Provenance, cross-key consistency and cache reuse remain driver-owned. Helpers require at least three real call sites; single-use record conversion stays inline.
Require independent specification, test-sufficiency, implementation, volume/scope and consistency reviews; cargo fmt --check, cargo clippy --all-targets --all-features -- -D warnings, cargo test.
Exit: complete retained cache-key bundles round-trip under exact caller path binding; aggregate diagnostics and composition remain subsequent tasks.
