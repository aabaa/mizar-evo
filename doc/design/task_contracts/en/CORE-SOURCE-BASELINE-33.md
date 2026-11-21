# Task CORE-SOURCE-BASELINE-33: first Core context/item baseline
Canonical language: English; [Japanese pointer](../ja/CORE-SOURCE-BASELINE-33.md).
Status: frozen for P20. Tier: full; first production identity-shell baseline.
Owner: [Core plan](../../mizar-core/en/00.crate_plan.md); consumer: [test plan](../../mizar-test/en/00.crate_plan.md).
Dependencies: completed Checker 248/259 and [33LB](CORE-SOURCE-LOCAL-BINDER-CONTEXT-33LB.md)/[33I259](CORE-SOURCE-PREDICATE-ITEM-CONTEXT-33I259.md).
Readiness: their immutable context and predicate-owner handoffs authenticate the bounded source shape.
Authority: [variables §4.2/4.6](../../../spec/en/04.variables_and_constants.md#42-variable-declarations-let), [predicate declarations §9.1](../../../spec/en/09.predicates.md#91-overview-and-syntax), [public default §12.5](../../../spec/en/12.modules_and_namespaces.md#125-visibility-control-privatepublic).
Design authority: [Core33 graph](../../mizar-core/en/source_family_decomposition.md#accepted-coreir-task-graph), [prepared consumer](../../mizar-core/en/source_family_decomposition.md#prepared-mizar-test-consumers), [context invariant](../../mizar-core/en/elaborator.md#core-33-task259-predicate-item-context-prerequisite), [Core source map](../../mizar-core/en/core_ir.md#source-map-and-provenance).
Gaps: `source_drift` in absent production composition; `test_gap` in the absent non-Task180 baseline. Missing module-context storage in CoreIr remains deferred `design_drift`.

## Frozen scope
Add `SourcePredicateCoreContextProducer::lower(source_context, checker_owner)`
in `crates/mizar-core/src/elaborator.rs`, returning the existing immutable
`SourcePredicateCoreContextHandoff` and a validated `CoreIr`, using the existing
error type. Core creates the exact public Predicate seed, checker provenance
and pending definition boundary, allocates variables through 33LB and validates
the complete association through 33I259 before publishing either output.
The CoreIr contains the item and its exact source map; all semantic-body
tables are empty. The retained handoff owns normalized free variables, binding
identity/order, local scope, visibility, source ranges and checker contexts.
Those binder/context contents are independently checked and are **not** part
of the CoreIr snapshot bytes. No parallel representation or new public type/field.

Install the first bounded `MT10-CIR-TE` runner path for the new real fixture
`pass_type_elaboration_core_context_baseline_001` under `tests/miz/pass/types/`,
with the existing Task259 source shape and unchanged language intent.
Require exact source/sidecar identity, domain `core.context`, active type tag,
pass kind/outcome and elaboration phase; reject metadata deviations before
checker dispatch. Independently rebuild and compare both outputs. Wrap the
complete `CoreIr::debug_text()` body in an existing CoreIr `SnapshotRecord`,
and verify its canonical bytes/hash against the committed `.core_ir.snap`.
No update mode or general snapshot registry activation. Existing Task180 and
checker-only fixtures/expectations remain unchanged.

Affected artifacts: existing runner/type-elaboration, expectation admission,
their existing test modules, the new fixture/sidecar/baseline, scoped trace row,
paired elaborator/harness/schema owner notes and Core TODO. Audit impact:
record only this identity-shell baseline; broad Core33--35 rows stay deferred.
General Core33, other eight associations, Core34--39, type results, guards,
definiens/correctness bodies, proof/algorithm/VC, GeneratedOrigin and semantic
acceptance remain deferred; no spec or protected expectation edits.

## Test matrix and exit

Case cap: six groups. (1) Real source: exact public item, fresh ordered binders,
source/provenance and empty semantic tables; independent deterministic replay.
(2) Missing/duplicate/reordered item, binding or context rows: fail closed.
(3) Cross-source/module/owner, recovered/stale/partial inputs: fail closed.
(4) Mutated stage/tag/phase/domain/id/path/kind/ref admission: rejected.
(5) Missing output/baseline, changed body, kind/hash or bytes: rejected.
(6) Existing Task180 and Checker248/259 routes preserve their outcomes.
Reuse existing association corruption oracles; add tests only for the new seam.
Reviews: specification/test intent, sufficiency, precision, volume/scope,
source/documentation consistency and quality scoring under full gates.
Verification: fmt, all-target/all-feature Clippy, both lint_policy suites,
`cargo test -p mizar-core`, affected mizar-test context/predicate/snapshot tests,
type-elaboration corpus and documentation link/fragment checks. Run affected
snapshot/admission matrices immediately; full cargo test only on a cadence
trigger. Exit: one genuine baseline, fail-closed composition, retained context,
documented residuals, required reviews/checks and task-only local commits.
