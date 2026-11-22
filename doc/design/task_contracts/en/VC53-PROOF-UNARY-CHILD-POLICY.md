# Task VC53-PROOF-UNARY-CHILD-POLICY: current-policy consumption
Canonical language: English; [Japanese pointer](../ja/VC53-PROOF-UNARY-CHILD-POLICY.md).
Status: frozen bounded consumer. Tier: full; new Proof policy-consumption boundary.
Owner: [Proof](../../mizar-proof/en/00.crate_plan.md). Consumer: mizar-test.
Dependency: [arena-local association](VC53-PROOF-UNARY-CHILD-ASSOCIATION.md).
Authority: [§§20.1.2, 20.2.6, 20.7.2, 20.13.1](../../../spec/en/20.algorithm_and_verification.md), [§16.7.3](../../../spec/en/16.theorems_and_proofs.md#1673-skeleton-encoding).
Owner APIs/invariants: [association](../../mizar-proof/en/status.md#arena-local-unary-child-request-association), [existing E4 policy rule](../../mizar-proof/en/status.md#original-e4-current-policy-consumption).
Gap: source_drift/test_gap; the whole child association exists without a current-policy consumer. No language-intent gap is demonstrated.

## Frozen scope and design
Add `consume_source_unary_child_call_existential(source, typed, symbols, core, request, policy)` in Proof status.rs returning existing `(SourceUnaryChildCallAssociation, PolicyFingerprint)`.
FIRST call unchanged `SourceUnaryChildCallAssociation::from_source` once, preserving authentication and Kernel errors. Inline require enabled `BuiltinKernelEvidence` and exact policy checker-schema equality with the association's actual Kernel result. Refuse with internal `algorithm.proof.unary_child_policy_rejected`; return the whole association and current `policy.policy_fingerprint()` atomically on success.
This reuses existing E4 policy eligibility without new policy semantics, type, field, detached input or helper. Each invocation checks the supplied current policy; earlier success/fingerprints grant no authority. Source/environment/filesystem currentness remains caller-owned. The unary object profile and lower-stage ownership remain unchanged.
Excluded: native retention/adoption, canonical ProofObligationIdentity/reuse, caller E4 acceptance, E1 discharge, generic substitution, broader profiles, owner-gated scope, evidence/status/promotion/publication/coverage credit. No language intent, diagnostics, AGENTS/protocol/spec/.miz/expectation/trace/audit-status changes.
Files: Proof status.rs, existing runner algorithm tests, paired Proof status/plan documents and this contract. Lower-stage production and direct-return APIs remain unchanged.

## Test matrix and exit
Cap: five families; reuse existing genuine unary fixtures and establish one consumer success before refusal expansion.
1. Genuine pair and existing rename/trivia, equal-spelling distinct-scope and optional-terminating variants: compare whole request/descriptor, same-Core target/witness/guard/actual schema to the genuine association; repeat calls preserve all inputs and products.
2. Current policy: default, builtin-only, development, interactive, changed profile and policy-schema values return exactly their current fingerprints. Missing builtin, empty/FormulaSubstitution-only formats and checker-schema mismatches refuse; other formats never compensate for missing builtin or mismatched schema. Prior success remains unchanged and a later valid invocation succeeds.
3. Existing whole request/Core mutation matrix, including valid wrong IDs, refuses with exact upstream errors even under denied policy. Equal cloned Core is consumable as a fresh association on that arena and cannot match the old arena.
4. Foreign/stale source/typed/environment and unsupported profile preserve calibrated lower errors before denied policy, including a combined drift precedence case. Genuine authenticated changed source consumes freshly without matching old association.
5. Existing descriptor/Kernel/direct-return controls retain their boundaries; no native adoption, scheduling, status or acceptance path is introduced.
Independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews required.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, Proof/VC/Kernel owner groups, focused unary/current-policy regressions, algorithm/source-return groups and short proof-verification E2E. Changed current-policy/authentication/association obligations run immediately; unchanged evidence/cache/aggregate-acceptance/edit-versus-clean heavy matrices defer to orchestrator integration. Full cargo test only on cadence; lane dispatch has no full/push trigger.
Exit: whole arena-local association consumed only under the supplied eligible current policy, with authentication-first refusal and no broader authority.
