# mizar-parser

`mizar-parser` implements the syntax grammar for Mizar Evo.

It should keep a narrow dependency on parser-facing token transfer objects and
syntax structures: frontend-adapted tokens in, `SurfaceAst` plus syntax
diagnostics out. Parser-assisted lexing is allowed only through explicit context
objects such as string-required positions and symbol-kind filters.

Status: the minimal crate exposes a parser entry point that consumes the
frontend-adapted token transfer object with session `SourceRange`s, returns
`mizar_syntax::SurfaceAst` plus syntax diagnostics, preserves token order and
ranges, exercises explicit operator fixity through a small Pratt parser, and
implements task-12 recovery passthrough plus task-28 nested missing-`end`
matching, missing string literals, and unrecoverable `ast = None`. Task 2
parser infrastructure is in place as private cursor, syntax-event,
expected-token diagnostic, synchronization, and recovery emission helpers.
Full grammar coverage remains planned.

Initial module specs:

- `grammar.md`
- `pratt.md`
- `recovery.md`

Implementation roadmap: [todo.md](./todo.md).
