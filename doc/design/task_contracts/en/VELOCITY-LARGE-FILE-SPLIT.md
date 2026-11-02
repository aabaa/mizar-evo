# Task VELOCITY-LARGE-FILE-SPLIT: split Rust files over 20k lines into modules
Canonical language: English; [Japanese pointer](../ja/VELOCITY-LARGE-FILE-SPLIT.md).
Status: planned. Tier: light. Primary owners: [checker plan](../../mizar-checker/en/00.crate_plan.md), [test plan](../../mizar-test/en/00.crate_plan.md).
Dependencies: none. Readiness: immediate; run while no other task edits the listed files.
Authority: [module boundary audit](../../mizar-checker/en/module_boundary_audit.md); [gate tiering](../../autonomous_crate_development.md#gate-tiering), light tier.
Gap: `design_drift`. Several Rust files exceed 20k lines. In `source_statement.rs` the production part is a small fraction; the rest is `#[cfg(test)]` oracles. Every slice therefore recompiles and re-reads a whole file that no agent can hold in context.

## Scope
Move each `#[cfg(test)]` body of the listed files into `<module>/tests/<family>.rs` submodules declared from `<module>.rs` (`#[cfg(test)] mod tests;`), grouped by the task or family prefix of the test names. Shared helpers go to `<module>/tests/support.rs`. Split `crates/mizar-driver/tests/source_load.rs` into `tests/source_load/main.rs` plus one file per family so it remains one test binary.
Files: `crates/mizar-checker/src/{source_statement,type_checker}.rs`, `crates/mizar-core/src/elaborator.rs`, `crates/mizar-test/src/runner/{proof_verification,tests}.rs`, `crates/mizar-driver/tests/source_load.rs`.
Production code moves only when a file's non-test part still exceeds 10k lines; then split along the existing `impl`/producer boundaries without changing any item's name, visibility, or body.
Update `checker_source_inventory.tsv` for the new paths only; no `split_required` or decision change.

## Forbidden
Any change to item bodies, names, visibility, test names, assertions, fixtures, `doc/spec`, `.miz`, expectations, trace status, or coverage credit. No new helper, trait, or public item. No reordering that changes a test's identity.

## Verification and exit
`cargo fmt --check`; Clippy per `AGENTS.md`; `cargo test -p <crate> -- --list` before and after with identical test sets; `cargo test -p` each touched crate; `bash tests/check.sh docs standard`. One independent equivalence review. The light-tier commit names the tier.
Exit: no Rust file over 20k lines, except a file whose production part alone exceeds that and is listed with its reason in the module boundary audit.
