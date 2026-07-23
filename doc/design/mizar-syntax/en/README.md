# mizar-syntax

> Canonical language: English. Japanese companion: [../ja/README.md](../ja/README.md).

`mizar-syntax` defines the `SurfaceAst` boundary for parsed Mizar Evo source.

The crate should provide syntax data structures that are stable enough for the parser, resolver, LSP, formatter, and tests to share, while still remaining internal compiler data rather than a stable public artifact schema.

Status: the crate owns a rowan-backed `SurfaceAst`, typed compatibility views,
deterministic snapshot rendering, syntax diagnostics, syntax-owned trivia side
tables, the task-35 surface vocabulary, the task-22 predicate redefinition
label follow-through paired with parser task 36, and the task-24 private AST
source split. Parser Task 48 later adds the post-exit
`PropertyImplementation` vocabulary increment: append-only `SyntaxKind` 192,
the matching surface node/accessor/snapshot/raw-kind/rowan contract, and active
parse-only pass/fail evidence. This is syntax-only coverage under
`SPEC-07-PI-PLACEMENT`; Task-39 property semantics and S-021 rustdoc summaries
remain deferred.

Autonomous crate-development kickoff plan:

- [00.crate_plan.md](./00.crate_plan.md)

Initial module specs:

- `ast.md`
- `trivia.md`
- `recovery.md`

Grammar-gate planning notes:

- `grammar_audit.md`
- `parse_only_acceptance_matrix.md`
- `parse_only_fixture_seed.md`

Cross-cutting audit notes:

- `source_spec_correspondence.md`
- `bilingual_documentation_synchronization.md`
- `crate_exit_report.md`

Implementation roadmap: [todo.md](./todo.md).
