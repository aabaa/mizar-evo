# Task VC53-PROOF-CALLER-E2-ASSOCIATION: arena-local caller postcondition association
Canonical language: English; [Japanese pointer](../ja/VC53-PROOF-CALLER-E2-ASSOCIATION.md).
Status: frozen association-only slice. Tier: full; new public Proof type/API fails pattern eligibility.
Owner: [Proof](../../mizar-proof/en/00.crate_plan.md). Consumer: mizar-test.
Dependency: [authenticated E2 schema handoff](VC53-UNARY-CALLER-E2-SCHEMA.md). Precedent: [typing association](VC53-PROOF-CALLER-E1-TYPING-ASSOCIATION.md).
Authority: [§§20.1.2, 20.2.6, 20.4.1, 20.7.2, 20.13.1 E2](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [Proof association precedent](../../mizar-proof/en/status.md#arena-local-caller-e1-typing-association), [authenticated E2 request](../../mizar-vc/en/generator.md#bounded-unary-caller-e2-request), [Kernel E2 rule](../../mizar-kernel/en/checker.md#bounded-unary-caller-e2-postcondition).
Gap: source_drift/test_gap; authenticated E2 checking exists without immutable Proof association/recheck. Existing authority fixes the meaning; no language-intent change or external prerequisite.

## Frozen design and boundaries
Add opaque `SourceUnaryCallerE2Association` in Proof status.rs. Private fields retain only the whole existing `SourceUnaryChildCallRequest`, exact `(LocalContext, [VcGeneratedFormula; 4])` tuple and same-call borrowed Core. Immutable `request` and `descriptor` getters expose the retained existing values.
`from_source(source, typed, symbols, core, request) -> Result<Self, String>` calls unchanged `check_source_unary_child_call_caller_e2_request` once on identical inputs; preserve every upstream/local error and retain the whole cloned request, exact returned tuple and borrowed Core atomically on success only.
`matches_source` takes the same five inputs. FIRST call unchanged `generate_source_unary_child_call_caller_e2_request` once, preserving authentication errors before mismatch; then require whole request/tuple equality and pointer-identical Core. Equal cloned arenas refuse; fresh associations on clones succeed. No repeated Kernel check, detached constructor, partial/hash-only comparison, parallel payload representation or generic helper. Currentness remains caller-owned.
Keep the genuine unary object profile, sole original caller guard, declared fresh binder, exact caller relation and ordered definition-free equality unchanged. Contract clauses, child/E4 evidence and optional terminating intent supply no premise or additional authority.
Exclude whole acceptance, status, currentness, complete footprints/cache, promotion/publication, identity/policy/native authority, schema reporting/storage, broader profiles and owner-gated scope. Preserve child/E4/E1/direct-return and raw unknown/incomplete/cache-miss boundaries; imported VC59 stays excluded.
No lower-stage production, language-intent, diagnostic, spec/.miz/expectation/trace/audit-status, AGENTS or protocol edits.
Files: Proof status.rs, existing [algorithm regressions](../../../../crates/mizar-test/src/runner/tests/algorithm.rs) under [runner/test design](../../mizar-test/en/harness.md#algorithm--logic), paired Proof status/plan/source-inventory documents and this contract. Audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: four existing genuine-fixture families; confirm one successful independent four-row association/recheck before expanding. No new shared fixture.
1. Independent existing E2 context/four-row oracle and whole request equality; same-Core association/recheck succeeds. Repeated construction/recheck preserves inputs and retained products without external child/E4 evidence.
2. Existing renames/trivia/equal spellings and optional terminating variants succeed. Equal cloned Core refuses old association after authentication; fresh cloned-Core association succeeds. Authentic changed sources succeed with their own inputs and refuse old association.
3. Complete existing request/Core drift and calibrated foreign/stale/forged source/typed/environment/combined errors: constructor and recheck preserve exact upstream producer errors before mismatch; original association recovers after refusals.
4. Existing unsupported/no-clause/direct-return controls refuse construction with the producer error; child/E4/E1/direct-return and raw unknown/incomplete/cache-miss controls remain unchanged. No later authority follows.
Existing oracle: [ensures return](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz), existing unary caller ensures/drift families. No test-intent changes.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, Proof/VC/Kernel owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed association/authentication matrices run; unchanged heavy evidence/policy/cache/aggregate-acceptance/edit-versus-clean defer to Muse sealed integration. Full cargo test only on cadence; lane has no full/push trigger.
Exit: freeze committed before implementation; immutable freshly checked caller E2 association and fail-closed same-arena recheck only.
