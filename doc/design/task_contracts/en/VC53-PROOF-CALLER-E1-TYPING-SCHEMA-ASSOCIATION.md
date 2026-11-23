# Task VC53-PROOF-CALLER-E1-TYPING-SCHEMA-ASSOCIATION: caller typing schema association
Canonical language: English; [Japanese pointer](../ja/VC53-PROOF-CALLER-E1-TYPING-SCHEMA-ASSOCIATION.md).
Status: frozen bounded schema association. Tier: full; new public Proof API fails pattern eligibility.
Owner: [Proof](../../mizar-proof/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [schema producer](VC53-UNARY-CALLER-E1-TYPING-SCHEMA.md), [association](VC53-PROOF-CALLER-E1-TYPING-ASSOCIATION.md), [passive identity](VC53-PROOF-CALLER-E1-TYPING-IDENTITY.md).
Authority: [§§20.1.2, 20.2.6, 20.7.2, 20.13.1 E1](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [Proof association](../../mizar-proof/en/status.md#arena-local-caller-e1-typing-association), [authenticated VC schema producer](../../mizar-vc/en/generator.md#bounded-unary-caller-e1-typing-request), [Kernel typing](../../mizar-kernel/en/checker.md#bounded-unary-caller-e1-typing).
Gap: source_drift/test_gap; VC reports the actual same-call schema, but Proof's existing association constructor discards it. Existing authority fixes typing; no language-intent gap or external prerequisite.

## Frozen design and boundaries
Add `SourceUnaryCallerE1TypingAssociation::from_source_with_schema(source, typed, symbols, core, request) -> Result<(Self, u16), String>`. Call the existing VC `check_source_unary_child_call_caller_e1_typing_request_with_schema` exactly once on those identical inputs; preserve every authentication/local-check error. Atomically construct the existing association from the whole cloned request, exact returned four-row context/formula tuple and same-call borrowed Core, returning its actual scalar unchanged.
Keep the old `from_source` signature and behavior by delegation, discarding only the scalar. No duplicate authentication/Kernel check, schema stamping, externally supplied schema, new type, field, holder, helper, detached constructor or parallel representation.
Leave `matches_source`, getters and `obligation_identity` unchanged: authenticate before whole request/tuple and Core-pointer matching; canonical equality across arenas grants no association. Preserve passive identity bytes, original caller guard/fresh binder, optional terminating intent and caller-owned currentness.
Exclude policy/native eligibility or adoption, opaque Kernel results, single-valuedness, E2, whole acceptance/status, complete footprints/cache, promotion/publication, broader profiles and owner-gated scope. Preserve raw unknown/incomplete/cache-miss states and direct-return/child/E4/inhabitation/lower-stage behavior. No language-intent, diagnostic, spec/.miz/expectation/trace/audit-status or AGENTS/protocol edits.
Files: existing Proof status.rs, [algorithm regressions](../../../../crates/mizar-test/src/runner/tests/algorithm.rs) under the [runner/test design](../../mizar-test/en/harness.md#algorithm--logic), paired Proof status/plan documents and this contract. Audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: four existing genuine-fixture families; establish one successful independent four-row/schema association case before expanding. No new shared fixture.
1. Independent four-row oracle and Kernel schema oracle; new/old constructors retain the same whole request/tuple and recheck/identity. Repetition preserves inputs and retained products; schema equals the actual VC result.
2. Existing renames/trivia/equal spellings and optional terminating variants preserve schema and canonical identity. Equal cloned Core refuses old association after authentication; fresh schema association on clone succeeds.
3. Complete existing request/Core drift and calibrated foreign/stale/forged source/typed/environment/combined errors: both constructors preserve exact upstream errors before mismatch; original association recovers, retaining purity.
4. Existing child/E4/inhabitation/direct-return and raw unknown/incomplete/cache-miss controls remain unchanged; unsupported direct-return input refuses the schema constructor with the producer error. No later authority follows.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, Proof/VC/Kernel owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed schema/association/authentication matrices run; unchanged heavy evidence/policy/cache/aggregate-acceptance/edit-versus-clean defer to Muse sealed integration. Full cargo test only on cadence; lane has no full/push trigger.
Exit: existing immutable caller result-typing association plus actual same-call schema only; excluded authority remains deferred.
