# Task VC53-UNARY-CALLER-E1-INHABITATION-REQUEST: bounded caller object inhabitation
Canonical language: English; [Japanese pointer](../ja/VC53-UNARY-CALLER-E1-INHABITATION-REQUEST.md).
Status: frozen request-only slice. Tier: full; new public API fails pattern eligibility.
Owner: [VC](../../mizar-vc/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [authenticated child existential](VC53-UNARY-CHILD-EXISTENTIAL.md), [caller descriptor](VC53-CALLER-E4-DESCRIPTOR.md), [native caller identity boundary](VC53-PROOF-CALLER-E4-NATIVE-IDENTITY.md).
Authority: [§§20.1.2, 20.2.6, 20.7.2, 20.13.1 E1](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [child tuple](../../mizar-vc/en/generator.md#bounded-unary-child-existential), [caller E4](../../mizar-vc/en/generator.md#bounded-unary-caller-e4-descriptor), [dependency ownership](../../mizar-vc/en/dependency_slice.md).
Gap: source_drift/test_gap; the authenticated tuple exists without a caller all-typed-input object-inhabitation request. No language-intent change or external prerequisite.

## Frozen design and boundaries
Add only `generate_source_unary_child_call_caller_e1_inhabitation_request(source, typed, symbols, core, request)` in VC generator.rs, returning the existing `Result<(LocalContext, [VcGeneratedFormula; 2], CoreTermId), String>` tuple.
First call unchanged `generate_source_unary_child_call_existential_request` once on the identical five inputs, preserving whole fresh source/typed/environment/Core/request authentication and upstream errors. Extract its fresh result from row zero; replace only that row's shape with `ProgramTypePredicate { subject: result, ty: object }`. Preserve context, row IDs/kinds/provenance, existential row and actual-input witness exactly. Impossible local shape refusal reuses `algorithms.unary_child.invalid_existential_request`.
Under the sole caller object-input guard, the request is `exists r. object(r)` for all typed inputs, assuming neither returns nor requires/ensures. The actual-input witness is retained for later checking, never an Apply, formal parameter or result-slot witness.
No new type, field, helper, fixture or parallel representation; lower-stage APIs and E4/direct-return behavior stay unchanged.
Exclude checking, native adoption, identity, status, acceptance, currentness, complete footprints/cache, promotion/publication, E1 typing/single-valuedness, E2, whole-algorithm credit, broader profiles and owner-gated scope. No spec/.miz/expectation/trace/audit-status, diagnostic, AGENTS or protocol edits.
Files: VC generator.rs, existing runner tests/algorithm.rs, paired VC generator/plan documents and this contract. Audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: four existing genuine-fixture families; establish one successful setup before expanding.
1. Exact independent formula/witness oracle and purity: sole original caller object guard, fresh declared binder, object(r) row and existential link; exact retained tuple metadata/witness; repeats preserve all inputs/products.
2. Genuine renames/trivia/equal spellings and optional terminating flags preserve authenticated joins; equal cloned Core regenerates the same tuple without granting arena-bound evidence.
3. Existing complete request/Core drift and calibrated source/typed/environment/forgery/combined errors refuse atomically with unchanged upstream errors; authentic changed source succeeds only with its own inputs; original recovers.
4. Existing E4/child/direct-return and raw unknown/incomplete/cache-miss boundaries remain unchanged; unsupported direct-return source refuses this caller route. No checking or proof credit follows.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, VC/Kernel/Proof owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed request/authentication matrices run immediately; unchanged heavy evidence/cache/aggregate-acceptance/edit-versus-clean groups defer to Muse sealed integration. Full cargo test only on cadence; lane has no full/push trigger.
Exit: only the authenticated caller all-typed-input object-inhabitation request becomes available, preserving every excluded authority.
