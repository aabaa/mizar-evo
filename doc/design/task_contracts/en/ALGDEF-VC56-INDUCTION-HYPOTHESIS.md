# Task ALGDEF-VC56-INDUCTION-HYPOTHESIS: algorithm induction-hypothesis context
Canonical language: English; [Japanese pointer](../ja/ALGDEF-VC56-INDUCTION-HYPOTHESIS.md).
Status: planned. Tier: full. Owner: [VC plan](../../mizar-vc/en/00.crate_plan.md); consumers: [test plan](../../mizar-test/en/00.crate_plan.md).
Dependencies: Core 46 and 52 (terminating intent, header measure, recursive-group membership); VC 43, 47, 51, and 52 (postcondition, invariant, and measure formulas) and VC 44 (call substitution) in the [algorithm VC graph](../../mizar-vc/en/source_vc_decomposition.md#accepted-algorithm-vc-task-graph).
Authority: specification §20.4.1 self-reference rule, §20.13.4 induction hypothesis, §20.13.1 and §20.13.3 VC schema; [architecture 07](../../architecture/en/07.vc_generation.md#algorithm-vcs-follow-structured-control-flow).
Gap: `test_gap` and `source_drift`; no VC context carries the induction hypothesis.

## Scope

For a `terminating` algorithm with a header `decreasing` measure, add to the
initial context of every VC of its body the §20.13.4 hypothesis for the
algorithm and for each member of its recursive group, guarded by a strictly
smaller lexicographic measure, with provenance to the contracts and the
measure. There is no hypothesis at the current arguments, and none at all
without a header measure.

## Forbidden

Current-argument hypotheses; hypotheses for algorithms that are not
`terminating` (their self-reference is rejected upstream by Core 46);
termination acceptance; promotion facts (VC 57); discharge, ATP, or kernel
work; specification, corpus, or expectation changes made to match behavior.

## Tests

Test-first real sources: `fact_loop` from §20.7.3, whose invariant and return
VCs carry the hypothesis and never the current-argument contract; a recursive
algorithm that uses the hypothesis for another group member; a near-miss
without a header measure that carries no hypothesis. Corruption tests cover a
wrong measure, owner, or group.

## Coverage, reviews, and exit

Coverage audit: the chapter 20 row records this ownership of the §20.13.4
context. Require independent specification, test-sufficiency, implementation,
volume/scope, and consistency reviews; `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
Exit: each owned VC context contains exactly the §20.13.4 hypothesis and
nothing about the current call.
