# Task ATP-SOURCE-REGISTRATION-PROJECTIONS: authentic bounded ATP input
Canonical language: English; [Japanese pointer](../ja/ATP-SOURCE-REGISTRATION-PROJECTIONS.md).
Status: implemented. Tier: full. Owner: [ATP plan](../../mizar-atp/en/00.crate_plan.md) (33); consumer: [Test plan](../../mizar-test/en/00.crate_plan.md), ATP30's later finder.
Dependencies: implemented Core/VC source registration profile and VC61 genuine default-discharge handoff. Readiness: no backend or accepted result is required for problem construction.
Authority: specification §§9.9.3–9.9.4/21.7.2/21.7.5; [Core profile](../../mizar-core/en/elaborator.md#source-existential-registration-proof); [VC profile](../../mizar-vc/en/generator.md#source-existential-registration-proof); [projection owner](../../mizar-atp/en/translator.md#source-registration-projections).
Gap: bounded structured source-projection `source_gap`; general source extraction and candidate transport/search remain deferred.

Scope: implement the projection owner's single existing-module function over authentic Core/VcSet/selected NeedsAtp and caller profile, then use the existing translator and concrete encoders. No new record/module/helper file or production dependency.
Preserve the whole source-profile replay, default-discharge result, actual source/target/context/provenance and cited premise set; do not substitute a chosen VC status or declaration identity.
Use structured Core/VC trees for payloads; explicit universal closure applies only to the real shared definition formal in premise formulas. Reject a free-variable goal or unsupported fragment/profile without changing profiles.
The type context repeats only the selected actual ground choice-type premise with its matching handoff provenance; retain that premise in the axiom set, add no distinct fact, and never assert a definition-formal guard universally.
Tests use the real original/renamed single and three-registration source pipelines, actual discharge, complete expected trees/declarations/binders/guards/type context/target/provenance, each real encoder under its own unchanged compatible caller profile, repeat determinism, non-NeedsAtp and coherent whole-set/foreign-Core corruptions.
Existing authority tests: `tests/miz/pass/types/pass_type_elaboration_argument_attribute_widening_001.miz`, `tests/miz/fail/overload/fail_advanced_semantics_overload_ambiguous_candidates_001.miz`; real replay/corruption controls: `crates/mizar-test/src/runner/tests.rs::source_registration_discharge_handoff_replays_actual_default_phase_and_rejects_corruption`.
Audit impact: §21 remains partial; this bounded source-projection owner narrows its projection deferral, while generic extraction, candidate evidence and acceptance remain deferred.
Affected sources: ATP translator and its exact public-function guard, existing Test runner tests plus a genuine ATP dev dependency/lock; paired owner/plan/TODO/audit inventory only as required. Existing tests and protected authority remain unchanged apart from the guard's explicitly owned additive API allowance.
Forbidden: decoding opaque handoff/debug/backend/source text as formulas, invented numeric kernel names, new public types/fields/adapters, fake producers/evidence/acceptance/cache hits, forced statuses, backend/kernel/Proof policy execution or protected spec/.miz/expectation/trace changes.
Deferral: full ATP30/Steps6–8, active ATP runner, generic projection production, candidate evidence, real backend/portfolio, accepted proof/witness/cache/artifact integration remain open; source problem construction supplies no proof credit.

Reviews: independent specification, test sufficiency, implementation, volume/scope and consistency. Parent checks: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
Exit: only matching genuine NeedsAtp inputs yield deterministic structured and encoded ATP problems; all unsupported/mismatched inputs yield no problem.
