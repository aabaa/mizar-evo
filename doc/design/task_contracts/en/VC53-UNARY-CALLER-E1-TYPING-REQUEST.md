# Task VC53-UNARY-CALLER-E1-TYPING-REQUEST: bounded caller result typing
Canonical language: English; [Japanese pointer](../ja/VC53-UNARY-CALLER-E1-TYPING-REQUEST.md).
Status: frozen request-only slice. Tier: full; new public producer fails pattern eligibility.
Owner: [VC](../../mizar-vc/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [caller E4 descriptor](VC53-CALLER-E4-DESCRIPTOR.md), [native E1 identity boundary](VC53-PROOF-CALLER-E1-INHABITATION-NATIVE-IDENTITY.md).
Authority: [§§20.1.2, 20.2.6, 20.7.2, 20.13.1 E1](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [caller E4](../../mizar-vc/en/generator.md#bounded-unary-caller-e4-descriptor), [child tuple](../../mizar-vc/en/generator.md#bounded-unary-child-existential), [dependency ownership](../../mizar-vc/en/dependency_slice.md).
Gap: source_drift/test_gap; authenticated caller returns exists without its bounded E1 typing request. Existing specification fixes the meaning; no language-intent change or external prerequisite.

## Frozen design and boundaries
Add only `generate_source_unary_child_call_caller_e1_typing_request(source, typed, symbols, core, request)` in VC generator.rs, returning existing `Result<(LocalContext, [VcGeneratedFormula; 4]), String>`.
First call unchanged `generate_source_unary_child_call_caller_e4_request` once on the identical five inputs, preserving whole fresh source/typed/environment/Core/request authentication and upstream errors. Retain its exact context and row zero caller relation, including caller/flow/Return/actual/fresh-result joins and metadata. Read row one's original existential binder; replace that row's shape with `ProgramTypePredicate { subject: result, ty: object }`, preserving metadata. Append SplitGoal rows with empty provenance: row two implies row zero to row one; row three universally quantifies the original binder over row two. IDs are their row indices. Impossible local shape refusal reuses `algorithms.unary_child.invalid_caller_e4_request`.
Under only the original caller object-input guard, this requests `forall r. returns_caller(actual,r) -> object(r)`. Returns is an antecedent, never an installed context fact; requires/ensures, child returns and function equality supply no premise or proof.
No new type, holder, field, helper, fixture or parallel representation; lower-stage APIs and E4/child/direct-return behavior stay unchanged.
Exclude typing/single-valuedness checking, inhabitation changes, native adoption, identity, E2, whole acceptance, status, currentness, complete footprints/cache, promotion/publication, broader profiles and owner-gated scope. No spec/.miz/expectation/trace/audit-status, diagnostic, AGENTS or protocol edits.
Files: VC generator.rs, existing runner tests/algorithm.rs, paired VC generator/plan documents and this contract. Audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: four existing genuine-fixture families; confirm one successful setup before expanding.
1. Independent exact formula oracle: sole original caller guard, fresh declared binder, exact caller relation, object result, implication and universal links; metadata/context retention, repeated output and input/product purity.
2. Genuine renames/trivia/equal spellings and optional terminating flags preserve authenticated joins; equal cloned Core regenerates the same tuple without arena-bound evidence.
3. Existing complete request/Core drift and calibrated source/typed/environment/forgery/combined errors refuse atomically with unchanged E4 upstream errors; authentic changed source succeeds only with its own inputs; original recovers.
4. Existing E4/child/inhabitation/direct-return and raw unknown/incomplete/cache-miss boundaries remain unchanged; unsupported direct-return source refuses this route. No checking or later authority follows.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, VC/Kernel/Proof owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed request/authentication matrices run immediately; unchanged heavy evidence/cache/aggregate-acceptance/edit-versus-clean groups defer to Muse sealed integration. Full cargo test only on cadence; lane has no full/push trigger.
Exit: only the authenticated bounded caller E1 typing request becomes available, preserving every excluded authority.
