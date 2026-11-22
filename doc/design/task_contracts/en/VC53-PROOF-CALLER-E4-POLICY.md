# Task VC53-PROOF-CALLER-E4-POLICY: current-policy consumption
Canonical language: English; [Japanese pointer](../ja/VC53-PROOF-CALLER-E4-POLICY.md).
Status: frozen bounded consumer. Tier: full; new public API fails the pattern criterion.
Owner: [Proof](../../mizar-proof/en/00.crate_plan.md). Consumer: mizar-test.
Dependency: [passive caller identity/association](VC53-PROOF-CALLER-E4-IDENTITY.md).
Authority: [§§20.1.2, 20.2.6, 20.7.2, 20.13.1](../../../spec/en/20.algorithm_and_verification.md), [§16.7.3](../../../spec/en/16.theorems_and_proofs.md#1673-skeleton-encoding).
Invariants: [caller association](../../mizar-proof/en/status.md#arena-local-caller-e4-association), [existing child policy rule](../../mizar-proof/en/status.md#unary-child-request-current-policy-consumption).
Gap: source_drift/test_gap; authenticated caller association exists without current-policy consumption. No language-intent gap or external prerequisite.

## Frozen scope and design
Add `consume_source_unary_caller_e4_existential(source, typed, symbols, core, request, policy)` in Proof status.rs returning existing `(SourceUnaryCallerE4Association, PolicyFingerprint)`.
FIRST call unchanged `SourceUnaryCallerE4Association::from_source` once, preserving authentication/Kernel errors. Inline require enabled `BuiltinKernelEvidence` and exact policy checker-schema equality with the association's actual Kernel result. Otherwise refuse with internal `algorithm.proof.unary_caller_e4_policy_rejected`; on success return the whole association and current `policy.policy_fingerprint()` atomically.
Use existing eligibility semantics without new type, field, detached input, helper or eager identity copy. Each invocation checks its supplied current policy; earlier success/fingerprints grant no authority. Passive identity remains unchanged and policy-independent; source/environment/filesystem currentness remains caller-owned.
Exclude native adoption, complete footprints/cache, E1/E2, whole acceptance, status, promotion/publication, generic evidence, broader profiles and owner-gated scope. No language intent, diagnostics, spec/.miz/expectation/trace/audit-status or AGENTS/protocol changes.
Files: Proof status.rs, existing runner algorithm tests and paired Proof status/plan documents. Lower-stage production, child/direct-return APIs and audit coverage/owner/deferral stay unchanged.

## Test matrix and exit
Cap: five families; reuse existing genuine fixtures and establish one consumer success before refusal expansion; no new shared fixture.
1. Genuine rename/trivia, equal-spelling distinct-scope and optional-terminating variants: compare whole request/descriptor, exact Core, caller target, actual witness, guard/context and actual schema with the genuine caller association. Repeated calls preserve inputs/products.
2. Current default, builtin-only, development, interactive, changed profile/policy-schema return exactly current fingerprints. Empty/FormulaSubstitution-only formats and checker-schema mismatches refuse; other formats cannot compensate. Prior products remain unchanged and later valid calls recover.
3. Reuse whole request/Core mutations, including valid wrong IDs: exact upstream errors precede denied policy. Equal cloned Core consumes freshly on that arena but cannot match an old association.
4. Calibrated foreign/stale source/typed/environment and unsupported profile preserve lower errors before denied policy, including combined drift. Authenticated changed source consumes freshly without matching old association.
5. Passive caller identity/raw unknown/incomplete/cache-miss, child policy/identity, descriptor/Kernel and direct-return boundaries remain unchanged; no native/status/acceptance path.
Independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews required.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites with links/fragments, Kernel/Proof/VC owner groups, algorithm/source-algorithm regressions and short proof-verification E2E. Changed policy/authentication/association matrices run; unaffected evidence/cache/aggregate-acceptance/edit-versus-clean heavy matrices defer to Muse sealed integration. Full cargo test only on cadence; lane has no full/push trigger.
Exit: whole arena-local caller association consumed only under supplied eligible current policy, authentication first, with passive identity and excluded authority unchanged.
