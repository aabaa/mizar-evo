# Task VC53-UNARY-CALLER-E1-INHABITATION-CHECK: fresh caller object-inhabitation checking
Canonical language: English; [Japanese pointer](../ja/VC53-UNARY-CALLER-E1-INHABITATION-CHECK.md).
Status: frozen checking-only slice. Tier: full; new public semantic-authority API fails pattern eligibility.
Owner: [VC](../../mizar-vc/en/00.crate_plan.md). Consumer: mizar-test.
Dependency: [authenticated caller request](VC53-UNARY-CALLER-E1-INHABITATION-REQUEST.md).
Authority: [§§20.1.2, 20.2.6, 20.7.2, 20.13.1 E1](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [caller request](../../mizar-vc/en/generator.md#bounded-unary-caller-e1-inhabitation-request), [Kernel introduction](../../mizar-kernel/en/checker.md#bounded-e1-inhabitation-witness-introduction), [dependency ownership](../../mizar-vc/en/dependency_slice.md).
Gap: source_drift/test_gap; the authentic caller request and Kernel rule exist without an atomic fresh caller checking API. No language-intent change or external prerequisite.

## Frozen design and boundaries
Add only `check_source_unary_child_call_caller_e1_inhabitation_result(source, typed, symbols, core, request)` in VC generator.rs, returning `Result<((LocalContext, [VcGeneratedFormula; 2], CoreTermId), ObjectInhabitationCheck<'core>), String>`.
Call unchanged `generate_source_unary_child_call_caller_e1_inhabitation_request` once first on the identical five inputs; preserve upstream errors. Extract the fresh declared binder, atomic object body and original caller guard from its tuple. Call unchanged Kernel `check_object_inhabitation_result` once with that target, the retained actual-input witness, and the original guard as both child obligation and sole context. Return the entire unchanged tuple and opaque result only on success; local refusal reuses `algorithms.unary_child.invalid_existential_request`.
No new type, field, helper, fixture or parallel representation; no lower-stage, E4 or direct-return changes. Checking proves only `exists r. object(r)` under the caller object-input guard, assuming neither returns nor requires/ensures.
Exclude native adoption, identity, status, acceptance, currentness, complete footprints/cache, promotion/publication, E1 typing/single-valuedness, E2, whole-algorithm credit, broader profiles and owner-gated scope. No spec/.miz/expectation/trace/audit-status, diagnostic, AGENTS or protocol edits.
Files: VC generator.rs, existing runner tests/algorithm.rs, paired VC generator/plan documents and this contract. Audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: four existing genuine-fixture families; confirm one successful setup before expanding.
1. Independent exact formula/witness oracle and purity: unchanged whole request tuple, same-Core result, fresh declared binder, sole original caller object guard and existing Kernel schema; repeated calls preserve all inputs/products.
2. Genuine renames/trivia/equal spellings and optional terminating flags succeed; equal cloned Core retains the exact tuple and produces a result bound to that clone.
3. Complete existing request/Core drift and calibrated source/typed/environment/forgery/combined errors refuse atomically with unchanged upstream errors; authentic changed source succeeds only with its own inputs; original recovers.
4. Existing child/E4/direct-return and raw unknown/incomplete/cache-miss boundaries remain unchanged; unsupported direct-return source refuses this caller route. No later authority follows.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, VC/Kernel/Proof owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed checking/authentication matrices run immediately; unchanged heavy evidence/cache/aggregate-acceptance/edit-versus-clean groups defer to Muse sealed integration. Full cargo test only on cadence; lane has no full/push trigger.
Exit: only atomic freshly authenticated caller object-inhabitation checking becomes available; every excluded authority remains separate.
