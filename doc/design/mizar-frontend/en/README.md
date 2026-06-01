# mizar-frontend

`mizar-frontend` coordinates source loading, preprocessing, lexing, and parsing.

The crate should wire together `mizar-session`, `mizar-lexer`, `mizar-syntax`, and `mizar-parser` without taking ownership of token internals, syntax node definitions, or grammar implementation.

Initial module specs:

- `source.md`
- `orchestration.md`

