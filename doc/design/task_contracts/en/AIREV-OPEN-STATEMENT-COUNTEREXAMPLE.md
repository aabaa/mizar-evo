# Task AIREV-OPEN-STATEMENT-COUNTEREXAMPLE: counterexample search before proving
Canonical language: English; [Japanese pointer](../ja/AIREV-OPEN-STATEMENT-COUNTEREXAMPLE.md).
Status: planned. Tier: full. Owner: [atp plan](../../mizar-atp/en/00.crate_plan.md); consumers: [driver plan](../../mizar-driver/en/00.crate_plan.md) (configuration), [lsp todo](../../mizar-lsp/en/todo.md) (editor action), [test plan](../../mizar-test/en/00.crate_plan.md).
Dependencies: the real backend runner and its counterexample classification (Step 7 item 1).
Authority: specification §22.4.3 (search on open or omitted proofs, E0310).
Gap: `test_gap`; counterexample search runs only after a failed proof step.

## Scope

Allow a counterexample search on a theorem, lemma, or scheme whose proof is
open or omitted, requested through verifier configuration or an editor
action. A found model reports E0310 at the statement with source-level
assignments mapped through provenance; a failed search changes nothing.

## Forbidden

Treating a failed search as evidence; running it by default on every
statement; changing proof status; specification changes.

## Tests and exit

A false open statement with a finite counterexample, a true open statement,
and a request through configuration and through the editor action. Require
independent specification, test-sufficiency, implementation, volume/scope,
and consistency reviews; `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
Exit: requested searches on open statements report E0310 exactly when a
source-mappable model is found.
