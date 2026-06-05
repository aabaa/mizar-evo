# mizar-syntax

`mizar-syntax` defines the `SurfaceAst` boundary for parsed Mizar Evo source.

The crate should provide syntax data structures that are stable enough for the parser, resolver, LSP, formatter, and tests to share, while still remaining internal compiler data rather than a stable public artifact schema.

Status: the task-11 minimal crate defines `SurfaceAst`, source-preserving nodes,
syntax diagnostics, and recovery markers sufficient for frontend parser-seam
integration. Full trivia and recovery modeling remain planned.

Initial module specs:

- `ast.md`
- `trivia.md`
- `recovery.md`
