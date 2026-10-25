# Task ALGDEF-PARSER-FAIL: Explicit algorithm failure syntax
Canonical language: English; [Japanese pointer](../ja/ALGDEF-PARSER-FAIL.md).
Status: bare syntax completed; message/report forms remain blocked. Tier: full.
Owner: [parser plan](../../mizar-parser/en/00.crate_plan.md); producer owners: [lexer plan](../../mizar-lexer/en/00.crate_plan.md), [syntax plan](../../mizar-syntax/en/00.crate_plan.md), [frontend plan](../../mizar-frontend/en/00.crate_plan.md).
Producers: lexer reservation, existing frontend token/cache handoff, syntax views.
Consumers: Core 43/48/53, VC 43/53 and mizar-test.
Dependencies: existing algorithm statement lists, term parser and string tokens;
message/report forms have the lexical spec_gap below; reflection reports also wait for AST-PARSE.
Authority: [§20.2.6](../../../spec/en/20.algorithm_and_verification.md#2026-jump-statements-return-break-continue-fail),
Appendix A A.20; [parser gap](../../mizar-parser/en/source_spec_audit.md#reserved-word-coverage).
Classification: source_drift/test_gap for bare fail lexical/parser/syntax coverage; the separate message/report lexical spec_gap remains below.

## Scope
Reserve fail in lexer and parse bare fail with semicolon at algorithm statement
positions. Message/comma-report forms remain a separate blocked increment. Retain authored token order, ranges, recovery and typed
syntax projection. Sole grammar/recovery owner: [bare failure](../../mizar-parser/en/grammar.md#bare-algorithm-failure); sole vocabulary/projection owner: [syntax](../../mizar-syntax/en/ast.md#bare-algorithm-failure). No parallel statement AST or single-use parser wrapper.
Spec_gap: §2.7 says strings appear only after ( or comma, while §20.2.6
places the optional message after fail. Resolve that EN/JA authority conflict
before extending string recognition. Reflected report terms also need AST-PARSE.

Existing regressions: [control flow](../../../../tests/miz/pass/parser/pass_parser_algorithm_control_flow_001.miz), [recovery](../../../../tests/miz/fail/parser/fail_parser_algorithm_control_flow_recovery_001.miz); [statement owner](../../mizar-parser/en/grammar.md#task-33-algorithm-control-flow).

## Boundaries and tests
Parser accepts syntax without deciding terminating/ghost/type restrictions.
Core 48 owns failure as no exit and return-or-fail exhaustiveness; Core 53 owns
forbidden failure placement and non-ghost report values; VC owns no-failure-VC
accounting and normal-return requirements. MVM owns evaluation/reporting/E0331.
Forbidden: execution, acceptance, public E0503 adoption, new language semantics,
protected spec/existing .miz/expectation changes and reflection implementation.
Test-first ready parser pass sources: bare fail and nested algorithm branches; parse-only terminating fail carries no
static acceptance claim. Ready negative sources cover malformed semicolon and non-algorithm placement.
After the lexical gap closes, message tests cover both delimiters, escapes,
comment shielding, malformed comma/report forms and non-fail quote/postfix near-misses.
Frontend controls cover recovery ordering, cache replay, deterministic syntax/source projection and source provenance. Use the [public parser namespace](../../mizar-frontend/en/parsing.md#bare-algorithm-failure-cache-namespace) v8 so identical caller-supplied token/parser inputs cannot reuse a v7 result after grammar changes; verify the same-token key distinction. The concrete coordinator already invalidates vocabulary-sensitive keys. Token versions/key shapes/storage/string recognition/cache algorithms stay unchanged.
Frozen fresh cases: pass_parser_algorithm_fail_bare_001, fail_parser_algorithm_fail_semicolon_001, fail_parser_algorithm_fail_placement_001; one new sorted spec.en.20.algorithm_fail_bare.parser trace row (all old row bytes/order retained). Missing-semicolon and top-level rejection use existing missing_semicolon/unexpected_top_level_token codes in authored order, with no new diagnostic enum.
Write scope: lexer tables.rs; parser module.rs; syntax ast.rs/ast/snapshot.rs; existing syntax ast/tests.rs and parser module/tests.rs vocabulary extensions; frontend parsing.rs parser-version constant, its two existing namespace assertions and new same-token/cache test; orchestration.rs only the six existing current-parser namespace assertion literals, preserving all surrounding oracles; mizar-test runner/tests.rs new real-Frontend checks and tests/metadata.rs fresh-case counts, three additive ordered active-case IDs and deferred-word removal. Only aggregate cells may change in runner/tests/type_elaboration/{source_attribute_definition.rs,source_mode_definition.rs,source_functor_definition.rs,source_property_implementation.rs}; preserve all old IDs/order/semantic cells. Add only the three fresh .miz/sidecar pairs and trace row; never change existing source/sidecar/trace oracles.
Documentation scope: paired parser grammar/TODO/source_spec_audit, syntax ast, frontend parsing, lexer/syntax/frontend plan indexes, these contracts, the existing global TODO bare-readiness row and chapter20/appendixA coverage rows only. No owner history, doc ratchet, Cargo, semantic/runtime oracle, token namespace or cache schema changes. Presented limits:70 production/450 test/60 documentation added, with docs2:1/200 and contracts60; actual test-first current-API baseline before production, then full protocol reviews/gates/commit/checkpoint and continue.

## Exit
Only fresh cases receive trace/coverage changes in the implementation task;
existing oracles remain unchanged. Update chapter20/appendixA parser deferral
for bare fail only, retaining message/reflection and semantic/runtime deferrals.
Require specification, test-sufficiency, implementation, volume/scope and
consistency reviews; cargo fmt --check; cargo clippy --all-targets --all-features
-- -D warnings; cargo test; exact local-link and paired-pointer validation.
