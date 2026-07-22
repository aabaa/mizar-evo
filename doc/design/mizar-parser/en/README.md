# mizar-parser

`mizar-parser` implements the syntax grammar for Mizar Evo.

It should keep a narrow dependency on parser-facing token transfer objects and
syntax structures: frontend-adapted tokens in, `SurfaceAst` plus syntax
diagnostics out. Parser-assisted lexing is allowed only through explicit context
objects such as string-required positions and symbol-kind filters.

Status: the crate exposes a parser entry point that consumes the
frontend-adapted token transfer object with session `SourceRange`s and returns
`mizar_syntax::SurfaceAst` plus syntax diagnostics. Task 2 parser
infrastructure is in place as private cursor, syntax-event, expected-token
diagnostic, synchronization, and recovery-emission helpers. The parser
workstream has grown through parser task 42, including module/import/export,
type/term/formula, statement/proof, definition, structure, registration,
template, algorithm, verification-clause, annotation surfaces, and the
predicate redefinition label repair, plus recovery consolidation and fail-corpus
expansion, parse-only `SurfaceAst` snapshot baselines, parser determinism
coverage, the parser-owned valid-UTF-8 fuzz target, and frontend passthrough
follow-through audit, plus a private annotation/test module-boundary split.
Task 43 source/spec correspondence and reserved-word coverage audit is
complete, task 44 bilingual documentation synchronization is complete, task
45 public enum policy is complete, and task 47 aligns all three canonical
`reconsider` tail forms. Task 48 is the next authorized nonempty Step-5 parser
task. Task 46 remains deferred for future concrete operator declarations.

Module specs and audits:

- `00.crate_plan.md`
- `grammar.md`
- `pratt.md`
- `recovery.md`
- `source_spec_audit.md`
- `bilingual_documentation_synchronization.md`

Implementation roadmap: [todo.md](./todo.md).
