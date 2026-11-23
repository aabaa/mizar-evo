# Task VC53-PROOF-CALLER-E1-SINGLE-VALUEDNESS-POLICY: current-policy consumption
Canonical language: English; [Japanese pointer](../ja/VC53-PROOF-CALLER-E1-SINGLE-VALUEDNESS-POLICY.md).
Status: frozen bounded consumer. Tier: full; new public API/refusal class fails pattern eligibility.
Owner: [Proof](../../mizar-proof/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [same-call schema association](VC53-PROOF-CALLER-E1-SINGLE-VALUEDNESS-SCHEMA-ASSOCIATION.md), [passive identity](VC53-PROOF-CALLER-E1-SINGLE-VALUEDNESS-IDENTITY.md).
Authority: [§§20.1.2, 20.2.6, 20.7.2, 20.13.1 E1/E3](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [single-valuedness association](../../mizar-proof/en/status.md#arena-local-caller-e1-single-valuedness-association), [existing policy rule](../../mizar-proof/en/status.md#caller-e1-typing-current-policy-consumption), [runner/test design](../../mizar-test/en/harness.md#algorithm--logic).
Gap: source_drift/test_gap; authenticated whole single-valuedness association and actual schema exist without current-policy consumption. No language-intent gap or external prerequisite.

## Frozen scope and design
Add `consume_source_unary_caller_e1_single_valuedness(source, typed, symbols, core, request, policy)` in Proof status.rs returning existing `(SourceUnaryCallerE1SingleValuednessAssociation, PolicyFingerprint)`.
FIRST call unchanged `SourceUnaryCallerE1SingleValuednessAssociation::from_source_with_schema` exactly once on identical inputs, preserving every upstream error. Inline require enabled `BuiltinKernelEvidence` and exact policy checker-schema equality with the actual same-call schema. Otherwise refuse with internal `algorithm.proof.unary_caller_e1_single_valuedness_policy_rejected`; success returns the whole association and current `policy.policy_fingerprint()` atomically.
Each invocation checks its supplied current policy; earlier success/fingerprints grant no authority. Preserve whole request/tuple, Core association, passive identity and caller-owned currentness. No schema stamping, detached input, new type, field, holder, helper or duplicate check.
Exclude native adoption, E2, whole acceptance/status, complete footprints/cache, promotion/publication, broader profiles and owner-gated scope. Preserve child/E4/inhabitation/typing/direct-return and raw unknown/incomplete/cache-miss boundaries. No language intent, diagnostics, spec/.miz/expectation/trace/audit-status or AGENTS/protocol changes.
Files: Proof status.rs, [existing algorithm regressions](../../../../crates/mizar-test/src/runner/tests/algorithm.rs), paired Proof status/plan documents and this contract. Lower-stage production and audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: four existing genuine-fixture families; establish one independent six-row/schema consumer success before expansion. No new shared fixture.
1. Independent tuple/Kernel schema oracle, existing rename/trivia/equal-spelling and optional-terminating variants: compare whole request/tuple, same-Core matching and passive identity. Repeat calls preserve inputs/products; equal cloned Core consumes freshly but cannot match old association.
2. Default, builtin-only, development, interactive, changed profile/policy-schema return current fingerprints. Empty/FormulaSubstitution-only formats and checker-schema mismatches refuse; other formats cannot compensate. Prior products remain unchanged and valid calls recover.
3. Complete existing request/Core drift matrix, including valid wrong IDs, preserves exact upstream errors before denied policy. Original association recovers without mutation.
4. Calibrated foreign/stale/forged source/typed/environment, combined errors and unsupported direct-return preserve producer errors before denied policy; authenticated changed input consumes freshly. Child/E4/inhabitation/raw controls retain excluded authority boundaries.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites with links/fragments, Proof/VC/Kernel owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed policy/authentication/association matrices run; unchanged heavy evidence/cache/aggregate-acceptance/edit-versus-clean defer to Muse sealed integration. Full cargo test only on cadence; lane has no full/push trigger.
Exit: whole arena-local single-valuedness association consumed under supplied eligible current policy, upstream errors first, with passive identity and excluded authority unchanged.
