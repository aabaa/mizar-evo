# Task VC53-UNARY-CALLER-E1-SINGLE-VALUEDNESS-REQUEST: bounded caller uniqueness
Canonical language: English; [Japanese pointer](../ja/VC53-UNARY-CALLER-E1-SINGLE-VALUEDNESS-REQUEST.md).
Status: frozen request-only slice. Tier: full; new public producer fails pattern eligibility.
Owner: [VC](../../mizar-vc/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [caller E4 descriptor](VC53-CALLER-E4-DESCRIPTOR.md), [native typing identity boundary](VC53-PROOF-CALLER-E1-TYPING-NATIVE-IDENTITY.md).
Authority: [§§20.1.2, 20.2.6, 20.7.2, 20.13.1 E1/E3](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [caller E4](../../mizar-vc/en/generator.md#bounded-unary-caller-e4-descriptor), [child tuple](../../mizar-vc/en/generator.md#bounded-unary-child-existential), [separate goals](../../mizar-vc/en/generator.md#bounded-direct-return-e1e4-producer-design).
Gap: source_drift/test_gap; authenticated caller E4 exists without a caller single-valuedness request. Existing specification fixes its meaning; no language-intent change or external prerequisite.

## Frozen design and boundaries
Add only `generate_source_unary_child_call_caller_e1_single_valuedness_request(source, typed, symbols, core, request)` in VC generator.rs, returning existing `Result<(LocalContext, [VcGeneratedFormula; 6]), String>`.
First call unchanged `generate_source_unary_child_call_caller_e4_request` once on identical inputs, preserving whole fresh authentication and upstream errors. Retain its context entries/policy inputs and exact row zero caller relation. Append a result BinderDeclaration with the next context ID, a distinct sort key and variable one greater than the original fresh result (checked addition); retain the original binder as the first declaration. Reuse row one's metadata for a second returns atom with only its result changed to the new binder. Rows two through five are And of both returns, ProgramEquals of the ordered results, Implies from And to equality, and Forall over both ordered declarations. IDs are row indices; appended rows are SplitGoal with empty provenance. Local shape/overflow/context refusal reuses `algorithms.unary_child.invalid_caller_e4_request`.
Under only the original caller object-input guard, request `forall r,s. (returns_caller(actual,r) & returns_caller(actual,s)) -> r=s`. Both relations share the exact caller/flow/Return/actual association. Returns remain antecedents; no E1/E3, requires/ensures, child return or function equation is installed as a premise or proof.
No new type, field, holder, helper, fixture or parallel representation; lower-stage APIs and E4/child/inhabitation/typing/direct-return behavior stay unchanged.
Exclude checking, Kernel/Proof authority, identity, policy/native adoption, E2, whole acceptance/status, caller-owned currentness, complete footprints/cache, promotion/publication, broader profiles and owner-gated scope. No spec/.miz/expectation/trace/audit-status, diagnostic, AGENTS or protocol edits.
Files: VC generator.rs, existing runner tests/algorithm.rs, paired VC generator/plan documents and this contract. Audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: four existing genuine-fixture families; confirm one successful setup before expanding.
1. Independent exact formula oracle: sole original object guard, two distinct fresh declared binders, exact shared caller relation, ordered And/equality/implication/universal links; retained metadata/context prefix and policy inputs, repeated output and input/product purity.
2. Genuine renames/trivia/equal spellings and optional terminating flags preserve joins; equal cloned Core regenerates the same tuple without arena-bound evidence.
3. Existing complete request/Core drift and calibrated source/typed/environment/forgery/combined errors refuse atomically with unchanged E4 upstream errors; authentic changed source succeeds only with its own inputs; original recovers.
4. Existing E4/child/inhabitation/typing/direct-return and raw unknown/incomplete/cache-miss boundaries remain unchanged; unsupported direct-return source refuses this route. No checking or later authority follows.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, VC/Kernel/Proof owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed request/authentication matrices run immediately; unchanged heavy evidence/cache/aggregate-acceptance/edit-versus-clean groups defer to Muse sealed integration. Full cargo test only on cadence; lane has no full/push trigger.
Exit: only the authenticated bounded caller E1 single-valuedness request becomes available, preserving every excluded authority.
