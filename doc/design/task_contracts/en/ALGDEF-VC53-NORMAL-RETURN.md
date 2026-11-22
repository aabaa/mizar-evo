# Task ALGDEF-VC53-NORMAL-RETURN: Normal-return and function-graph prerequisites
Canonical language: English; [Japanese pointer](../ja/ALGDEF-VC53-NORMAL-RETURN.md).
Status: bounded requests/adoption; broader proof blocked. Tier: full gates. Owners: [Core plan](../../mizar-core/en/00.crate_plan.md), [VC plan](../../mizar-vc/en/00.crate_plan.md), [proof plan](../../mizar-proof/en/00.crate_plan.md), [Kernel plan](../../mizar-kernel/en/00.crate_plan.md).
Consumers: mizar-test.
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
Replay emits relation/Open4: E1 typing, single-valuedness, all-typed-input inhabitation and independent E4 normal-return existence. Capture-free binders retain typed-input premises and postcondition/Deferred termination.
Exclude calls, loops, Pick, recursion, groups, premise exposure, symbol introduction, promotion, acceptance/coverage, fixture/expectation changes. Parser reconciliation and mixed-group E2 remain excluded.
## Existing request boundaries
[Proof identity](../../mizar-proof/en/status.md#source-algorithm-direct-return-request-identity): same-source regeneration, fixed four-role schemas, normalized goal/context and owner binding; inputs remain incomplete/reuse-ineligible, currentness caller-owned.
VC [witness reduction](../../mizar-vc/en/generator.md#e1-inhabitation-witness-reduction-request), [E4 reduction](../../mizar-vc/en/generator.md#e4-operational-reduction-request) and [validator handoff](../../mizar-vc/en/generator.md#e4-authenticated-validator-handoff) retain original identities, binders, witnesses and raw slices; detached substitution/local associations supply no proof. Kernel [conditional validator](../../mizar-kernel/en/checker.md#bounded-varsole-return-local-validation) leaves initialized/typed-input satisfaction to callers.
## Dependency boundaries
VC [structural/authenticated collector](../../mizar-vc/en/dependency_slice.md#authenticated-direct-return-core-contribution) retains ID-free alias structure, authenticated reachable Core contributions and every unknown/raw remainder. Plain collection remains structural; slices stay incomplete/reuse-ineligible.

## Original E4 Proof request-identity association
[Proof same-call association](../../mizar-proof/en/status.md#original-e4-request-identity-association) owns authenticated E4 checking, identity regeneration and whole raw-slice equality without policy authority.
Preserve Open4/Deferred, caller currentness, raw unknowns, IncompleteUncacheable/cache misses. Formula-cycle hardening remains PRODUCER-MISSING; no formula/general-cycle safety claim. R3-Q2/R5-Q2 remain G/RESERVED/unassigned.

## Original E1 typing descriptor
[VC descriptor](../../mizar-vc/en/generator.md#original-e1-typing-descriptor) owns original typing projections and raw slice.

## Bounded E1 typing check
[Kernel](../../mizar-kernel/en/checker.md#bounded-original-e1-typing) checks immutable-parameter/sole-Return object typing independently of E4; [VC](../../mizar-vc/en/generator.md#original-e1-typing-descriptor) authenticates the original descriptor first and returns it unchanged only on success. Core object guards/original projections remain checked.

## Original E1 typing Proof request-identity association
Proof [same-call typing association](../../mizar-proof/en/status.md#original-e1-typing-request-identity-association) retains authenticated VC checking, fixed-schema identity regeneration and whole raw-slice equality without policy/adoption.

## Original E1 single-valuedness checking
[Kernel](../../mizar-kernel/en/checker.md#bounded-original-e1-single-valuedness) and [VC](../../mizar-vc/en/generator.md#original-e1-single-valuedness-descriptor) own immutable-parameter/sole-Return checks, original ordered projections and unchanged six-tuple/raw slice; descriptor authentication precedes Kernel checking independently of typing/E4.

## Original E1 single-valuedness Proof request-identity association
Proof [same-call single-valuedness association](../../mizar-proof/en/status.md#original-e1-single-valuedness-request-identity-association) preserves VC-first checking, fixed-schema identity regeneration and whole raw-slice equality without policy/adoption.

Typing checker/generator and Proof association above own schema reporting; [typing consumption](../../mizar-proof/en/status.md#original-e1-typing-current-policy-consumption) owns current-policy gating.
Single-valuedness checker/generator above own success-only Kernel schema reporting, authenticated descriptor/schema handoff and delegation. Single-valuedness [consumption](../../mizar-proof/en/status.md#original-e1-single-valuedness-current-policy-consumption) owns current-policy gating.
## Native original-E1 single-valuedness adoption
Authority: §20.1.2/§20.2.6/E1. Full tier; paired Proof status/API owners.
Add constructor-only `NativeE1SingleValuedness`, following [typing adoption](../../mizar-proof/en/status.md#native-original-e1-typing-adoption): call unchanged single-valuedness consumer once; privately retain original identity, whole raw slice, snapshot, success-only schema and borrowed Core. Adoption regenerates identity/slice FIRST, preserving errors; return false unless complete values, snapshot and identical Core match. Then require current enabled Builtin/schema agreement; use existing single-valuedness-policy refusal. Fingerprints grant no authority.
Only status.rs, existing source-return tests, contract pair and paired Proof owner/API locators change. Exclude lower producers/tokens, E2/aggregate acceptance, symbols, promotion and reuse. Preserve caller currentness, Open4/Deferred, raw unknown/incomplete/cache misses; protected artifacts stay unchanged.
Tests, four-group cap, genuine fixture: repeated adoption/purity; wrong roles/changed source/check associations; foreign snapshot/distinct equal Core; current-policy refusal/variation and upstream-error precedence. Constructor preserves consumer failures.
Exit: specification/test-intent, sufficiency, precision/trust, volume, source/docs/ENJA and quality reviews; fmt, Clippy, both lint_policy suites, Proof/VC owners, source-return, proof-verification E2E and links/fragments; full tests before push.
