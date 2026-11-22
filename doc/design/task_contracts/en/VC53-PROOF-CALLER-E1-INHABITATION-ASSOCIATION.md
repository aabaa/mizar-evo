# Task VC53-PROOF-CALLER-E1-INHABITATION-ASSOCIATION: arena-local caller object-inhabitation association
Canonical language: English; [Japanese pointer](../ja/VC53-PROOF-CALLER-E1-INHABITATION-ASSOCIATION.md).
Status: frozen association-only slice. Tier: full; new public Proof type/API fails pattern eligibility.
Owner: [Proof](../../mizar-proof/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [caller inhabitation check](VC53-UNARY-CALLER-E1-INHABITATION-CHECK.md), [child association](VC53-PROOF-UNARY-CHILD-ASSOCIATION.md).
Authority: [§§20.1.2, 20.2.6, 20.7.2, 20.13.1 E1](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [Proof association](../../mizar-proof/en/status.md#arena-local-unary-child-request-association), [caller request/check](../../mizar-vc/en/generator.md#bounded-unary-caller-e1-inhabitation-request), [Kernel introduction](../../mizar-kernel/en/checker.md#bounded-e1-inhabitation-witness-introduction).
Gap: source_drift/test_gap; authenticated caller object-inhabitation checking exists without immutable Proof association/recheck. No language-intent change or external prerequisite.

## Frozen design and boundaries
Add opaque `SourceUnaryCallerE1InhabitationAssociation` in Proof status.rs. Private fields retain only the whole existing `SourceUnaryChildCallRequest`, exact `(LocalContext, [VcGeneratedFormula; 2], CoreTermId)` tuple and same-call borrowed `ObjectInhabitationCheck`. Immutable `request`, `descriptor` and `result` getters expose those existing values.
`from_source(source, typed, symbols, core, request)` calls the unchanged landed caller inhabitation checker once on identical inputs; preserve upstream errors, clone the whole request and retain its tuple/result atomically.
`matches_source` takes the same five inputs. FIRST regenerate the unchanged authenticated caller E1 inhabitation request, preserving errors before mismatch; then require whole request/tuple equality and pointer-identical Core to the retained result. Equal cloned arenas refuse; fresh associations on clones succeed. No repeated Kernel check, detached constructor, partial/hash-only comparison, duplicate payload representation or generic helper. Currentness remains caller-owned.
The genuine unary object profile, original caller guard, fresh binder and actual-input witness remain unchanged. Optional terminating intent grants no additional authority.
Exclude native adoption, passive E1 identity, typing/single-valuedness checking, E2, whole acceptance, status, currentness, complete footprints/cache, promotion/publication, broader profiles and owner-gated scope. Preserve child/E4/direct-return and raw unknown/incomplete/cache-miss boundaries. No lower-stage production, spec/.miz/expectation/trace/audit-status, diagnostic, AGENTS or protocol edits.
Files: Proof status.rs, existing runner tests/algorithm.rs, paired Proof status/plan documents and this contract. Audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: four existing genuine-fixture families; establish one successful association/recheck before expanding.
1. Independent whole request/tuple and same-Core Kernel-result oracle, original guard/schema, fresh binder and actual witness; repeated constructor/recheck preserves inputs and retained products.
2. Existing renames/trivia/equal spelling and optional terminating variants; equal cloned Core refuses old association after authentication, fresh cloned-Core association succeeds. Authentic changed sources succeed only with their own inputs.
3. Complete existing request/Core drift plus calibrated source/typed/environment/forgery and combined errors: constructor and recheck preserve upstream errors before mismatch, and the original association recovers after refusals.
4. Existing child/E4/direct-return controls and raw unknown/incomplete/cache-miss boundaries remain unchanged; unsupported direct-return source refuses this caller route. No later authority follows.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, Proof/VC/Kernel owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed association/authentication matrices run immediately; unchanged heavy evidence/policy/cache/aggregate-acceptance/edit-versus-clean groups defer to Muse sealed integration. Full cargo test only on cadence; lane has no full/push trigger.
Exit: immutable freshly authenticated caller object-inhabitation association and fail-closed same-arena recheck only.
