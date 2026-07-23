# mizar-syntax Design

Canonical language: English. Japanese companion: [ja/README.md](./ja/README.md).

This directory contains implementation-facing design notes for the `mizar-syntax` crate.

`mizar-syntax` owns the source-shaped syntax layer produced by `mizar-parser` and consumed by `mizar-resolve`, diagnostics, LSP features, formatters, and tests. It should remain semantic-free: no resolved symbol identities, inferred types, overload resolution results, cluster facts, or proof obligations belong here.

Status: the crate owns the rowan-backed `SurfaceAst`, typed compatibility
views, deterministic snapshot rendering, syntax diagnostics, syntax-owned
trivia side tables, task-35 surface vocabulary, parser task-36 predicate
redefinition label follow-through, task-24 private AST source split, and the
post-exit parser Task 48 `PropertyImplementation` vocabulary increment and
parser Task 46 `OperatorDeclaration` vocabulary increment. Task 46 adds
append-only raw kind 193 plus syntax-only typed/snapshot/rowan support; it does
not own operator activation, resolution, or precedence semantics. S-021
rustdoc summaries remain explicitly deferred.

## Crate Plan

- [en/00.crate_plan.md](./en/00.crate_plan.md) - autonomous crate-development kickoff plan, known gaps, task split, and exit gates

## Expected Module Specs

- [en/ast.md](./en/ast.md) - `SurfaceAst`, rowan storage, compatibility node views, and syntax node kinds
- [en/trivia.md](./en/trivia.md) - comments, doc-comment attachment targets, whitespace/trivia retention
- [en/recovery.md](./en/recovery.md) - error nodes, skipped tokens, and parser recovery markers

## Grammar Gate Notes

- [en/grammar_audit.md](./en/grammar_audit.md) - Appendix A consistency audit before AST node vocabulary growth
- [en/parse_only_acceptance_matrix.md](./en/parse_only_acceptance_matrix.md) - parse-only matrix and fixture ownership plan before AST snapshots
- [en/parse_only_fixture_seed.md](./en/parse_only_fixture_seed.md) - inactive fixture seed manifest for later parser activation

## Cross-Cutting Audits

- [en/source_spec_correspondence.md](./en/source_spec_correspondence.md) - S-025 plus parser Tasks 48/46 source/spec/test correspondence
- [en/bilingual_documentation_synchronization.md](./en/bilingual_documentation_synchronization.md) - S-025 plus parser Tasks 48/46 bilingual synchronization
- [en/crate_exit_report.md](./en/crate_exit_report.md) - historical close-out evidence plus parser Tasks 48/46 post-exit addenda
- [en/todo.md](./en/todo.md) - implementation roadmap and deferred S-021 trigger
