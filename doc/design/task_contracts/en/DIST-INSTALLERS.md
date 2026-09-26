# Task DIST-INSTALLERS: installers for non-VS Code users
Canonical language: English; [Japanese pointer](../ja/DIST-INSTALLERS.md).
Status: planned. Tier: light. Owner: [driver plan](../../mizar-driver/en/00.crate_plan.md); consumers: users of other editors.
Dependencies: DIST-TOOLCHAIN-MANAGER; EDITOR-LSP-ENTRY for editor setup instructions.
Authority: specification §1 (LSP for editors such as Emacs and Neovim).
Gap: `design_drift`; users outside VS Code have no supported installation path.

## Scope

Provide a one-line install script for macOS and Linux and a signed Windows
installer (also published through winget), each installing the toolchain
manager and the current toolchain, plus a Homebrew formula. Document how to
configure common editors to run `mizar lsp`.

## Forbidden

Installers that bypass the toolchain manager; unsigned Windows or macOS
binaries in releases; editor-specific language features.

## Tests and exit

Clean-machine installation on each platform, rerun idempotence, and
uninstall. Require specification, implementation, volume/scope, and
consistency reviews; the verification commands of `AGENTS.md` for any Rust
change. Exit: on each supported platform one documented command yields a
working toolchain and editor setup instructions.
