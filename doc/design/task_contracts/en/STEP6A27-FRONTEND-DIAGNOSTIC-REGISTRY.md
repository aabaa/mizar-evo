# Task STEP6A27-FRONTEND-DIAGNOSTIC-REGISTRY: frontend diagnostic allocations
Canonical language: English; [Japanese pointer](../ja/STEP6A27-FRONTEND-DIAGNOSTIC-REGISTRY.md).
Status: frozen. Tier: full. Owner: [diagnostics plan](../../mizar-diagnostics/en/00.crate_plan.md); consumer: future source-bound frontend diagnostic bridge.
Authority: specification 22.2.3 and 22.7; [registry owner](../../mizar-diagnostics/en/registry.md#frontend-allocations); frontend discriminator mapping remains frontend-owned.
Gap: specified E0013–E0052 cannot yet be referenced by shared drafts/records because descriptors are absent.
Scope: register exactly the forty approved Syntax/Error codes with canonical names and summaries, and meaning keys equal to semantic names; use a distinct frontend allocation version marker. Preserve all existing metadata and codes.
Tests: compare every new descriptor against the canonical specification table, validate registry uniqueness/compatibility, and construct shared draft/record values for all newly allocated codes. This is registry/value-layer coverage, not evidence of producer emission.
Forbidden: spec or existing corpus/expectation changes, new diagnostic meanings, message-based classification, frontend conversion/emission/service activation, parser recovery or publication/proof acceptance changes.
Coverage audit: chapter22 remains partial; registry allocation is available while source-bound conversion and producer adoption remain deferred to the future driver bridge.
Require independent specification, test-sufficiency, implementation, volume/scope and consistency reviews; cargo fmt --check, cargo clippy --all-targets --all-features -- -D warnings, cargo test.
Exit: approved descriptors are usable by shared records; producer-facing adoption remains separate work.
