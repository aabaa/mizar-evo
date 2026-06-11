# mizar-syntax

`mizar-syntax` defines the `SurfaceAst` boundary for parsed Mizar Evo source.

The crate should provide syntax data structures that are stable enough for the parser, resolver, LSP, formatter, and tests to share, while still remaining internal compiler data rather than a stable public artifact schema.

Status: the representation foundation owns a rowan-backed `SurfaceAst`, typed
compatibility views, deterministic snapshot rendering, syntax diagnostics, and
recovery nodes sufficient for frontend parser-seam integration. Full trivia and
expanded recovery modeling remain planned.

Initial module specs:

- `ast.md`
- `trivia.md`
- `recovery.md`

Implementation roadmap: [todo.md](./todo.md).
