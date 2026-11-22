# Task VC53-PROOF-CALLER-E4-NATIVE-IDENTITY: bounded native caller identity
Canonical language: English; [Japanese pointer](../ja/VC53-PROOF-CALLER-E4-NATIVE-IDENTITY.md).
Status: frozen bounded identity. Tier: full; new public method fails pattern eligibility.
Owner: [Proof](../../mizar-proof/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [native adoption](VC53-PROOF-CALLER-E4-NATIVE.md), [passive caller identity](VC53-PROOF-CALLER-E4-IDENTITY.md).
Authority: [§§20.1.2, 20.2.6, 20.7.2, 20.13.1](../../../spec/en/20.algorithm_and_verification.md), [§16.7.3](../../../spec/en/16.theorems_and_proofs.md#1673-skeleton-encoding), [identity boundary](../../architecture/en/22.incremental_verification_contract.md#identity-model-vcid-vs-obligationanchor).
Invariants: [native caller adoption](../../mizar-proof/en/status.md#native-caller-e4-adoption), [whole caller association](../../mizar-proof/en/status.md#arena-local-caller-e4-association).
Gap: source_drift/test_gap; authentic adoption and passive identity exist without their native association. No language-intent gap or external prerequisite.

## Frozen scope
Add only `NativeUnaryCallerE4Existential::obligation_identity(source, typed, symbols, core, request, policy)` in Proof status.rs, returning `Result<Option<(ProofObligationIdentity, DependencySlice)>, String>`. FIRST call unchanged `adopt_original_request` once on all six inputs; propagate errors and return None on false. On true return the retained association's unchanged `obligation_identity` on identical five inputs. Preserve caller hashes, domain, anchor and whole raw slice; no child identity substitution.
Current policy is checked each call; earlier success grants no authority. Authentication precedes whole-request/descriptor/exact-arena mismatch, which precedes policy refusal. Currentness remains caller-owned.
No new producer, field, helper, fixture or lower-stage production change. Exclude complete footprints/cache, E1/E2, whole acceptance, status, promotion/publication, generic evidence, broader profiles and owner-gated scope. No language intent, diagnostics, spec/.miz/expectation/trace/audit-status or AGENTS/protocol changes.
Files: Proof status.rs, existing runner tests/algorithm.rs, paired Proof status/plan documents and this contract. Lower owners and child/direct-return APIs stay unchanged; audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: five existing genuine-fixture families; establish one native identity success before expansion.
1. Independent whole-tuple canonical oracle and purity: native identity equals existing independently verified passive caller identity and whole slice; repeat without mutating inputs/products.
2. Normalization/owners/flags/clones: existing trivia/alpha variants preserve identity, owner/terminating changes participate; old holder returns None for equal cloned Core, fresh clone holder returns equal identity only in its own arena.
3. Policy refusal/recovery: eligible policies return the unchanged tuple; denied formats/Builtin/schema retain caller policy error; prior success grants no authority and valid recovery succeeds.
4. Request/Core/authentication/changed-source ordering: reuse every existing drift and calibrated error under denied policy; authenticated cross-source mismatch returns None, fresh own-source holder succeeds.
5. Unchanged passive/child/direct-return/raw-state boundaries: passive identity remains policy-independent; unknown/incomplete/cache-miss and no status/acceptance authority persist.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, Kernel/Proof/VC owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed identity/policy/authentication matrices run; unaffected heavy evidence/cache/aggregate-acceptance/edit-versus-clean matrices defer to Muse sealed integration. Full cargo test only on cadence; lane has no full/push trigger.
Exit: native identity only after original caller adoption under current policy, preserving all excluded authority.
