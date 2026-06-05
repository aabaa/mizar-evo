# mizar-parser

`mizar-parser` implements the syntax grammar for Mizar Evo.

It should keep a narrow dependency on parser-facing token transfer objects and
syntax structures: frontend-adapted tokens in, `SurfaceAst` plus syntax
diagnostics out. Parser-assisted lexing is allowed only through explicit context
objects such as string-required positions and symbol-kind filters.

Status: the task-11 minimal crate exposes a parser entry point that consumes the
frontend-adapted token transfer object with session `SourceRange`s, returns
`mizar_syntax::SurfaceAst` plus syntax diagnostics, preserves token order and
ranges, and exercises explicit operator fixity through a small Pratt parser.
Full grammar coverage and recovery remain planned.

Initial module specs:

- `grammar.md`
- `pratt.md`
- `recovery.md`
