# Task AST-PARSE: Reflected syntax input preparation
Canonical language: English; [Japanese pointer](../ja/AST-PARSE.md).
Status: partial; expr-type syntax complete. Tier: full gates. Primary owner: [mizar-parser](../../mizar-parser/en/00.crate_plan.md). Producer owners: [lexer](../../mizar-lexer/en/00.crate_plan.md), [syntax](../../mizar-syntax/en/00.crate_plan.md), [frontend](../../mizar-frontend/en/00.crate_plan.md).
Consumers: mizar-checker, mizar-core and mizar-test.
Readiness: additive syntax slice ready; legacy exhaustive retirement blocked.
Dependencies: existing lexer/frontend/syntax pipeline; extend its reserved
vocabulary and syntax handoff in this slice rather than assume they exist.
Authority: Chapters [2](../../../spec/en/02.lexical_structure.md#24-reserved-words),
[3](../../../spec/en/03.type_system.md#33-type-expressions),
[13.10](../../../spec/en/13.term_expression.md#1310-function-binder--),
[20](../../../spec/en/20.algorithm_and_verification.md#2011-algorithm-definition-block),
and [A.20](../../../spec/en/appendix_a.grammar_summary.md#a20-algorithms-and-computation).
Gap: source_drift/test_gap remains for Binder and contextual inputs.
Scope: additive expr/|-> token support, expr types, Binder precedence,
term/formula syntax candidates in algorithm inputs/RHS/patterns, explicit
computation calls, boolean replay options and terminating definition labels;
retain label/name syntax and positions needed by resolver-owned shell projection for RESOLVE33.
Parser enforces grammar/position: terminating-only definition labels, thesis
directly under thus/hence for reflected/computation inputs, and B1 call/citation
forms; existing ordinary formula syntax stays separate. Expected types, resolved
reference scope/identity, UID/context formation and execution remain later-stage.
Blocked: the protected [pass fixture](../../../../tests/miz/pass/parser/pass_parser_algorithm_control_flow_001.miz)
uses exhaustive; its unchanged pass expectation conflicts with §20.2.5.
Retiring it requires authorized test-intent resolution; preserve this blocker.
Forbidden: modifying existing .miz/expectations, semantic legacy modes, MML
certification, ghost valuation, runtime execution or reflection coverage credit.
Owner-local invariants/tests: [grammar.md](../../mizar-parser/en/grammar.md#reflected-syntax-input-boundary).
Owner details: [grammar](../../mizar-parser/en/grammar.md#task-8-type-expressions),
[precedence](../../mizar-parser/en/pratt.md#term-pratt-contract),
[syntax extension](../../mizar-syntax/en/ast.md#vocabulary-increment-contract).
Tests: use the linked grammar matrix; existing parser control-flow/justification
fixtures are regressions. Resolver owns shell/reference semantics.
Audit: retain absent semantic/runtime coverage; later parser activation must
describe only its actual syntax route.

Reviews: specification/documentation, test sufficiency, implementation,
volume/scope, and source/document consistency under the canonical protocol.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features --
-D warnings; cargo test; affected owner tests and local-link/parity checks.
Exit: frozen scope satisfied by authentic producer inputs and reviewed tests;
no completion or coverage claim for deferred consumers.

## Frozen expr-type increment
Authority: Chapters2/3.3 and AppendixA.A3; source_drift/test_gap. Reserve expr only; keep |-> and all Binder/contextual/reflection semantics deferred. Existing TypeHead owns expr, required of and nested TypeExpression; no new public node/type/helper. Preserve applicable required-type policies and bounded generic qua syntax.
Reuse existing type diagnostics: missing of diagnoses at the next token while retaining a valid operand; missing operand inserts MissingTypeExpression at the owning delimiter without consuming it. Only explicit expr-led of/over actuals take the type route; all ordinary value actuals retain their route. Keep lookahead/qua consistent with the recursive type head.
Fresh frozen parser cases: pass_parser_expr_types_001, fail_parser_expr_missing_of_001 (one malformed_type_expression), fail_parser_expr_missing_target_001 (two). Add only their six files and one sorted spec.en.03.expr_types.parser trace block; preserve all old row bytes/order and source/expectation intent.
Source write scope: lexer tables.rs; parser module.rs; frontend parsing.rs namespace v8 to v9. Test scope: new blocks only in parser module/tests.rs, frontend parsing.rs and mizar-test runner/tests.rs; metadata.rs only expr deferred-word removal, three sorted active IDs and119 parser counts. Existing current namespace assertions in parsing.rs/orchestration.rs/runner/tests.rs may move to v9; retain historical version controls. Four source_{attribute_definition,mode_definition,functor_definition,property_implementation}.rs aggregate cells only: (561,513) to (564,514), (316,245) to (317,247), and parser116 to119; semantic cells/order stay intact.
Documentation write scope: this contract, paired parser grammar/TODO/source_spec_audit, syntax ast and frontend parsing owners; lexer/syntax/frontend plan index pointers, global AST-PARSE readiness row and Chapter3/AppendixA3 coverage cells only. Paired JA contract ownership links track the producer plan indexes. Protected spec, existing .miz/sidecars/trace rows, Cargo, storage schemas, public diagnostics and ratchet stay unchanged.
Presented limits:100 production/450 test/70 documentation added; docs2:1/200 and contract60 apply. Parent witnesses current-API source baseline before production; then actual affected tests, full protocol reviews/gates/commit/checkpoint and continued inventory. This increment grants only expr-type lexical/parser syntax coverage, not full AST-PARSE or MBP-01 completion.
