# Task AST-PARSE: Reflected syntax input preparation
Canonical language: English; [Japanese pointer](../ja/AST-PARSE.md).
Status: partial; expr types and bounded Binder roots complete. Tier: full gates. Primary owner: [mizar-parser](../../mizar-parser/en/00.crate_plan.md). Producer owners: [lexer](../../mizar-lexer/en/00.crate_plan.md), [syntax](../../mizar-syntax/en/00.crate_plan.md), [frontend](../../mizar-frontend/en/00.crate_plan.md).
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

## Frozen Binder-root increment
Authority:13.10/A13 and the [root grammar owner](../../mizar-parser/en/grammar.md#binder-expression-roots); source_drift/test_gap. Reserve |-> and admit unary/multiple heads only at set equating RHS, parenthesized-term interiors and recursive Binder bodies. Keep shared qualified-term/before-field-list entrypoints unchanged. Bare algorithm calls are specified but deferred here; formula/ordinary-call breadth stays deferred without declaring it invalid. No MB1/MBP-01/full AST-PARSE completion claim.
Representation: [syntax owner](../../mizar-syntax/en/ast.md#binder-expression-roots) adds only BinderExpression raw195, appended SurfaceNodeKind/serde order, valid direct ordered head tokens, arrow and one body TermExpression (existing direct MissingTerm occupies the body slot when absent); consumers use existing generic node_view/kind/children. No head node, stored arity, new view/helper wrapper, context flag, UID, binding/type/formation/proof/reflection/Core/runtime or acceptance API. One private Binder-capable parser has real set/parenthesis/recursive-body call sites.
Recovery: confirm only identifier-immediate-arrow or balanced-parentheses-immediate-arrow within existing statement/item boundaries before mutation; semicolon/end/item/statement-keyword barriers leave a prefix unconfirmed even inside parentheses. Valid parenthesized heads contain2+ distinct identifiers. A confirmed invalid head gets one existing MalformedTermExpression at its start and the existing emit_malformed_tail_recovery over the known head span as SkippedToken, then retains arrow/body and recovered status. Missing body uses existing zero-width MissingTerm at the owning delimiter, leaving it unconsumed. Unconfirmed prefixes keep existing recovery; no comma scan claiming a multi-head or new generic recovery/code.
Frozen fresh pairs: pass_parser_binder_roots_001, pass_parser_binder_grouped_hosts_001; fail_parser_binder_heads_001 (8 malformed_term_expression:6 confirmed heads plus2 existing set-equating errors), fail_parser_binder_missing_body_001 (2), fail_parser_binder_operator_operands_001 (2). Add only these10files and sorted spec.en.13.binder_roots.parser trace block; preserve all old source/sidecar/trace bytes/order. Local | uses a genuine source functor/default-fixity producer; no fabricated import fixture.
Source scope: lexer tables.rs; parser module.rs; syntax ast.rs and ast/snapshot.rs; frontend parsing.rs namespacev9 to v10. Tests: new blocks only in syntax ast/tests.rs, frontend parsing.rs and mizar-test runner/tests.rs, with optional focused parser module/tests.rs/lexer existing tests only if unique coverage needs them. Existing12current-version assertions in parsing.rs/orchestration.rs/runner/tests.rs may advance; historical controls remain. metadata.rs only5sorted active IDs and parser119 to124; four source_{attribute_definition,mode_definition,functor_definition,property_implementation}.rs aggregate cells only: (564,514) to (569,515), (317,247) to (319,250), parser119 to124; semantic cells/order unchanged.
Documentation: this contract, paired parser grammar/current source_spec_audit/TODO, syntax ast, frontend parsing; existing AST-MULTI-BINDER-PLAN readiness/MBP-01 partial status, global AST-PARSE sequencing and Chapter13/AppendixA13 live audit coverage/deferrals only. Producer plan indexes and paired ownership links already exist. Preserve frozen owner histories, protected exhaustive intent, other spec gaps, spec/Cargo/schemas/public codes/doc ratchet and all existing .miz/expectations.
Presented limits:200production/700test/75documentation added, docs2:1/200 and contract60 apply. Parent first actual serial current-API baseline before production; independent SPEC/trust/TEST/CODE/VOLUME/CONSISTENCY/EXIT, narrow then fmt/Clippy-all-targets-all-features-Dwarnings/full cargo test, exact links/protection/gates and commit/checkpoint/continued inventory. Only bounded root syntax credit; all semantic and remaining host/pattern consumers stay deferred.
