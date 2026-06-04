# Module: lexing

> Canonical language: English. Japanese companion: [../ja/lexing.md](../ja/lexing.md).

Status: planned.

## Purpose

This module implements the frontend pipeline Step 4 (lexing / disambiguation). It drives the
`mizar-lexer` raw scanner, scope-skeleton pre-scan, and context-sensitive
disambiguator to turn a `PreprocessedSource` plus an `ActiveLexicalEnvironment`
into a `TokenStream` whose spans are `mizar-session` `SourceRange` values.

It owns the wiring and the span bridging; it does not own the longest-match
rules, the scope-skeleton construction, or the parser-lex context semantics
(those live in `mizar-lexer`). It makes no semantic decisions about whether an
identifier is defined or which overload applies.

See
[architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md)
"Step 4: Lex", "Lexing Is Raw First, Then Contextually Disambiguated",
"Dot Handling Is Split Across Lexer, Parser, and Resolver", and
"String Literals Are Fully Tokenized by the Lexer".

## Public API

```rust
pub struct TokenStream {
    pub source_id: SourceId,
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<LexDiagnostic>,
}

pub struct Token {
    pub kind: TokenKind,
    pub span: SourceRange,
    pub text: InternedText,
}

pub struct TokenizeRequest<'a> {
    pub preprocessed: &'a PreprocessedSource,
    pub environment: &'a ActiveLexicalEnvironment,
    pub parser_context: Option<&'a ParserLexContext>,
}

pub fn tokenize(request: TokenizeRequest<'_>, bridge: &SpanBridge) -> TokenStream;
```

`TokenKind`, `ParserLexContext`, `ParserLexMode`, and `LexDiagnostic` are
re-exported from `mizar-lexer`. The frontend's `Token` differs from the lexer's
internal token only in that `span` is a session `SourceRange` rather than a
lexical-text byte span: `tokenize` maps each lexer span through `span_bridge`.

`parser_context` carries the narrow grammar-derived signals the disambiguator
needs — string-required positions and symbol-kind filters — without exposing
arbitrary parser state. When `None`, lexing runs in the default mode and the
parser-integrated path supplies context incrementally during Step 5.

## Dependencies

- Internal: `preprocess` (lexical text + map), `lexical_env`
  (`ActiveLexicalEnvironment`), `span_bridge` (lexical-byte → `SourceRange`),
  `parsing` (consumes `TokenStream`).
- External: `mizar-lexer` (`scan_raw`, `build_scope_skeleton`, `ScopeLexView`,
  `disambiguate`, `lex`, `Token`, `TokenKind`, `ParserLexContext`,
  `LexDiagnostic`), `mizar-session` (`SourceId`, `SourceRange`).

This module is consumed by parsing and, through the diagnostics, by the
orchestration merge.

## Data Structures

### Token Stream

`TokenStream` is the complete, source-faithful token sequence for one file. Each
`Token` preserves its original spelling (`text`) and a session `SourceRange`.
`TokenKind` includes `LexemeRun` for raw units that were not split,
`UserSymbol`, `ReservedWord`, `ReservedSymbol`, `Identifier`, `Numeral`,
`StringLiteral`, and `ErrorRecovery`. `StringLiteral` appears only at
grammar-defined string-required positions signalled through `parser_context`.

### Scope Lex View

The scope-skeleton pre-scan (`build_scope_skeleton`) produces a read-only
`ScopeLexView` over raw lexer output, used by the disambiguator for scoped
identifier override rules. The frontend builds this view and passes it to the
disambiguator; it does not build scopes itself, and the view records lexical
shape only — never resolved bindings.

## Algorithm / Logic

### Tokenize a preprocessed source

1. Raw-scan the lexical text (`scan_raw`) into `LexemeRun`s with preserved spans.
2. Build the `ScopeSkeleton` / `ScopeLexView` from the raw tokens for scoped
   identifier override.
3. Run `disambiguate` (or the parser-integrated `lex`) with longest-match
   against, in order: active user symbols, reserved special symbols, reserved
   words, identifier/numeral rules, and parser expectation / scoped override
   where the language requires it.
4. Recognize `StringLiteral` tokens only at the string-required positions
   indicated by `parser_context`; outside those positions, quote characters stay
   ordinary symbol characters.
5. Map each resulting lexer span through `span_bridge` to a `SourceRange` scoped
   by `source_id`.
6. Collect lexer diagnostics and return the `TokenStream`.

Compound reserved tokens (`.{`, `.*`, `.=`, `...`) are recognized by the lexer;
selector/namespace roles for `.` are left to the parser and resolver. The lexer
never decides definedness, applicability, or overload selection.

## Error Handling

Lexer recovery is preserved end to end:

- malformed spans emit `TokenKind::ErrorRecovery` with the original source range
  retained for diagnostics;
- scanning resumes at whitespace, reserved delimiters, or line boundaries;
- `LexDiagnostic`s (unknown/malformed token, invalid numeral, malformed string
  literal in a string-required position) are collected without dropping
  recoverable tokens.

`tokenize` always returns a `TokenStream`; lexical failures degrade to recovery
tokens and diagnostics rather than aborting, so the parser can still attempt
recovery.

## Tests

Key scenarios:

- a user symbol that shares spelling with an identifier is classified by
  longest-match against the active lexical environment;
- import-order tie-breaking selects the expected user symbol for equal-length
  matches;
- compound reserved tokens (`.{`, `.*`, `.=`, `...`) lex as single tokens;
- a quote character lexes as a user-symbol character outside string-required
  positions and as a `StringLiteral` at an annotation/operator-declaration
  argument position;
- a malformed token emits `ErrorRecovery` with the correct `SourceRange` and
  scanning resumes;
- every emitted token's `span` is a valid `SourceRange` for `source_id`,
  reproducing the original spelling through the source map.

## Constraints and Assumptions

- This module orchestrates the lexer; it does not own longest-match, scope
  skeleton, or disambiguation rules.
- The lexer recognizes compound reserved tokens and active user symbols but not
  semantic selector/namespace roles.
- `StringLiteral` tokens are emitted only at grammar-defined string-required
  positions.
- The `TokenStream` cache key is the preprocessed hash plus the active lexical
  environment fingerprint.
- All token spans are session `SourceRange` values produced through `span_bridge`.
