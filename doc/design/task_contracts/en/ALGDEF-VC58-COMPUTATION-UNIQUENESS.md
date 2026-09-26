# Task ALGDEF-VC58-COMPUTATION-UNIQUENESS: computation uniqueness obligation
Canonical language: English; [Japanese pointer](../ja/ALGDEF-VC58-COMPUTATION-UNIQUENESS.md).
Status: planned. Tier: full. Owner: [VC plan](../../mizar-vc/en/00.crate_plan.md); consumers: [test plan](../../mizar-test/en/00.crate_plan.md).
Dependencies: the existing [bounded computation request](../../mizar-vc/en/generator.md#bounded-computation-request-generation); Core 42, 44, and 46 (Pick sites, collection loops, resolved calls); VC 44 (call substitution).
Authority: specification §20.9.2 determinism requirement, §20.3 and §20.2.4 computation restrictions, §22.4.5 E0330; [architecture 07](../../architecture/en/07.vc_generation.md#algorithm-vcs-follow-structured-control-flow).
Gap: `test_gap` and `source_drift`; no computation step carries the uniqueness obligation.

## Scope

For a `by computation` step, compute the computation closure over resolved
calls within the same package. An imported callee whose body is available only
through a computational export (§20.9.1, parked with the MVM) leaves the
closure unestablished: the step stays open and nothing is guessed. For each
value-returning algorithm in the closure that contains a non-ghost Pick
site or a `for ... in` loop, attach the §20.9.2 uniqueness obligation to the
step with provenance. When `ensures` has the form `result = t` or
`result = true iff φ` with no `result` in `t` or `φ`, record the exemption
instead of an obligation.

## Forbidden

MVM evaluation or computation acceptance (parked); public E0330 projection
(proof-status projection is downstream); a canonical iteration order; treating
ghost-only Pick as a source; specification, corpus, or expectation changes
made to match behavior.

## Tests

Test-first real sources: `exists_greater` from §20.9.2 (exempt), `find_even`
from §20.3 (obligation generated and left open), a closure that reaches such
an algorithm transitively, and a deterministic loop algorithm with no
obligation, and an imported callee that leaves the closure unestablished.
Corruption tests cover a missing callee, a stale closure, and a forged
exemption.

## Coverage, reviews, and exit

Coverage audit: the chapter 20 row records this ownership of the §20.9.2
obligation. Require independent specification, test-sufficiency,
implementation, volume/scope, and consistency reviews; `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
Exit: every computation step carries exactly the obligations and exemptions
that §20.9.2 requires, and no computation result is accepted.
