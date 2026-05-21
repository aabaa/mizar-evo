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
| [raw_lexer.md](./raw_lexer.md) | `crates/mizar-lexer/src/raw_lexer.rs`, with source/tables boundary notes | Raw scanning, `LexemeRun`, source preprocessing handoff, reserved tables, and the disambiguation boundary | Draft |
| [import_prescan.md](./import_prescan.md) | `crates/mizar-lexer/src/import_prescan.rs` | Import prelude scanning and `ImportStub` extraction from raw tokens | Draft |
| [lexical_environment.md](./lexical_environment.md) | `crates/mizar-lexer/src/lexical_environment.rs` | Active lexical environment construction from reserved tables and module lexical summaries | Draft |
| [scope_skeleton.md](./scope_skeleton.md) | `crates/mizar-lexer/src/scope_skeleton.rs` | Reserved-keyword-based lexical scope skeleton and `ScopeLexView` projection | Draft |
| [disambiguator.md](./disambiguator.md) | `crates/mizar-lexer/src/disambiguator.rs` | Context-sensitive longest-match token disambiguation from `LexemeRun` to final tokens | Draft |
| [test_and_implementation_plan.md](./test_and_implementation_plan.md) | `tests/lexical`, `tests/coverage/spec_trace.toml`, `crates/mizar-lexer` | Ordered lexer test corpus and implementation checklist | Draft |

## Crate Boundary

`mizar-lexer` provides:

- LF-only raw lexical scanning after source loading has normalized newlines;
- source preprocessing helpers for comment stripping, LF/ASCII diagnostics, and module source naming contracts;
- source span preserving raw lexical units;
- helper APIs for identifier, numeral, layout, reserved word, and symbol-shape recognition;
- final token disambiguation support that consumes lexical environment, parser expectation, and scoped bindings.

It must not:

- read files or normalize platform newlines;
- resolve imports by itself;
- load full module IR for imported files;
- decide that an identifier is undefined;
- perform type checking, overload resolution, or proof-related semantics.
