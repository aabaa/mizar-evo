# Task ALGDEF-VC53-NORMAL-RETURN: Normal-return and function-graph prerequisites
Canonical language: English; [Japanese pointer](../ja/ALGDEF-VC53-NORMAL-RETURN.md).
Status: bounded producer ready; request identity frozen; proof/adoption blocked. Tier: full gates. Owners: [Core plan](../../mizar-core/en/00.crate_plan.md), [VC plan](../../mizar-vc/en/00.crate_plan.md), [proof plan](../../mizar-proof/en/00.crate_plan.md).
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
void/value fallthrough, ghost/report leakage, empty Pick and E1 failures. Compare
whole Core/CFG/VC outputs and stale/cross-call/group/provenance refusals; distinguish
postconditions, normal return and finite-budget failure.
Acceptance: only complete source-authenticated inputs produce the specified
contexts/goals; only genuine checked evidence enables local promotion.
Forbidden: MVM/E5 execution, imported receipts, public diagnostic allocation,
existing .miz/expectation rebaselining, fabricated proofs or coverage credit.
Audit: retain §20 deferred ownership until executable source slices land.
Reviews and verification: full protocol gates, including EN/JA consistency.

## Bounded direct-return producer design

Implement the [idalgo fixture](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz) profile in the [VC owner](../../mizar-vc/en/generator.md#bounded-direct-return-e1e4-producer-design): one object parameter/result, equality ensures, sole direct return, authenticated terminating intent.
A source/check/symbol/Core replayed producer emits an untrusted execution-relation atom and four Open requests using existing VcSet/formula/context representations: E1 return typing, execution single-valuedness, all-typed-input result inhabitation and independent E4 normal-return existence. Bind fresh result variables without capture; keep only the typed-input premise. Preserve the existing postcondition output and Deferred termination accounting.
Producer matrix: whole output/context/provenance; missing/cross-bound typing; two-result uniqueness; all-input inhabitation without return/P; E4 isolation with stale/cross-source/Core/CFG/schema controls. Proof-dependent controls remain deferred.
Exclude calls, loops, Pick, recursion, groups, premise exposure, Proof/Kernel checking/adoption, symbol introduction, promotion, acceptance/coverage, fixture/expectation changes and Lane 2 integration. Parser reconciliation and mixed-group E2 stay outside scope.
Exit: independent whole-unit reviews, VC owner tests, source-to-request E2E, fmt, Clippy, both lint_policy suites and local links/fragments; full regression only on cadence. Representation changes no semantics; proof/adoption remain deferred.

## Bounded Proof request inputs and identity
Freeze then implement canonical inputs and identity for four idalgo requests; retain Core46 terminating-intent transport and Deferred accounting.
Regenerate from the same source/check/symbols with fixed schemas; reject old postcondition, absent selections and unsupported profiles. Reuse semantic Core, guard and syntax producers, CanonicalJson, Hash, ProofObligationIdentity and whole raw DependencySlice; no parallel representation or VC API expansion.
The [Proof owner](../../mizar-proof/en/status.md#source-algorithm-direct-return-request-identity) fixes role-specific domains, normalized goal/context content and source-owner binding. Inputs remain request-only, incomplete and reuse-ineligible; source/snapshot/filesystem currentness stays caller-owned.
Five-case matrix: independent four-role canonical/hash/whole-slice oracle; independent candidate/final identity vectors and purity; real trivia/relocation/owner/parameter/snapshot edits; exact upstream-error ordering with genuine cross-source/check/symbol inputs; old-postcondition/absent/profile refusals and unchanged Open/Deferred accounting. Reuse lower corruption controls; never invent proof evidence.
Exclude checking/adoption, symbol introduction, promotion, acceptance/coverage, publication and all broader blocked work. Inhabitation awaits genuine existential-witness evidence; typing/uniqueness/E4 await an authenticated operational bridge and Kernel checking, owned by VC/Kernel/Proof.
Exit: independent full reviews; Proof owner and source-return tests, proof-verification E2E, fmt, Clippy, both lint_policy suites, links/fragments and cadence-triggered full regression. No language decision.
