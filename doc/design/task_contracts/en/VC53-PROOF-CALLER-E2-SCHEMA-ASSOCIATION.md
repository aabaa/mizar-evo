# Task VC53-PROOF-CALLER-E2-SCHEMA-ASSOCIATION: caller postcondition schema association
Canonical language: English; [Japanese pointer](../ja/VC53-PROOF-CALLER-E2-SCHEMA-ASSOCIATION.md).
Status: frozen bounded schema association. Tier: full; new public Proof API fails pattern eligibility.
Owner: [Proof](../../mizar-proof/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [authenticated E2 schema handoff](VC53-UNARY-CALLER-E2-SCHEMA.md), [E2 association](VC53-PROOF-CALLER-E2-ASSOCIATION.md). Precedent: [typing schema association](VC53-PROOF-CALLER-E1-TYPING-SCHEMA-ASSOCIATION.md).
Authority: [§§20.1.2, 20.2.6, 20.4.1, 20.7.2, 20.13.1 E2](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [Proof association](../../mizar-proof/en/status.md#arena-local-caller-e2-association), [authenticated E2 request](../../mizar-vc/en/generator.md#bounded-unary-caller-e2-request), [Kernel E2 rule](../../mizar-kernel/en/checker.md#bounded-unary-caller-e2-postcondition).
Gap: source_drift/test_gap; VC reports the actual same-call E2 schema, but Proof's existing constructor discards it. Existing authority fixes the meaning; no language-intent change or external prerequisite.

## Frozen design and boundaries
Add `SourceUnaryCallerE2Association::from_source_with_schema(source, typed, symbols, core, request) -> Result<(Self, u16), String>`. Call the existing VC `check_source_unary_child_call_caller_e2_request_with_schema` exactly once on identical inputs; preserve every authentication/local-check error. Atomically construct the existing association from the whole cloned request, exact returned `(LocalContext, [VcGeneratedFormula; 4])` tuple and same-call borrowed Core, returning its actual scalar unchanged.
Keep old `from_source` signature and behavior by delegation, discarding only the scalar. No duplicate authentication/Kernel check, schema stamping, externally supplied schema, new type, field, holder, helper, detached constructor or parallel representation.
Leave `matches_source` and getters unchanged: authenticate before whole request/tuple equality and Core-pointer matching; equal cloned arenas refuse old association and fresh associations on clones succeed. No repeated Kernel check on recheck; currentness remains caller-owned.
Preserve the genuine unary object profile, sole original caller guard, fresh declared binder, exact caller relation, ordered definition-free equality and optional terminating intent. Contract clauses and child/E4 evidence supply no premise or additional authority.
Exclude whole acceptance, status, currentness, complete footprints/cache, promotion/publication, later E2 Proof identity/policy/native authority, schema storage, broader profiles and owner-gated scope. Preserve child/E4/E1/direct-return and raw unknown/incomplete/cache-miss boundaries; imported VC59 stays excluded. Muse integration/sealed-tree verification is separate.
No lower-stage production, language-intent, diagnostic, spec/.miz/expectation/trace/audit-status, AGENTS or protocol edits.
Files: existing Proof status.rs, [algorithm regressions](../../../../crates/mizar-test/src/runner/tests/algorithm.rs) under [runner/test design](../../mizar-test/en/harness.md#algorithm--logic), paired Proof status/plan/source-inventory documents and this contract. Audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: four existing genuine-fixture families; confirm one successful independent four-row/schema association before expanding. No new shared fixture.
1. Independent E2 context/four-row and Kernel schema oracles; new/old constructors retain the whole request and exact tuple and recheck successfully. Repetition preserves inputs/products; schema equals the actual VC result without external child/E4 evidence.
2. Existing renames/trivia/equal spellings and optional terminating variants preserve schema. Equal cloned Core refuses old association after authentication; fresh schema association on clone succeeds. Authentic changed sources succeed with their own inputs and refuse old association.
3. Complete existing request/Core drift and calibrated foreign/stale/forged source/typed/environment/combined errors: both constructors preserve exact upstream producer errors before mismatch; original association recovers after refusals.
4. Existing unsupported/no-clause/direct-return controls refuse schema construction with the producer error; child/E4/E1/direct-return and raw unknown/incomplete/cache-miss controls remain unchanged. No later authority follows.
Existing oracle: [ensures return](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz), existing unary caller ensures/drift families. No test-intent changes.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, Proof/VC/Kernel owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed schema/association/authentication matrices run; unchanged heavy evidence/policy/cache/aggregate-acceptance/edit-versus-clean defer to Muse sealed integration. Full cargo test only on cadence; lane has no full/push trigger.
Exit: freeze committed before implementation; existing immutable caller E2 association plus actual same-call scalar only.
