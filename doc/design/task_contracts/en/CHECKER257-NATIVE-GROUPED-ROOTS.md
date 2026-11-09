# Task CHECKER257-NATIVE-GROUPED-ROOTS: grouped quantifier headers
Canonical language: English; [Japanese pointer](../ja/CHECKER257-NATIVE-GROUPED-ROOTS.md).
Status: bounded implementation complete; broader Tasks257/258 remain open. Tier: full.
Owner: [checker plan](../../mizar-checker/en/00.crate_plan.md); consumer: [test plan](../../mizar-test/en/00.crate_plan.md).
Dependency: completed CHECKER257B7-NATIVE-QUANTIFIER-COMPOSITION.
Authority: EN14 §§14.1/14.4/14.6/14.8; EN04 §§4.5/4.6; EN15 §15.2.1; EN16 §§16.1/16.2/16.4.
Gap: source_drift/test_gap; genuine lower and original composition refuse parentheses outside an otherwise admitted quantifier header.
## Frozen boundary
Admit finite transparent ParenthesizedFormula chains around existing explicit builtin-Set universal or existential theorem/lemma headers.
Keep every existing header body, declaration, restriction, fixed-connective and proof-body rule.
Reuse the existing source nodes, contexts, types, composite/composition tables, wrapper rows and original replay; add no public API, type, field, helper or parallel representation.
The effective quantified formula remains the sole owner root; outer wrappers retain actual sites/ranges, incoming context and outer-to-inner ordinals.
Grouping adds no binding scope, declaration, operand, formula semantics or accepted fact/proof.
Keep implicit/reserved binders, repetition, numeric leaves, grouped proof conclusions and nonquantified header roots outside this family.
Native258 statement construction remains direct-universal-root-only; successful grouped composition does not admit grouped statement owners.
No spec, existing MIZ, expectation, trace, coverage credit, Core/VC or Driver acceptance change is authorized.
Owner details: [TypeChecker](../../mizar-checker/en/type_checker.md#original-source-universal-composition-intake), [composite](../../mizar-checker/en/source_composite_formula.md#original-source-universal-composition), [TypedAst](../../mizar-checker/en/typed_ast.md), [ResolvedTypedAst](../../mizar-checker/en/resolved_typed_ast.md).
## Test-first and maintenance
Use existing private native quantifier/connective fixtures through generated grouped variants; no new fixture or helper file.
Before implementation freeze genuine public lower and complete upper/root/wrapper expectations for single/deep groups, both quantifier kinds, restricted/nested headers and a lemma/layout/name control.
Correct only the B7 lower test's grouped-root exclusion after independent specification/test-intent/trust review: its valid grouped universal now succeeds with unchanged primitive terms/equalities and genuine contexts/types.
Preserve every other B7 assertion, malformed-source control and unrelated exclusion; no output-derived expected value is permitted.
Witness compiled lower RED, lower GREEN and unchanged upper RED before implementing upper composition; retain actual logs outside doc/.
Run complete Typed/Resolved original replay, causal wrapper/root/context refusals, input purity and the native258 grouped-owner refusal after successful grouped composition.
## Reviews and verification
Independent specification/test-intent/trust PRE, POST implementation/sufficiency, removable-only volume and source/document consistency reviews precede nine gates and score.
Update paired module documents only for the accepted-input boundary; coverage audit remains unchanged because semantic/trace credit does not change.
Run fmt, workspace Clippy, both lint_policy suites, affected checker/context/harness/Core owners, verifier/resolver standard and short formula/type/CLI checks.
Changed wrapper/root/replay obligations run immediately; unchanged resolver/evidence heavy matrices defer to shared-push full integration.
Full cargo test must pass before shared push; full failure stops feature work until repair and full success.
Exit: grouped genuine lower and complete original composition/final replay pass with all protected boundaries preserved.
