# Task ALGDEF-RESOLVE33-ALGORITHM-REFERENCES: algorithm synonyms and citations
Canonical language: English; [Japanese pointer](../ja/ALGDEF-RESOLVE33-ALGORITHM-REFERENCES.md).
Status: planned. Tier: full. Owner: [resolve plan](../../mizar-resolve/en/00.crate_plan.md); consumers: [checker plan](../../mizar-checker/en/00.crate_plan.md) (synonym replay and use sites), [VC plan](../../mizar-vc/en/00.crate_plan.md) (VC 57 premises), [test plan](../../mizar-test/en/00.crate_plan.md).
Dependencies: resolver synonym relation records and algorithm declaration shells ([symbols](../../mizar-resolve/en/symbols.md)); parser support for optional terminating definition labels; label resolution; checker synonym replay.
Readiness: dependency-blocked on optional definition-label syntax and authentic algorithm/synonym projections.
Authority: specification §11.1.2 and §11.5.1 algorithm synonyms, §16.5.1 citation of promoted algorithms, §20.1.1; [architecture 03](../../architecture/en/03.module_and_symbol_resolution.md#label-resolution-is-scoped-separately-from-item-resolution).
Gap: `test_gap` and `source_drift`; synonym targets and citations do not admit algorithms.

## Scope

Accept an algorithm application as the original pattern of a functor synonym
and record the algorithm identity as its target. The checker replays the
synonym as the original call and allows it only where that call may appear.
Resolve an optional definition label of a terminating algorithm in `by`;
VC 57 supplies its verified promotion premises. Algorithm names and
`by alg(args)` are not citations (§16.5.1).

## Forbidden

A logical encoding for synonyms of algorithms that are not promoted;
fabricated promotion facts or acceptance; notation that changes lexing outside
the existing synonym rules; specification, corpus, or expectation changes made
to match behavior.

## Tests

Test-first real sources: `synonym n ! for factorial(n);` used in a theorem;
a synonym of an algorithm that is not promoted, rejected as a functor actual
exactly as its original call is (§18); `by FactorialDef` resolved to its definition-label reference; rejection of
a bare algorithm name, `by alg(args)`, and an unknown definition label.

## Coverage, reviews, and exit

Coverage audit: the chapter 11 and 16 rows record this ownership. Require
independent specification, test-sufficiency, implementation, volume/scope,
and consistency reviews; `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
Exit: algorithm synonyms and citations resolve with the same use-site limits
as the original algorithm, and no premise is fabricated.
