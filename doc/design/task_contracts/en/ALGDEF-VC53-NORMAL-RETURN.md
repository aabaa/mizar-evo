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
VC 53 requires independently checked exact-call normal-return evidence for
terminating child calls/ranges; stopping/equality is insufficient. Core owns
identities, VC capture-free substitution, Proof acceptance. Open goals grant no status.
E1 needs single-valuedness, typing and inhabitation wherever return typing is asserted; E2 needs body correctness, E4 verified normal return. Keep environment/InputOK
guards for syntax-bearing inputs with AST-CONTRACT. Ghost Pick requires static
discharge; non-ghost unsupported set Pick may fail only in partial execution.
Missing evidence stays Open.

Regressions: [return](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz), [ghost rejection](../../../../tests/miz/fail/algorithms/fail_type_elaboration_algorithm_ghost_isolation_001.miz).
## Tests and acceptance

General matrix: continuation, failure/fallthrough, ghost/Pick/E1, provenance and postcondition/return/budget distinctions.
Forbidden: MVM/E5, imported receipts, public diagnostics, .miz/expectation rebaselining, fabricated proofs/credit.
Audit: retain §20 deferrals.
## Bounded direct-return producer design

[idalgo fixture](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz) [VC profile](../../mizar-vc/en/generator.md#bounded-direct-return-e1e4-producer-design): object parameter/result, equality ensures, sole return, authenticated terminating intent.
Authenticated replay emits untrusted relation/Open4: E1 typing, single-valuedness, all-typed-input inhabitation and independent E4 normal-return existence. Fresh capture-free binders retain typed-input premises; preserve postcondition/Deferred termination.
Producer matrix remains VC-owned: output, typing, uniqueness, inhabitation and E4 isolation/refusals; proof-dependent controls deferred.
Exclude calls, loops, Pick, recursion, groups, premise exposure, symbol introduction, promotion, acceptance/coverage, fixture/expectation changes and Lane 2. Parser reconciliation and mixed-group E2 remain excluded.
Exit: use the witness-request full-review and verification gates below.
## Existing request boundaries
[Proof identity](../../mizar-proof/en/status.md#source-algorithm-direct-return-request-identity): same-source regeneration, fixed four-role schemas, normalized goal/context and owner binding; inputs remain incomplete/reuse-ineligible, currentness caller-owned.
VC [witness reduction](../../mizar-vc/en/generator.md#e1-inhabitation-witness-reduction-request), [E4 reduction](../../mizar-vc/en/generator.md#e4-operational-reduction-request) and [validator handoff](../../mizar-vc/en/generator.md#e4-authenticated-validator-handoff) retain original identities, binders, witnesses and raw slices; detached substitution/local associations supply no proof. Kernel [conditional validator](../../mizar-kernel/en/checker.md#bounded-varsole-return-local-validation) leaves initialized/typed-input satisfaction to callers. E4 checking/introduction stays deferred.
## Dependency boundaries
VC [structural/authenticated collector](../../mizar-vc/en/dependency_slice.md#authenticated-direct-return-core-contribution) retains ID-free alias structure, authenticated reachable Core contributions and every unknown/raw remainder. Plain collection remains structural; slices stay incomplete/reuse-ineligible.

## Bounded checked E4 local consumer
Existing [native E1 adoption](../../mizar-proof/en/status.md#native-e1-inhabitation-adoption) remains limited to inhabitation; typing/single-valuedness and broader adoption stay deferred.
Full tier: a new E4 checked consumer fails same-consumer and semantic-class criteria. Internal connection design under §20.1.2 and §20.2.6, with no language change.
VC adds `check_source_algorithm_e4_local_request` alongside the [handoff](../../mizar-vc/en/generator.md#e4-authenticated-validator-handoff). Call that producer first, preserving its errors; invoke unchanged Kernel `check_sole_return_var` on exactly its returned inputs. Require actual statement/term equality and borrowed-value pointer identity with its expected output; refusal returns an error atomically. Return only the unchanged original descriptor after success, without a capability, new representation or generic evidence conversion. Keep arbitrary T without Eq/Clone.
Initialization and semantic parameter-guard satisfaction remain explicit caller premises. Local success proves no original AlgorithmReturns relation, E4 existential, semantic environment association, Proof identity/current-policy binding or adoption. Preserve Open4/Deferred, raw unknowns, IncompleteUncacheable/cache misses and all existing exclusions; no generic status/reuse, result typing/contracts, E1 uniqueness, currentness, symbols, publication, promotion or coverage credit.
Five cases: (1) genuine fixture and independent direct-Kernel statement/term/value-pointer oracle; (2) wrong selection/role/source/Core/environment association refuses, retaining supported alpha/ID cases; (3) producer-first errors and atomic refusal, reusing lower malformed/profile controls; (4) non-Clone/non-Eq borrowed-value identity, purity/determinism; (5) unchanged original descriptor/whole raw slice, Open4/Deferred and remaining proof deferrals. Cap five; reuse existing source-return matrices rather than duplicate them.
Affected artifacts: VC generator, paired owner documents and existing source-return tests. No Kernel/Proof behavior, spec, fixture/expectation, trace/coverage, protocol or AGENTS edits.
Exit: independent specification/test-intent, sufficiency, precision/trust, volume, final source/docs/ENJA and quality reviews; fmt, all-target/all-feature Clippy, both lint_policy suites, Kernel/VC/Proof owners, source-return, proof-verification E2E and scoped links/fragments. Full cargo test follows cadence.
