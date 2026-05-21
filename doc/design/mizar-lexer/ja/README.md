# Module Specifications: mizar-lexer

> Canonical language: English. English canonical version: [../en/README.md](../en/README.md).

`mizar-lexer` は、Mizar Evo の source text を token 化するための基盤機能を担当します。

Mizar の字句分類は、import、active user symbol、parser position、scoped identifier binding に依存します。そのため、この crate では raw lexical scanning と final token disambiguation を明確に分けて扱います。

## Context

- [doc/spec/en/02.lexical_structure.md](../../../spec/en/02.lexical_structure.md) - lexical structure, identifiers, symbols, layout, literals, annotations
- [doc/spec/en/11.symbol_management.md](../../../spec/en/11.symbol_management.md) - user-defined symbols and active lexicons
- [doc/design/architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md) - frontend pipeline and source/token boundaries
- [doc/design/architecture/en/03.module_and_symbol_resolution.md](../../architecture/en/03.module_and_symbol_resolution.md) - module imports, namespaces, and symbol resolution
- [doc/design/internal/en/07.crate_module_layout.md](../../internal/en/07.crate_module_layout.md) - crate/module ownership map

## Index

| Document | Maps To | Description | Status |
|---|---|---|---|
| [raw_lexer.md](./raw_lexer.md) | `crates/mizar-lexer/src/raw_lexer.rs`, source/tables 境界の補足を含む | Raw scanning, `LexemeRun`, source preprocessing handoff, reserved tables, disambiguation boundary | Draft |
| [import_prescan.md](./import_prescan.md) | `crates/mizar-lexer/src/import_prescan.rs` | Import prelude scanning and `ImportStub` extraction from raw tokens | Draft |
| [lexical_environment.md](./lexical_environment.md) | `crates/mizar-lexer/src/lexical_environment.rs` | Active lexical environment construction from reserved tables and module lexical summaries | Draft |
| [scope_skeleton.md](./scope_skeleton.md) | `crates/mizar-lexer/src/scope_skeleton.rs` | Reserved-keyword-based lexical scope skeleton and `ScopeLexView` projection | Draft |
| [disambiguator.md](./disambiguator.md) | `crates/mizar-lexer/src/disambiguator.rs` | Context-sensitive longest-match token disambiguation from `LexemeRun` to final tokens | Draft |
| [test_and_implementation_plan.md](./test_and_implementation_plan.md) | `tests/lexical`, `tests/coverage/spec_trace.toml`, `crates/mizar-lexer` | Ordered lexer test corpus and implementation checklist | Draft |

## Crate Boundary

`mizar-lexer` provides:

- source loading が LF-only に正規化した後の raw lexical scanning;
- comment stripping、LF/ASCII diagnostics、module source naming contract を扱う source preprocessing helpers;
- source span を保持する raw lexical units;
- identifier, numeral, layout, reserved word, symbol shape recognition の helper API;
- lexical environment, parser expectation, scoped bindings を受け取る final token disambiguation support.

この crate は以下を行いません。

- file read や platform newline normalization;
- import resolution;
- imported file の full module IR を読み込む;
- identifier が undefined かどうかを決める;
- type checking、overload resolution、proof-related semantics.
