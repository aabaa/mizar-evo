# Task EDITOR-LSP-COMPLETION: code completion
Canonical language: English; [Japanese pointer](../ja/EDITOR-LSP-COMPLETION.md).
Status: planned. Tier: full. Owner: [lsp todo](../../mizar-lsp/en/todo.md); consumers: editor clients.
Dependencies: mizar-lsp tasks 6-7 (open-buffer snapshots) and 12-15 (metadata and navigation).
Authority: specification §1 (code completion through LSP), §23.5 and §23.6 (artifacts and hover data); [architecture 12](../../architecture/en/12.diagnostics_and_lsp.md); [internal 03](../../internal/en/03.diagnostics_model_and_lsp_bridge.md).
Gap: `design_drift`. The specification promises code completion, but no mizar-lsp task owns it.

## Scope

Serve completion items from the current snapshot and published metadata:
visible symbols and their notation, theorem and definition labels for `by`
citations, namespace path segments, and keywords valid at the position. Rank
deterministically, mark items from stale metadata as stale, and document the
module in a `completion.md` spec written in the same task.

## Forbidden

Semantic analysis inside the LSP server beyond published metadata and snapshot
data; proposing proof steps or premises (that belongs to the explanation and
agent interfaces); specification changes.

## Tests and exit

Completion in type, term, formula, and citation positions, including imported
and aliased names, stale metadata, and recovered buffers; determinism across
repeated requests. Require independent specification, test-sufficiency,
implementation, volume/scope, and consistency reviews; `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
Exit: completion requests return deterministic items drawn only from
published metadata and the current snapshot.
