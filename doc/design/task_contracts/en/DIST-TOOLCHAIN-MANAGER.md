# Task DIST-TOOLCHAIN-MANAGER: toolchain manager for mizar and ATP backends
Canonical language: English; [Japanese pointer](../ja/DIST-TOOLCHAIN-MANAGER.md).
Status: planned; the project version pin needs a specification change, and bundling needs the license review below. Tier: full. Owner: [driver plan](../../mizar-driver/en/00.crate_plan.md); consumers: [lsp todo](../../mizar-lsp/en/todo.md) (VS Code extension), installers, playground.
Dependencies: release builds of `mizar` per platform; a manifest field that pins the toolchain version of a project (specification §23.1 change, to be proposed separately).
Authority: specification §23.1 (manifest), §21.7.3 (ATP backends); [ATP backend integration](../../architecture/en/10.atp_backend_integration.md).
Gap: `design_drift`. Verification needs the `mizar` binary and external ATP backends, and results depend on their versions, but no component installs or pins them.

## Scope

Provide a small separate binary (working name `mizarup`) that installs
released `mizar` toolchains and pinned ATP backend builds for the host
platform, selects the toolchain pinned by the current project, and updates or
removes toolchains. Before bundling any backend, record the redistribution
terms of each (E is GPL-licensed; Vampire, cvc5, and z3 terms must be checked)
and choose bundling or download per backend.

## Forbidden

Changing verification behavior; silently upgrading a pinned toolchain;
downloading without integrity verification; bundling a backend whose terms
have not been reviewed.

## Tests and exit

Install, select, update, and remove on each supported platform; a project pin
that selects an older toolchain; checksum failure and offline behavior. Require
independent specification, test-sufficiency, implementation, volume/scope, and
consistency reviews; `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
Exit: a clean machine obtains a working pinned toolchain, including ATP
backends, with one command.
