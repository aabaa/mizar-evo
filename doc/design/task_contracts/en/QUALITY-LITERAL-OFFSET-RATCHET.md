# Task QUALITY-LITERAL-OFFSET-RATCHET: no new literal source offsets in production
Canonical language: English; [Japanese pointer](../ja/QUALITY-LITERAL-OFFSET-RATCHET.md).
Status: planned. Tier: light. Owner: [test plan](../../mizar-test/en/00.crate_plan.md) (repository lint); consumers: every crate with production `src`.
Dependencies: none.
Authority: [autonomous_crate_development.md](../../autonomous_crate_development.md#authority-order); the existing documentation-volume ratchet is the model for baseline handling.
Gap: `design_drift`. Production code in `mizar-checker` and `mizar-core` compares against source ranges and node indices written as integer literals, which only matches specific corpus files.

## Scope

Add a repository lint that counts, per production source file outside test
modules, source-range constructions with integer-literal offsets and
`expected_ranges` / `expected_sites` arrays. Record the current counts in a
baseline whose rows may only be lowered or removed, like
`tests/coverage/doc_volume_baseline.tsv`. A new file must have a count of zero.

## Forbidden

Rewriting production code in this task; counting test modules or fixtures;
raising a baseline row without explicit user approval recorded in the commit
body.

## Tests and exit

Unit tests cover the counter on literal and non-literal ranges, test-module
exclusion, and stale or raised baseline rows. Require specification,
implementation, volume/scope, and consistency reviews; `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
Exit: the lint passes on the current tree, and any new literal offset in
production code fails it.
