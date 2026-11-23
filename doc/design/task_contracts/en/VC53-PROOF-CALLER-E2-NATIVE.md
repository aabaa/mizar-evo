# Task VC53-PROOF-CALLER-E2-NATIVE: original caller postcondition adoption
Canonical language: English; [Japanese pointer](../ja/VC53-PROOF-CALLER-E2-NATIVE.md).
Status: frozen bounded adoption. Tier: full; new public holder/API and adoption boundary fail pattern eligibility.
Owner: [Proof](../../mizar-proof/en/00.crate_plan.md). Consumer: mizar-test.
Dependency: [current-policy consumer](VC53-PROOF-CALLER-E2-POLICY.md). Precedent: [single-valuedness adoption](VC53-PROOF-CALLER-E1-SINGLE-VALUEDNESS-NATIVE.md).
Authority: [§§20.1.2, 20.2.6, 20.4.1, 20.7.2, 20.13.1 E2](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [whole association](../../mizar-proof/en/status.md#arena-local-caller-e2-association), [policy consumer](../../mizar-proof/en/status.md#caller-e2-current-policy-consumption), [runner/test design](../../mizar-test/en/harness.md#algorithm--logic).
Gap: source_drift/test_gap; authenticated whole E2 association and current-policy consumer lack original postcondition adoption. No language-intent gap or external prerequisite.

## Frozen scope and design
Add opaque `NativeUnaryCallerE2<'core>` in Proof status.rs retaining only the existing whole `SourceUnaryCallerE2Association<'core>` in one private field.
`from_source(source, typed, symbols, core, request, policy)` calls the unchanged consumer exactly once with identical inputs, propagates errors and discards only its fingerprint.
`adopt_original_request` takes the same six inputs. FIRST call retained association's unchanged `matches_source`, preserving authentication errors; whole request/tuple/Core mismatch returns false even under denied policy. On match freshly call the unchanged consumer once under supplied current policy, preserving errors, then return true only for the original caller E2 postcondition sub-obligation.
The extra Kernel check avoids retaining schema, fingerprints or Kernel results. No detached constructor, getter, identity method, helper or alternate API. Prior success grants no current-policy authority; currentness stays caller-owned.
Exclude E2 identity/native identity, schema storage, whole caller acceptance/status/currentness receipts, complete footprints/cache, promotion/publication, broader profiles and owner-gated scope. Preserve child/E4/E1/direct-return and raw unknown/incomplete/cache-miss boundaries; imported VC59 remains excluded. Muse sealed-tree integration is separate.
No lower-stage production, language-intent, diagnostic, spec/.miz/expectation/trace/audit-status, AGENTS or protocol edits.
Files: Proof status.rs, [existing algorithm regressions](../../../../crates/mizar-test/src/runner/tests/algorithm.rs), paired Proof status/plan/API-inventory documents and this contract. Audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: four existing genuine-fixture families; confirm independent four-row/Kernel-backed construction and adoption success before expansion. No new shared fixture.
1. Independent tuple/schema oracle, existing rename/trivia/equal-spelling and optional-terminating variants adopt on original Core. Repeat/purity and recovery preserve whole request/tuple and inputs.
2. Default, builtin-only, development, interactive and changed profile/policy-schema adopt; empty/FormulaSubstitution-only formats and wrong checker schemas refuse at construction and adoption. Earlier success cannot bypass supplied current policy.
3. Complete existing request/Core drift matrix preserves upstream errors; authenticated whole request/tuple mismatch and equal cloned Core return false before denied policy. Fresh holder on cloned Core succeeds.
4. Calibrated foreign/stale/forged source/typed/environment, combined errors and unsupported/no-clause/direct-return preserve upstream errors before denied policy at construction/adoption. Authenticated changed inputs construct freshly; excluded-authority controls remain unchanged.
Existing oracle: [ensures return](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz) and existing unary caller ensures/drift families. No test-intent changes.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites with links/fragments, Proof/VC/Kernel owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed policy/authentication/association/adoption matrices run; unchanged heavy evidence/cache/aggregate-acceptance/edit-versus-clean defer to Muse sealed integration. Full cargo test only on cadence; lane has no full/push trigger.
Exit: freeze committed before implementation; only original arena-local E2 postcondition adopted under freshly checked current policy without later authority.
