# Task STEP5C13-DISTINCT-LOCI: source overload selection

Canonical language: English; [Japanese pointer](../ja/STEP5C13-DISTINCT-LOCI.md).
Owning plans: [checker](../../mizar-checker/en/00.crate_plan.md#task-index),
[resolve](../../mizar-resolve/en/00.crate_plan.md#task-index), [test](../../mizar-test/en/00.crate_plan.md#task-index).

## Frozen assignment

- Status: implemented; bounded distinct-loci and attributed-ambiguity profiles; full tier; checker owns typing and overload selection.
- Dependencies: bounded 5A.2/5C.5 plus the source formal lookup introduced by 5C.12.
- Authority: [spec19](../../../spec/en/19.overload_resolution.md) §§19.4.1–3,19.6.3;
  [spec3](../../../spec/en/03.type_system.md) builtin/structure types;
  [spec5](../../../spec/en/05.structures.md) fields/selectors;
  [spec10](../../../spec/en/10.functors.md) equals bodies; spec17 §§17.3/17.3.4/17.9.1 existential proof, full-pattern and constructor inhabitation.
- Case: pass_advanced_semantics_overload_distinct_loci_001, unchanged overload_resolution endpoint.
- Classification: source signature/binding/type/overload producer and execution gaps.
- The attributed consumer uses the implemented OA/OB/OAB prerequisite and actual overload results.

## Retained distinct-loci runner

Authenticate whole source/environment/typed-arena correspondence, declaration owners and source order.
Resolve both ordinary functor parameters, theorem and proof-let bindings to their actual declarations.
Derive both overload candidates from source declarations and normalize all signatures/actuals together.
Check builtin set identity body and the structure-field selector body with real member identities.
Validate the bare structure's constructor inhabitation from its required builtin-set field.
Each of the two applications independently retains both candidates and consumes the unchanged
collection, expansion, viability, specificity and selection pipeline. Only the actual argument type
may select a root; the theorem's expected type and proof citation cannot choose the overload.
Derive the exposed result type from the selected declaration; unsupported shapes fail closed.
No proof acceptance, registrations, views, attributed actuals, general templates or Core/VC credit.
Use existing pipeline/type outputs; no parallel signature IR, source DTO or invented variable SymbolId.
Keep overload_resolution.rs on its explicit-payload boundary and preserve its import lint.
Source intake belongs to type_checker.rs under its existing exact syntax-enum import exception.
Minimally extend resolver formal lookup to ordinary DefinitionParameter and functor-pattern roles;
retain prior template/registration lookup, scope, recovery and ordering boundaries.
The harness owns exact id/source/sidecar/stage/phase/pass/tags and consumes checker outputs only.
Activate only the positive; beyond those witness insertions preserve .miz/expectation intent, trace and activation map.
Owner APIs/tests: [type checker](../../mizar-checker/en/type_checker.md#distinct-loci-overload-source-checking),
[overload](../../mizar-checker/en/overload_resolution.md), [names](../../mizar-resolve/en/names.md#unbounded-template-formal-resolution),
[harness](../../mizar-test/en/harness.md#distinct-loci-overload-admission).
Audit impact: Chapter19 retains distinct-loci selection and tracks the bounded attributed continuation below.

## Source-proof and attributed continuation

Use unchanged `tests/miz/fail/overload/fail_advanced_semantics_overload_ambiguous_candidates_001.miz` to authenticate and prove its three original registrations through existing checker/Core/VC/proof signatures.
[Proof status](../../mizar-proof/en/status.md#three-registration-source-proof-extension) requires six fresh normal-kernel leaves, three independent nonempty seeds and exact same-source full-pattern activation; OAB retains both guarded definitions and substitutions; the [harness](../../mizar-test/en/harness.md#three-registration-source-proof-prerequisite) owns its controls.
Preserve the original R profile, source/expectation semantics/trace/map and positive C13 outcome. No general Task274/import, closure, callback, receipt or Step6/MVM scope.
The [attributed checker consumer](../../mizar-checker/en/type_checker.md#attributed-set-overload-consumer) owns binding gates, return/argument widening and actual root comparison; [harness admission](../../mizar-test/en/harness.md#attributed-ambiguity-admission) owns fresh same-input proof orchestration and exact failure mapping.
Only completion of that pipeline may activate the existing ambiguity tag/note; no lower-stage or missing-evidence failure earns its expected key.

## Exit

Independent specification/docs, test-sufficiency, implementation/soundness, volume and consistency reviews.
Test real source and consistent renaming, both candidates/sites, selected roots and result types,
wrong owner/binder/member/callee/type/order, missing or duplicate candidates and malformed provenance.
A structure-argument source variation must select the other genuine root without changing the oracle.
The retained distinct-loci runner requires deterministic existing outputs, no accepted effects and exact admission/cross-stage/missing-row checks.
Run focused tests and corpus, fmt, warnings-denied full Clippy and cargo test; commit this bounded increment.
The bounded 5C.13 profiles execute both original oracles at their frozen endpoint; broader overload semantics remain deferred.
