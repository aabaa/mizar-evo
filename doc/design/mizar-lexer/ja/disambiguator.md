# Module: disambiguator

> Canonical language: English. English canonical version: [../en/disambiguator.md](../en/disambiguator.md).

## Purpose

This module converts raw lexer output into final tokens.

It owns context-sensitive longest-match over `LexemeRun` content. It consumes the active lexical environment, parser lexical context, and read-only scope view. It does not build those inputs.

## Public API

Expected API direction:

```rust
pub struct TokenStream {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<LexDiagnostic>,
}

pub struct Token {
    pub kind: TokenKind,
    pub span: SourceRange,
    pub lexeme: String,
}

pub enum TokenKind {
    Identifier,
    ReservedWord(ReservedWord),
    ReservedSymbol(ReservedSymbol),
    UserSymbol(SymbolId),
    Numeral,
    StringLiteral,
    Error,
}

pub fn disambiguate(
    raw: &RawTokenStream,
    lexical_env: &ActiveLexicalEnvironment,
    parser_context: &ParserLexContext,
    scope_view: &dyn ScopeLexView,
) -> TokenStream;
```

## Candidate Selection

At each position inside a `LexemeRun`, the disambiguator gathers candidates:

1. reserved compound symbols;
2. active user symbols;
3. reserved words;
4. identifier syntax;
5. numeral syntax for digit-starting runs;
6. parser context が許可する場合のみ、`LexemeRun` 内で scan される string-literal candidates;
7. fallback error candidates.

The selected candidate is the longest valid candidate after parser expectation and scope override rules are applied.

## Identifier and Symbol Override

When a spelling is both identifier-shaped and an active user symbol:

1. `ScopeLexView` に、この source position で scoped identifier binding が symbol を override するか問い合わせる。
2. `ParserLexContext` に、この grammar position で identifier が legal か問い合わせる。
3. override and parser context が許す場合のみ `Identifier` を emit する。
4. Otherwise, active `UserSymbol` candidate を emit する。

Undefined identifiers are still emitted as `Identifier`; name resolution reports undefined-name diagnostics later.

## Parser Context

`ParserLexContext` is a narrow input that expresses lexical expectations, not a parser callback into arbitrary syntax logic.

Examples:

- identifier-required position;
- symbol/operator-admitting expression position;
- string-literal-required argument position;
- namespace-path position;
- recovery mode after a syntax error.

The disambiguator must not mutate parser state except by returning tokens and diagnostics.

Quote characters は raw scanning 中は `LexemeRun` の一部として残ります。Disambiguator は、`ParserLexContext` が current position を string-required と示す場合にのみ string literals を認識します。それ以外の位置では、quotes は ordinary symbol characters として user-symbol matching に参加します。

## Error Handling

Disambiguation diagnostics include:

- no valid token candidate in a raw run;
- parser context rejects every candidate;
- equal-length ambiguity without a deterministic shadowing rule;
- malformed string literal in a string-required position;
- unsupported numeral form.

Whenever possible, the disambiguator emits an `Error` token with the original source span and resumes at the next recoverable boundary.

## Tests

Tests should cover:

- longest-match over punctuation-shaped user symbols;
- identifier-shaped user symbol vs scoped identifier override;
- reserved word emission;
- namespace-path context;
- string literal only in string-required positions;
- equal-length import tie breaking through lexical environment;
- recovery emits stable `Error` tokens and diagnostics.
