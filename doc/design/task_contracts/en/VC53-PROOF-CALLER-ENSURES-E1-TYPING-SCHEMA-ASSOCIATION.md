# Task VC53-PROOF-CALLER-ENSURES-E1-TYPING-SCHEMA-ASSOCIATION: ensures typing schema association
Canonical language: English; [Japanese pointer](../ja/VC53-PROOF-CALLER-ENSURES-E1-TYPING-SCHEMA-ASSOCIATION.md).
Status: frozen bounded schema association. Tier: full; new public Proof API fails pattern eligibility.
Owner: [Proof](../../mizar-proof/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [ensures schema producer](VC53-CALLER-ENSURES-E1-TYPING-SCHEMA.md), [ensures association](VC53-PROOF-CALLER-ENSURES-E1-TYPING-ASSOCIATION.md), [no-clause schema association](VC53-PROOF-CALLER-E1-TYPING-SCHEMA-ASSOCIATION.md).
Authority: [§§20.1.2, 20.2.6, 20.7.2, 20.13.1 E1](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [Proof association](../../mizar-proof/en/status.md#arena-local-caller-e1-typing-association), [VC typing](../../mizar-vc/en/generator.md#clause-bearing-unary-caller-e1-typing-request), [Kernel typing](../../mizar-kernel/en/checker.md#bounded-unary-caller-e1-typing).
Gap: source_drift/test_gap; VC reports the actual same-call ensures typing schema, but Proof's existing ensures constructor discards it. No language-intent gap or external prerequisite.

## Frozen design and boundaries
Add only `SourceUnaryCallerE1TypingAssociation::from_source_with_ensures_and_schema(source, typed, symbols, core, request) -> Result<(Self, u16), String>`. Call existing VC `check_source_unary_child_call_caller_ensures_e1_typing_request_with_schema` exactly once on identical inputs; preserve every upstream error. Construct the existing association from the whole cloned request, exact returned context/four-formula tuple and same-call borrowed Core, returning its actual scalar unchanged.
Old `from_source_with_ensures` delegates, discarding only the scalar. No duplicate authentication/Kernel check, schema stamping, new type, field, helper, fixture, storage, detached constructor or parallel representation.
Leave getters, rechecks and identity unchanged: authenticate before whole request/tuple equality and Core-pointer matching; canonical equality across arenas grants no association. Preserve sole original guard, fresh binder, target/object conclusion, optional terminating intent, purity and caller-owned currentness.
Legacy clause-bearing identity/policy/native and lower-stage refusals remain. Exclude single-valuedness, E2/E4 adoption, whole acceptance/status/currentness receipts, complete footprints/cache, promotion/publication, broader profiles and owner-gated scope. No lower-stage production, language-intent, diagnostic, spec/.miz/expectation/trace/audit-status, AGENTS or protocol edits.
Files: Proof status.rs, existing [algorithm regressions](../../../../crates/mizar-test/src/runner/tests/algorithm.rs) under [runner design](../../mizar-test/en/harness.md#algorithm--logic), paired Proof status/plan documents and this contract.

## Test matrix and exit
Cap: four existing ensures fixture families; confirm one genuine successful schema association before expanding. Reuse independent tuple/Kernel-schema oracles and [ensures return fixture](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz); no new fixture or changed test intent.
1. Exact whole request/tuple and actual VC/Kernel schema; old-constructor parity, repeated same-Core construction/recheck and input/prior-product purity. Legacy recheck/identity refuse the new association.
2. Existing rename/trivia/equal-spelling/optional-terminating and authentic changed-input variants retain tuple/schema success; old association refuses changed inputs and equal cloned Core after authentication, fresh cloned-Core schema association succeeds.
3. Complete existing request/Core drift and calibrated foreign/stale/forged source/typed/environment/combined errors: both constructors preserve exact upstream errors before mismatch; original association recovers.
4. Unsupported/malformed/no-clause/direct-return controls preserve exact producer refusals; legacy identity/policy/native and lower-stage refusals plus no-clause successes remain. No later authority follows.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, Proof/VC/Kernel owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed schema/association/authentication matrices run; unchanged heavy evidence/policy/cache/aggregate-acceptance/edit-versus-clean defer to Muse sealed integration. Full cargo test only on cadence; lane push:no has no full trigger.
Exit: freeze committed before implementation; only immutable clause-bearing typing association and actual same-call schema handoff, preserving excluded authority.
