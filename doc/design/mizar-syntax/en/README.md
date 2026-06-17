# mizar-syntax

`mizar-syntax` defines the `SurfaceAst` boundary for parsed Mizar Evo source.

The crate should provide syntax data structures that are stable enough for the parser, resolver, LSP, formatter, and tests to share, while still remaining internal compiler data rather than a stable public artifact schema.

Status: the crate owns a rowan-backed `SurfaceAst`, typed compatibility views,
deterministic snapshot rendering, syntax diagnostics, syntax-owned trivia side
tables, and the task-35 surface vocabulary. The S-019 source/spec audit found
no missing implementation for the public API and behavior promises in the
module specs.

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
