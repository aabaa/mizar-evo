# Task VC53-PROOF-CALLER-E1-SINGLE-VALUEDNESS-SCHEMA-ASSOCIATION: caller uniqueness schema association
Canonical language: English; [Japanese pointer](../ja/VC53-PROOF-CALLER-E1-SINGLE-VALUEDNESS-SCHEMA-ASSOCIATION.md).
Status: frozen bounded schema association. Tier: full; new public Proof API fails pattern eligibility.
Owner: [Proof](../../mizar-proof/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [schema producer](VC53-UNARY-CALLER-E1-SINGLE-VALUEDNESS-SCHEMA.md), [association](VC53-PROOF-CALLER-E1-SINGLE-VALUEDNESS-ASSOCIATION.md). Precedent: [typing schema association](VC53-PROOF-CALLER-E1-TYPING-SCHEMA-ASSOCIATION.md).
Authority: [§§20.1.2, 20.2.6, 20.7.2, 20.13.1 E1/E3](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [Proof association](../../mizar-proof/en/status.md#arena-local-caller-e1-single-valuedness-association), [authenticated VC schema producer](../../mizar-vc/en/generator.md#bounded-unary-caller-e1-single-valuedness-request), [Kernel uniqueness](../../mizar-kernel/en/checker.md#bounded-unary-caller-e1-single-valuedness).
Gap: source_drift/test_gap; VC reports the actual same-call schema, but Proof's association constructor discards it. No language-intent gap or external prerequisite.

## Frozen design and boundaries
Add `SourceUnaryCallerE1SingleValuednessAssociation::from_source_with_schema(source, typed, symbols, core, request) -> Result<(Self, u16), String>`. Call existing VC `check_source_unary_child_call_caller_e1_single_valuedness_request_with_schema` exactly once on identical inputs; preserve every authentication/local-check error. Construct the existing association from the whole cloned request, exact returned six-row context/formula tuple and same-call borrowed Core, returning its actual scalar unchanged.
Keep old `from_source` signature and behavior by delegation, discarding only the scalar. No duplicate authentication/Kernel check, schema stamping/storage, externally supplied schema, new type, field, holder, helper, detached constructor or parallel representation.
Leave `matches_source` and getters unchanged: authenticate before whole request/tuple and Core-pointer matching; equal cloned arenas grant no association. Keep original caller guard, both fresh binders and ordered definition-free equality; optional terminating intent grants no additional authority.
Exclude Proof identity/policy/native authority, E2, whole acceptance/status/currentness, complete footprints/cache, promotion/publication, broader profiles and owner-gated scope. Preserve child/E4/inhabitation/typing/direct-return and raw unknown/incomplete/cache-miss boundaries; imported VC59 stays excluded. No language-intent, diagnostic, lower-stage production, spec/.miz/expectation/trace/audit-status or AGENTS/protocol edits.
Files: Proof status.rs, existing [algorithm regressions](../../../../crates/mizar-test/src/runner/tests/algorithm.rs) under [runner/test design](../../mizar-test/en/harness.md#algorithm--logic), paired Proof status/plan documents and this contract. Audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: four existing genuine-fixture families; confirm one successful independent six-row/schema association case before expanding. No new shared fixture.
1. Independent six-row and Kernel schema oracles: new/old constructors retain the same whole request/tuple and same-Core recheck; actual scalar equals VC result. Repetition preserves inputs and retained products.
2. Existing renames/trivia/equal spellings and optional terminating variants preserve schema. Equal cloned Core refuses old associations after authentication; fresh schema association on clone succeeds. Authentic changed sources succeed with their own inputs.
3. Complete existing request/Core drift and calibrated foreign/stale/forged source/typed/environment/combined errors: both constructors preserve exact upstream errors before mismatch; original association recovers after refusals.
4. Existing child/E4/inhabitation/typing/direct-return and raw unknown/incomplete/cache-miss controls remain unchanged; unsupported direct-return source refuses schema construction with producer error. No later authority follows.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, Proof/VC/Kernel owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed schema/association/authentication matrices run; unchanged heavy evidence/policy/cache/aggregate-acceptance/edit-versus-clean defer to Muse sealed integration. Full cargo test only on cadence; lane has no full/push trigger.
Exit: existing immutable caller uniqueness association plus actual same-call schema only; excluded authority remains deferred.
