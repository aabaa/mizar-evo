# Task EDITOR-VSCODE-EXTENSION: VS Code extension on the Marketplace
Canonical language: English; [Japanese pointer](../ja/EDITOR-VSCODE-EXTENSION.md).
Status: planned. Tier: full. Owner: [lsp todo](../../mizar-lsp/en/todo.md); the package lives in `editors/vscode/`, outside the Rust workspace.
Dependencies: EDITOR-LSP-ENTRY; DIST-TOOLCHAIN-MANAGER for automatic toolchain installation.
Authority: specification §1 (editor integration), §2 and Appendix A (lexical syntax for highlighting), §23.6 (LSP features).
Gap: `design_drift`; the user-facing gate requires that installing the extension is enough to start verifying.

## Scope

A VS Code extension with a language client that starts `mizar lsp`, a
TextMate grammar derived from the lexical syntax for highlighting before the
server starts, language configuration (comments, brackets, indentation), and
a server-path setting. On first activation without a toolchain, it asks for
consent and installs the project's pinned toolchain through the toolchain
manager. Publish to the Marketplace through a release pipeline.

## Forbidden

Language semantics in the extension; installing without consent; a
highlighting grammar that contradicts the specification's lexical rules.

## Tests and exit

Extension integration tests for activation, toolchain bootstrap, diagnostics
display, and server restart; grammar tests on representative corpus files.
Require specification, test-sufficiency, implementation, volume/scope, and
consistency reviews. Exit: on a clean machine, installing the extension and
opening a Mizar project yields diagnostics without manual setup.
