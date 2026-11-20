# Task ALGDEF-VC53-NORMAL-RETURN: Normal-return and function-graph prerequisites
Canonical language: English; [Japanese pointer](../ja/ALGDEF-VC53-NORMAL-RETURN.md).
Status: bounded requests, validation and structural dependencies; proof/adoption blocked. Tier: full gates. Owners: [Core plan](../../mizar-core/en/00.crate_plan.md), [VC plan](../../mizar-vc/en/00.crate_plan.md), [proof plan](../../mizar-proof/en/00.crate_plan.md), [Kernel plan](../../mizar-kernel/en/00.crate_plan.md).
Consumers: mizar-test/promotion; imports: ALGDEF-VC59-IMPORTED-PROMOTION-FACTS.
Dependencies: ALGDEF-PARSER-FAIL; Core42–53/CFG, VC formulas, proof/kernel results; producer Core46.
Authority: [§20.2.6, §20.3, §20.7.2, E1–E5 and VC schema](../../../spec/en/20.algorithm_and_verification.md); [algorithm graph](../../mizar-vc/en/source_vc_decomposition.md#accepted-algorithm-vc-task-graph).
Classification: source_drift/test_gap; profiles grant no acceptance.
Owner details: [Core/CFG graph](../../mizar-core/en/source_family_decomposition.md#accepted-phase-10-controlflowir-task-graph), [VC generator](../../mizar-vc/en/generator.md#step-4-algorithm-vcs), [proof status](../../mizar-proof/en/status.md#source-existential-registration-proof).
## Boundary and invariants

Core43/48/53: failure gives no continuation/postcondition; void fallthrough returns, value fallthrough rejects; ghost cannot fail/influence runtime.
VC44 retains partial-call returns/ensures and recursive E2; external E2 needs verified body correctness. Continuation proves no E4.
VC 53 requires independently checked exact-call normal-return evidence for
terminating child calls/ranges; stopping/equality is insufficient. Core owns
identities, VC capture-free substitution, Proof acceptance. Open goals grant no status.
E1 needs single-valuedness, typing and inhabitation wherever return typing is asserted; E2 needs body correctness, E4 verified normal return. Keep environment/InputOK
guards for syntax-bearing inputs with AST-CONTRACT. Ghost Pick requires static
discharge; non-ghost unsupported set Pick may fail only in partial execution.
Missing evidence stays Open.

Regressions: [return](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz), [ghost rejection](../../../../tests/miz/fail/algorithms/fail_type_elaboration_algorithm_ghost_isolation_001.miz).
## Tests and acceptance

General matrix: continuation, failure/fallthrough, ghost/Pick/E1, provenance and postcondition/return/budget distinctions; promotion needs evidence.
Forbidden: MVM/E5, imported receipts, public diagnostics, .miz/expectation rebaselining, fabricated proofs/credit.
Audit: retain §20 deferrals.
## Bounded direct-return producer design

[idalgo fixture](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz) [VC profile](../../mizar-vc/en/generator.md#bounded-direct-return-e1e4-producer-design): object parameter/result, equality ensures, sole return, authenticated terminating intent.
Authenticated replay emits untrusted relation/Open4: E1 typing, single-valuedness, all-typed-input inhabitation and independent E4 normal-return existence. Fresh capture-free binders retain typed-input premises; preserve postcondition/Deferred termination.
Producer matrix remains VC-owned: output, typing, uniqueness, inhabitation and E4 isolation/refusals; proof-dependent controls deferred.
Exclude calls, loops, Pick, recursion, groups, premise exposure, adoption, symbol introduction, promotion, acceptance/coverage, fixture/expectation changes and Lane 2. Parser reconciliation and mixed-group E2 remain excluded.
Exit: use the witness-request full-review and verification gates below.
## Existing request boundaries
[Proof identity](../../mizar-proof/en/status.md#source-algorithm-direct-return-request-identity): same-source regeneration, fixed four-role schemas, normalized goal/context and owner binding; inputs remain incomplete/reuse-ineligible, currentness caller-owned.
VC [witness reduction](../../mizar-vc/en/generator.md#e1-inhabitation-witness-reduction-request), [E4 reduction](../../mizar-vc/en/generator.md#e4-operational-reduction-request) and [validator handoff](../../mizar-vc/en/generator.md#e4-authenticated-validator-handoff) retain original identities, binders, witnesses and raw slices; detached substitution/local associations supply no proof. Kernel [conditional validator](../../mizar-kernel/en/checker.md#bounded-varsole-return-local-validation) leaves initialized/typed-input satisfaction to callers. E4 checking/introduction and Proof adoption stay deferred.
## Dependency boundaries
VC [structural/authenticated collector](../../mizar-vc/en/dependency_slice.md#authenticated-direct-return-core-contribution) retains ID-free alias structure, authenticated reachable Core contributions and every unknown/raw remainder. Plain collection remains structural; slices stay incomplete/reuse-ineligible.

## Bounded native E1 result consumption
Authority: [§16.3.3](../../../spec/en/16.theorems_and_proofs.md#1633-skeleton-decomposition-rules), [§16.7.3](../../../spec/en/16.theorems_and_proofs.md#1673-skeleton-encoding) and E1 above. Internal representation only. Full tier: new Proof consumer/native-policy seam fails same-owner/consumer/public-seam criteria.
Kernel [native result](../../mizar-kernel/en/checker.md#bounded-e1-inhabitation-witness-introduction) retains immutable Core, original binder/context and fixed positive object existential; expose its Kernel-owned checker schema version, never copy the requested policy version. Opaque type encodes rule.
Proof obtains that result through the authentic VC producer for the exact selected request and supplied Core/snapshot, using identity schemas. Preserve producer errors first; derive unchanged original identity and compare whole slices. Return identity, whole slice, native result and current policy fingerprint atomically. No caller token/hash/slice/success or duplicate checking. Freshness remains caller-owned.
Proof [policy](../../mizar-proof/en/policy.md#kernel-scheduling-policy) consumes the genuine positive result in a separate narrow path: require enabled built-in Kernel evidence and exact checker schema agreement; otherwise refuse. Fingerprint the supplied policy only after these checks. Post-check eligibility grants no selection class. Existing generic result/hash/polarity paths remain unchanged.
Five cases: (1) genuine fixture: independent original identity/context/fixed conclusion/whole-slice oracle and native policy consumption; (2) wrong role/selection, foreign source/Core or binder/context/schema/snapshot association refuses, reusing producer controls and preserving supported alpha/ID cases; (3) default/development/interactive, disabled format and mismatched checker policies, changed policy cannot reuse prior authorization; (4) producer refusal/upstream error precedence over policy and atomic return/purity; (5) preserve unknown/raw remainder, IncompleteUncacheable/cache misses, Open4/Deferred and independent typing/single-valuedness. Cap five.
Defer adoption, selection, status projection, E4 checking/introduction, evidence/symbol/promotion/publication and acceptance/coverage credit. No generic Accepted/check-kind/origin/taint/axiom/SAT/final-goal fields, caller fingerprint, refutation polarity or accepted-evidence hash. No spec/test-oracle/trace/coverage/protocol/AGENTS edits.
Exit: independent specification/test-intent, sufficiency, precision/trust, volume, final source/docs/ENJA and quality reviews; fmt, all-target/all-feature Clippy, both lint_policy suites, Kernel/VC/Proof owners, source-return, proof-verification E2E and scoped links/fragments. Changed policy/identity obligations run; full follows cadence.
