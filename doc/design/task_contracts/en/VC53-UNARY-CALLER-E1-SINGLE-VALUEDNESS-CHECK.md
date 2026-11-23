# Task VC53-UNARY-CALLER-E1-SINGLE-VALUEDNESS-CHECK: fresh caller uniqueness
Canonical language: English; [Japanese pointer](../ja/VC53-UNARY-CALLER-E1-SINGLE-VALUEDNESS-CHECK.md).
Status: frozen checking-only slice. Tier: full; new public semantic-authority API fails pattern eligibility.
Owner: [VC](../../mizar-vc/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [authenticated uniqueness request](VC53-UNARY-CALLER-E1-SINGLE-VALUEDNESS-REQUEST.md), [Kernel uniqueness rule](VC53-UNARY-CALLER-E1-SINGLE-VALUEDNESS-KERNEL.md).
Authority: [§§20.1.2, 20.2.6, 20.7.2, 20.13.1 E1/E3](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [uniqueness request](../../mizar-vc/en/generator.md#bounded-unary-caller-e1-single-valuedness-request), [Kernel uniqueness](../../mizar-kernel/en/checker.md#bounded-unary-caller-e1-single-valuedness), [dependency ownership](../../mizar-vc/en/dependency_slice.md).
Gap: source_drift/test_gap; authenticated caller uniqueness requests and the local Kernel boolean exist without their fresh VC checking join. Existing specification fixes the meaning; no language-intent change or external prerequisite.

## Frozen design and boundaries
Add only `check_source_unary_child_call_caller_e1_single_valuedness_request(source, typed, symbols, core, request)` in VC generator.rs, returning existing `Result<(LocalContext, [VcGeneratedFormula; 6]), String>`.
First call unchanged `generate_source_unary_child_call_caller_e1_single_valuedness_request` once on identical five inputs, preserving whole fresh authentication and upstream errors. Project both exact caller targets, both original declared binders, ordered definition-free equality and sole original Core guard. Rebuild the authenticated Core CFG to obtain the callee flow. Pass those projections and the same request/Core to unchanged Kernel `check_unary_caller_return_single_valuedness` once. Return the whole unchanged tuple only on true; impossible local shape/flow or Kernel refusal reuses `algorithms.unary_child.invalid_caller_e4_request`.
This checks only `forall r,s. (returns_caller(actual,r) & returns_caller(actual,s)) -> r=s` under the original object-input guard. Returns remain antecedents; typing, E1/E3, requires/ensures, child/E4 evidence and function equality supply no context premise or proof.
No new type, holder, field, helper, fixture or parallel representation; lower-stage APIs and child/E4/inhabitation/typing/direct-return behavior stay unchanged.
Exclude schema reporting, Proof authority, identity, policy/native adoption, E2, whole acceptance/status, currentness, complete footprints/cache, promotion/publication, broader profiles and owner-gated scope. No spec/.miz/expectation/trace/audit-status, diagnostic, AGENTS or protocol edits.
Files: VC generator.rs, existing runner tests/algorithm.rs, paired VC generator/plan documents and this contract. Audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: four existing genuine-fixture families; confirm one successful setup before expanding.
1. Independent existing six-row oracle and Kernel projections: fresh VC check succeeds and returns the exact whole producer tuple repeatedly, retaining both binders, ordered equality and sole original guard; inputs/products remain unchanged without child/E4 or typing evidence.
2. Genuine renames/trivia/equal spellings and optional terminating flags succeed; equal cloned Core regenerates and checks the same tuple independently.
3. Existing complete request/Core drift and calibrated source/typed/environment/forgery/combined errors refuse atomically with exact upstream producer errors; authentic changed source succeeds only with its own inputs; original recovers. Existing Kernel target/binder/equality/context/flow controls remain effective.
4. Existing child/E4/inhabitation/typing/direct-return and raw unknown/incomplete/cache-miss boundaries remain unchanged; unsupported direct-return source refuses this caller route. No later authority follows.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, VC/Kernel/Proof owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed checking/authentication/uniqueness matrices run immediately; unchanged heavy evidence/cache/policy/aggregate-acceptance/edit-versus-clean groups defer to Muse sealed integration. Full cargo test only on cadence; lane has no full/push trigger.
Exit: only atomic freshly authenticated bounded caller single-valuedness becomes checkable, preserving every excluded authority.
