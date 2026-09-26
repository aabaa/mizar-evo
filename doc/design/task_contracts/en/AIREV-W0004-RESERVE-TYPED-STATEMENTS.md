# Task AIREV-W0004-RESERVE-TYPED-STATEMENTS: reserve-typed public statements
Canonical language: English; [Japanese pointer](../ja/AIREV-W0004-RESERVE-TYPED-STATEMENTS.md).
Status: planned. Tier: full. Owner: [resolve plan](../../mizar-resolve/en/00.crate_plan.md); consumers: [test plan](../../mizar-test/en/00.crate_plan.md); public code registration follows the resolver diagnostics adoption (resolver task 30).
Dependencies: resolver reserve bindings and theorem visibility.
Authority: specification §22.6.7 (W0004), §4.3 (reserve closure and binder typing), §11.4 (visibility).
Gap: `test_gap`; no producer detects reserve-typed public statements.

## Scope

For each public `theorem`, `lemma`, or scheme proposition, report every
variable whose type comes only from `reserve`, through implicit universal
closure or an untyped binder, with a stable internal detail key, the reserve
declaration as a secondary location, and the explicit binder as the fix.
Private items and proof-local uses produce nothing.

## Forbidden

Reporting private items or proof-local propositions; changing reserve
semantics; allocating the public code before diagnostics adoption;
specification changes.

## Tests and exit

Real sources: a public theorem relying on closure, one relying on an untyped
binder, an explicitly typed public theorem, a private lemma, and a
suppressed case. Require independent specification, test-sufficiency,
implementation, volume/scope, and consistency reviews; `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
Exit: exactly the reserve-typed variables of public statements are reported.
