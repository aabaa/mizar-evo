# Task RESOLVE-P19-BINDER-SOURCE-CORRESPONDENCE: fresh Binder source correspondence
Canonical language: English; [Japanese pointer](../ja/RESOLVE-P19-BINDER-SOURCE-CORRESPONDENCE.md).
Status: frozen bounded unit; MBP-02 remains partial. Tier: full.
Owners: [Resolver plan](../../mizar-resolve/en/00.crate_plan.md), [Checker plan](../../mizar-checker/en/00.crate_plan.md); consumer: existing lexical BindingEnv projection.
Purpose: retain authenticated ordered head/body candidates and exact outer declaration/type-source correspondence in the existing admitted mathematical profile.
Dependencies: completed [fresh resolution](RESOLVE257-FRESH-FUNCTION-BINDERS.md), [BindingEnv metadata](CHECKER257-FRESH-FUNCTION-BINDING-CONTEXTS.md), and [scoped references](CHECKER257-SCOPED-FUNCTION-REFERENCES.md); these partially implement [MBP-02](AST-MULTI-BINDER-PLAN.md), without formation.
Authority: [EN13 §13.10](../../../spec/en/13.term_expression.md#1310-function-binder--), [EN04 §4.6](../../../spec/en/04.variables_and_constants.md#46-scoping-and-shadowing), [EN20 §20.5.3](../../../spec/en/20.algorithm_and_verification.md#2053-reflected-syntax); existing Binder parser corpus remains parse-only.
Gap: source_drift/test_gap; copied Set/Object payloads lose the selected declaration and physical type origin; existing replay authenticates no Binder-root candidate row.

## Frozen boundary
Retain actual Binder root, incoming construction scope, ordered head tokens and final body in the existing sealed ResolvedVariableScope. These source-only candidates precede fresh declaration introduction; they do not implement general mode selection.
For each fresh head retain the immediate selected outer binding and ultimate physical TypeExpression node from an admitted prior Let/Reserve. An untyped Let keeps its own identity while its type originates at the applicable Reserve; nested heads inherit that physical origin transitively.
Reuse existing incoming-context lookup, simultaneous fresh introduction, shadow winners, source order and free identities. Preserve all existing admission/refusal behavior, including repeated-reservation refusal and unsupported origin/mode positions.
Minimal public provenance on existing receipts is permitted for the real cross-crate consumer; no parallel term/type representation or speculative formation fields. Keep production additions within 300 lines.
Original-source replay authenticates every new candidate and correspondence alongside all existing binding/reference rows. Checker consumes this authenticated handoff in its existing lexical projection, retaining original identities and Missing type sites.
No expected-type inference, Set/Object carrier inference, mode expansion, captures, reconstruction, E0206 adoption, checked Binder, Core/VC/Proof, acceptance or semantic/trace credit. MBP-03–09 and later MBP-02 units remain separate owners.

## Ownership and artifacts
Production: names.rs, necessary symbols.rs replay and type_checker.rs lexical-consumer seams; binding_env.rs only if needed. Tests: existing names/tests.rs, labels/tests.rs and existing source-derived Checker/public scoped tests; no new fixture/module.
Owner invariants: [Resolver names](../../mizar-resolve/en/names.md#step-5c1-variable-scope-resolution), [original replay](../../mizar-resolve/en/symbols.md#original-source-proof-lexical-maps), [Checker BindingEnv](../../mizar-checker/en/binding_env.md), [lexical intake](../../mizar-checker/en/type_checker.md#original-source-theorem-binding-contexts).
Update paired owner boundaries and necessary plan/TODO links; correct the umbrella plan's stale completion mapping. No doc/spec, old MIZ/expectations, protocol, diagnostics, recovery, hashes, corpus activation or element-rule ledger edits. Coverage-audit ownership/credit/deferrals remain unchanged.

## Test matrix and exit
Use genuine existing parser/Frontend, collectors, original source and replay fixtures; at most six case families:
1. Unary and ordered heterogeneous multi-head Let/Reserve: exact root/head/body, immediate outer binding and physical type node/range.
2. Untyped Let default and explicit Let override: distinguish immediate declaration from physical Reserve/Let type origin, including equal-payload distinct type nodes.
3. Nested/sibling Binders: nearest outer fresh declaration with transitive physical origin; sibling isolation.
4. Simultaneous incoming lookup, shadows and body/free references across all three existing occurrence profiles; fresh value identities remain distinct.
5. Corrupt root/body/order, outer binding, equal-payload physical type node, scope/source/module; coherent source changes with fresh collectors and pristine controls must reject stale correspondence.
6. Existing missing/forward/duplicate/default ambiguity, unsupported origins/modes and omitted winners refuse; Binder BindingEnv stays Missing and higher consumers refuse while non-Binder controls pass.
Exit: exact authenticated correspondence reaches the existing Checker lexical consumer without changing higher-stage admission.

## Reviews and verification
Independent specification/test-intent/trust review before freeze; sufficiency, implementation/trust, removable-only volume, source/document consistency and final full-gate quality review.
Run fmt, workspace all-target/all-feature Clippy -D warnings, both lint_policy suites, Resolver names/labels owners, Checker lexical/scoped owners and representative formula/type/metadata CLI checks with CARGO_BUILD_JOBS=4.
Run changed identity/original-replay obligations immediately; unchanged evidence-policy and edit-versus-clean matrices defer to integration cadence. No lane full-test trigger or push; freeze and implementation count once at integration.
