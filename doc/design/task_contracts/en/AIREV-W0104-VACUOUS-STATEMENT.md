# Task AIREV-W0104-VACUOUS-STATEMENT: statements that hold only vacuously
Canonical language: English; [Japanese pointer](../ja/AIREV-W0104-VACUOUS-STATEMENT.md).
Status: planned. Tier: full. Owner: [proof plan](../../mizar-proof/en/00.crate_plan.md); consumers: [kernel plan](../../mizar-kernel/en/00.crate_plan.md) (read-only check), [test plan](../../mizar-test/en/00.crate_plan.md).
Dependencies: kernel-accepted evidence for real obligations (Step 7).
Authority: specification §22.6.2 (W0104), §22.5.1 (E0401 takes precedence), §21.7.5.
Gap: `test_gap`; no producer detects vacuous statements.

## Scope

For a theorem, lemma, or scheme proposition of the form `for x st P holds Q`
or `P implies Q` discharged by a single justification, re-check its accepted
evidence with the negated conclusion removed. If the premises are still
refuted, report W0104 with a stable internal detail key, unless the
contradiction lies in declared axioms (E0401). The check is diagnostic only
and never changes a proof status.

Use the kernel's existing public checking entry. If that entry cannot express
the re-check without changing acceptance behavior, stop and request a
kernel-owned read-only diagnostic entry; that is a kernel-scope change and
needs user consultation.

## Forbidden

Changing proof status or acceptance; checking structured proofs or
contradiction goals; searching for new evidence; specification changes.

## Tests and exit

The §22.6.2 example, a non-vacuous theorem with the same shape, a structured
proof that is skipped, and an axiom contradiction reported as E0401 only.
Require independent specification, test-sufficiency, implementation,
volume/scope, and consistency reviews; `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
Exit: W0104 appears exactly for vacuous single-justification statements.
