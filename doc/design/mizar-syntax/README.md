# mizar-syntax Design

Canonical language: English. Japanese companion: [ja/README.md](./ja/README.md).

This directory contains implementation-facing design notes for the `mizar-syntax` crate.

`mizar-syntax` owns the source-shaped syntax layer produced by `mizar-parser` and consumed by `mizar-resolve`, diagnostics, LSP features, formatters, and tests. It should remain semantic-free: no resolved symbol identities, inferred types, overload resolution results, cluster facts, or proof obligations belong here.

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
