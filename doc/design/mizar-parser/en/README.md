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
diagnostic, synchronization, and recovery-emission helpers. Grammar coverage has
grown through parser task 39, including module/import/export,
type/term/formula, statement/proof, definition, structure, registration,
template, algorithm, verification-clause, annotation surfaces, and the
predicate redefinition label repair, plus recovery consolidation and fail-corpus
expansion, parse-only `SurfaceAst` snapshot baselines, and parser determinism
coverage. Hardening/audit tasks 40-45 remain planned.

Initial module specs:

- `grammar.md`
- `pratt.md`
- `recovery.md`

Implementation roadmap: [todo.md](./todo.md).
