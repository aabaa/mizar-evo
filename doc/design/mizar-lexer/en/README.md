# Module Specifications: mizar-lexer

> Canonical language: English. Japanese companion: [../ja/README.md](../ja/README.md).

`mizar-lexer` owns source-text tokenization primitives for Mizar Evo.

The crate must preserve the distinction between raw lexical scanning and final token disambiguation because Mizar lexical classification depends on imports, active user symbols, parser position, and scoped identifier bindings.

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

- LF-only raw lexical scanning after source loading has normalized newlines;
- source span preserving raw lexical units;
- helper APIs for identifier, numeral, layout, reserved word, and symbol-shape recognition;
- future disambiguation support that can consume lexical environment, parser expectation, and scoped bindings.

It must not:

- read files or normalize platform newlines;
- resolve imports by itself;
- load full module IR for imported files;
- decide that an identifier is undefined;
- perform type checking, overload resolution, or proof-related semantics.
