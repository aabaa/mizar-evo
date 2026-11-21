# Task RESOLVE-P19-BINDER-PATTERN-INTAKE: pattern positions and precommit intake
Canonical language: English; [Japanese pointer](../ja/RESOLVE-P19-BINDER-PATTERN-INTAKE.md).
Status: bounded implementation; MBP-02 remains partial. Tier: full.
Owners: [Resolver plan](../../mizar-resolve/en/00.crate_plan.md), [Checker plan](../../mizar-checker/en/00.crate_plan.md); consumer: source lexical BindingEnv intake.
Purpose: authenticate construction positions before head interpretation and prepare immutable branch-local pattern names without semantic capture.
Predecessor: [fresh source correspondence](RESOLVE-P19-BINDER-SOURCE-CORRESPONDENCE.md); [MBP-02](AST-MULTI-BINDER-PLAN.md) owns the remainder.
Authority: [EN13 §13.10.1](../../../spec/en/13.term_expression.md#13101-capture-reconstruction-and-r2), [EN20 §20.2.5](../../../spec/en/20.algorithm_and_verification.md#2025-ast-pattern-match-match), [EN04 §4.6](../../../spec/en/04.variables_and_constants.md#46-scoping-and-shadowing).
Gap: source_drift/test_gap; existing algorithm call exclusions retain spelling/range sets, while mathematical candidates commit fresh heads without a construction-position handoff to Checker.
Full tier fails the predecessor's pattern criterion: new public position/branch provenance and Checker pattern-name consumer.

## Frozen boundary
Reuse SourceFunctionBinderCandidate for ordered roots, heads, final bodies and incoming scope. Derive mathematical, pattern and unsupported construction positions from actual ancestors before fresh declaration selection; retain enclosing case/pattern/branch nodes for patterns.
Admit source-only Binder-root case patterns with bare-name bodies, parenthesization and nested Binders. Preserve ordered head groups and all bare pattern-name occurrences; repeated occurrences share one branch-local name, while each head group remains distinct. Other pattern shapes remain unsupported in this intake.
Authenticate through the existing source-owned structural projection and exact symbol-collection replay, with no caller-supplied positions or names; raw-text authentication is outside this seam. Existing mathematical scope replay must include position provenance.
Checker chooses its fresh or pattern intake from candidates before interpreting head declarations. The theorem lexical route requires mathematical candidates; pattern intake creates only immutable pending name metadata in branch-owned contexts, with Missing type sites and source-local identity distinct from stored Var UIDs.
Names are visible only in their own branch, including nested branch scopes; sibling and post-match scopes do not acquire them. Existing outer declarations, source order and mathematical source/type correspondence stay unchanged.
Unsupported expectation-driven/reflected/reconstruction positions fail closed with no fresh fallback. Existing algorithm type checking and all higher Binder consumers continue to refuse; call-target candidate exclusions keep their existing behavior.
Exclude semantic matching, captured UIDs/types/domain/C, carrier inference, formation, reconstruction, R2, E0206, checked Binder, Core/VC/Proof, acceptance and semantic/trace credit. MBP-03–09 remain deferred.

## Artifacts and ownership
Production: names.rs and symbols.rs source authentication; type_checker.rs intake; binding_env.rs only the pending pattern-name kind needed by that consumer. Reuse existing types and source-derived fixtures; no new source/test module.
Owner invariants: [Resolver names](../../mizar-resolve/en/names.md#step-5c1-variable-scope-resolution), [original replay](../../mizar-resolve/en/symbols.md#original-source-proof-lexical-maps), [BindingEnv](../../mizar-checker/en/binding_env.md), [theorem intake](../../mizar-checker/en/type_checker.md#original-source-theorem-binding-contexts).
Update paired owner boundaries and umbrella mapping only where changed. Coverage-audit ownership, credit and deferrals remain unchanged. No doc/spec, existing MIZ/expectations, trace, diagnostics, protocol or call-candidate acceptance edits.

## Test matrix and exit
Use real parser/Frontend originals, collectors and replay; at most six families:
1. Unary and ordered heterogeneous multi-head mathematical controls preserve fresh identities, exact outer/type sources, Missing sites and refusal; position is mathematical before commitment.
2. Unary/multi-head case patterns retain exact root/head/body/case/branch and source order without preceding Let/Reserve; Checker pending names have Missing types and no fresh FunctionBinder identity.
3. Repeated names, parenthesized/nested Binder patterns retain all occurrences and ordered groups; distinct branches with equal spellings have distinct local identities.
4. Branch visibility: same-branch lookup, nested branch shadowing, sibling isolation and post-match escape; pattern names cannot become mutable algorithm locals.
5. Wrong source/module/symbols/recovery and coherent changed originals cannot reuse stale position/name provenance; pristine originals and deterministic repeated acquisition pass.
6. Algorithm-body fresh, reflected/expected-AST/reconstruction, unsupported patterns and missing mathematical types refuse without mode fallback; non-Binder algorithm controls and call-target exclusions remain unchanged.
Exit: actual Checker intake consumes authenticated positions before committing fresh head interpretation and exposes only pending immutable pattern names, preserving higher-stage refusal.

## Reviews and verification
Independent specification/test-intent/trust review before freeze; sufficiency, implementation/trust, removable-only volume, source/document consistency and final full-gate quality review.
Run fmt, workspace all-target/all-feature Clippy -D warnings, both lint_policy suites, Resolver and Checker owner groups, affected public scoped/call-target tests and representative formula/type/metadata CLI checks with CARGO_BUILD_JOBS=4.
Run changed original-source/identity obligations immediately; unchanged evidence-policy and edit-versus-clean matrices defer to integration cadence. No lane full-test trigger or push; freeze and implementation count once at integration.
