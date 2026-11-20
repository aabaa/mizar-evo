# Task ALGDEF-VC53-NORMAL-RETURN: Normal-return and function-graph prerequisites
Canonical language: English; [Japanese pointer](../ja/ALGDEF-VC53-NORMAL-RETURN.md).
Status: bounded request producers; proof/adoption blocked. Tier: full gates. Owners: [Core plan](../../mizar-core/en/00.crate_plan.md), [VC plan](../../mizar-vc/en/00.crate_plan.md), [proof plan](../../mizar-proof/en/00.crate_plan.md).
Consumers: mizar-test, local promotion; imports remain ALGDEF-VC59-IMPORTED-PROMOTION-FACTS.
Dependencies: ALGDEF-PARSER-FAIL; authentic Core 42–53 call/range/group/CFG inputs and replayable VC formulas; proof/kernel results. Bounded producer uses Core46.
Authority: [§20.2.6, §20.3, §20.7.2, E1–E5 and VC schema](../../../spec/en/20.algorithm_and_verification.md); [algorithm graph](../../mizar-vc/en/source_vc_decomposition.md#accepted-algorithm-vc-task-graph).
Classification: source_drift/test_gap; bounded profiles confer no general acceptance.
Owner details: [Core/CFG graph](../../mizar-core/en/source_family_decomposition.md#accepted-phase-10-controlflowir-task-graph), [VC generator](../../mizar-vc/en/generator.md#step-4-algorithm-vcs), [proof status](../../mizar-proof/en/status.md#source-existential-registration-proof).

## Boundary and invariants

Core 43/48/53 carry validated failure placement, runtime report terms and
return-or-fail closure; failure contributes no normal/return/break/continue
context or postcondition. Void fallthrough returns implicitly; value fallthrough
is rejected. Ghost execution cannot fail or influence runtime execution.
VC 44 adds fresh returns and declared ensures on a partial body's normal call
continuation, including recursive induction while checking E2; external E2
adoption requires verified body correctness. Continuation proves no E4.
VC 53 admits independently checked exact-call normal-return evidence for
terminating child calls and iteration ranges; stopping or value equality is
insufficient. Core supplies identities; VC substitutes without capture; proof
owns actual acceptance. Declarations/open goals grant no status.
Before E1 symbol introduction justify single-valued execution, typing and
inhabitation on every input where its return type is asserted; E2 needs body
correctness, E4 independently verified normal return. Keep environment/InputOK
guards for syntax-bearing inputs with AST-CONTRACT. Ghost Pick requires static
discharge; non-ghost unsupported set Pick may fail only in partial execution.
Missing evidence stays incomplete/open.

Regressions: [return](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz), [ghost rejection](../../../../tests/miz/fail/algorithms/fail_type_elaboration_algorithm_ghost_isolation_001.miz).

## Tests and acceptance

General matrix: partial/recursive continuation, child/range return evidence, failure,
void/value fallthrough, ghost/report leakage, empty Pick and E1 failures. Compare whole outputs and stale/cross-call/group/provenance refusals; distinguish postconditions, normal return and budget failure.
Acceptance: only complete source-authenticated inputs produce the specified
contexts/goals; only genuine checked evidence enables local promotion.
Forbidden: MVM/E5 execution, imported receipts, public diagnostic allocation,
existing .miz/expectation rebaselining, fabricated proofs or coverage credit.
Audit: retain §20 deferrals.

## Bounded direct-return producer design

Implement the [idalgo fixture](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz) profile in the [VC owner](../../mizar-vc/en/generator.md#bounded-direct-return-e1e4-producer-design): one object parameter/result, equality ensures, sole direct return, authenticated terminating intent.
A source/check/symbol/Core replayed producer emits an untrusted execution-relation atom and four Open requests using existing VcSet/formula/context representations: E1 return typing, execution single-valuedness, all-typed-input result inhabitation and independent E4 normal-return existence. Bind fresh result variables without capture; keep only the typed-input premise. Preserve the existing postcondition output and Deferred termination accounting.
Producer matrix: whole output/context/provenance; missing/cross-bound typing; two-result uniqueness; all-input inhabitation without return/P; E4 isolation with stale/cross-source/Core/CFG/schema controls. Proof-dependent controls remain deferred.
Exclude calls, loops, Pick, recursion, groups, premise exposure, Proof/Kernel checking/adoption, symbol introduction, promotion, acceptance/coverage, fixture/expectation changes and Lane 2 integration. Parser reconciliation and mixed-group E2 stay outside scope.
Exit: use the witness-request full-review and verification gates below.

## Bounded Proof request inputs and identity
The [Proof owner](../../mizar-proof/en/status.md#source-algorithm-direct-return-request-identity) owns canonical inputs and original identities for all four requests: same-source regeneration, fixed schemas, role-specific domains, normalized goal/context and source-owner binding. Reuse Core/guard/syntax producers, CanonicalJson, Hash, ProofObligationIdentity and whole raw DependencySlice; no parallel representation. Reject old postconditions, absent selections and unsupported profiles. Inputs remain incomplete/reuse-ineligible; currentness stays caller-owned.
Five-case matrix: independent four-role hashes/whole slices; candidate/final identity vectors and purity; trivia/relocation/owner/parameter/snapshot edits; exact upstream errors with cross-source/check/symbol inputs; profile/selection refusals and Open/Deferred preservation. No checking, adoption, evidence, symbol introduction, promotion, publication or acceptance/coverage. Typing/uniqueness/E4 await an operational bridge and Kernel/Proof checking.

## E1 inhabitation witness-reduction request
VC freezes the [bounded witness descriptor](../../mizar-vc/en/generator.md#e1-inhabitation-witness-reduction-request): authenticate the whole four-request graph, bind actual returned a, substitute its fresh existential binder without capture and return child object(a), original existential identity and whole raw slice. This is request-only; checking and Proof adoption remain deferred.
Five-case cap: independent whole descriptor/parent-identity/slice oracle; actual renamed/relocated witness and fresh binder; other-role/absent selection refusals; stale/cross-source/check/symbol/Core refusals with upstream error order; schema/snapshot isolation and unchanged Open/Deferred/purity. Reuse existing lower corruption controls. No fixture, expectation, language-specification or Lane 2 changes.
Exit: independent full reviews; VC and Proof owner/source-return tests, proof-verification E2E, fmt, Clippy, both lint_policy suites, links/fragments; full regression only on cadence.
