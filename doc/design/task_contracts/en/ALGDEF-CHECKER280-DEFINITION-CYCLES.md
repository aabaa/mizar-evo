# Task ALGDEF-CHECKER280-DEFINITION-CYCLES: definition dependency cycles
Canonical language: English; [Japanese pointer](../ja/ALGDEF-CHECKER280-DEFINITION-CYCLES.md).
Status: frozen for P28 direct/mutual func cycles. Tier: full. Owner: [checker plan](../../mizar-checker/en/00.crate_plan.md); consumers: [resolve plan](../../mizar-resolve/en/00.crate_plan.md) (dependency edges), [test plan](../../mizar-test/en/00.crate_plan.md).
Dependencies: completed checker definition producers (Tasks 259-262) and resolver [selected definiens dependencies](../../mizar-resolve/en/symbols.md#planned-algorithm-call-dependencies). Algorithm recursive-group production is owned by [P7](ALGDEF-RESOLVE-CALL-EDGES.md), not a prerequisite for this func-only slice.
Readiness: selected source-authenticated definiens edges are available; ordinary scope candidates do not supply self/forward selections.
Authority: [§10.3.3 recursion confinement](../../../spec/en/10.functors.md#1033-non-recursive-definitional-layer), [§22.5.2 E0410](../../../spec/en/22.error_handling_and_diagnostics.md#2252-circular-definitions), [§20.8.2 recursive groups](../../../spec/en/20.algorithm_and_verification.md#2082-mutual-recursion).
Gap: `test_gap` for direct/mutual func cycles; `design_drift` in the former blanket readiness blocker.

## Scope

Add `type_checker::check_source_functor_definition_cycles(source, symbols,
selected, originals) -> Result<(), String>` using the existing resolver maps
and `source_definiens_dependencies`. Authenticate all supplied associations
before checking reachability over canonical func-to-func definiens edges.
Reject direct and mutual cycles, including longer cycles, with the stable
internal detail key `logic.circular_definition`. No new public type or field.
The caller owns selection correctness and raw-source currentness. Empty or
partial selections check only the supplied graph and confer no completeness,
typing, proof or acceptance credit. Never infer selections from names or
singleton candidates. Preserve immutable input arenas, environments and maps.

## Forbidden

New public diagnostic allocation (E0410/logic.circular_definition is already
registered; source-cycle emission/adoption remains a separate producer task); termination checking (VC 52);
treating synonyms or algorithm contracts as definition edges (§10.3.3 names
definiens and algorithm calls only); specification, corpus, or expectation
changes made to match behavior.
Deferred: pred/mode/attr or mixed cycles, algorithm-body edges and their
recursive-group exemption, automatic self/forward target selection, promotion
and runner adoption. P7 availability alone does not implement those consumers.

## Tests

Test-first real `tests/miz/fail/definitions/` sources for direct func
self-reference and the §22.5.2 mutual func cycle, consumed by Checker unit
tests with explicit occurrence selections. No corpus expectation/trace activation.
Matrix (six families, at most six test functions): (1) real direct/mutual
sources reject with the exact key; (2) longer cycle with an acyclic tail and
repeated edges rejects independent of selection order; (3) acyclic chain,
diamond, empty/partial selections succeed only for the supplied graph;
(4) equal-spelling overload identities remain distinct; (5) original synonym
targets participate but relation-only and algorithm-body calls create no func
edges; (6) malformed/foreign/stale associations refuse before cycle reporting,
including invalid-plus-cyclic input, with inputs unchanged.

## Coverage, reviews, and exit

Coverage audit and trace credit remain unchanged: this is an explicitly
selected Checker producer, not source diagnostic adoption. Require independent specification, test-sufficiency,
implementation, volume/scope, and consistency reviews; `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, both lint_policy
suites, Checker tests and declaration-symbol corpus E2E; full `cargo test`
only on a cadence trigger. Exit: authenticated selected func graphs reject
every cycle and preserve acyclic graphs without broader acceptance claims.
