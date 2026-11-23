# Task VC53-PROOF-CALLER-E1-TYPING-NATIVE: original caller typing adoption
Canonical language: English; [Japanese pointer](../ja/VC53-PROOF-CALLER-E1-TYPING-NATIVE.md).
Status: frozen bounded adoption. Tier: full; new public holder/API and adoption boundary fail pattern eligibility.
Owner: [Proof](../../mizar-proof/en/00.crate_plan.md). Consumer: mizar-test.
Dependency: [current-policy consumer](VC53-PROOF-CALLER-E1-TYPING-POLICY.md).
Authority: [§§20.1.2, 20.2.6, 20.7.2, 20.13.1 E1](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [whole association](../../mizar-proof/en/status.md#arena-local-caller-e1-typing-association), [policy consumer](../../mizar-proof/en/status.md#caller-e1-typing-current-policy-consumption), [runner/test design](../../mizar-test/en/harness.md#algorithm--logic).
Gap: source_drift/test_gap; existing authenticated association and current-policy consumer lack original typing adoption. No language-intent gap or external prerequisite.

## Frozen scope and design
Add opaque `NativeUnaryCallerE1Typing<'core>` in Proof status.rs retaining only the existing whole `SourceUnaryCallerE1TypingAssociation<'core>` in one private field.
`from_source(source, typed, symbols, core, request, policy)` calls the unchanged consumer exactly once with identical inputs, propagates errors and discards only its fingerprint.
`adopt_original_request` takes the same six inputs. FIRST call retained association's unchanged `matches_source`, preserving authentication errors; whole request/tuple/Core mismatch returns false even under denied policy. On match freshly call the unchanged consumer once under supplied current policy, preserving its errors, then return true only for the original caller typing sub-obligation.
The extra Kernel check avoids retaining schema, fingerprints or Kernel results. No schema stamping, detached constructor, getters, identity method, helper or alternate API. Prior success grants no current-policy authority; currentness stays caller-owned.
Exclude native identity, single-valuedness, E2, whole acceptance/status/currentness receipts, complete footprints/cache, promotion/publication, broader profiles and owner-gated scope. Preserve passive identity, child/E4/inhabitation/direct-return and raw unknown/incomplete/cache-miss boundaries. No language intent, diagnostics, spec/.miz/expectation/trace/audit-status or AGENTS/protocol changes.
Files: Proof status.rs, [existing algorithm regressions](../../../../crates/mizar-test/src/runner/tests/algorithm.rs), paired Proof status/plan documents and this contract. Lower-stage production and audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: four existing genuine-fixture families; confirm independent four-row/Kernel-backed construction and adoption success before expanding. No new shared fixture.
1. Existing rename/trivia/equal-spelling and optional-terminating variants adopt on original Core. Repeat/purity and recovery preserve association, passive identity, descriptors and inputs.
2. Default, builtin-only, development, interactive and changed profile/policy-schema adopt; empty/FormulaSubstitution-only formats and wrong checker schemas refuse at construction and adoption. Earlier success cannot bypass current policy.
3. Complete existing request/Core drift matrix preserves upstream errors; authenticated whole request/tuple mismatch and equal cloned Core return false before denied policy. Fresh holder on cloned Core succeeds.
4. Calibrated foreign/stale/forged source/typed/environment, combined errors and unsupported direct-return preserve upstream errors before denied policy at construction/adoption. Authenticated changed inputs construct freshly; excluded-authority controls remain unchanged.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites with links/fragments, Proof/VC/Kernel owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed policy/authentication/association/adoption matrices run; unchanged heavy evidence/cache/aggregate-acceptance/edit-versus-clean defer to Muse sealed integration. Full cargo test only on cadence; lane has no full/push trigger.
Exit: only original arena-local caller typing is adopted under freshly checked current policy; passive identity and excluded authority remain unchanged.
