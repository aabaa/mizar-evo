# Task QUALITY-EXACT-PROFILE-RETIREMENT: retire fixture-shaped production profiles
Canonical language: English; [Japanese pointer](../ja/QUALITY-EXACT-PROFILE-RETIREMENT.md).
Status: planned. Tier: full. Owner: [checker plan](../../mizar-checker/en/00.crate_plan.md); consumers: [Core plan](../../mizar-core/en/00.crate_plan.md), [test plan](../../mizar-test/en/00.crate_plan.md).
Dependencies: QUALITY-LITERAL-OFFSET-RATCHET (baseline to lower) and QUALITY-METAMORPHIC-GUARD (evidence of generality); checker Task 258, which owns the general statement producer.
Authority: [source statement design](../../mizar-checker/en/source_statement.md); the authority order of [autonomous_crate_development.md](../../autonomous_crate_development.md#authority-order).
Gap: `design_drift`. `SourceStatementProducer` accepts only the task-named statement profiles, each matching exact byte ranges and node indices of one corpus file; only mizar-test consumes it.

## Scope

Replace each task-named exact profile with shape validation over the
authenticated handoffs, or, where a profile serves only a test, move it into
the test module. Apply the same rule to the remaining literal-offset checks in
`source_functor_definition` and in Core. Lower the ratchet baseline to match.
Behavior on every active case is unchanged, and the metamorphic variants of
the affected cases pass.

## Forbidden

Changing specification, corpus, expectations, trace status, or coverage
credit; widening accepted semantics beyond what the handoffs authenticate;
leaving a fixture-specific branch in production code.

## Tests and exit

Existing active cases and their metamorphic variants; negative cases showing
that malformed or foreign handoffs are still rejected without exact offsets.
Require independent specification, test-sufficiency, implementation,
volume/scope, and consistency reviews; `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
Exit: no production file compares against literal source offsets, and the
ratchet baseline for them is removed.
