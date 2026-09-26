# Task ALGDEF-CHECKER280-DEFINITION-CYCLES: definition dependency cycles
Canonical language: English; [Japanese pointer](../ja/ALGDEF-CHECKER280-DEFINITION-CYCLES.md).
Status: planned. Tier: full. Owner: [checker plan](../../mizar-checker/en/00.crate_plan.md); consumers: [resolve plan](../../mizar-resolve/en/00.crate_plan.md) (dependency edges), [test plan](../../mizar-test/en/00.crate_plan.md).
Dependencies: completed checker definition producers (Tasks 259-262); resolver declaration dependency edges and resolved algorithm call targets ([architecture 03](../../architecture/en/03.module_and_symbol_resolution.md)). Preflight must confirm that both exist; if either is missing, stop for a named resolver prerequisite instead of reconstructing it.
Authority: specification §10.3.3 recursion confinement, §22.5.2 E0410, §20.8.2 recursive groups.
Gap: `test_gap`; no owner currently checks `logic.circular_definition`.

## Scope

Build the definition dependency graph over `func`, `pred`, `mode`, and `attr`
definiens and over algorithm bodies. Reject every cycle that
passes through a definiens, including one that also passes through an
algorithm, with a stable internal detail key. Accept cycles confined to
algorithm call edges, which form a recursive group.

## Forbidden

Public diagnostic registration (the E0410 code stays spec-reserved until a
separately authorized diagnostics adoption); termination checking (VC 52);
treating synonyms or algorithm contracts as definition edges (§10.3.3 names
definiens and algorithm calls only); specification, corpus, or expectation
changes made to match behavior.

## Tests

Test-first real sources: fail cases for a direct `func` self-reference, the
§22.5.2 mutual `func` cycle, and the §10.3.3 `func`/algorithm cycle; pass
cases for a recursive algorithm, a mutually recursive algorithm group, and a
`func` definiens that uses a promoted algorithm.

## Coverage, reviews, and exit

Coverage audit: the chapter 10 and 22 rows record this ownership of E0410
semantics. Require independent specification, test-sufficiency,
implementation, volume/scope, and consistency reviews; `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
Exit: every definition cycle outside an algorithm recursive group is rejected
and every confined algorithm recursion is accepted.
