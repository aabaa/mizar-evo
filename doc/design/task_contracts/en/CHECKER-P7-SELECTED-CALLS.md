# Task CHECKER-P7-SELECTED-CALLS: checked source direct-call targets
Canonical language: English; [Japanese pointer](../ja/CHECKER-P7-SELECTED-CALLS.md).
Status: implemented for the initial profile. Tier: full (new checked selection seam).
Owner: [Checker plan](../../mizar-checker/en/00.crate_plan.md); consumer: [test runner](../../mizar-test/en/00.crate_plan.md).
Dependencies: authenticated source symbols and explicit selected-edge Resolver APIs; no acceptance prerequisite is discharged.
Authority: [module/symbol ownership](../../architecture/en/03.module_and_symbol_resolution.md), [selection ownership](../../architecture/en/05.overload_resolution.md#overload-resolution-finalizes-surface-semantics), [viability](../../../spec/en/19.overload_resolution.md#1942-viable-candidates), [synonyms](../../../spec/en/11.symbol_management.md#1112-functor-synonyms), [definition cycles](../../../spec/en/10.functors.md#1033-non-recursive-definitional-layer), [recursive groups](../../../spec/en/20.algorithm_and_verification.md#208-recursive-definitions).
Classification: source_drift/test_gap; Resolver inventories candidates and consumes explicit selections, but no checked source producer supplies algorithm targets.

## Scope and ownership
Expose check_source_algorithm_call_targets in type_checker, with private source_overload selection implementation, returning existing maps of definiens-reference targets, algorithm-body application targets and original synonym-declaration targets.
Authenticate complete neutral TypedAst/source/environment correspondence, node kinds, recovery and exact occurrence/declaration ranges before returning maps; malformed associations fail atomically.
Resolver retains name lookup, scope/order/visibility, declaration identities, name-level relations and dependency/group graph algorithms. Checker consumes those identities through existing name/candidate APIs and performs actual phase-8 viability and selection; candidate counts alone never supply an edge.
Initial profile: same-module non-overloaded unary direct calls, builtin set/object formal and result types, direct definition-parameter actuals; no local binders, templates, attributed types, conversions, imported or qualified callees. Unsupported sites remain unselected.
Select through existing normalization, collection, expansion, viability, specificity and selection engines. Missing evidence, type mismatch or multiple roots never produces a selected target; expected/result types cannot choose a root.
Support exact unary direct synonym patterns and earlier local original chains with a checked locus bijection; preserve alias identity and declaration-range original links. Do not invent Resolver relation metadata or mutate environments.
Consumer feeds the separate exact-site maps to source_definiens_dependencies and source_algorithm_recursive_groups. Missing body selections preserve None; no partial group is exposed. Inputs remain immutable and reruns deterministic.
This selects dependency identities only. SourceAlgorithmCheck, general call typing/lowering, proof/termination/promotion/cycle acceptance, diagnostics, trace status, audit credit, G19 and other owner-gated work stay unchanged.
Owner boundary: [type checker](../../mizar-checker/en/type_checker.md#static-source-algorithm-checking); lower-stage boundary: [Resolver selected edges](../../mizar-resolve/en/symbols.md#planned-algorithm-call-dependencies).

## Test matrix and exit
Cap: six test families, no more than six test functions; existing genuine frontend fixtures supply setup.
1. Empty/call-free and direct self calls: exact selected symbol/range and group; immutable deterministic inputs.
2. Mutual calls, body order/repetition and source renames: exact maps and maximal source-ordered groups.
3. Direct unary synonym chains: real alias/original identities and ranges; no relation-only body edge.
4. Ordinary definiens calls: exact reference anchors feed real dependency edges without algorithm-body sites leaking into that map.
5. Singleton type mismatch, overloads, unsupported actuals/local binding/template/qualification: no fabricated selection; incomplete body coverage yields None.
6. Foreign/stale/augmented/recovered source, environment or typed projection and forged kind/owner/range/children: atomic refusal.
Reviews: independent authority/test intent before freeze; sufficiency, precision, volume/scope and final source/document consistency.
Verification: fmt; all-target/all-feature Clippy with warnings denied; both lint_policy suites including contract links/fragments; affected checker overload tests and Resolver selected-edge tests; genuine mizar-test producer-to-graph end-to-end check. Full cargo test follows cadence only.
Exit: both consumer graph APIs execute on genuinely checked maps, frozen cases pass, no language intent or acceptance change, and scoped full gates pass.
