# Task ATPK-BENCHMARK: re-proving benchmark for ATP effectiveness
Canonical language: English; [Japanese pointer](../ja/ATPK-BENCHMARK.md).
Status: planned. Tier: light. Owner: [test plan](../../mizar-test/en/00.crate_plan.md); consumers: [atp plan](../../mizar-atp/en/00.crate_plan.md), [kernel plan](../../mizar-kernel/en/00.crate_plan.md).
Dependencies: ATPK-INSTANCE-FINDER and a real backend runner; ATPK-PREMISE-RANKING for the suggestion condition.
Authority: specification §21.7 and §22.4.4.
Gap: `test_gap`; ATP effectiveness is unmeasured.

## Scope

For each proved theorem of a chosen library slice, remove its proof and try to
re-prove it under three conditions: its original citations only, a bulk
citation of its modules, and the facts its original proof actually used.
Report, per condition and per class (pure logic, equational, arithmetic), the
backend success rate and the kernel acceptance rate. Results are written to a
report outside `doc/`; the benchmark earns no coverage credit.

## Forbidden

Using results to change expectations or proof status; storing scores in
`doc/`; nondeterministic sampling.

## Tests and exit

The harness is tested on a small fixed slice with known outcomes. Require
specification, implementation, volume/scope, and consistency reviews;
`cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`,
`cargo test`. Exit: the benchmark runs reproducibly and reports both rates for
every condition and class.
