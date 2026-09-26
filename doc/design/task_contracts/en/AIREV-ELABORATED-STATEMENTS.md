# Task AIREV-ELABORATED-STATEMENTS: elaborated statements in metadata and hover
Canonical language: English; [Japanese pointer](../ja/AIREV-ELABORATED-STATEMENTS.md).
Status: planned. Tier: full. Owner: [checker plan](../../mizar-checker/en/00.crate_plan.md) (producer); consumers: [artifact plan](../../mizar-artifact/en/00.crate_plan.md) (`statements` array), [lsp todo](../../mizar-lsp/en/todo.md) (hover and code action), [test plan](../../mizar-test/en/00.crate_plan.md).
Dependencies: real statement producers (checker Task 258) and resolver reserve bindings; the `*.mizir.json` writer for publication; mizar-lsp metadata tasks 12-13 for hover.
Authority: specification §21.1.1, §23.5.1 (`statements` array), §23.6.1 (hover and make-binders-explicit action), §4.3 (reserve closure).
Gap: `test_gap` and `source_drift`; no producer emits elaborated statements.

## Scope

For every theorem, lemma, scheme, and definition item, produce its elaborated
statement: explicit binders and binder types (including implicit universal
closure and reserve-supplied types), the reserve-typed variables, and the
resolved fully-qualified name of every symbol spelling. For a definition, use
its definitional axiom. The artifact writer publishes the entries; the LSP
server shows them on hover and offers the make-binders-explicit action.

## Forbidden

Printing text whose meaning differs from the source in the item's scope;
recomputing semantics in the artifact writer or the LSP server; new source
syntax; specification changes.

## Tests and exit

Real sources with reserve closure, untyped binders, overloaded and imported
symbols, and each item kind; a round trip showing that replacing the source
binders with the elaborated binders leaves the checked statement unchanged.
Require independent specification, test-sufficiency, implementation,
volume/scope, and consistency reviews; `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
Exit: every item has an elaborated statement with the same meaning as its
source, and the artifact and hover expose it.
