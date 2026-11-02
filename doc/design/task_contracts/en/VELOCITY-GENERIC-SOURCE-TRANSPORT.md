# Task VELOCITY-GENERIC-SOURCE-TRANSPORT: one producer and one oracle for original-source transport
Canonical language: English; [Japanese pointer](../ja/VELOCITY-GENERIC-SOURCE-TRANSPORT.md).
Status: in progress; steps 1-3 complete. Tier: full for steps 2-3 and the initial shared native statement consumer; pattern tier for subsequent single-shape family slices. Primary owner: [checker plan](../../mizar-checker/en/00.crate_plan.md); consumers: [test plan](../../mizar-test/en/00.crate_plan.md), [Core plan](../../mizar-core/en/00.crate_plan.md).
Dependencies: VELOCITY-LARGE-FILE-SPLIT. Subsumes checker Task 281 ([QUALITY-EXACT-PROFILE-RETIREMENT](./QUALITY-EXACT-PROFILE-RETIREMENT.md)); its exit criterion is exit criterion 3 below.
Readiness: step 1 froze the producer, boundary and oracle contract in the [source statement design](../../mizar-checker/en/source_statement.md#generic-original-source-transport-task-284-design) and the classification ledger `tests/coverage/source_transport_classification.tsv`; statement/witness migration may start.
Authority: [source statement design](../../mizar-checker/en/source_statement.md); [resolved typed AST](../../mizar-checker/en/resolved_typed_ast.md); the specification sections cited by the existing per-element contracts; [authority order](../../autonomous_crate_development.md#authority-order).
Gap: `design_drift`. Original-source transport is implemented one syntax element at a time (Mode, Functor, Property, Structure, and Attribute coordinates; empty and singleton enumeration and bare-set choice witnesses), each as a lower/pair/preservation triple with hand-written literal-offset oracles. This dominated the week ending 2026-11-02 without moving the roadmap.

## Scope
1. Design (docs-only commit): inventory the syntax elements and witness/layout forms of Tasks 257-279 that still lack original-source transport. Classify each as (a) derivable by a generic walk over the authenticated frontend tree, (b) needing an element-specific rule, or (c) semantic and deferred. Record the result in the source statement design document and freeze the generic producer's input/output contract and its refusal conditions.
2. Generic producer: one crate-private walk over the authenticated `TypedAst`/frontend handoff that derives sites, ranges, parents, and ordering for every (a) element from the actual tree, replacing the per-element `from_original`-style constructors. Element-specific rules (b) plug into the walk and do not duplicate it.
3. Generic oracle: tests compare producer output with the frontend's own spans and structure for every active corpus case (Root-reachable shape, documented recovery anchors, ordering, and metamorphic layout variants; unrecoverable frontend absence retains its diagnostics), replacing literal-offset profiles. Retire the `exact_*_profile` functions and lower `tests/coverage/literal_source_offset_baseline.tsv` to zero for the affected modules.
4. Unit of work: a remaining (b) element is one pattern-tier slice under one family contract; the lower/pair/preservation triple is no longer the unit.

## Family slices
Pattern predecessor: the completed initial shared native statement consumer; subsequent slices repeat its authenticated table and oracle pattern.
- [x] step 1 design and classification ledger
- [x] step 2 generic walk and `TypedAst` slot, with the widened type-only kind import in the syntax-import lint
- [x] step 3 generic runner oracle and metamorphic route
- [x] initial shared native statement consumer (full): read installed table; preserve canonical receiver comparison
- [x] bare unnamed finite enumeration witness: ordered references to the sole reserved set variable
- [x] unused named finite enumeration witness: fresh name, same reserved-reference elements, no binding activation
- [x] unused named bare-set Choice witness: preserve pending requests, no abbreviation activation
- [x] dormant unnamed numeral witness: retire its raw profile into private tests; numeric semantics remain deferred
- [x] dormant unnamed Qua witness: retire its raw profile into private tests; widening semantics remain deferred
- [x] dormant unnamed comprehension witness: retire its raw profile into private tests; generator and proof semantics remain deferred
- [x] dormant numeric enumeration witness: retire its raw profile into private tests; numeric payload and witness semantics remain deferred
- [x] dormant wrapped application witness: retire its raw profile into private tests; bare application and lower APIs remain unchanged
- [x] dormant bare application witness: retire its raw profile into private tests; preserve authenticated lower donors and native paths
- [x] dormant structure-update witness: retire its raw profile into private tests; preserve independent lower structure handoffs
- [x] dormant structure-selector witness: retire its raw profile into private tests; preserve independent lower structure handoffs
- [x] dormant structure-constructor witness: retire the final raw witness profile and now-unused shared raw internals into private tests
- [x] dormant explicit-universal upper statement: retire raw B4A; preserve active lower-only universal transport
- [x] dormant nested-quantifier upper statement: retire raw B4C; preserve active lower-only nested transport
- [x] dormant connective-grouping upper statement: retire final raw B4 profile; preserve independent lower transport
- [x] dormant imported-theorem citation: retire raw B5B; preserve native local references and import provenance tests
- [ ] statement/witness family migration; retire its exact profiles and baseline rows
- [x] isolated unused proof-local Let syntax: generic table sites and lexical layouts/names; C/CT canonical dependency boundary unchanged
- [x] isolated unused proof-local Given syntax: generic table sites and lexical layouts/names; G/GT canonical dependency boundary unchanged
- [x] isolated Given-use syntax: generic sites with matching declaration/use names; share receiver at three real consumers, preserve canonical donors
- [x] isolated Given-condition syntax: actual condition-reference sites and raw hashes; preserve GC/GCT/GCU canonical dependencies
- [x] descendant Given/Now/Set-chain syntax: generic sites and ordering; preserve canonical donors and deferred Set/capture semantics
- [x] proof-local Let type arena: derive existing overlay anchors from authenticated lower and binding sites; preserve canonical type donor
- [x] proof-local Given type arena: derive existing overlay anchors from authenticated lower and binding sites; preserve canonical type donor
- [ ] proof-local declaration family (Task 269)
- [ ] type, term and formula families
- [ ] class `b` element rules: Tasks 257, 269, 270, 271, 272, 279 per the ledger

## Forbidden
Fabricating sites or ranges absent from the frontend tree; widening admitted semantics; changing `doc/spec`, `.miz`, expectations, trace status, or coverage credit; keeping a fixture-specific branch in production code; adding a parallel AST or span representation where an existing one can be referenced.

## Verification and exit
Existing active cases and their metamorphic variants pass unchanged; negative cases show malformed or foreign handoffs are still refused without literal offsets. Reviews: specification, test-sufficiency, implementation, volume/scope, consistency. `cargo fmt --check`; Clippy per `AGENTS.md`; verifier and corpus checks under the cadence; full `cargo test` at exit.
Exit: (1) every (a) element transports through the generic walk; (2) no (a) element needs its own transport contract; (3) no production file compares against literal source offsets and their baseline rows are removed; (4) remaining (b) and (c) items are listed with owners in the checker TODO.
