# Task VC53-PROOF-CALLER-E1-INHABITATION-NATIVE: bounded native caller inhabitation adoption
Canonical language: English; [Japanese pointer](../ja/VC53-PROOF-CALLER-E1-INHABITATION-NATIVE.md).
Status: frozen bounded adoption. Tier: full; new public Proof type/API and native adoption boundary fail pattern eligibility.
Owner: [Proof](../../mizar-proof/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [caller inhabitation association](VC53-PROOF-CALLER-E1-INHABITATION-ASSOCIATION.md), [existing object policy consumption](../../mizar-proof/en/policy.md#bounded-native-positive-result-policy).
Authority: [§§20.1.2, 20.2.6, 20.7.2, 20.13.1 E1](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [caller association](../../mizar-proof/en/status.md#arena-local-caller-e1-inhabitation-association), [Kernel introduction](../../mizar-kernel/en/checker.md#bounded-e1-inhabitation-witness-introduction), [native E1 pattern](../../mizar-proof/en/status.md).
Gap: source_drift/test_gap; whole authenticated association and actual-schema/current-policy consumption exist without native caller inhabitation adoption. No language-intent change or external prerequisite.

## Frozen design and boundaries
Add opaque `NativeUnaryCallerE1Inhabitation<'core>` in Proof status.rs with only one private existing `SourceUnaryCallerE1InhabitationAssociation<'core>` field.
`from_source(source, typed, symbols, core, request, policy)` calls the unchanged association constructor once, then consumes its retained actual Kernel result through unchanged `ProofPolicyEvaluator::consume_object_inhabitation`. Preserve upstream errors before policy refusal; discard the fingerprint and retain the whole association.
`adopt_original_request` takes the same six inputs. FIRST call the unchanged association `matches_source`, preserving authentication errors. Return false for whole-request/tuple or pointer-identical Core mismatch before policy checks, even under denied policy. On a match, consume the retained result through the same evaluator using the supplied current policy; return true only for this original caller object-inhabitation obligation. Keep existing `algorithm.proof.native_e1_policy_rejected` refusal.
Each adoption reevaluates current policy; previous success grants no authority. Currentness remains caller-owned. No detached constructor, getter, copied payload, snapshot, fingerprint, passive identity, generic helper or repeated Kernel check on adoption.
Preserve original guard/context/schema, fresh binder and actual-input witness; optional terminating intent grants no additional authority. Exclude passive E1 identity, typing/single-valuedness checking, E2, whole acceptance, status, currentness, complete footprints/cache, promotion/publication, broader profiles and owner-gated scope.
Files: Proof status.rs, existing runner tests/algorithm.rs, paired Proof status/plan documents and this contract. Lower-stage production, child/E4/direct-return and raw unknown/incomplete/cache-miss boundaries remain unchanged. No spec/.miz/expectation/trace/audit-status, diagnostic, AGENTS or protocol changes. Audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: four existing genuine-fixture families; establish one successful native construction/adoption before refusal expansion; no new shared fixture.
1. Genuine rename/trivia/equal-spelling and optional-terminating variants construct/adopt repeatedly; preserve existing independent whole tuple/Kernel, guard/schema/fresh-binder/actual-witness oracles and input/product purity.
2. Eligible current policies construct/adopt; empty/FormulaSubstitution-only, missing Builtin and wrong-schema policies refuse with the existing E1 policy error. Other formats cannot compensate; prior success cannot authorize denied policy, and valid recovery succeeds.
3. Complete existing whole request/Core drift and calibrated foreign/stale source/typed/environment/forgery plus combined errors preserve upstream errors before denied policy in constructor/adoption. Equal cloned Core refuses old holder before policy; fresh holder adopts its own arena. Fresh changed-source holder succeeds; cross-adoption refuses before policy.
4. Existing child/E4/direct-return controls, unsupported caller profile and raw unknown/incomplete/cache-miss boundaries remain unchanged; adoption grants no later authority.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, Proof/VC/Kernel owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed adoption/authentication/policy matrices run immediately; unchanged heavy evidence/cache/aggregate-acceptance/edit-versus-clean groups defer to Muse sealed integration. Full cargo test only on cadence; lane has no full/push trigger.
Exit: native adoption of only the whole original arena-local caller inhabitation request under current eligible policy, authentication before mismatch before policy, preserving all excluded authority.
