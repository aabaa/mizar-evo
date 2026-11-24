# Task VC53-PROOF-CALLER-ENSURES-E4-ASSOCIATION: clause-bearing caller E4 association
Canonical language: English; [Japanese pointer](../ja/VC53-PROOF-CALLER-ENSURES-E4-ASSOCIATION.md).
Status: frozen bounded association. Tier: full; new public APIs fail pattern eligibility.
Owner: [Proof](../../mizar-proof/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [checked ensures E4](VC53-CALLER-ENSURES-E4-CHECK.md), [existing E4 holder](VC53-PROOF-CALLER-E4-ASSOCIATION.md).
Authority: [§20.1.2, §20.2.6, §20.4.1, §20.7.2, §20.13.1 E4](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [Proof association](../../mizar-proof/en/status.md#arena-local-caller-e4-association), [VC request/check](../../mizar-vc/en/generator.md#clause-bearing-unary-caller-e4-request), [Kernel composition](../../mizar-kernel/en/checker.md#bounded-unary-caller-operational-composition).
Gap: source_drift/test_gap; the clause-bearing checked caller result exists but the existing Proof holder lacks explicit association/recheck methods. No language-intent change or external prerequisite.

## Frozen design and boundaries
Add only `SourceUnaryCallerE4Association::from_source_with_ensures(source, typed, symbols, core, request) -> Result<Self, String>` and `matches_source_with_ensures` with the existing recheck's five inputs and `Result<bool, String>`.
Construction calls unchanged VC `check_source_unary_child_call_caller_ensures_e4_result` exactly once on identical inputs; preserve errors and atomically retain the whole cloned request, exact returned descriptor and same-call borrowed checked result in existing fields. Existing immutable getters expose these products.
Recheck FIRST calls unchanged VC `generate_source_unary_child_call_caller_ensures_e4_request`, preserving errors before whole request/descriptor equality and pointer-identical Core matching. Equal cloned arenas refuse the old association; fresh construction on clones succeeds. No repeated Kernel check; currentness remains caller-owned.
Preserve the exact clause-bearing unary object profile, caller/CFG/Return/Apply/callee/formal/actual/result-slot joins, sole original guard, fresh result binder, actual witness and actual Kernel schema. Optional terminating intent grants no promotion.
Keep all old constructors/rechecks, canonical identity, policy consumption and native adoption unchanged: they still refuse caller ensures, including identity on the new association. No new type, field, helper, fixture, schema store, detached constructor or parallel representation.
Exclude E1/E2 adoption, whole acceptance/status, currentness receipts, complete footprints/cache, publication/promotion, broader profiles and owner-gated scope. No lower-stage production, language-intent, diagnostic, spec/.miz/expectation/trace/audit-status, AGENTS or protocol edits.
Files: Proof status.rs, existing [algorithm tests](../../../../crates/mizar-test/src/runner/tests/algorithm.rs), paired Proof status/plan documents and this contract. [Runner design](../../mizar-test/en/harness.md#algorithm--logic) remains the test owner; audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: four existing fixture families; confirm one genuine successful association/recheck before expanding. No new shared fixture.
1. Compare whole retained request and descriptor with independent existing VC/Kernel oracles; verify exact caller target, guard, fresh binder, actual witness, schema and Core pointer. Repetition preserves all inputs/products; legacy recheck/identity refuse the new association.
2. Existing rename/trivia/equal-spelling/optional-terminating variants and authentic changed sources construct/recheck with their own inputs; old association refuses changed inputs and equal cloned Core after authentication, fresh cloned-Core association succeeds.
3. Complete existing request/Core drift and calibrated foreign/stale/forged source/typed/environment/combined-error matrices: constructor and recheck preserve exact upstream errors before mismatch; original association recovers.
4. Existing unsupported/malformed/no-clause/direct-return controls preserve producer refusals; legacy clause-bearing descriptor/E1/E4/child/Proof identity/policy/adoption refusals and no-clause controls remain. No later authority follows.
Existing source oracle: [ensures return](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz); existing caller-ensures and drift matrices. No changed test intent.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy denied warnings, both lint_policy suites including links/fragments, Proof/VC/Kernel owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed association/authentication matrices run; unchanged heavy evidence/policy/cache/aggregate-acceptance/edit-versus-clean defer to Muse sealed integration. Full cargo test only on cadence; lane push:no has no full trigger.
Exit: freeze committed before implementation; explicit immutable clause-bearing association/recheck with all legacy authority boundaries preserved.
