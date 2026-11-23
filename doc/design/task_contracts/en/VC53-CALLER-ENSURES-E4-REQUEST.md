# Task VC53-CALLER-ENSURES-E4-REQUEST: clause-bearing caller existential request
Canonical language: English; [Japanese pointer](../ja/VC53-CALLER-ENSURES-E4-REQUEST.md).
Status: frozen request-only producer. Tier: full; new public API fails pattern eligibility.
Owner: [VC plan](../../mizar-vc/en/00.crate_plan.md). Consumer: mizar-test.
Dependency: [authenticated E2 request](VC53-UNARY-CALLER-E2-REQUEST.md).
Authority: [§20.1.2, §20.2.6, §20.7.2, §20.13.1 E2/E4](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [E2 authentication](../../mizar-vc/en/generator.md#bounded-unary-caller-e2-request), [legacy E4](../../mizar-vc/en/generator.md#bounded-unary-caller-e4-descriptor).
Gap: source_drift/test_gap; authenticated clause-bearing caller relations lack an existential request. No language-intent change or external prerequisite.

## Frozen design and boundaries
Add only `generate_source_unary_child_call_caller_ensures_e4_request(source, typed, symbols, core, request)` in VC generator.rs, returning existing `Result<(LocalContext, [VcGeneratedFormula; 2]), String>`.
Call the unchanged E2 request producer once first on identical inputs and propagate errors unchanged. Retain its exact context (sole original caller object guard, fresh result BinderDeclaration, empty policy inputs) and AlgorithmReturns row, including caller/flow/Return/actual/result joins and metadata.
Project the original quantified binder into Exists with body Generated(0) and row ID 1; preserve SplitGoal kind and empty provenance. Discard the equality, implication and universal goal. Impossible local shape refusal reuses `algorithms.unary_child.invalid_caller_e2_request`.
The output requests only `object(actual) |- exists r. returns_caller(actual,r)`. It installs no return, equality, ensures or terminating intent as a premise and provides no witness, checking, Kernel/Proof evidence, identity, VcSet, seed or acceptance credit.
Keep inputs immutable and repeatable; equal cloned Core regenerates the tuple. Legacy descriptor/E1/E4/child/direct-return/Proof routes still refuse caller ensures, while E2 and this API require the existing exact clause.
Exclude schema retention, policy/native adoption, whole acceptance/status, caller-owned currentness, complete footprints/cache, promotion/publication, broader profiles and owner-gated work. No lower-stage, spec/.miz/expectation/trace, diagnostic, AGENTS or protocol edits. Audit coverage/owner/deferral unchanged.
Files: VC generator.rs, existing runner tests/algorithm.rs, paired VC generator/plan documents and this contract; no new type, field, helper, fixture or representation.

## Test matrix and exit
Cap: four existing fixture families; reuse the established genuine clause-bearing E2 setup and pass one E4 projection case before expanding.
1. Independent exact existential oracle: sole original guard, fresh binder, exact caller relation, Exists/body/IDs/metadata, empty policy inputs; exact E2 context/relation projection and purity/repetition.
2. Existing genuine rename/trivia/equal-spelling/optional-terminating variants and equal cloned Core preserve joins and regenerate the tuple.
3. Complete existing request/Core drift, source/typed/environment and combined-error cases refuse with the exact E2 error in authentication order; authentic changed inputs succeed only together and original recovers.
4. Existing unsupported/malformed/no-clause/direct-return controls preserve E2 refusals; old clause-bearing E1/E4/descriptor/child/Proof refusals and no-clause downstream controls remain valid; no later credit.
Existing source oracle: [ensures return](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz); existing runner caller-ensures and drift matrices. No changed test intent.
Independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews required.
Run fmt, all-target/all-feature Clippy denied warnings, both lint_policy suites including links/fragments, VC owner tests, algorithm/source_algorithm regressions and short proof-verification E2E. Changed request/authentication matrices run; unaffected heavy evidence/policy/cache/aggregate-acceptance/edit-versus-clean defer to Muse sealed integration. Full cargo test only on cadence; lane push:no has no full trigger.
Exit: freeze committed before implementation; exact freshly authenticated unproved existential request available, legacy boundaries preserved, required gates pass.
