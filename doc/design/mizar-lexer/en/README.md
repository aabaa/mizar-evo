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
| [todo.md](./todo.md) | `crates/mizar-lexer`, `tests/lexical`, review follow-ups | Quality-review follow-up tasks | Living |

## Crate Boundary

`mizar-lexer` provides:

- LF-only raw lexical scanning after source loading has normalized newlines;
- source preprocessing helpers for comment stripping, LF/ASCII diagnostics, and module source naming contracts;
- strict ASCII-only code-region spelling rules without Unicode normalization;
- source span preserving raw lexical units;
- lightweight source-span to line/column conversion helpers for lexer diagnostics and tests;
- helper APIs for identifier, numeral, layout, reserved word, and symbol-shape recognition;
- final token disambiguation support that consumes lexical environment, parser expectation, and scoped bindings.

It must not:

- read files or normalize platform newlines;
- normalize Unicode source text or accept non-ASCII code identifiers/symbols by normalization;
- resolve imports by itself;
- load full module IR for imported files;
- decide that an identifier is undefined;
- perform type checking, overload resolution, or proof-related semantics.

## Responsibility Boundaries

The lexer crate may keep shallow pre-parser helpers when they are required to
construct the final lexical token stream without a full parser. These helpers
must remain spelling-based, source-span-preserving, and deterministic. They may
recover enough structure for the lexer handoff, but they must not become
authoritative syntax, source, or semantic services.

| Capability | Long-term owner | Current `mizar-lexer` role |
|---|---|---|
| File I/O, source discovery, package-root enforcement, symlink/case policy, source identity, snapshots | `mizar-session` or a frontend/source service | No file I/O. The crate-local byte-loading and module-name helpers are executable boundary contracts for tests and early integration, not filesystem ownership. |
| UTF-8 validation, leading BOM stripping, CRLF-to-LF normalization, and original-byte loading maps | Source/session layer | `load_source_text_from_bytes` mirrors the boundary for tests and simple callers. Session/source code should own the production loading map and may reuse or mirror the same behavior. |
| Comment stripping, documentation-comment trivia, lexical-text source maps, and lexer-boundary malformed text diagnostics | Lexer for the lexical handoff; session/source for retained source-map services | `preprocess_source_for_lexing` stays in `mizar-lexer` because raw scanning, import pre-scan, and scope skeletons consume the comment-stripped lexical text. Rich retained maps and editor snapshots stay outside lexer. |
| Raw scanning, reserved tables, identifier/numeral/symbol spelling helpers, final token spans | `mizar-lexer` | Owned directly by `mizar-lexer`; spans remain byte offsets into the exact scanner input. |
| Import prelude shape extraction | `mizar-lexer` for pre-parser extraction; module resolver/build planner for resolution | `scan_import_prelude` may extract raw import stubs before final disambiguation. It must not resolve modules, load summaries, check visibility, or validate import placement beyond prelude termination. |
| Active lexical environment from resolved imports and lexical summaries | Boundary between module resolver and lexer | `build_lexical_environment` consumes already resolved imports and module lexical summaries. Producing those summaries and deciding the import graph is outside lexer. |
| Scope skeleton needed for lexical overrides | `mizar-lexer` until the parser provides an equivalent pre-tokenization handoff | `build_scope_skeleton` may conservatively recover binding ranges needed by `ScopeLexView`. The parser/resolver owns the authoritative AST, syntax acceptance, name lookup, and semantic lifetimes. |
| Parser lexical context | Parser | `ParserLexContext` is a parser-facing request object consumed by disambiguation. The lexer does not decide grammar progress beyond honoring the supplied context. |
| Human-facing diagnostics, rendering, tab expansion, one-based columns, LSP UTF-16 positions | Diagnostic/frontend/LSP adapter crates | Lexer diagnostics keep stable codes and byte spans. Human/protocol coordinate conversion stays explicit outside tokens and outside core lexer state. |
| Type checking, overload resolution, proof semantics, undefined-name diagnostics | Resolver/elaborator/kernel-facing phases | No lexer ownership. User-symbol tokens carry spelling and span; downstream phases recover candidates and choose meanings. |

The intended dependency direction is:

```text
session/source/frontend
  -> raw/preprocessed source text
  -> mizar-lexer
  -> raw tokens, import stubs, lexical summaries/environment handoff, final tokens
  -> parser
  -> resolver/elaborator/proof phases
  -> diagnostics/LSP adapters for rendering and protocol conversion
```

Adapters may depend on both `mizar-lexer` and session/source crates to bridge
coordinate spaces, but `mizar-lexer` should not depend on parser, resolver,
session snapshot, diagnostic rendering, or LSP protocol crates. If a future
frontend crate owns the complete source-to-token handoff, it may wrap or move
the executable source-loading helpers, but the lexer must keep its token spans
byte-oriented and its pre-parser helpers limited to lexical handoff data.
