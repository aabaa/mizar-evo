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

General matrix: continuation, child/range evidence, failure/fallthrough, ghost/report/Pick/E1, provenance, postcondition/return/budget distinctions; promotion needs evidence.
Forbidden: MVM/E5, imported receipts, public diagnostics, .miz/expectation rebaselining, fabricated proofs/credit.
Audit: retain §20 deferrals.
## Bounded direct-return producer design

[idalgo fixture](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz) [VC profile](../../mizar-vc/en/generator.md#bounded-direct-return-e1e4-producer-design): object parameter/result, equality ensures, sole return, authenticated terminating intent.
Authenticated replay emits untrusted relation/Open4: E1 typing, single-valuedness, all-typed-input inhabitation and independent E4 normal-return existence. Fresh capture-free binders retain typed-input premises; preserve postcondition/Deferred termination.
Producer matrix remains VC-owned: output, typing, uniqueness, inhabitation and E4 isolation/refusals; proof-dependent controls deferred.
Exclude calls, loops, Pick, recursion, groups, premise exposure, checking/adoption, symbol introduction, promotion, acceptance/coverage, fixture/expectation changes and Lane 2. Parser reconciliation and mixed-group E2 remain excluded.
Exit: use the witness-request full-review and verification gates below.
## Existing request boundaries
[Proof identity](../../mizar-proof/en/status.md#source-algorithm-direct-return-request-identity): same-source regeneration, fixed four-role schemas, normalized goal/context and owner binding; inputs remain incomplete/reuse-ineligible, currentness caller-owned.
VC [witness reduction](../../mizar-vc/en/generator.md#e1-inhabitation-witness-reduction-request), [E4 reduction](../../mizar-vc/en/generator.md#e4-operational-reduction-request) and [validator handoff](../../mizar-vc/en/generator.md#e4-authenticated-validator-handoff) retain original identities, binders, witnesses and raw slices; detached substitution/local associations supply no proof. Kernel [conditional validator](../../mizar-kernel/en/checker.md#bounded-varsole-return-local-validation) leaves initialized/typed-input satisfaction to callers. Checking, existential introduction and adoption stay deferred.
## ID-free AlgorithmReturns dependencies
VC [collector](../../mizar-vc/en/dependency_slice.md#fingerprint-contract): fingerprint variable alias, definition presence, mutual equality and equality to return statement; retain unresolved semantics. No numeric IDs/API/type/proof/credit.
Five-case cap: ID-renumbering invariance; variable-alias sensitivity; definition-presence sensitivity; definition-equality/statement-alias sensitivity; nested collection preserving unresolved markers/IncompleteUncacheable/cache misses/statuses. Run VC/Proof/cache, source-return, E2E and witness-request gates; regression on cadence.

## Authenticated Core-semantic dependency contribution
VC [collector](../../mizar-vc/en/dependency_slice.md#authenticated-direct-return-core-contribution) regenerates the whole same-source/Core/CFG graph before comparing the supplied VcSet, using its snapshot/schema and the supplied generation schema. Reuse Core's existing semantic payload; accept no detached bytes or copied normalization. Append role/binder-bound contributions only for reachable original relations in typing, single-valuedness and E4 slices; leave inhabitation/postcondition unchanged. Exact regenerated graph equality binds algorithm/flow/statement, parameter/result roles, original binder/witness scope and selection. Plain collection remains structural.
Retain every unknown and raw remainder, IncompleteUncacheable/cache misses, Open4/Deferred. This is internal dependency input, not complete CFG/body interpretation, terminating policy, proof/adoption, evidence, symbol, promotion, publication or acceptance/coverage credit.
Five-case cap: authentic graph/contribution/remainder oracle; alpha/ID/relocation/trivia invariance with owner sensitivity; supported semantic changes and mismatched graph/profile refusals; stale/cross-source/check/symbol/Core/wrong-or-absent selection and schema/snapshot error precedence without partial output; independent identity/hash oracle, cache misses, statuses and purity. Reuse existing lower controls; integrate original descriptors and Proof identity consumer.
Exit: independent specification/test-intent, sufficiency, precision/trust, volume, final source/docs/ENJA and quality reviews; fmt, all-target/all-feature Clippy, both lint_policy suites, VC/Proof/cache groups, source-return tests, proof-verification E2E and scoped links/fragments. Full regression only on cadence. No fixture, expectation, trace or coverage changes.
