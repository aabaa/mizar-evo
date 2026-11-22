# Task VC53-CORE46-UNARY-CHILD-REQUEST: bounded unary child-call request producer
Canonical language: English; [Japanese pointer](../ja/VC53-CORE46-UNARY-CHILD-REQUEST.md).
Status: frozen bounded producer. Tier: full; new Checker/Core consumer and typed call/result handoff fail predecessor same-owner/consumer criteria.
Owner: [Checker plan](../../mizar-checker/en/00.crate_plan.md); consumer: [Core plan](../../mizar-core/en/00.crate_plan.md), mizar-test.
Dependency: existing checked direct-call selection; [VC53 boundary](ALGDEF-VC53-NORMAL-RETURN.md).
Authority: [§§20.1.1–2, 20.4.1, 20.7.2, 20.13.1/3](../../../spec/en/20.algorithm_and_verification.md).
Owner APIs: [selected calls and static types](../../mizar-checker/en/type_checker.md#selected-direct-call-dependencies), [lowering](../../mizar-core/en/elaborator.md#static-source-algorithm-lowering), [Core46](../../mizar-core/en/source_family_decomposition.md#accepted-algorithm-coreir-task-graph).
Gap: source_drift/test_gap; selected callee identities exist, authenticated caller/actual/formal/result linkage is absent. No demonstrated language-intent gap.

## Frozen profile and design
Authenticate exactly two source-ordered public same-module algorithm definitions, each with its own definition-block object parameter and object result. The earlier callee solely returns its formal; the later caller solely returns one direct unary application of that callee to its own parameter. Optional terminating declarations retain intent only.
Use genuine Frontend/Resolver source, complete neutral TypedAst authentication and existing checked call selection; do not select by spelling or add another overload algorithm. Resolve ordinary header/body occurrences to their actual declaration identities, distinguishing caller actual from callee formal even with equal spellings.
An opaque borrowed Checker seal carries checked source/type/owner joins to Core. Core consumes only that seal and uses existing algorithm, binder, Apply and Return representations. One request refers to the actual caller/callee, application/statement, typed actual/formal and distinct callee/caller result identities with source provenance; it requests later substitution/normal-return work without performing it.
Refuse atomically malformed/foreign source, environment, neutral typed nodes or declaration associations, missing/ambiguous selection, recursion, unsupported syntax or mismatched types. Preserve immutable inputs and existing APIs/profiles.
No language intent, public diagnostic, substitution, VC53 admission, evidence adoption, proof/acceptance/promotion, ranges/loops, reflection/execution, imported receipts or coverage credit. No owner-gated work. VC53 Open/Deferred and raw/evidence policy remain unchanged. Broader calls, synonyms, overloads, contracts, locals, literals, qualified/imported/template/group calls stay deferred.
Files: type_checker.rs, elaborator.rs, existing Checker and mizar-test test modules, paired owner API documents and this contract. No new parallel syntax/term representation; no AGENTS/protocol/spec/expectation/trace edits.

## Test matrix and exit
Cap: six families, genuine source setup first; establish one successful caller/callee fixture before expanding tests.
1. Genuine nonrecursive unary object pair: selected callee is the actual declaration; exact owner/application/Return/source joins, caller actual and callee formal retain distinct declaration identities and authenticated compatible type associations, distinct result slots and existing Apply operands.
2. Rename/trivia and equal-spelling distinct-scope variants: regenerate genuine source, retain correct joins; optional terminating flags confer no proof credit. Repeat/purity checks preserve all inputs.
3. Real self/mutual recursion, missing/ambiguous or wrong target, extra application and unsupported call/statement/profile shapes: refuse without request.
4. Real type mismatch or unsupported signature/actual: refuse, never fabricate typed actual/formal/result links.
5. Foreign/stale source/environment/typed projection, changed node/child/range/recovery or declaration association: atomic refusal; original remains usable.
6. Existing static/direct-return and call-selection controls remain unchanged; downstream direct-return VC entry refuses this child-call profile, no obligations or evidence/adoption credit is introduced by request production.
Tests reuse existing genuine selection setup in Checker tests/support/source_overload.rs and mizar-test runner tests; existing [direct return](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz) remains an unchanged control.
Independent specification/test-intent/trust, sufficiency, precision, removable-only volume, source/docs/EN-JA and final quality reviews are required.
Run fmt, all-target/all-feature Clippy with denied warnings, both lint_policy suites (links/fragments), Checker/Core owner tests, explicit new regressions and short proof-verification E2E. Full cargo test only on cadence; lane dispatch has no full/push trigger. Changed source-authentication/identity obligations run focused tests; unchanged evidence/policy/cache/edit-versus-clean heavy matrices defer to integration.
Exit: frozen profile produces one authenticated request with unchanged refusal/credit boundaries and passing required gates. Audit/trace status remains unchanged.
