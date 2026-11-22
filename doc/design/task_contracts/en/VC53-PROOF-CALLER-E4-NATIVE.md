# Task VC53-PROOF-CALLER-E4-NATIVE: bounded native caller adoption
Canonical language: English; [Japanese pointer](../ja/VC53-PROOF-CALLER-E4-NATIVE.md).
Status: frozen bounded adoption. Tier: full; new public type/API fails pattern eligibility.
Owner: [Proof](../../mizar-proof/en/00.crate_plan.md). Consumer: mizar-test.
Dependency: [caller current-policy consumption](VC53-PROOF-CALLER-E4-POLICY.md).
Authority: [§§20.1.2, 20.2.6, 20.7.2, 20.13.1](../../../spec/en/20.algorithm_and_verification.md), [§16.7.3](../../../spec/en/16.theorems_and_proofs.md#1673-skeleton-encoding).
Invariants: [whole caller association](../../mizar-proof/en/status.md#arena-local-caller-e4-association), [caller consumer](../../mizar-proof/en/status.md#caller-e4-current-policy-consumption), [child native pattern](../../mizar-proof/en/status.md#native-unary-child-request-adoption).
Gap: source_drift/test_gap; authenticated caller association and policy consumer exist without native adoption. No language-intent gap or external prerequisite.

## Frozen scope and design
Add opaque `NativeUnaryCallerE4Existential<'core>` in Proof status.rs with one private existing `SourceUnaryCallerE4Association<'core>` field. `from_source(source, typed, symbols, core, request, policy)` calls the unchanged caller consumer once, retaining the whole association and discarding its fingerprint.
`adopt_original_request` takes the same six inputs. FIRST call the retained association's unchanged `matches_source`, preserving authentication errors. Return false on whole-request/descriptor or exact Core-arena mismatch, even under denied policy. Then inline require enabled BuiltinKernelEvidence and exact checker-schema agreement with the retained actual Kernel result; refusal remains `algorithm.proof.unary_caller_e4_policy_rejected`. Return true only for this original caller existential.
Each adoption checks the supplied current policy; prior success grants no authority. Source/environment/filesystem currentness remains caller-owned. No detached constructor, getter, copied payload, fingerprint, snapshot, canonical/native identity or helper is added.
Exclude complete footprints/cache, E1/E2 discharge, whole acceptance, status, promotion/publication, generic evidence, broader profiles and owner-gated scope. No language intent, public diagnostics, spec/.miz/expectation/trace/audit-status or AGENTS/protocol changes.
Files: Proof status.rs, existing runner tests/algorithm.rs, paired Proof status/plan documents and this contract. Lower-stage production, passive identity and child/direct-return APIs remain unchanged. Audit coverage/owner/deferral remains unchanged.

## Test matrix and exit
Cap: five families; reuse existing genuine fixtures and establish one native success before refusal expansion; no new shared fixture.
1. Genuine rename/trivia, equal-spelling distinct-scope and optional-terminating variants construct/adopt repeatedly. Preserve existing whole request/descriptor, exact Core, caller target, actual witness, guard/context/schema oracles and input/product purity.
2. Construction and adoption accept existing eligible current policies. Empty/FormulaSubstitution-only formats, missing Builtin and schema mismatches refuse with the caller policy error. Other formats cannot compensate; prior success cannot authorize denied policy and valid recovery succeeds.
3. Existing whole request/Core mutations, including valid wrong IDs, preserve upstream errors before denied policy in construction/adoption. Equal cloned Core returns false against the old holder before policy refusal; a fresh holder adopts only its own arena.
4. Calibrated foreign/stale source/typed/environment, unsupported profile and combined drift preserve lower errors before denied policy. Authenticated changed source constructs/adopts freshly; cross-adoption returns false before policy refusal.
5. Passive caller identity/raw unknown/incomplete/cache-miss, child policy/identity, descriptor/Kernel and direct-return boundaries remain unchanged; adoption grants no status or acceptance.
Independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews required.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, Kernel/Proof/VC owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed adoption/authentication/policy matrices run; unaffected evidence/cache/aggregate-acceptance/edit-versus-clean heavy matrices defer to Muse sealed integration. Full cargo test only on cadence; lane has no full/push trigger.
Exit: native adoption of only the whole original arena-local caller existential under current eligible policy, authentication before mismatch before policy, preserving all excluded authority.
