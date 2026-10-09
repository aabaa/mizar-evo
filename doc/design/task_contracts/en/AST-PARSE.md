# Task AST-PARSE: Reflected syntax input preparation
Canonical language: English; [Japanese pointer](../ja/AST-PARSE.md).
Status: planned. Tier: full gates. Primary owner: [mizar-parser](../../mizar-parser/en/00.crate_plan.md).
Consumers: mizar-checker, mizar-core and mizar-test.
Readiness: additive syntax slice ready; legacy exhaustive retirement blocked.
Dependencies: existing lexer/frontend/syntax pipeline; extend its reserved
vocabulary and syntax handoff in this slice rather than assume they exist.
Authority: Chapters [2](../../../spec/en/02.lexical_structure.md#24-reserved-words),
[3](../../../spec/en/03.type_system.md#33-type-expressions),
[13.10](../../../spec/en/13.term_expression.md#1310-function-binder--),
[20](../../../spec/en/20.algorithm_and_verification.md#2011-algorithm-definition-block),
and [A.20](../../../spec/en/appendix_a.grammar_summary.md#a20-algorithms-and-computation).
Gap: source_drift/test_gap for expr, Binder and contextual inputs.
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
