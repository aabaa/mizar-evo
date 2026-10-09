# Task ALGDEF-PARSER-FAIL: Explicit algorithm failure syntax
Canonical language: English; [Japanese pointer](../ja/ALGDEF-PARSER-FAIL.md).
Status: planned; bare fail is implementation-ready; message forms are blocked. Tier: full.
Owner: [parser plan](../../mizar-parser/en/00.crate_plan.md).
Producers: lexer reservation, frontend string/cache handoff, syntax views.
Consumers: Core 43/48/53, VC 43/53 and mizar-test.
Dependencies: existing algorithm statement lists, term parser and string tokens;
message/report forms have the lexical spec_gap below; reflection reports also wait for AST-PARSE.
Authority: [§20.2.6](../../../spec/en/20.algorithm_and_verification.md#2026-jump-statements-return-break-continue-fail),
Appendix A A.20; [parser gap](../../mizar-parser/en/source_spec_audit.md#reserved-word-coverage).
Gap: source_drift/test_gap: lexer lacks fail reservation; parser/syntax lack fail_stmt.

## Scope
Reserve fail in lexer and parse bare fail with semicolon at algorithm statement
positions. Message/comma-report forms remain a separate blocked increment. Retain authored token order, ranges, recovery and typed
syntax projection. Preserve existing frontend cache/recovery handoffs; no parallel statement AST.
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
Frontend controls cover recovery ordering and cache replay; deterministic syntax/source projection and source provenance are checked.

## Exit
Only fresh cases receive trace/coverage changes in the implementation task;
existing oracles remain unchanged. Update chapter20/appendixA parser deferral
for bare fail only, retaining message/reflection and semantic/runtime deferrals.
Require specification, test-sufficiency, implementation, volume/scope and
consistency reviews; cargo fmt --check; cargo clippy --all-targets --all-features
-- -D warnings; cargo test; exact local-link and paired-pointer validation.
