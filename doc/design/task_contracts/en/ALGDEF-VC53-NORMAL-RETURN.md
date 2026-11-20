# Task ALGDEF-VC53-NORMAL-RETURN: Normal-return and function-graph prerequisites
Canonical language: English; [Japanese pointer](../ja/ALGDEF-VC53-NORMAL-RETURN.md).
Status: bounded requests and local operational validation; proof/adoption blocked. Tier: full gates. Owners: [Core plan](../../mizar-core/en/00.crate_plan.md), [VC plan](../../mizar-vc/en/00.crate_plan.md), [proof plan](../../mizar-proof/en/00.crate_plan.md), [Kernel plan](../../mizar-kernel/en/00.crate_plan.md).
Consumers: mizar-test/promotion; imports: ALGDEF-VC59-IMPORTED-PROMOTION-FACTS.
Dependencies: ALGDEF-PARSER-FAIL; Core 42–53 call/range/group/CFG and replayable VC formulas; proof/kernel results. Producer uses Core46.
Authority: [§20.2.6, §20.3, §20.7.2, E1–E5 and VC schema](../../../spec/en/20.algorithm_and_verification.md); [algorithm graph](../../mizar-vc/en/source_vc_decomposition.md#accepted-algorithm-vc-task-graph).
Classification: source_drift/test_gap; profiles grant no acceptance.
Owner details: [Core/CFG graph](../../mizar-core/en/source_family_decomposition.md#accepted-phase-10-controlflowir-task-graph), [VC generator](../../mizar-vc/en/generator.md#step-4-algorithm-vcs), [proof status](../../mizar-proof/en/status.md#source-existential-registration-proof).

## Boundary and invariants

Core43/48/53 validate failure placement, report terms and return-or-fail closure;
failure contributes no continuation/postcondition. Void fallthrough returns;
value fallthrough rejects. Ghost execution cannot fail or influence runtime.
VC 44 retains partial-call returns/ensures, including recursive E2 induction;
external E2 requires verified body correctness. Continuation proves no E4.
VC 53 requires independently checked exact-call normal-return evidence for
terminating child calls/ranges; stopping/equality is insufficient. Core owns
identities, VC capture-free substitution, Proof acceptance. Open goals grant no status.
Before E1 symbol introduction justify single-valued execution, typing and
inhabitation on every input where its return type is asserted; E2 needs body
correctness, E4 independently verified normal return. Keep environment/InputOK
guards for syntax-bearing inputs with AST-CONTRACT. Ghost Pick requires static
discharge; non-ghost unsupported set Pick may fail only in partial execution.
Missing evidence stays incomplete/open.

Regressions: [return](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz), [ghost rejection](../../../../tests/miz/fail/algorithms/fail_type_elaboration_algorithm_ghost_isolation_001.miz).

## Tests and acceptance

General matrix: continuation, child/range evidence, failure/fallthrough, ghost/report/Pick/E1 refusals, provenance, postcondition/normal-return/budget distinctions. Only complete authenticated inputs yield contexts/goals; genuine checked evidence is required for promotion.
Forbidden: MVM/E5 execution, imported receipts, public diagnostic allocation,
existing .miz/expectation rebaselining, fabricated proofs or coverage credit.
Audit: retain §20 deferrals.

## Bounded direct-return producer design

The [idalgo fixture](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz) [VC profile](../../mizar-vc/en/generator.md#bounded-direct-return-e1e4-producer-design): object parameter/result, equality ensures, sole return, authenticated terminating intent.
Authenticated replay emits an untrusted relation and four Open requests: E1 typing, single-valuedness, all-typed-input inhabitation and independent E4 normal-return existence. Fresh capture-free binders retain typed-input premises; preserve postcondition/Deferred termination.
Producer matrix remains VC-owned: output, typing, uniqueness, inhabitation and E4 isolation/refusals; proof-dependent controls deferred.
Exclude calls, loops, Pick, recursion, groups, premise exposure, checking/adoption, symbol introduction, promotion, acceptance/coverage, fixture/expectation changes and Lane 2. Parser reconciliation and mixed-group E2 remain excluded.
Exit: use the witness-request full-review and verification gates below.

## Bounded Proof request inputs and identity
The [Proof owner](../../mizar-proof/en/status.md#source-algorithm-direct-return-request-identity) owns four-request canonical inputs/identities: same-source regeneration, fixed schemas, role domains, normalized goal/context and source-owner binding. Reuse existing identities/slices; reject postconditions, absent selections and unsupported profiles. Inputs remain incomplete/reuse-ineligible; currentness caller-owned.
Five cases: four-role hashes/slices; candidate/final vectors/purity; trivia/relocation/owner/parameter/snapshot; upstream cross-source/check/symbol errors; profile/selection/Open/Deferred. No proof/adoption/evidence/acceptance.

## E1 inhabitation witness-reduction request
VC freezes the [bounded witness descriptor](../../mizar-vc/en/generator.md#e1-inhabitation-witness-reduction-request): authenticate the whole four-request graph, bind actual returned a, substitute its fresh existential binder without capture and return child object(a), original existential identity and whole raw slice. This is request-only; checking and Proof adoption remain deferred.
Five cases: descriptor/parent-identity/slice oracle; renamed/relocated witness/fresh-binder; other-role/absent; stale/cross-source/check/symbol/Core/upstream-error-order; schema/snapshot/Open/Deferred/purity. Reuse lower controls; no fixture/expectation/specification/Lane2 changes.
Exit: independent full reviews; VC and Proof owner/source-return tests, proof-verification E2E, fmt, Clippy, both lint_policy suites, links/fragments; full regression only on cadence.

## E4 operational-reduction request
VC owns the [sole-return reduction](../../mizar-vc/en/generator.md#e4-operational-reduction-request): regenerate the whole graph, select Open E4, return original VcSet/binder/raw slice, actual returned CoreTermId, detached instantiated returns(a,a) and residual True shapes. These request evaluation of immutable a and normal exit of the sole Return; neither shape is evidence. No equality alias, trusted premise, graph rewrite or new schedulable VC. Operational checking, existential introduction and adoption remain deferred.
Five cases: descriptor/identity/slice oracle; renamed/relocated witness/fresh-binder; other-role/absent; upstream source/check/symbol/Core-before-selection; schema/snapshot/Open/Deferred/purity. Reuse witness-request gates/lower controls.

## Kernel Var/sole-Return local validator
Kernel [conditional evaluation](../../mizar-kernel/en/checker.md#bounded-varsole-return-local-validation) uses existing Core/environment tuples. Semantic typing/initialization remain caller premises; guard IDs authenticate no values. Return borrowed value/local statement/term association only. Reject malformed/unsupported/mismatched inputs. Source/CFG authentication, original E4 proof, existential introduction, Proof identity/policy binding/adoption remain deferred; four Open requests and Deferred termination persist.
Five-case cap: opaque typed input/renaming; missing/duplicate/cross-variable/guard bindings; mutable/ghost/uninitialized/non-Var/extra-work; ownership/error/missing-guard/result-alias; authentic E4 descriptor/identity/slice and Open/Deferred/purity. Refusals yield no output. Run Kernel/source-return and witness-request gates; full regression on cadence. No evidence/credit.
