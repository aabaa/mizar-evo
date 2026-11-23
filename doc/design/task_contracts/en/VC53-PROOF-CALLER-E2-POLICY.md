# Task VC53-PROOF-CALLER-E2-POLICY: caller postcondition current-policy consumption
Canonical language: English; [Japanese pointer](../ja/VC53-PROOF-CALLER-E2-POLICY.md).
Status: frozen bounded consumer. Tier: full; new public Proof API/refusal class fails pattern eligibility.
Owner: [Proof](../../mizar-proof/en/00.crate_plan.md). Consumer: mizar-test.
Dependency: [same-call E2 schema association](VC53-PROOF-CALLER-E2-SCHEMA-ASSOCIATION.md). Precedent: [typing policy consumer](VC53-PROOF-CALLER-E1-TYPING-POLICY.md).
Authority: [§§20.1.2, 20.2.6, 20.4.1, 20.7.2, 20.13.1 E2](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [E2 association](../../mizar-proof/en/status.md#arena-local-caller-e2-association), [authenticated E2 request](../../mizar-vc/en/generator.md#bounded-unary-caller-e2-request), [Kernel E2 rule](../../mizar-kernel/en/checker.md#bounded-unary-caller-e2-postcondition), [runner/test design](../../mizar-test/en/harness.md#algorithm--logic).
Gap: source_drift/test_gap; whole authenticated E2 association and actual schema exist without current-policy consumption. No language-intent gap or external prerequisite.

## Frozen scope and design
Add `consume_source_unary_caller_e2(source, typed, symbols, core, request, policy)` in Proof status.rs returning existing `(SourceUnaryCallerE2Association, PolicyFingerprint)`.
FIRST call unchanged `SourceUnaryCallerE2Association::from_source_with_schema` exactly once on identical inputs, preserving every upstream error. Inline require enabled `BuiltinKernelEvidence` and exact policy checker-schema equality with the actual same-call schema. Otherwise refuse with internal `algorithm.proof.unary_caller_e2_policy_rejected`; success returns the whole association and current `policy.policy_fingerprint()` atomically.
Each invocation checks its supplied policy; prior success/fingerprints grant no authority. Preserve whole request/tuple, borrowed Core association and caller-owned currentness. No identity prerequisite, schema storage/stamping, detached input, new type, field, holder, helper or duplicate check.
Exclude native adoption, whole caller acceptance/status, currentness authority, complete footprints/cache, promotion/publication, broader profiles and owner-gated scope. Preserve child/E4/E1/direct-return and raw unknown/incomplete/cache-miss boundaries; imported VC59 remains excluded. Muse sealed-tree integration is separate.
No lower-stage production, language-intent, diagnostic, spec/.miz/expectation/trace/audit-status, AGENTS or protocol edits.
Files: existing Proof status.rs, [algorithm regressions](../../../../crates/mizar-test/src/runner/tests/algorithm.rs), paired Proof status/plan/API-inventory documents and this contract. Audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: four existing genuine-fixture families; confirm one independent four-row/schema consumer success before expansion. No new shared fixture.
1. Independent tuple/Kernel schema oracle, existing rename/trivia/equal-spelling and optional-terminating variants: compare whole request/tuple and same-Core matching. Repetition preserves inputs/products; equal cloned Core consumes freshly but cannot match old association.
2. Default, builtin-only, development, interactive, changed profile/policy-schema return current fingerprints. Empty/FormulaSubstitution-only formats and checker-schema mismatches refuse; other formats cannot compensate. Prior products remain unchanged and valid calls recover.
3. Complete existing request/Core drift, including valid wrong IDs, preserves exact upstream errors before denied policy. Original association recovers without mutation.
4. Calibrated foreign/stale/forged source/typed/environment, combined errors and unsupported/no-clause/direct-return inputs preserve producer errors before denied policy; authenticated changed input consumes freshly. Existing child/E4/E1/raw controls retain excluded authority boundaries.
Existing oracle: [ensures return](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz) and existing unary caller ensures/drift families. No test-intent changes.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites with links/fragments, Proof/VC/Kernel owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed policy/authentication/association matrices run; unchanged heavy evidence/cache/aggregate-acceptance/edit-versus-clean defer to Muse sealed integration. Full cargo test only on cadence; lane has no full/push trigger.
Exit: freeze committed before implementation; whole arena-local E2 association consumed under supplied eligible current policy, upstream errors first, without later authority.
