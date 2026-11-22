# Task VC53-PROOF-UNARY-CHILD-NATIVE: bounded native adoption
Canonical language: English; [Japanese pointer](../ja/VC53-PROOF-UNARY-CHILD-NATIVE.md).
Status: frozen bounded adoption. Tier: full; new Proof adoption boundary.
Owner: [Proof](../../mizar-proof/en/00.crate_plan.md). Consumer: mizar-test.
Dependency: [current-policy consumption](VC53-PROOF-UNARY-CHILD-POLICY.md).
Authority: [§§20.1.2, 20.2.6, 20.7.2, 20.13.1](../../../spec/en/20.algorithm_and_verification.md), [§16.7.3](../../../spec/en/16.theorems_and_proofs.md#1673-skeleton-encoding).
Owner invariants: [whole association](../../mizar-proof/en/status.md#arena-local-unary-child-request-association), [consumer](../../mizar-proof/en/status.md#unary-child-request-current-policy-consumption), [native E4 pattern](../../mizar-proof/en/status.md#native-original-e4-adoption).
Gap: source_drift/test_gap; current-policy consumption exists without native child-request adoption. No language-intent gap is demonstrated.

## Frozen scope and design
Add opaque `NativeUnaryChildCallExistential<'core>` in Proof status.rs with one private existing `SourceUnaryChildCallAssociation<'core>` field. `from_source(source, typed, symbols, core, request, policy)` calls the unchanged consumer once, retaining the whole association and discarding its fingerprint. No detached constructor, copied payload, snapshot or canonical identity is added.
`adopt_original_request` takes the same six inputs. FIRST call the retained association's unchanged `matches_source`, preserving authentication errors. Return false on whole-request/descriptor or exact Core-arena mismatch, even under denied policy. Then inline require enabled BuiltinKernelEvidence and exact checker-schema agreement with the retained actual Kernel result; refusal remains `algorithm.proof.unary_child_policy_rejected`. Return true only for this original child existential.
Each adoption evaluates the supplied current policy; earlier success/fingerprints grant no authority. Currentness remains caller-owned. This boolean changes no graph, scheduling, status, raw unknown/incomplete/cache-miss state or acceptance. The existing unary object profile and lower-stage ownership remain unchanged.
Excluded: canonical ProofObligationIdentity/reuse, caller E4 acceptance, E1/E2 discharge, generic substitution/evidence conversion, broader profiles, owner-gated scope, promotion/publication/coverage credit. No language intent, public diagnostics, AGENTS/protocol/spec/.miz/expectation/trace/audit-status changes.
Files: Proof status.rs, existing runner tests/algorithm.rs, paired Proof status/plan documents and this contract. Lower-stage production and direct-return APIs remain unchanged.

## Test matrix and exit
Cap: five families; reuse existing genuine unary fixtures and establish one native success before refusal expansion.
1. Existing genuine, rename/trivia, equal-spelling distinct-scope and optional-terminating variants construct and adopt repeatedly. Existing whole request/descriptor and actual Kernel target/witness/guard/schema oracles remain; inputs and retained products stay unchanged.
2. Construction and adoption accept the existing six eligible current policies. Missing Builtin, empty/FormulaSubstitution-only formats and schema mismatches refuse under the existing policy error. Prior native success cannot authorize a denied policy; valid recovery succeeds.
3. Existing request/Core mutation matrix preserves upstream errors in construction/adoption even with denied policy. Equal cloned Core returns false against the retained holder before policy refusal; a fresh holder adopts only that arena.
4. Existing foreign/stale source/typed/environment, unsupported-profile and combined-drift cases preserve calibrated upstream precedence. Fresh authenticated changed source constructs/adopts, while cross-adoption returns false before denied policy.
5. Existing descriptor/Kernel/direct-return controls retain their boundaries. No status, caller E4, generic evidence or reuse path is introduced.
Independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews required. Audit coverage/owner/deferral remains unchanged.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, Proof/VC/Kernel owner groups, focused unary/current-policy/native regressions, algorithm/source-return groups and short proof-verification E2E. Changed adoption/authentication/policy obligations run immediately; unchanged heavy evidence/cache/aggregate-acceptance/edit-versus-clean matrices defer to orchestrator integration. Full cargo test only on cadence; lane dispatch has no full/push trigger.
Exit: native adoption of only the whole original arena-local child existential under current eligible policy, preserving authentication-first refusal and all excluded authority.
