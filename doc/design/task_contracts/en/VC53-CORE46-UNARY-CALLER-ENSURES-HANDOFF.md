# Task VC53-CORE46-UNARY-CALLER-ENSURES-HANDOFF: caller postcondition handoff
Canonical language: English; [Japanese pointer](../ja/VC53-CORE46-UNARY-CALLER-ENSURES-HANDOFF.md).
Status: frozen bounded producer. Tier: full; expands accepted unary inputs.
Owner: [Checker plan](../../mizar-checker/en/00.crate_plan.md), [Core plan](../../mizar-core/en/00.crate_plan.md); consumer: VC, mizar-test.
Dependency: [unchanged unary pair](VC53-CORE46-UNARY-CHILD-REQUEST.md), [VC53 boundary](ALGDEF-VC53-NORMAL-RETURN.md).
Authority: [§20.1.2, §20.4.1, §20.7.2, §20.13.1/3](../../../spec/en/20.algorithm_and_verification.md).
Owner APIs: [Checker authentication](../../mizar-checker/en/type_checker.md#selected-direct-call-dependencies), [Core lowering](../../mizar-core/en/elaborator.md#static-source-algorithm-lowering), [VC boundary](../../mizar-vc/en/generator.md#bounded-direct-return-e1e4-producer-design).
Gap: source_drift/test_gap; unary authentication currently refuses caller contracts and lowering stores empty contracts. No language-intent change.

## Frozen scope
Keep the existing two public same-module unary object algorithms, scoped parameters, callee return and selected caller application/return unchanged. Admit zero or one caller clause of exactly `ensures result = <caller parameter>`; retain optional terminating intent without credit. The callee remains contract-free.
Authenticate the complete genuine source/environment/neutral typed projection before accepting the clause. The left operand denotes the caller's implicit result; the right resolves to its actual definition parameter, including equal-spelling distinct scopes. Refuse a caller parameter named `result` when a clause is present.
Reuse existing checked binding/inference storage and Core terms/formulas; preserve the equality's operands, range and provenance in caller CoreContractSet.ensures using the existing distinct caller result slot. No parallel payload or new public API/type/field is needed.
Keep no-clause output and old APIs unchanged. Refuse atomically malformed, duplicate, foreign, recovered or unsupported clauses, reversed/non-equality formulas, other bindings, callee clauses and requires. Inputs remain immutable.
Existing unary VC descriptor entry remains contract-free: clause-bearing Core is refused before earlier obligation/identity/Proof consumers can acquire authority under their old identities. This handoff creates no obligations, proof/evidence adoption, acceptance, status, promotion, publication or execution credit.
Caller E2/whole acceptance, later Proof authority, complete footprints/cache and promotion remain deferred; currentness remains caller-owned. Schema storage is unnecessary here. No owner-gated work, spec/.miz/expectation/trace or AGENTS/protocol changes. Audit coverage/owner/deferral stays unchanged.
Files: Checker type_checker.rs, Core elaborator.rs, VC generator.rs; existing Checker source_overload.rs and mizar-test runner/tests/algorithm.rs; paired Checker/Core/VC API owner paragraphs, plan backlinks and this contract. VC guard preserves its existing profile boundary.
Test maintenance: replace the old exact caller-ensures refusal with an unsupported requires control and add positive coverage; §20.1.2/20.4.1 and this bounded handoff determine intent, without changing unrelated refusals.

## Test matrix and exit
Cap: six families; first establish one genuine clause-bearing Checker/Core fixture and pass its setup before expanding the matrix.
1. Exact caller equality: actual caller result/parameter Core variables, equality source/provenance, sole caller ensures, empty callee contract and obligation tables; unchanged selected Apply/Return joins.
2. Regenerated rename/trivia, same-spelling scopes and terminating variants: correct binding/result separation, repeatability and input purity; no-clause controls unchanged.
3. Requires, callee clause, duplicate clause, reversed/other equality, non-equality/compound/grouped formulas, unknown/foreign parameter and result-shadowing: refuse with no seal.
4. Existing recursive, type, selection and unsupported body/signature controls continue to refuse.
5. Clause-bearing source with foreign/stale environment or changed typed clause/operand/range/child/recovery: refuse atomically; original remains reusable.
6. Clause-bearing output remains refused by old static/direct-return and unary descriptor/identity/Proof routes; no-clause downstream controls remain valid.
Existing oracles: [direct return](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz), Checker source_overload and mizar-test algorithm genuine-source fixtures. No acceptance expectation changes.
Independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews are required.
Run fmt, all-target/all-feature Clippy denied warnings, both lint_policy suites including links/fragments, Checker/Core/VC owner groups, algorithm regressions and short proof-verification E2E. Full cargo test only on cadence; lane push:no has no full trigger. Run changed authentication/contract/profile matrices; defer unaffected evidence/policy/cache/edit-versus-clean heavy groups to sealed integration.
Exit: freeze committed before implementation; exact optional clause authenticated and retained, credit boundaries preserved, required gates pass.
