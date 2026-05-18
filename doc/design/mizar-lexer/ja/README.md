# Module Specifications: mizar-lexer

> Canonical language: English. English canonical version: [../en/README.md](../en/README.md).

`mizar-lexer` は Mizar Evo の source-text tokenization primitives を担当します。

Mizar の字句分類は import、active user symbols、parser position、scoped identifier bindings に依存します。そのため、この crate は raw lexical scanning と final token disambiguation を分けて扱います。

## Context

- [doc/spec/en/02.lexical_structure.md](../../../spec/en/02.lexical_structure.md) - lexical structure, identifiers, symbols, layout, literals, annotations
- [doc/spec/en/11.symbol_management.md](../../../spec/en/11.symbol_management.md) - user-defined symbols and active lexicons
- [doc/design/architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md) - frontend pipeline and source/token boundaries
- [doc/design/architecture/en/03.module_and_symbol_resolution.md](../../architecture/en/03.module_and_symbol_resolution.md) - module imports, namespaces, and symbol resolution
- [doc/design/internal/en/07.crate_module_layout.md](../../internal/en/07.crate_module_layout.md) - crate/module ownership map

## Index

| Document | Maps To | Description | Status |
|---|---|---|---|
| [raw_lexer.md](./raw_lexer.md) | `crates/mizar-lexer/src/lib.rs` initially; later split modules | Raw scanning, `LexemeRun`, disambiguation boundary, scope-sensitive symbol/identifier rules | Draft |

## Crate Boundary

`mizar-lexer` provides:

- source loading が LF-only に正規化した後の raw lexical scanning;
- source span を保持する raw lexical units;
- identifier, numeral, layout, reserved word, symbol shape recognition の helper API;
- lexical environment, parser expectation, scoped bindings を受け取る将来の disambiguation support.

It must not:

- file read や platform newline normalization を行う;
- import resolution を単独で行う;
- imported file の full module IR を読み込む;
- identifier が undefined かどうかを決める;
- type checking, overload resolution, proof-related semantics を行う.
