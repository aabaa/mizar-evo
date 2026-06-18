# mizar-syntax

`mizar-syntax` defines the `SurfaceAst` boundary for parsed Mizar Evo source.

The crate should provide syntax data structures that are stable enough for the parser, resolver, LSP, formatter, and tests to share, while still remaining internal compiler data rather than a stable public artifact schema.

Status: the crate owns a rowan-backed `SurfaceAst`, typed compatibility views,
deterministic snapshot rendering, syntax diagnostics, syntax-owned trivia side
tables, the task-35 surface vocabulary, and the task-22 predicate redefinition
label follow-through paired with parser task 36. The S-023 follow-up audits
found no remaining source/spec or bilingual documentation gap for that repair.

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
