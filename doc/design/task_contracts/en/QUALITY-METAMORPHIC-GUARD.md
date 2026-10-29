# Task QUALITY-METAMORPHIC-GUARD: semantics-preserving corpus variants
Canonical language: English; [Japanese pointer](../ja/QUALITY-METAMORPHIC-GUARD.md).
Status: complete. Tier: full. Owner: [test plan](../../mizar-test/en/00.crate_plan.md); consumers: every producer crate whose active corpus cases it varies.
Dependencies: the active semantic corpus and its runners; no producer change.
Readiness: Ready through current real producers; failed variants remain named producer defects rather than expectation changes.
Authority: the authority order of [autonomous_crate_development.md](../../autonomous_crate_development.md#authority-order) (tests and specification outrank implementation); specification §§2.2/2.9 (layout/comments), 4.6 (variable scope), 15.10 (label scope), and 22.1 (source locations).
Gap: `test_gap`. Active cases prove behavior only on their exact bytes; nothing shows that producers generalize to sources they were not fitted to.

## Scope

For every active semantic corpus case, derive deterministic variants that
preserve meaning: a leading blank line and comments/whitespace inserted at
verified intertoken layout boundaries. Also rename an eligible bound variable
or proof-local label capture-free, preserving duplicate and invalid-reference
relations. Record rename non-applicability when no safe target exists; it is
neither a producer exception nor permission to omit layout variants.
Run variants through the same producers; compare actual outcome, phase,
failure category and stable detail key. Transport retained producer ranges
through the explicit byte-edit map; report unavailable locations as such. Variants are generated in the test, not committed as
corpus files, and never earn coverage credit.

Case-identity admission that binds exact source bytes keeps its credit role;
the guard exercises the producers directly and does not relax admission.
Select active cases from the unchanged originals, including advanced semantics.
Test-only input substitution must keep original admission separate from variant
execution. Snapshot byte identities are not semantic observations; preserve
existing snapshot assertions and compare raw producer outputs for variants.

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
Exit: every active semantic case either passes all applicable variants or has a named
exception owned by a producer task.
