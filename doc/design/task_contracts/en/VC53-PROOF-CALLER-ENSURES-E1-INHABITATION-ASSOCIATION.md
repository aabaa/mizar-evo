# Task VC53-PROOF-CALLER-ENSURES-E1-INHABITATION-ASSOCIATION: clause-bearing caller object-inhabitation association
Canonical language: English; [Japanese pointer](../ja/VC53-PROOF-CALLER-ENSURES-E1-INHABITATION-ASSOCIATION.md).
Status: frozen association-only slice. Tier: full; new public APIs fail pattern eligibility.
Owner: [Proof](../../mizar-proof/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [ensures E1 check](VC53-CALLER-ENSURES-E1-INHABITATION-CHECK.md), [existing holder](VC53-PROOF-CALLER-E1-INHABITATION-ASSOCIATION.md).
Authority: [§§20.1.2, 20.2.6, 20.7.2, 20.13.1 E1](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [Proof association](../../mizar-proof/en/status.md#arena-local-caller-e1-inhabitation-association), [VC request/check](../../mizar-vc/en/generator.md#clause-bearing-unary-caller-e1-inhabitation-request), [Kernel introduction](../../mizar-kernel/en/checker.md#bounded-e1-inhabitation-witness-introduction).
Gap: source_drift/test_gap; authenticated clause-bearing E1 checking exists without explicit Proof association/recheck. No language-intent gap or external prerequisite.

## Frozen design and boundaries
Add only `SourceUnaryCallerE1InhabitationAssociation::from_source_with_ensures(source, typed, symbols, core, request) -> Result<Self, String>` and `matches_source_with_ensures` with the same five inputs and `Result<bool, String>`.
Construction calls unchanged VC `check_source_unary_child_call_caller_ensures_e1_inhabitation_result` exactly once on identical inputs, preserving errors; atomically retain the whole cloned request, exact returned tuple and same-call borrowed result in existing fields. Existing immutable getters expose those products.
Recheck FIRST calls unchanged VC `generate_source_unary_child_call_caller_ensures_e1_inhabitation_request`, preserving errors before whole request/tuple equality and pointer-identical Core comparison. Equal cloned arenas refuse old association; fresh construction on clones succeeds. No repeated Kernel check; currentness remains caller-owned.
Preserve the bounded unary object profile, authenticated caller/CFG/Return/Apply/callee/formal/actual/result-slot joins, fresh declared binder, actual-input witness, sole original caller guard and actual Kernel schema. Neither clauses nor optional terminating intent grant further authority.
Keep all old constructors/rechecks, canonical identity, policy consumption and native adoption unchanged: they still refuse caller ensures, including identity on the new association. No new type, field, helper, fixture, schema storage, detached constructor or parallel representation.
Exclude E1 typing/single-valuedness, E2/E4 adoption, whole acceptance/status, currentness receipts, complete footprints/cache, promotion/publication, broader profiles and owner-gated scope. No lower-stage production, language-intent, diagnostic, spec/.miz/expectation/trace/audit-status, AGENTS or protocol edits.
Files: Proof status.rs, existing [algorithm tests](../../../../crates/mizar-test/src/runner/tests/algorithm.rs), paired Proof status/plan documents and this contract. [Runner design](../../mizar-test/en/harness.md#algorithm--logic) remains test owner; audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: four existing ensures fixture families; confirm one genuine successful association/recheck before expanding. No new shared fixture.
1. Independent whole request/tuple and same-Core Kernel-result oracle, sole original guard/schema, fresh binder and actual witness; repetition preserves inputs/products. Legacy recheck/identity refuse the new association.
2. Existing rename/trivia/equal-spelling/optional-terminating variants and authentic changed inputs construct/recheck together; old association refuses changed inputs and equal cloned Core after authentication, fresh cloned-Core association succeeds.
3. Complete existing request/Core drift and calibrated foreign/stale/forged source/typed/environment/combined-error matrices: constructor and recheck preserve exact upstream errors before mismatch; original association recovers.
4. Existing unsupported/malformed/no-clause/direct-return controls preserve producer refusals; legacy clause-bearing identity/policy/native and lower-stage refusals plus no-clause successes remain. No later authority follows.
Reuse existing oracles and [ensures return fixture](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz); no changed test intent.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, Proof/VC/Kernel owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed association/authentication matrices run; unchanged heavy evidence/policy/cache/aggregate-acceptance/edit-versus-clean defer to Muse sealed integration. Full cargo test only on cadence; lane push:no has no full trigger.
Exit: freeze committed before implementation; immutable clause-bearing association/recheck only, with all legacy authority boundaries preserved.
