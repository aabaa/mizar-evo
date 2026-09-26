# Task QUALITY-METAMORPHIC-GUARD: semantics-preserving corpus variants
Canonical language: English; [Japanese pointer](../ja/QUALITY-METAMORPHIC-GUARD.md).
Status: planned. Tier: full. Owner: [test plan](../../mizar-test/en/00.crate_plan.md); consumers: every producer crate whose active corpus cases it varies.
Dependencies: the active semantic corpus and its runners; no producer change.
Authority: the authority order of [autonomous_crate_development.md](../../autonomous_crate_development.md#authority-order) (tests and specification outrank implementation); specification chapter 2 (layout, comments, identifiers) for which rewrites preserve meaning.
Gap: `test_gap`. Active cases prove behavior only on their exact bytes; nothing shows that producers generalize to sources they were not fitted to.

## Scope

For every active semantic corpus case, derive deterministic variants that
preserve meaning: a leading blank line, inserted comments and whitespace
between tokens, and renaming of a bound variable or proof label to a fresh
identifier. Run each variant through the same producers and require the same
outcome, phase, failure category, and stable detail key, with source ranges
shifted consistently. Variants are generated in the test, not committed as
corpus files, and never earn coverage credit.

Case-identity admission that binds exact source bytes keeps its credit role;
the guard exercises the producers directly and does not relax admission.

## Forbidden

Rebaselining, weakening, or skipping an expectation to make a variant pass;
rewrites that change meaning (reordering statements, renaming exported
symbols); committing generated variants; producer changes in this task.

## Tests and exit

A failing variant is a generalization defect of its producer, reported with the
case, rewrite, and first divergence, and recorded as a named exception only
with the owning producer task that will fix it. Require independent
specification, test-sufficiency, implementation, volume/scope, and consistency
reviews; `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
Exit: every active semantic case either passes all its variants or has a named
exception owned by a producer task.
