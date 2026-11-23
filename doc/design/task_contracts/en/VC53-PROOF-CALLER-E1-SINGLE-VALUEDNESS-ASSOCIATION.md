# Task VC53-PROOF-CALLER-E1-SINGLE-VALUEDNESS-ASSOCIATION: arena-local caller uniqueness association
Canonical language: English; [Japanese pointer](../ja/VC53-PROOF-CALLER-E1-SINGLE-VALUEDNESS-ASSOCIATION.md).
Status: frozen association-only slice. Tier: full; new public Proof type/API fails pattern eligibility.
Owner: [Proof](../../mizar-proof/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [fresh uniqueness check](VC53-UNARY-CALLER-E1-SINGLE-VALUEDNESS-CHECK.md), [schema handoff](VC53-UNARY-CALLER-E1-SINGLE-VALUEDNESS-SCHEMA.md). Precedent: [typing association](VC53-PROOF-CALLER-E1-TYPING-ASSOCIATION.md).
Authority: [§§20.1.2, 20.2.6, 20.7.2, 20.13.1 E1/E3](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [Proof association](../../mizar-proof/en/status.md#arena-local-caller-e1-typing-association), [authenticated uniqueness request/check](../../mizar-vc/en/generator.md#bounded-unary-caller-e1-single-valuedness-request), [Kernel uniqueness](../../mizar-kernel/en/checker.md#bounded-unary-caller-e1-single-valuedness).
Gap: source_drift/test_gap; freshly authenticated uniqueness checking exists without immutable Proof association/recheck. No language-intent change or external prerequisite.

## Frozen design and boundaries
Add opaque `SourceUnaryCallerE1SingleValuednessAssociation` in Proof status.rs. Private fields retain only the whole existing `SourceUnaryChildCallRequest`, exact `(LocalContext, [VcGeneratedFormula; 6])` tuple and same-call borrowed Core. Immutable `request` and `descriptor` getters expose those existing values.
`from_source(source, typed, symbols, core, request)` calls unchanged `check_source_unary_child_call_caller_e1_single_valuedness_request` once on identical inputs; preserve upstream errors, clone the whole request and retain its tuple and borrowed Core atomically on success only.
`matches_source` takes the same five inputs. FIRST call unchanged `generate_source_unary_child_call_caller_e1_single_valuedness_request` once, preserving authentication errors before mismatch; then require whole request/tuple equality and pointer-identical Core. Equal cloned arenas refuse; fresh associations on clones succeed. No repeated Kernel check, detached constructor, partial/hash-only comparison, duplicate payload representation or generic helper. Currentness remains caller-owned.
The proposition remains only uniqueness of two caller returns for the same complete input under the original caller object-input guard. Keep both original fresh binders and ordered definition-free equality. Optional terminating intent grants no additional authority. No schema reporting/storage or opaque Kernel result is needed for this passive association.
Exclude identity, policy/native authority, E2, whole acceptance/status/currentness, complete footprints/cache, promotion/publication, broader profiles and owner-gated scope; imported VC59 remains blocked-reserved and excluded. Preserve child/E4/inhabitation/typing/direct-return and raw unknown/incomplete/cache-miss boundaries. No lower-stage production, spec/.miz/expectation/trace/audit-status, diagnostic, AGENTS or protocol edits.
Files: Proof status.rs, existing runner tests/algorithm.rs, paired Proof status/plan documents and this contract. Audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: four existing genuine-fixture families; confirm one successful association/recheck against the independent six-row oracle before expanding, with no new shared fixture.
1. Whole request and independently checked six-row tuple equality, same-Core association/recheck, repeated construction/recheck and input/product purity, sole original guard and both fresh binders without external child/E4 or typing evidence.
2. Existing renames/trivia/equal spellings and optional terminating variants succeed; equal cloned Core refuses old association after authentication, fresh cloned-Core association succeeds. Authentic changed sources succeed only with their own inputs.
3. Complete existing request/Core drift and calibrated source/typed/environment/forgery/combined errors: constructor and recheck preserve exact upstream producer errors before mismatch; original association recovers after refusals.
4. Existing child/E4/inhabitation/typing/direct-return and raw unknown/incomplete/cache-miss controls remain unchanged; unsupported direct-return source refuses this caller route. No later authority follows.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, Proof/VC/Kernel owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed association/authentication matrices run immediately; unchanged heavy evidence/policy/cache/aggregate-acceptance/edit-versus-clean groups defer to Muse sealed integration. Full cargo test only on cadence; lane has no full/push trigger.
Exit: immutable freshly checked caller uniqueness association and fail-closed same-arena recheck only.
