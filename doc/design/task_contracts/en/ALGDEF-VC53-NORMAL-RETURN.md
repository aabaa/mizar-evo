# Task ALGDEF-VC53-NORMAL-RETURN: Normal-return and function-graph prerequisites
Canonical language: English; [Japanese pointer](../ja/ALGDEF-VC53-NORMAL-RETURN.md).
Status: bounded requests/adoption; broader proof blocked. Tier: full gates. Owners: [Core plan](../../mizar-core/en/00.crate_plan.md), [VC plan](../../mizar-vc/en/00.crate_plan.md), [proof plan](../../mizar-proof/en/00.crate_plan.md), [Kernel plan](../../mizar-kernel/en/00.crate_plan.md).
Consumers: mizar-test/promotion.
Dependencies: ALGDEF-PARSER-FAIL; Core42–53/CFG, VC formulas, proof/kernel results; producer Core46.
Authority: [§20.2.6, §20.3, §20.7.2, E1–E5 and VC schema](../../../spec/en/20.algorithm_and_verification.md); [algorithm graph](../../mizar-vc/en/source_vc_decomposition.md#accepted-algorithm-vc-task-graph).
Classification: source_drift/test_gap.
Owner details: [Core/CFG graph](../../mizar-core/en/source_family_decomposition.md#accepted-phase-10-controlflowir-task-graph), [VC generator](../../mizar-vc/en/generator.md#step-4-algorithm-vcs), [proof status](../../mizar-proof/en/status.md#source-existential-registration-proof).
## Boundary and invariants

Core43/48/53: failure gives no continuation/postcondition; void fallthrough returns, value fallthrough rejects; ghost cannot fail/influence runtime.
VC44 retains partial-call returns/ensures and recursive E2; external E2 needs verified body correctness. Continuation proves no E4.
VC53 requires exact-call evidence for terminating children/ranges; stopping/equality is insufficient. Core owns identities, VC substitution, Proof acceptance. Open grants no status.
E1 needs single-valuedness, typing and inhabitation wherever return typing is asserted; E2 needs body correctness, E4 verified normal return. Keep environment/InputOK
guards for syntax-bearing inputs with AST-CONTRACT. Ghost Pick requires static
discharge; non-ghost unsupported set Pick may fail only in partial execution.
Missing evidence stays Open.

Regressions: [return](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz), [ghost rejection](../../../../tests/miz/fail/algorithms/fail_type_elaboration_algorithm_ghost_isolation_001.miz).
## Tests and acceptance

Forbidden: MVM/E5, imported receipts, public diagnostics, .miz/expectation rebaselining, fabricated proofs/credit.
## Bounded direct-return producer design

[idalgo fixture](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz) [VC profile](../../mizar-vc/en/generator.md#bounded-direct-return-e1e4-producer-design): object parameter/result, equality ensures, sole return, authenticated terminating intent.
Replay emits relation/Open4: E1 typing, single-valuedness, all-typed-input inhabitation and independent E4 normal-return existence. Fresh capture-free binders retain typed-input premises; preserve postcondition/Deferred termination.
Exclude calls, loops, Pick, recursion, groups, premise exposure, symbol introduction, promotion, acceptance/coverage, fixture/expectation changes. Parser reconciliation and mixed-group E2 remain excluded.
## Existing request boundaries
[Proof identity](../../mizar-proof/en/status.md#source-algorithm-direct-return-request-identity): same-source regeneration, fixed four-role schemas, normalized goal/context and owner binding; inputs remain incomplete/reuse-ineligible, currentness caller-owned.
VC [witness reduction](../../mizar-vc/en/generator.md#e1-inhabitation-witness-reduction-request), [E4 reduction](../../mizar-vc/en/generator.md#e4-operational-reduction-request) and [validator handoff](../../mizar-vc/en/generator.md#e4-authenticated-validator-handoff) retain original identities, binders, witnesses and raw slices; detached substitution/local associations supply no proof. Kernel [conditional validator](../../mizar-kernel/en/checker.md#bounded-varsole-return-local-validation) leaves initialized/typed-input satisfaction to callers.
## Dependency boundaries
VC [structural/authenticated collector](../../mizar-vc/en/dependency_slice.md#authenticated-direct-return-core-contribution) retains ID-free alias structure, authenticated reachable Core contributions and every unknown/raw remainder. Plain collection remains structural; slices stay incomplete/reuse-ineligible.

## Original E4 Proof request-identity association
[Existing existential rule](../../mizar-kernel/en/checker.md#original-algorithmreturns-existential) and [VC result producer](../../mizar-vc/en/generator.md#e4-checked-original-existential) retain exact tuple, binder, witness, context and rebuilt CFG checks. [Native E1 adoption](../../mizar-proof/en/status.md#native-e1-inhabitation-adoption) stays separate.
Authority: §20.1.2, §20.2.6, §20.7.2, §20.13.1 and [existential introduction §16.7.3](../../../spec/en/16.theorems_and_proofs.md#1673-skeleton-encoding).
Proof [same-call association](../../mizar-proof/en/status.md#original-e4-request-identity-association) authenticates the VC result before regenerating original identity on identical inputs and requiring whole raw-slice equality; errors remain producer-first, with no partial association.
Association alone supplies no policy authority. Source/snapshot/filesystem currentness remains caller-owned. Preserve Open4/Deferred, raw unknowns, IncompleteUncacheable/cache misses and all exclusions. Formula-cycle hardening remains separate PRODUCER-MISSING; no formula-cycle/general-cycle safety claim. R3-Q2/R5-Q2 remain G/RESERVED/unassigned.

## Original E1 typing descriptor
The [VC descriptor](../../mizar-vc/en/generator.md#original-e1-typing-descriptor) retains the authentic original Forall result binder, returns antecedent, object conclusion, sole typed-input guard and whole graph/raw slice. Source/Core/CFG remain producer-owned.

## Bounded E1 typing check
Full tier: new public Kernel checking seam. Authority: immutable parameters §20.1.2, sole Return §20.2.6 and E1 §20.13.1. Kernel checks only `forall r. returns(a,r) implies object(r)` under the original `object(a)` guard: one distinct parameter/result, sole owned `Return(Var(a))`, exact relation/CFG, fresh result binder, matching atomic object conclusion and original singleton typed-input context. Validate both Core parameter/result object guards; reject opaque types, malformed/cross-bound inputs and diagnostics. Use tuples/references and boolean success; no new result/evidence type. Immutable-parameter/sole-Return semantics justify result typing, independently of E4 existence or converse reasoning.
VC `check_source_algorithm_e1_typing_request` first regenerates the existing descriptor, preserving producer errors, then projects its original binder/antecedent/conclusion/guard to Kernel. Return the unchanged descriptor only on success; otherwise refuse.
Scope: Kernel checker, VC generator, Kernel/source-return tests and paired Kernel/VC owner/API documents. No Proof/Core production, environment interpretation, single-valuedness, native evidence/adoption, symbol introduction, promotion, acceptance/coverage, reuse, fixture/expectation or trace changes. Preserve Open4/Deferred, unknowns, incomplete/cache-misses, caller currentness, registered questions and cycle exclusions above.
Tests, cap four groups: Kernel success plus VC descriptor/purity/repeatability; Kernel wrong relation/binder/conclusion/context/guard/sole-return/CFG/diagnostic refusals; VC wrong-role/ID and source/Core/guard/CFG mutation refusals with upstream errors first; renamed/relocated input and fresh binder success. Reuse existing snapshot/schema and raw-state controls.
Exit: specification/test-intent, sufficiency, precision/trust, volume, source/docs/ENJA and quality reviews; fmt, Clippy, both lint_policy suites, Kernel/VC owners, source-return, proof-verification E2E and links/fragments; full tests follow lane cadence.
