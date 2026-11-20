# Task ALGDEF-VC53-NORMAL-RETURN: Normal-return and function-graph prerequisites
Canonical language: English; [Japanese pointer](../ja/ALGDEF-VC53-NORMAL-RETURN.md).
Status: bounded requests, validation and structural dependencies; bounded inhabitation adoption; broader proof blocked. Tier: full gates. Owners: [Core plan](../../mizar-core/en/00.crate_plan.md), [VC plan](../../mizar-vc/en/00.crate_plan.md), [proof plan](../../mizar-proof/en/00.crate_plan.md), [Kernel plan](../../mizar-kernel/en/00.crate_plan.md).
Consumers: mizar-test/promotion; imports: ALGDEF-VC59-IMPORTED-PROMOTION-FACTS.
Dependencies: ALGDEF-PARSER-FAIL; Core42–53/CFG, VC formulas, proof/kernel results; producer Core46.
Authority: [§20.2.6, §20.3, §20.7.2, E1–E5 and VC schema](../../../spec/en/20.algorithm_and_verification.md); [algorithm graph](../../mizar-vc/en/source_vc_decomposition.md#accepted-algorithm-vc-task-graph).
Classification: source_drift/test_gap; profiles grant no acceptance.
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
Audit: retain §20 deferrals.
## Bounded direct-return producer design

[idalgo fixture](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz) [VC profile](../../mizar-vc/en/generator.md#bounded-direct-return-e1e4-producer-design): object parameter/result, equality ensures, sole return, authenticated terminating intent.
Replay emits relation/Open4: E1 typing, single-valuedness, all-typed-input inhabitation and independent E4 normal-return existence. Fresh capture-free binders retain typed-input premises; preserve postcondition/Deferred termination.
Exclude calls, loops, Pick, recursion, groups, premise exposure, symbol introduction, promotion, acceptance/coverage, fixture/expectation changes and Lane 2. Parser reconciliation and mixed-group E2 remain excluded.
## Existing request boundaries
[Proof identity](../../mizar-proof/en/status.md#source-algorithm-direct-return-request-identity): same-source regeneration, fixed four-role schemas, normalized goal/context and owner binding; inputs remain incomplete/reuse-ineligible, currentness caller-owned.
VC [witness reduction](../../mizar-vc/en/generator.md#e1-inhabitation-witness-reduction-request), [E4 reduction](../../mizar-vc/en/generator.md#e4-operational-reduction-request) and [validator handoff](../../mizar-vc/en/generator.md#e4-authenticated-validator-handoff) retain original identities, binders, witnesses and raw slices; detached substitution/local associations supply no proof. Kernel [conditional validator](../../mizar-kernel/en/checker.md#bounded-varsole-return-local-validation) leaves initialized/typed-input satisfaction to callers. E4 existential introduction stays deferred.
## Dependency boundaries
VC [structural/authenticated collector](../../mizar-vc/en/dependency_slice.md#authenticated-direct-return-core-contribution) retains ID-free alias structure, authenticated reachable Core contributions and every unknown/raw remainder. Plain collection remains structural; slices stay incomplete/reuse-ineligible.

## Symbolic instantiated E4 child checking
Existing [local consumer](../../mizar-vc/en/generator.md#e4-checked-local-request) retains caller premises; [native E1 adoption](../../mizar-proof/en/status.md#native-e1-inhabitation-adoption) remains inhabitation-only.
Full tier: new Kernel rule/VC consumer fail same-owner, same-consumer and semantic-class criteria. Internal design under §20.1.2, §20.2.6 and §20.13.1; no language change.
Kernel adds boolean `check_instantiated_algorithm_returns(core, target, witness, context)`. Tuple target: algorithm, flow, statement, input/result `(var, definition)`. Require input=result=the immutable parameter with no definition, actual witness Var(parameter), sole owned Return of that witness, valid parameter/result roles and guards, and sole context equal to the parameter guard resolving to object(parameter). Reject errors/diagnostics/ghost effects. Recompute Core CFG; match algorithm-to-flow mapping and flow owner. No opaque runtime value supplies a premise.
VC adds `check_source_algorithm_e4_instantiated_request` with the reduction producer's arguments. Regenerate that authentic original descriptor first, preserving errors; require its typed-input CheckerFact, result binder and premise association, then pass the entire child tuple, witness and sole Core assumption to Kernel. Refusal is atomic; success returns only the unchanged original descriptor, without capability, new representation or generic evidence.
This checks only substituted returns(a,a) under the symbolic typed-input assumption. Defer original E4 existential introduction, semantic environment association, Proof identity/current-policy binding/adoption and separate E1 duties. Preserve Open4/Deferred, whole raw unknowns, IncompleteUncacheable/cache misses and existing exclusions.
Five cases: (1) genuine reduction with independent full-tuple/witness/context Kernel oracle; (2) wrong algorithm/flow/statement/input/result, local definition, witness or context guard/subject/type/missing/extra rejects; (3) producer-first errors and wrong role/selection/source/check/Core, supported alpha/ID relocation; (4) purity/determinism and original binder/descriptor unchanged, opaque values supply no premise; (5) whole raw slice/unknowns/incomplete/cache misses/Open4/Deferred and proof/credit exclusions. Cap five; reuse profile/source-return controls.
Affected: Kernel checker, VC generator, existing tests and paired owner documents; correct Kernel's blanket Proof deferral wording to link bounded E1 adoption. No Proof behavior, spec, fixture/expectation, trace/coverage, protocol or AGENTS edits.
Exit: independent specification/test-intent, sufficiency, precision/trust, volume, final source/docs/ENJA and quality reviews; fmt, all-target/all-feature Clippy, both lint_policy suites, Kernel/VC/Proof owners, source-return, proof-verification E2E and scoped links/fragments. Full cargo test follows cadence.
