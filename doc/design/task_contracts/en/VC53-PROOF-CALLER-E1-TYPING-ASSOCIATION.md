# Task VC53-PROOF-CALLER-E1-TYPING-ASSOCIATION: arena-local caller result-typing association
Canonical language: English; [Japanese pointer](../ja/VC53-PROOF-CALLER-E1-TYPING-ASSOCIATION.md).
Status: frozen association-only slice. Tier: full; new public Proof seam fails pattern eligibility.
Owner: [Proof](../../mizar-proof/en/00.crate_plan.md). Consumer: mizar-test.
Dependency: [fresh caller typing check](VC53-UNARY-CALLER-E1-TYPING-CHECK.md).
Authority: [§§20.1.2, 20.2.6, 20.7.2, 20.13.1 E1](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [Proof association](../../mizar-proof/en/status.md#arena-local-caller-e1-typing-association), [caller typing request/check](../../mizar-vc/en/generator.md#bounded-unary-caller-e1-typing-request), [Kernel typing](../../mizar-kernel/en/checker.md#bounded-unary-caller-e1-typing).
Gap: source_drift/test_gap; fresh authenticated caller typing checking exists without immutable Proof association/recheck. No language-intent change or external prerequisite.

## Frozen design and boundaries
Add opaque `SourceUnaryCallerE1TypingAssociation` in Proof status.rs. Private fields retain only the whole existing `SourceUnaryChildCallRequest`, exact `(LocalContext, [VcGeneratedFormula; 4])` tuple and same-call borrowed Core. Immutable `request` and `descriptor` getters expose the retained existing values.
`from_source(source, typed, symbols, core, request)` calls unchanged `check_source_unary_child_call_caller_e1_typing_request` once on identical inputs; preserve upstream errors, clone the whole request and retain its tuple and borrowed Core atomically on success only.
`matches_source` takes the same five inputs. FIRST call unchanged `generate_source_unary_child_call_caller_e1_typing_request` once, preserving authentication errors before mismatch; then require whole request/tuple equality and pointer-identical Core. Equal cloned arenas refuse; fresh associations on clones succeed. No repeated Kernel check, detached constructor, partial/hash-only comparison, duplicate payload representation or generic helper. Currentness remains caller-owned.
The genuine unary object profile, sole original caller guard and fresh declared binder remain unchanged. Optional terminating intent grants no additional authority. No schema or opaque Kernel result is needed for this passive association.
Exclude identity, policy/native adoption, single-valuedness checking, E2, whole acceptance, status, currentness, complete footprints/cache, promotion/publication, broader profiles and owner-gated scope. Preserve child/E4/inhabitation/direct-return and raw unknown/incomplete/cache-miss boundaries. No lower-stage production, spec/.miz/expectation/trace/audit-status, diagnostic, AGENTS or protocol edits.
Files: Proof status.rs, existing runner tests/algorithm.rs, paired Proof status/plan documents and this contract. Audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: four existing genuine-fixture families; confirm one successful association/recheck before expanding.
1. Independent existing four-row typing oracle and whole request equality; same-Core association/recheck succeeds. Repeated construction/recheck preserves inputs and retained products, sole original guard and fresh binder, without external child/E4 evidence.
2. Existing renames/trivia/equal spellings and optional terminating variants succeed; equal cloned Core refuses old association after authentication, fresh cloned-Core association succeeds. Authentic changed sources succeed only with their own inputs.
3. Complete existing request/Core drift and calibrated source/typed/environment/forgery/combined errors: constructor and recheck preserve exact upstream producer errors before mismatch; original association recovers after refusals.
4. Existing child/E4/inhabitation/direct-return and raw unknown/incomplete/cache-miss controls remain unchanged; unsupported direct-return source refuses this caller route. No later authority follows.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, Proof/VC/Kernel owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed association/authentication matrices run immediately; unchanged heavy evidence/policy/cache/aggregate-acceptance/edit-versus-clean groups defer to Muse sealed integration. Full cargo test only on cadence; lane has no full/push trigger.
Exit: immutable freshly checked caller result-typing association and fail-closed same-arena recheck only.
