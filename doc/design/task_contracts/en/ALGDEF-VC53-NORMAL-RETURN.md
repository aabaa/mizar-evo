# Task ALGDEF-VC53-NORMAL-RETURN: Normal-return and function-graph prerequisites
Canonical language: English; [Japanese pointer](../ja/ALGDEF-VC53-NORMAL-RETURN.md).
Status: bounded requests, validation and structural dependencies; bounded inhabitation adoption; broader proof blocked. Tier: full gates. Owners: [Core plan](../../mizar-core/en/00.crate_plan.md), [VC plan](../../mizar-vc/en/00.crate_plan.md), [proof plan](../../mizar-proof/en/00.crate_plan.md), [Kernel plan](../../mizar-kernel/en/00.crate_plan.md).
Consumers: mizar-test/promotion; import: ALGDEF-VC59-IMPORTED-PROMOTION-FACTS.
Dependencies: ALGDEF-PARSER-FAIL; Core42–53/CFG, VC formulas, proof/kernel results; producer Core46.
Authority: [§20.2.6, §20.3, §20.7.2, E1–E5 and VC schema](../../../spec/en/20.algorithm_and_verification.md); [algorithm graph](../../mizar-vc/en/source_vc_decomposition.md#accepted-algorithm-vc-task-graph).
Classification: source_drift/test_gap; no profile acceptance.
Owner details: [Core/CFG graph](../../mizar-core/en/source_family_decomposition.md#accepted-phase-10-controlflowir-task-graph), [VC generator](../../mizar-vc/en/generator.md#step-4-algorithm-vcs), [proof status](../../mizar-proof/en/status.md#source-existential-registration-proof).
## Boundary and invariants

Core43/48/53: failure gives no continuation/postcondition; void fallthrough returns, value fallthrough rejects; ghost cannot fail/influence runtime.
VC44 retains partial-call returns/ensures and recursive E2; external E2 needs verified body correctness. Continuation proves no E4.
VC 53 requires independently checked exact-call evidence for
terminating child calls/ranges; stopping/equality is insufficient. Core owns
identities, VC capture-free substitution, Proof acceptance. Open goals grant no status.
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
VC [witness reduction](../../mizar-vc/en/generator.md#e1-inhabitation-witness-reduction-request), [E4 reduction](../../mizar-vc/en/generator.md#e4-operational-reduction-request) and [validator handoff](../../mizar-vc/en/generator.md#e4-authenticated-validator-handoff) retain original identities, binders, witnesses and raw slices; detached substitution/local associations supply no proof. Kernel [conditional validator](../../mizar-kernel/en/checker.md#bounded-varsole-return-local-validation) leaves initialized/typed-input satisfaction to callers. Original E4 checking and Proof identity association are bounded below; adoption stays deferred.
## Dependency boundaries
VC [structural/authenticated collector](../../mizar-vc/en/dependency_slice.md#authenticated-direct-return-core-contribution) retains ID-free alias structure, authenticated reachable Core contributions and every unknown/raw remainder. Plain collection remains structural; slices stay incomplete/reuse-ineligible.

## Original E4 Proof request-identity association
[Existing existential rule](../../mizar-kernel/en/checker.md#original-algorithmreturns-existential) and [VC result producer](../../mizar-vc/en/generator.md#e4-checked-original-existential) retain exact tuple, binder, witness, context and rebuilt CFG checks. [Native E1 adoption](../../mizar-proof/en/status.md#native-e1-inhabitation-adoption) stays separate.
Authority: §20.1.2, §20.2.6, §20.7.2, §20.13.1 and [existential introduction §16.7.3](../../../spec/en/16.theorems_and_proofs.md#1673-skeleton-encoding).
Proof [same-call association](../../mizar-proof/en/status.md#original-e4-request-identity-association) authenticates the VC result before regenerating original identity on identical inputs and requiring whole raw-slice equality; errors remain producer-first, with no partial association.
Association alone supplies no policy authority. Source/snapshot/filesystem currentness remains caller-owned. Preserve Open4/Deferred, raw unknowns, IncompleteUncacheable/cache misses and all exclusions. Formula-cycle hardening remains separate PRODUCER-MISSING; no formula-cycle/general-cycle safety claim. R3-Q2/R5-Q2 remain G/RESERVED/unassigned.

## Kernel-owned E4 checker-schema provenance
`AlgorithmReturnsExistentialCheck::checker_schema_version() -> u16` reports Kernel-owned schema `1` for the existing original-existential rule, without caller-requested versions or a new type/field.

## Proof-only E4 current-policy consumption
Full tier: new public seam and E4 policy consumer fail the pattern criteria. Add `consume_source_algorithm_e4_existential(source, check, symbols, core, snapshot, vc_id, policy)` around the unchanged association. Propagate association errors first; inline require enabled BuiltinKernelEvidence and exact agreement between current policy and genuine result checker schemas, otherwise `algorithm.proof.native_e4_policy_rejected`. Atomically return original identity, whole raw slice, borrowed result and actual `policy.policy_fingerprint()`.
No helper/type/field, detached inputs, generic evidence/status conversion, adoption, semantic environment interpretation, separate E1 discharge or broader credit. Affect Proof status, existing source-return tests and necessary paired Proof owners/inventories only; no Kernel/VC/Core production, spec, fixture/expectation, trace/coverage, protocol or AGENTS edits.
Two focused test groups, cap two: (1) genuine association outputs match whole identity/slice/result and actual fingerprints under permitted current-policy variants, with repeatability/purity; (2) disabled Builtin/unsupported formats and mismatching checker schemas refuse, while real upstream errors precede policy refusals. Reuse lower tuple/binder/role/id, raw-state, alpha/ID relocation and E1 controls.
Exit: independent specification/test-intent, sufficiency, precision/trust, volume, final source/docs/ENJA and quality reviews; fmt, all-target/all-feature Clippy, both lint_policy suites, Kernel/VC/Proof owners, source-return, proof-verification E2E and scoped links/fragments. Full cargo test follows cadence.
Pattern follow-up: predecessor `42e7becee`; add Builtin-only policy to success variants, preserving assertions.
