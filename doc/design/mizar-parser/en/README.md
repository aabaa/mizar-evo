# mizar-parser

`mizar-parser` implements the syntax grammar for Mizar Evo.

It should keep a narrow dependency on lexer output and syntax structures: `TokenStream` in, `SurfaceAst` plus syntax diagnostics out. Parser-assisted lexing is allowed only through explicit context objects such as string-required positions and symbol-kind filters.

Initial module specs:

- `grammar.md`
- `pratt.md`
- `recovery.md`

